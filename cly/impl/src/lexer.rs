// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ast::Span;
use crate::result::{ParseError, ParseResult};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Ident(usize),
    Number(i128),
    Const,
    Typedef,
    Unnamed,
    BitsPerByte,
    PragmaPack,
    AttrPacked,
    Align,
    Sizeof,
    SizeofBits,
    OffsetOf,
    OffsetOfBits,
    Opaque,
    Enum,
    Struct,
    Union,
    Unit,
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    Char,
    Signed,
    Unsigned,
    Short,
    Int,
    Long,
    F32,
    F64,
    Float,
    Double,
    Ptr,
    LeftParen,
    LeftBrace,
    LeftBracket,
    RightParen,
    RightBrace,
    RightBracket,
    Comma,
    Dot,
    Eq,
    EqEq,
    NotEq,
    Le,
    Lt,
    Ge,
    Gt,
    Plus,
    Minus,
    Star,
    Div,
    Mod,
    Not,
    OrOr,
    AndAnd,
    At,
    Colon,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Token::Ident(_) => return f.write_str("identifier"),
            Token::Number(_) => return f.write_str("integer literal"),
            Token::Const => "const",
            Token::Typedef => "typedef",
            Token::Unnamed => "_",
            Token::BitsPerByte => "BITS_PER_BYTE",
            Token::PragmaPack => "pragma_pack",
            Token::AttrPacked => "attr_packed",
            Token::Align => "align",
            Token::Sizeof => "sizeof",
            Token::SizeofBits => "sizeof_bits",
            Token::OffsetOf => "offsetof",
            Token::OffsetOfBits => "offsetof_bits",
            Token::Opaque => "opaque",
            Token::Enum => "enum",
            Token::Struct => "struct",
            Token::Union => "union",
            Token::Unit => "unit",
            Token::Bool => "bool",
            Token::U8 => "u8",
            Token::I8 => "i8",
            Token::U16 => "u16",
            Token::I16 => "i16",
            Token::U32 => "u32",
            Token::I32 => "i32",
            Token::U64 => "u64",
            Token::I64 => "i64",
            Token::U128 => "u128",
            Token::I128 => "i128",
            Token::Char => "char",
            Token::Signed => "signed",
            Token::Unsigned => "unsigned",
            Token::Short => "short",
            Token::Int => "int",
            Token::Long => "long",
            Token::F32 => "f32",
            Token::F64 => "f64",
            Token::Float => "float",
            Token::Double => "double",
            Token::Ptr => "ptr",
            Token::LeftParen => "(",
            Token::LeftBrace => "{",
            Token::LeftBracket => "[",
            Token::RightParen => ")",
            Token::RightBrace => "}",
            Token::RightBracket => "]",
            Token::Comma => ",",
            Token::Dot => ".",
            Token::Eq => "=",
            Token::EqEq => "==",
            Token::NotEq => "!=",
            Token::Le => "<=",
            Token::Lt => "<",
            Token::Ge => ">=",
            Token::Gt => ">",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::Div => "/",
            Token::Mod => "%",
            Token::Not => "!",
            Token::OrOr => "||",
            Token::AndAnd => "&&",
            Token::At => "@",
            Token::Colon => ":",
        };
        write!(f, "token `{}`", s)
    }
}

impl Token {
    fn spanned(self, span: Span) -> Spanned<Self> {
        Spanned { span, val: self }
    }
}

#[derive(Copy, Clone)]
pub struct Spanned<T> {
    pub span: Span,
    pub val: T,
}

pub fn lex(chars: &[u8]) -> ParseResult<(Vec<Spanned<Token>>, Vec<String>)> {
    Lexer::new(chars).lex()
}

struct Lexer<'a> {
    pos: usize,
    chars: &'a [u8],
    strings: Vec<String>,
}

impl<'a> Lexer<'a> {
    fn new(chars: &'a [u8]) -> Self {
        Self {
            pos: 0,
            chars,
            strings: vec![],
        }
    }

    fn lex(mut self) -> ParseResult<(Vec<Spanned<Token>>, Vec<String>)> {
        let mut tokens = vec![];
        while let Some(token) = self.lex_one()? {
            tokens.push(token);
        }
        Ok((tokens, self.strings))
    }

