// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ast::Span;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub msg: String,
    pub span: Span,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.msg)
    }
}

impl Error for ParseError {}

pub type ParseResult<T> = std::result::Result<T, ParseError>;
