use crate::ast::Span;
use crate::result::CustomError;
use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;
use num_traits::Num;
use repr_c_impl::layout::FieldLayout;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

type Error<'input> = ParseError<usize, Token<'input>, CustomError>;

fn get_value<'input, T: Clone>(
    components: &HashMap<String, (T, Span)>,
    key: &str,
    span: Span,
) -> Result<T, Error<'input>> {
    match components.get(key) {
        Some(k) => Ok(k.0.clone()),
        _ => Err(ParseError::User {
            error: CustomError {
                span,
                msg: format!("Missing component `{}`", key),
            },
        }),
    }
}

fn err<'input, T>(msg: String, span: Span) -> Result<T, Error<'input>> {
    Err(ParseError::User {
        error: CustomError { span, msg },
    })
}

fn check_unknown<'input, T>(
    components: &HashMap<String, (T, Span)>,
    known: &[&str],
) -> Result<(), Error<'input>> {
    let known: HashSet<&str> = known.iter().copied().collect();
    for c in components.iter() {
        if !known.contains(&**c.0) {
            return err(format!("Unknown component `{}`", c.0), c.1 .1);
        }
    }
    Ok(())
}

pub fn parse_type_layout<'input, T: Clone>(
    c: &HashMap<String, (T, Span)>,
    span: Span,
    required_alignment_default: T,
) -> Result<[T; 4], Error<'input>> {
    const SIZE: &str = "size";
    const ALIGNMENT: &str = "alignment";
    const POINTER_ALIGNMENT: &str = "pointer_alignment";
    const FIELD_ALIGNMENT: &str = "field_alignment";
    const REQUIRED_ALIGNMENT: &str = "required_alignment";
    check_unknown(
        c,
        &[
            SIZE,
            ALIGNMENT,
            POINTER_ALIGNMENT,
            FIELD_ALIGNMENT,
            REQUIRED_ALIGNMENT,
        ],
    )?;
    let (field_alignment_bits, pointer_alignment_bits) = match (
        c.get(ALIGNMENT),
        c.get(FIELD_ALIGNMENT),
        c.get(POINTER_ALIGNMENT),
    ) {
        (Some(a), None, None) => (a.0.clone(), a.0.clone()),
        (None, Some(a), Some(b)) => (a.0.clone(), b.0.clone()),
        (None, None, _) => {
            return err(
                format!(
                    "Neither `{}` nor `{}` is specified",
                    ALIGNMENT, FIELD_ALIGNMENT
                ),
                span,
            )
        }
        (None, _, None) => {
            return err(
                format!(
                    "Neither `{}` nor `{}` is specified",
                    ALIGNMENT, POINTER_ALIGNMENT
                ),
                span,
            )
        }
        (Some(_), Some(_), _) => {
            return err(
                format!(
                    "Both `{}` and `{}` are specified",
                    ALIGNMENT, FIELD_ALIGNMENT
                ),
                span,
            )
        }
        (Some(_), _, Some(_)) => {
            return err(
                format!(
                    "Both `{}` and `{}` are specified",
                    ALIGNMENT, POINTER_ALIGNMENT
                ),
                span,
            )
        }
    };
    Ok([
        get_value(c, SIZE, span)?,
        field_alignment_bits,
        pointer_alignment_bits,
        c.get(REQUIRED_ALIGNMENT)
            .map(|v| v.0.clone())
            .unwrap_or(required_alignment_default),
    ])
}

pub fn components_to_hashmap<'input, T>(
    c: Vec<(String, T, Span)>,
) -> Result<HashMap<String, (T, Span)>, Error<'input>> {
    let mut res = HashMap::new();
    for c in c {
        match res.entry(c.0) {
            Entry::Vacant(v) => {
                v.insert((c.1, c.2));
            }
            Entry::Occupied(o) => {
                return err(format!("`{}` specified multiple times", o.key()), c.2)
            }
        }
    }
    Ok(res)
}

pub fn parse_field_layout<'input>(
    c: &HashMap<String, (u64, Span)>,
    span: Span,
) -> Result<FieldLayout, Error<'input>> {
    const SIZE: &str = "size";
    const OFFSET: &str = "offset";
    check_unknown(c, &[SIZE, OFFSET])?;
    Ok(FieldLayout {
        offset_bits: get_value(c, OFFSET, span)?,
        size_bits: get_value(c, SIZE, span)?,
    })
}

pub fn parse_number<N: Num>(s: &str, span: Span) -> Result<N, Error> {
    let s = s.replace('_', "");
    for (p, r) in [("0x", 16), ("0o", 8), ("0b", 2)].iter() {
        if let Some(s) = s.strip_prefix(p) {
            return match N::from_str_radix(s, *r) {
                Ok(v) => Ok(v),
                _ => err(format!("Out of bounds integer"), span),
            };
        }
    }
    match N::from_str_radix(&s, 10) {
        Ok(v) => Ok(v),
        _ => err(format!("Out of bounds integer"), span),
    }
}