    fn lex_one(&mut self) -> ParseResult<Option<Spanned<Token>>> {
        self.skip_whitespace();
        if self.pos == self.chars.len() {
            return Ok(None);
        }

        let mut span = Span(self.pos, self.pos + 1);
        let cur = self.chars[self.pos];
        self.pos += 1;

        if self.pos < self.chars.len() {
            let next = self.chars[self.pos];
            let token = match (cur, next) {
                (b'=', b'=') => Some(Token::EqEq),
                (b'!', b'=') => Some(Token::NotEq),
                (b'<', b'=') => Some(Token::Le),
                (b'>', b'=') => Some(Token::Ge),
                (b'|', b'|') => Some(Token::OrOr),
                (b'&', b'&') => Some(Token::AndAnd),
                _ => None,
            };
            if let Some(token) = token {
                self.pos += 1;
                span.1 += 1;
                return Ok(Some(token.spanned(span)));
            }
        }

        let token = match cur {
            b',' => Some(Token::Comma),
            b'.' => Some(Token::Dot),
            b'(' => Some(Token::LeftParen),
            b'{' => Some(Token::LeftBrace),
            b'[' => Some(Token::LeftBracket),
            b')' => Some(Token::RightParen),
            b'}' => Some(Token::RightBrace),
            b']' => Some(Token::RightBracket),
            b'=' => Some(Token::Eq),
            b'>' => Some(Token::Gt),
            b'<' => Some(Token::Lt),
            b'+' => Some(Token::Plus),
            b'-' => Some(Token::Minus),
            b'*' => Some(Token::Star),
            b'/' => Some(Token::Div),
            b'%' => Some(Token::Mod),
            b'!' => Some(Token::Not),
            b'@' => Some(Token::At),
            b':' => Some(Token::Colon),
            b'_' => {
                if self.pos < self.chars.len() && is_ident_cont(self.chars[self.pos]) {
                    None
                } else {
                    Some(Token::Unnamed)
                }
            }
            _ => None,
        };
        if let Some(token) = token {
            return Ok(Some(token.spanned(span)));
        }

        if !is_ident_cont(cur) {
            return Err(ParseError {
                msg: format!("Unknown symbol {:?}", cur as char),
                span,
            });
        }

        if matches!(cur, b'0'..=b'9') {
            let mut base = 10;
            if cur == b'0' && self.pos < self.chars.len() {
                let next = self.chars[self.pos];
                if is_ident_cont(next) {
                    match next {
                        b'b' => base = 2,
                        b'o' => base = 8,
                        b'x' => base = 16,
                        _ => {}
                    }
                }
            }
            let mut input = vec![];
            if base == 10 {
                input.push(cur);
            } else {
                self.pos += 1;
                span.1 += 1;
            }
            while self.pos < self.chars.len() {
                let next = self.chars[self.pos];
                match next {
                    b'_' => {}
                    b'0'..=b'1' => {}
                    b'2'..=b'7' if base > 2 => {}
                    b'8'..=b'9' if base > 8 => {}
                    b'a'..=b'f' | b'A'..=b'F' if base > 10 => {}
                    _ => break,
                }
                if next != b'_' {
                    input.push(next);
                }
                self.pos += 1;
                span.1 += 1;
            }
            if input.is_empty() {
                return Err(ParseError {
                    msg: "Empty number literal".to_string(),
                    span,
                });
            }
            let s = unsafe { std::str::from_utf8_unchecked(&input) };
            return match i128::from_str_radix(s, base) {
                Ok(v) => Ok(Some(Token::Number(v).spanned(span))),
                _ => Err(ParseError {
                    msg: "Out of bounds number literal".to_string(),
                    span,
                }),
            };
        }

        let mut ident = vec![];
        ident.push(cur);
        while self.pos < self.chars.len() {
            let next = self.chars[self.pos];
            if !is_ident_cont(next) {
                break;
            }
            ident.push(next);
            self.pos += 1;
        }
        span.1 += ident.len() - 1;

        let ident = unsafe { String::from_utf8_unchecked(ident) };

        let keyword = match &*ident {
            "const" => Some(Token::Const),
            "typedef" => Some(Token::Typedef),
            "BITS_PER_BYTE" => Some(Token::BitsPerByte),
            "pragma_pack" => Some(Token::PragmaPack),
            "attr_packed" => Some(Token::AttrPacked),
            "align" => Some(Token::Align),
            "sizeof" => Some(Token::Sizeof),
            "sizeof_bits" => Some(Token::SizeofBits),
            "offsetof" => Some(Token::OffsetOf),
            "offsetof_bits" => Some(Token::OffsetOfBits),
            "opaque" => Some(Token::Opaque),
            "enum" => Some(Token::Enum),
            "struct" => Some(Token::Struct),
            "union" => Some(Token::Union),
            "unit" => Some(Token::Unit),
            "bool" => Some(Token::Bool),
            "u8" => Some(Token::U8),
            "i8" => Some(Token::I8),
            "u16" => Some(Token::U16),
            "i16" => Some(Token::I16),
            "u32" => Some(Token::U32),
            "i32" => Some(Token::I32),
            "u64" => Some(Token::U64),
            "i64" => Some(Token::I64),
            "u128" => Some(Token::U128),
            "i128" => Some(Token::I128),
            "char" => Some(Token::Char),
            "signed" => Some(Token::Signed),
            "unsigned" => Some(Token::Unsigned),
            "short" => Some(Token::Short),
            "int" => Some(Token::Int),
            "long" => Some(Token::Long),
            "f32" => Some(Token::F32),
            "f64" => Some(Token::F64),
            "float" => Some(Token::Float),
            "double" => Some(Token::Double),
            "ptr" => Some(Token::Ptr),
            _ => None,
        };

        let token = match keyword {
            Some(k) => k,
            None => {
                self.strings.push(ident);
                Token::Ident(self.strings.len() - 1)
            }
        };

        Ok(Some(token.spanned(span)))
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                b'/' if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == b'/' => {
                    self.pos += 2;
                    while self.pos < self.chars.len() && self.chars[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                }
                _ => return,
            }
        }
    }
}

fn is_ident_cont(c: u8) -> bool {
    matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9')
}
