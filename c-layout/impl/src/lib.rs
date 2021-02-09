// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ast::{Declaration, Span};
use anyhow::{anyhow, Result};
pub use converter::{compute_layouts, extract_layouts};
pub use enhancer::enhance_declarations;
pub use printer::{printer, Printer};
use std::fmt;
use std::fmt::{Display, Formatter};

pub mod ast;
pub mod converter;
mod enhancer;
mod lexer;
mod parser;
mod printer;
mod result;
#[cfg(test)]
mod tests;

pub fn parse(input: &str) -> Result<Vec<Declaration>> {
    match parser::parse(input.as_bytes()) {
        Ok(d) => Ok(d),
        Err(e) => Err(anyhow!("At {}: {}", to_span(input, e.span), e.msg)),
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
