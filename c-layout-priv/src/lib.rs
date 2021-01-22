use crate::ast::{Declaration, Span, State};
use crate::result::{CustomError, ParseError};
pub use converter::{compute_layouts, extract_layouts};
pub use enhancer::enhance_declarations;
use lalrpop_util::lexer::Token;
pub use printer::{printer, Printer};
use repr_c::target::{
    AArch64PcWindowsMsvc, I686PcWindowsMsvc, Target, Thumbv7aPcWindowsMsvc, X8664PcWindowsMsvc,
};
use std::fmt;
use std::fmt::{Display, Formatter};

macro_rules! g {
    ($c:expr, $name:expr, $lo:expr, $hi:expr) => {
        g!($c, $name, $lo, $hi, None)
    };
    ($c:expr, $name:expr, $lo:expr, $hi:expr, $def:expr) => {{
        let iter = || $c.iter().filter(|c| c.0 == $name);
        if iter().count() > 1 {
            let second = iter().skip(1).next().unwrap();
            return Err(ParseError::User {
                error: CustomError {
                    msg: format!("multiple {}", $name),
                    span: second.2,
                },
            });
        }
        iter()
            .map(|c| c.1.clone())
            .next()
            .or($def)
            .ok_or_else(|| ParseError::User {
                error: CustomError {
                    msg: format!("missing {}", $name),
                    span: Span($lo, $hi),
                },
            })?
    }};
}

macro_rules! h {
    ($c:expr, $($name:expr),+) => {{
        for n in &$c {
            if $(n.0 != $name)&&+ {
                return Err(ParseError::User { error: CustomError { msg: format!("unknown layout component {}", n.0), span: n.2 }});
            }
        }
    }}
}

pub mod ast;
pub mod converter;
mod enhancer;
mod parser;
mod printer;
mod result;

pub fn parse(input: &str) -> Result<Vec<Declaration>, ParseError> {
    let mut state = State { next_id: 0 };
    match parser::TopParser::new().parse(&mut state, input) {
        Ok(d) => Ok(d),
        Err(e) => Err(handle_lalrpop_error(input, e)),
    }
}

fn handle_lalrpop_error(
    input: &str,
    e: lalrpop_util::ParseError<usize, Token<'_>, CustomError>,
) -> ParseError {
    match e {
        lalrpop_util::ParseError::InvalidToken { location } => ParseError {
            msg: format!("At {}: Invalid token", to_line_column(input, location)),
            span: Span(location, location),
        },
        lalrpop_util::ParseError::UnrecognizedEOF { location, expected } => ParseError {
            msg: format!(
                "At {}: Unexpected EOF. {}",
                to_line_column(input, location),
                expected_fmt(&expected)
            ),
            span: Span(location, location),
        },
        lalrpop_util::ParseError::UnrecognizedToken {
            token: (start, token, end),
            expected,
        } => ParseError {
            msg: format!(
                "At {}: Unrecognized token `{}`. {}",
                to_span(input, Span(start, end)),
                token,
                expected_fmt(&expected)
            ),
            span: Span(start, end),
        },
        lalrpop_util::ParseError::ExtraToken {
            token: (start, token, end),
        } => ParseError {
            msg: format!(
                "At {}: Unexpected token `{}`",
                to_span(input, Span(start, end)),
                token
            ),
            span: Span(start, end),
        },
        lalrpop_util::ParseError::User { error } => ParseError {
            msg: format!("At {}: {}", to_span(input, error.span), error.msg),
            span: error.span,
        },
    }
}

struct LC(usize, usize);

fn to_line_column(input: &str, pos: usize) -> LC {
    impl Display for LC {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}:{}", self.0, self.1)
        }
    }
    let newlines: Vec<_> = input[..pos]
        .char_indices()
        .filter(|c| c.1 == '\n')
        .collect();
    LC(
        newlines.len() + 1,
        newlines
            .last()
            .copied()
            .map(|v| pos - v.0 - 1)
            .unwrap_or(pos),
    )
}

struct S(LC, LC);

fn to_span(input: &str, span: Span) -> S {
    impl Display for S {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            if self.0 .0 == self.1 .0 && self.0 .1 + 1 > self.1 .1 {
                write!(f, "{}", self.0)
            } else {
                write!(f, "{} - {}", self.0, self.1)
            }
        }
    }
    S(to_line_column(input, span.0), to_line_column(input, span.1))
}

fn expected_fmt<'a>(e: &'a [String]) -> impl Display + 'a {
    struct D<'a>(&'a [String]);

    impl<'a> Display for D<'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            if self.0.len() == 1 {
                return write!(f, "Expected {}", self.0[0]);
            }
            write!(f, "Expected one of ")?;
            for (i, e) in self.0.iter().enumerate() {
                if i > 0 {
                    if self.0.len() > 2 {
                        write!(f, ",")?;
                    }
                    write!(f, " ")?;
                    if i + 1 == self.0.len() {
                        write!(f, "or ")?;
                    }
                }
                write!(f, "{}", e)?;
            }
            Ok(())
        }
    }

    D(e)
}

pub static TEST_TARGETS: &'static [&'static dyn Target] = &[
    &AArch64PcWindowsMsvc,
    &Thumbv7aPcWindowsMsvc,
    &X8664PcWindowsMsvc,
    &I686PcWindowsMsvc,
];
