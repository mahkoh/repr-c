use crate::ast::{
    Annotation, Array, BinaryExprType, BuiltinExpr, Declaration, DeclarationType, Expr, ExprType,
    Index, IndexType, OffsetofType, OpaqueTypeLayout, Record, RecordField, Span, Type,
    TypeExprType, TypeVariant, UnaryExprType,
};
use crate::lexer;
use crate::lexer::{Spanned, Token};
use crate::result::{ParseError, ParseResult};
use repr_c_impl::layout::{BuiltinType, FieldLayout, RecordKind, TypeLayout};
use repr_c_impl::util::BITS_PER_BYTE;

pub fn parse(input: &[u8]) -> ParseResult<Vec<Declaration>> {
    let (tokens, strings) = lexer::lex(input)?;
    Parser::new(tokens, strings).parse_declarations()
}

struct Parser {
    type_id: usize,
    pos: usize,
    tokens: Vec<Spanned<Token>>,
    strings: Vec<String>,
}

impl Parser {
    fn new(tokens: Vec<Spanned<Token>>, strings: Vec<String>) -> Self {
        Self {
            type_id: 0,
            pos: 0,
            tokens,
            strings,
        }
    }

    fn parse_declarations(mut self) -> ParseResult<Vec<Declaration>> {
        let mut res = vec![];
        while self.pos < self.tokens.len() {
            res.push(self.parse_declaration()?);
        }
        Ok(res)
    }

    fn parse_declaration(&mut self) -> ParseResult<Declaration> {
        let cur = self.peek()?;
        match cur.val {
            Token::Ident(_) => self.parse_type_declaration(),
            Token::Const => self.parse_const_declaration(),
            _ => Err(ParseError {
                msg: format!("Unexpected {}. Expected `const` or identifier.", cur.val),
                span: cur.span,
            }),
        }
    }

    fn parse_type_declaration(&mut self) -> ParseResult<Declaration> {
        let (name, span) = self.parse_ident()?;
        self.parse_token(Token::Eq)?;
        let ty = self.parse_type()?;
        Ok(Declaration {
            name,
            span,
            ty: DeclarationType::Type(ty),
        })
    }

    fn parse_const_declaration(&mut self) -> ParseResult<Declaration> {
        self.parse_token(Token::Const)?;
        let (name, span) = self.parse_ident()?;
        self.parse_token(Token::Eq)?;
        let expr = self.parse_top_level_expr()?;
        Ok(Declaration {
            name,
            span,
            ty: DeclarationType::Const(expr),
        })
    }

    fn parse_top_level_expr(&mut self) -> ParseResult<Expr> {
        let lo = self.peek()?.span.0;
        let (val, hi) = self.parse_expr_value()?;
        let layout_hi = hi.unwrap_or(lo);
        let mut expr = self.parse_expr()?;
        if val.is_some() {
            expr.value = val;
        } else if let ExprType::Lit(v) = expr.ty {
            expr.value = Some(v);
        }
        expr.span.0 = lo;
        expr.value_hi = layout_hi;
        Ok(expr)
    }

    fn parse_expr_value(&mut self) -> ParseResult<(Option<i128>, Option<usize>)> {
        if self.peek()?.val != Token::LeftBrace {
            return Ok((None, None));
        }
        self.pos += 1;
        let cur = self.parse_token(Token::Number(0))?;
        let val = match cur.val {
            Token::Number(n) => n,
            _ => unreachable!(),
        };
        let hi = self.parse_token(Token::RightBrace)?.span.1;
        Ok((Some(val), Some(hi)))
    }

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        let mut e_stack = vec![];
        let mut o_stack = vec![];

        e_stack.push(self.parse_atomic_expr()?);

        let reduce = |e_stack: &mut Vec<Expr>, o_stack: &mut Vec<Token>, next_predecence| {
            while let Some(&last) = o_stack.last() {
                if precedence(last) >= next_predecence {
                    o_stack.pop();
                    let right = e_stack.pop().unwrap();
                    let left = e_stack.pop().unwrap();
                    let ty = match last {
                        Token::EqEq => BinaryExprType::Eq,
                        Token::NotEq => BinaryExprType::NotEq,
                        Token::Le => BinaryExprType::Le,
                        Token::Lt => BinaryExprType::Lt,
                        Token::Ge => BinaryExprType::Ge,
                        Token::Gt => BinaryExprType::Gt,
                        Token::Plus => BinaryExprType::Add,
                        Token::Minus => BinaryExprType::Sub,
                        Token::Star => BinaryExprType::Mul,
                        Token::Div => BinaryExprType::Div,
                        Token::Mod => BinaryExprType::Mod,
                        Token::OrOr => BinaryExprType::LogicalOr,
                        Token::AndAnd => BinaryExprType::LogicalAnd,
                        _ => unreachable!(),
                    };
                    e_stack.push(Expr {
                        span: Span(left.span.0, right.span.1),
                        value: None,
                        value_hi: 0,
                        ty: ExprType::Binary(ty, Box::new(left), Box::new(right)),
                    });
                } else {
                    break;
                }
            }
        };

        while self.pos < self.tokens.len() {
            let next = self.tokens[self.pos];
            match next.val {
                Token::EqEq
                | Token::NotEq
                | Token::Le
                | Token::Lt
                | Token::Ge
                | Token::Gt
                | Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Div
                | Token::Mod
                | Token::OrOr
                | Token::AndAnd => {}
                _ => break,
            }
            self.pos += 1;
            reduce(&mut e_stack, &mut o_stack, precedence(next.val));
            o_stack.push(next.val);
            e_stack.push(self.parse_atomic_expr()?);
        }

        reduce(&mut e_stack, &mut o_stack, 0);

        assert_eq!(o_stack.len(), 0);
        assert_eq!(e_stack.len(), 1);

        Ok(e_stack.pop().unwrap())
    }

    fn parse_atomic_expr(&mut self) -> ParseResult<Expr> {
        let cur = self.next()?;
        let ty = match cur.val {
            Token::Not => ExprType::Unary(UnaryExprType::Not, Box::new(self.parse_atomic_expr()?)),
            Token::Minus => {
                ExprType::Unary(UnaryExprType::Neg, Box::new(self.parse_atomic_expr()?))
            }
            Token::LeftParen => {
                let expr = self.parse_expr()?;
                self.parse_token(Token::RightParen)?;
                expr.ty
            }
            Token::BitsPerByte => ExprType::Builtin(BuiltinExpr::BitsPerByte),
            Token::Number(v) => ExprType::Lit(v),
            Token::Sizeof | Token::AlignOf => {
                let kind = match cur.val {
                    Token::Sizeof => TypeExprType::Sizeof,
                    _ => TypeExprType::Alignof,
                };
                self.parse_token(Token::LeftParen)?;
                let dst = self.parse_type()?;
                self.parse_token(Token::RightParen)?;
                ExprType::TypeExpr(kind, Box::new(dst))
            }
            Token::OffsetOf | Token::OffsetOfBits => {
                let kind = match cur.val {
                    Token::OffsetOf => OffsetofType::Bytes,
                    _ => OffsetofType::Bits,
                };
                self.parse_token(Token::LeftParen)?;
                let dst = self.parse_type()?;
                self.parse_token(Token::Comma)?;
                let indices = self.parse_offsetof_path()?;
                self.parse_token(Token::RightParen)?;
                ExprType::Offsetof(kind, dst, indices)
            }
            Token::Ident(i) => ExprType::Name(self.strings[i].clone()),
            _ => {
                return Err(ParseError {
                    msg: format!("Unexpected {}. Expected an expression.", cur.val),
                    span: cur.span,
                })
            }
        };
        Ok(Expr {
            span: Span(cur.span.0, self.tokens[self.pos - 1].span.1),
            value: None,
            value_hi: 0,
            ty,
        })
    }

    fn parse_offsetof_path(&mut self) -> ParseResult<Vec<Index>> {
        let mut res = vec![];
        loop {
            let cur = self.next()?;
            let ty = match cur.val {
                Token::LeftBracket => {
                    let expr = self.parse_top_level_expr()?;
                    self.parse_token(Token::RightBracket)?;
                    IndexType::Array(Box::new(expr))
                }
                Token::Ident(i) => IndexType::Field(self.strings[i].clone()),
                _ => {
                    return Err(ParseError {
                        msg: format!("Unexpected {}. Expected `[` or an identifier", cur.val),
                        span: cur.span,
                    })
                }
            };
            res.push(Index {
                span: Span(cur.span.0, self.tokens[self.pos - 1].span.1),
                ty,
            });
            let next = self.peek()?;
            match next.val {
                Token::LeftBracket => {}
                Token::Dot => self.pos += 1,
                Token::RightParen => break,
                _ => {
                    return Err(ParseError {
                        msg: format!("Unexpected {}. Expected `[`, `.`, or `)`", cur.val),
                        span: cur.span,
                    })
                }
            }
        }
        Ok(res)
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        let id = {
            self.type_id += 1;
            self.type_id
        };
        let lo = self.peek()?.span.0;
        let (layout, hi) = self.parse_static_type_layout()?;
        let layout_hi = hi.unwrap_or(lo);
        let annotations = self.parse_annotations()?;
        let variant = self.parse_type_variant(id)?;
        Ok(Type {
            id,
            lo,
            layout,
            layout_hi,
            annotations,
            variant,
        })
    }

    fn parse_type_variant(&mut self, parent_id: usize) -> ParseResult<TypeVariant> {
        let next = self.peek()?;
        match next.val {
            Token::Ident(id) => {
                self.pos += 1;
                Ok(TypeVariant::Name(self.strings[id].clone(), next.span))
            }
            Token::Typedef => self.parse_typedef(),
            Token::Opaque => self.parse_opaque(),
            Token::Enum => self.parse_enum(),
            Token::Struct | Token::Union => self.parse_record(parent_id),
            Token::LeftBracket => self.parse_array(),
            _ => self.parse_builtin_type(),
        }
    }

    fn parse_typedef(&mut self) -> ParseResult<TypeVariant> {
        self.parse_token(Token::Typedef)?;
        let dst = self.parse_type()?;
        Ok(TypeVariant::Typedef(Box::new(dst)))
    }

    fn parse_builtin_type(&mut self) -> ParseResult<TypeVariant> {
        let cur = self.next()?;

        if self.pos + 1 < self.tokens.len() {
            let next = self.tokens[self.pos];
            let after = self.tokens[self.pos + 1];
            let bi = match (cur.val, next.val, after.val) {
                (Token::Unsigned, Token::Long, Token::Long) => Some(BuiltinType::UnsignedLongLong),
                (Token::Signed, Token::Long, Token::Long) => Some(BuiltinType::LongLong),
                _ => None,
            };
            if let Some(bi) = bi {
                self.pos += 2;
                return Ok(TypeVariant::Builtin(bi));
            }
        }

        if self.pos < self.tokens.len() {
            let next = self.tokens[self.pos];
            let bi = match (cur.val, next.val) {
                (Token::Long, Token::Long) => Some(BuiltinType::LongLong),
                (Token::Signed, Token::Char) => Some(BuiltinType::SignedChar),
                (Token::Signed, Token::Short) => Some(BuiltinType::Short),
                (Token::Signed, Token::Int) => Some(BuiltinType::Int),
                (Token::Signed, Token::Long) => Some(BuiltinType::Long),
                (Token::Unsigned, Token::Char) => Some(BuiltinType::UnsignedChar),
                (Token::Unsigned, Token::Short) => Some(BuiltinType::UnsignedShort),
                (Token::Unsigned, Token::Int) => Some(BuiltinType::UnsignedInt),
                (Token::Unsigned, Token::Long) => Some(BuiltinType::UnsignedLong),
                _ => None,
            };
            if let Some(bi) = bi {
                self.pos += 1;
                return Ok(TypeVariant::Builtin(bi));
            }
        }

        let bi = match cur.val {
            Token::Unit => BuiltinType::Unit,
            Token::Bool => BuiltinType::Bool,
            Token::U8 => BuiltinType::U8,
            Token::I8 => BuiltinType::I8,
            Token::U16 => BuiltinType::U16,
            Token::I16 => BuiltinType::I16,
            Token::U32 => BuiltinType::U32,
            Token::I32 => BuiltinType::I32,
            Token::U64 => BuiltinType::U64,
            Token::I64 => BuiltinType::I64,
            Token::U128 => BuiltinType::U128,
            Token::I128 => BuiltinType::I128,
            Token::Char => BuiltinType::Char,
            Token::Signed => BuiltinType::Int,
            Token::Unsigned => BuiltinType::UnsignedInt,
            Token::Short => BuiltinType::Short,
            Token::Int => BuiltinType::Int,
            Token::Long => BuiltinType::Long,
            Token::F32 => BuiltinType::F32,
            Token::F64 => BuiltinType::F64,
            Token::Float => BuiltinType::Float,
            Token::Double => BuiltinType::Double,
            Token::Ptr => BuiltinType::Pointer,
            _ => {
                return Err(ParseError {
                    msg: format!("Unexpected {}. Expected a type.", cur.val),
                    span: cur.span,
                })
            }
        };
        Ok(TypeVariant::Builtin(bi))
    }

    fn parse_static_type_layout(&mut self) -> ParseResult<(Option<TypeLayout>, Option<usize>)> {
        if self.peek()?.val != Token::LeftBrace {
            return Ok((None, None));
        }
        let (size, field, pointer, required, span) = self.parse_type_layout(Self::parse_u64)?;
        Ok((
            Some(TypeLayout {
                size_bits: size,
                field_alignment_bits: field,
                pointer_alignment_bits: pointer,
                required_alignment_bits: required.unwrap_or(BITS_PER_BYTE),
            }),
            Some(span.1),
        ))
    }

    fn parse_opaque(&mut self) -> ParseResult<TypeVariant> {
        self.parse_token(Token::Opaque)?;
        let (size, field, pointer, required, span) =
            self.parse_type_layout(|slf| slf.parse_expr())?;
        Ok(TypeVariant::Opaque(OpaqueTypeLayout {
            size_bits: Box::new(size),
            pointer_alignment_bits: Box::new(pointer),
            field_alignment_bits: Box::new(field),
            required_alignment_bits: Box::new(required.unwrap_or(Expr {
                span,
                value: Some(BITS_PER_BYTE as i128),
                value_hi: span.1,
                ty: ExprType::Lit(BITS_PER_BYTE as i128),
            })),
        }))
    }

    fn parse_enum(&mut self) -> ParseResult<TypeVariant> {
        self.parse_token(Token::Enum)?;
        let mut expr = vec![];
        self.parse_brace_list(|slf| {
            expr.push(slf.parse_top_level_expr()?);
            Ok(())
        })?;
        Ok(TypeVariant::Enum(expr))
    }

    fn parse_annotations(&mut self) -> ParseResult<Vec<Annotation>> {
        let mut res = vec![];
        while let Token::At = self.peek()?.val {
            res.push(self.parse_annotation()?);
        }
        Ok(res)
    }

    fn parse_annotation(&mut self) -> ParseResult<Annotation> {
        self.parse_token(Token::At)?;
        let cur = self.next()?;
        let a = match cur.val {
            Token::PragmaPack => {
                self.parse_token(Token::LeftParen)?;
                let val = self.parse_top_level_expr()?;
                self.parse_token(Token::RightParen)?;
                Annotation::PragmaPack(Box::new(val))
            }
            Token::AttrPacked => Annotation::AttrPacked,
            Token::Align => {
                self.parse_token(Token::LeftParen)?;
                let val = self.parse_top_level_expr()?;
                self.parse_token(Token::RightParen)?;
                Annotation::Aligned(Box::new(val))
            }
            _ => {
                return Err(ParseError {
                    msg: format!(
                        "Unexpected {}. Expected `pragma_pack`, `attr_packed`, or `align`.",
                        cur.val
                    ),
                    span: cur.span,
                })
            }
        };
        Ok(a)
    }

    fn parse_array(&mut self) -> ParseResult<TypeVariant> {
        self.parse_token(Token::LeftBracket)?;
        let num_elements = match self.peek()?.val {
            Token::RightBracket => None,
            _ => Some(Box::new(self.parse_top_level_expr()?)),
        };
        self.parse_token(Token::RightBracket)?;
        let ty = self.parse_type()?;
        Ok(TypeVariant::Array(Array {
            element_type: Box::new(ty),
            num_elements,
        }))
    }

    fn parse_record(&mut self, parent_id: usize) -> ParseResult<TypeVariant> {
        let kind = match self.next()?.val {
            Token::Struct => RecordKind::Struct,
            Token::Union => RecordKind::Union,
            _ => unreachable!(),
        };
        let mut fields = vec![];
        self.parse_brace_list(|slf| {
            fields.push(slf.parse_record_field(parent_id)?);
            Ok(())
        })?;
        let mut i = 0;
        for field in &mut fields {
            if field.name.is_some() {
                field.pos = Some(i);
                i += 1;
            }
        }
        Ok(TypeVariant::Record(Record { kind, fields }))
    }

    fn parse_record_field(&mut self, parent_id: usize) -> ParseResult<RecordField> {
        let lo = self.peek()?.span.0;
        let (layout, hi) = self.parse_field_layout()?;
        let layout_hi = hi.unwrap_or(lo);
        let annotations = self.parse_annotations()?;
        let cur = self.next()?;
        let name = match cur.val {
            Token::Unnamed => None,
            Token::Ident(id) => Some(self.strings[id].clone()),
            _ => {
                return Err(ParseError {
                    msg: format!("Unexpected {}. Expected `_` or identifier.", cur.val),
                    span: cur.span,
                })
            }
        };
        let ty = self.parse_type()?;
        let bit_width = match self.peek()?.val {
            Token::Colon => {
                self.pos += 1;
                Some(Box::new(self.parse_top_level_expr()?))
            }
            _ => None,
        };
        Ok(RecordField {
            parent_id,
            pos: None,
            lo,
            layout,
            layout_hi,
            annotations,
            name,
            bit_width,
            ty,
        })
    }

    fn parse_u64(&mut self) -> ParseResult<u64> {
        let cur = self.parse_token(Token::Number(0))?;
        let num = match cur.val {
            Token::Number(n) => n,
            _ => unreachable!(),
        };
        if num as u64 as i128 != num {
            return Err(ParseError {
                msg: format!("Out of bounds integer literal {}", num),
                span: cur.span,
            });
        }
        Ok(num as u64)
    }

    fn parse_field_layout(&mut self) -> ParseResult<(Option<FieldLayout>, Option<usize>)> {
        if self.peek()?.val != Token::LeftBrace {
            return Ok((None, None));
        }
        let mut size = None;
        let mut offset = None;
        let keys = &mut [("size", &mut size), ("offset", &mut offset)][..];
        let span = self.parse_key_value_list(Self::parse_u64, keys)?;
        for key in keys {
            if key.1.is_none() {
                return Err(ParseError {
                    msg: format!("Missing key {}", key.0),
                    span,
                });
            }
        }
        Ok((
            Some(FieldLayout {
                offset_bits: offset.unwrap(),
                size_bits: size.unwrap(),
            }),
            Some(span.1),
        ))
    }

    fn parse_brace_list<P>(&mut self, mut p: P) -> ParseResult<Span>
    where
        P: FnMut(&mut Self) -> ParseResult<()>,
    {
        let lo = self.parse_token(Token::LeftBrace)?.span.0;
        let hi;
        loop {
            let next = self.peek()?;
            if next.val == Token::RightBrace {
                hi = next.span.1;
                self.pos += 1;
                break;
            }
            p(self)?;
            let next = self.peek()?;
            match next.val {
                Token::Comma => self.pos += 1,
                Token::RightBrace => {}
                _ => {
                    return Err(ParseError {
                        msg: format!("Unexpected {}. Expected `,` or `}}`", next.val),
                        span: next.span,
                    })
                }
            }
        }
        Ok(Span(lo, hi))
    }

    fn parse_key_value_list<V, P>(
        &mut self,
        p: P,
        keys: &mut [(&str, &mut Option<V>)],
    ) -> ParseResult<Span>
    where
        P: Fn(&mut Self) -> ParseResult<V>,
    {
        self.parse_brace_list(|slf| {
            let cur = slf.next()?;
            let key = match cur.val {
                Token::Ident(id) => slf.strings[id].clone(),
                _ => {
                    return Err(ParseError {
                        msg: format!("Unexpected {}. Expected identifier or `}}`", cur.val),
                        span: cur.span,
                    })
                }
            };
            slf.parse_token(Token::Colon)?;
            let val = p(slf)?;
            let mut known = false;
            for (name, dst) in keys.iter_mut() {
                if *name == key {
                    known = true;
                    if dst.is_some() {
                        return Err(ParseError {
                            msg: format!("{} specified multiple times", name),
                            span: cur.span,
                        });
                    }
                    **dst = Some(val);
                    break;
                }
            }
            if !known {
                return Err(ParseError {
                    msg: format!("Unknown key {}", key),
                    span: cur.span,
                });
            }
            Ok(())
        })
    }

    fn parse_type_layout<V: Clone, P>(&mut self, p: P) -> ParseResult<(V, V, V, Option<V>, Span)>
    where
        P: Fn(&mut Self) -> ParseResult<V>,
    {
        let mut size = None;
        let mut alignment = None;
        let mut pointer_alignment = None;
        let mut required_alignment = None;
        let mut field_alignment = None;
        let span = self.parse_key_value_list(
            p,
            &mut [
                ("size", &mut size),
                ("alignment", &mut alignment),
                ("required_alignment", &mut required_alignment),
                ("pointer_alignment", &mut pointer_alignment),
                ("field_alignment", &mut field_alignment),
            ],
        )?;
        let size = match size {
            Some(s) => s,
            _ => {
                return Err(ParseError {
                    msg: "Missing key size".to_string(),
                    span,
                })
            }
        };
        let (field_alignment, pointer_alignment) =
            match (alignment, field_alignment, pointer_alignment) {
                (Some(a), None, None) => (a.clone(), a),
                (None, Some(l), Some(r)) => (l, r),
                (None, Some(_), None) => {
                    return Err(ParseError {
                        msg: "Missing key pointer_alignment".to_string(),
                        span,
                    })
                }
                (None, None, Some(_)) => {
                    return Err(ParseError {
                        msg: "Missing key field_alignment".to_string(),
                        span,
                    })
                }
                (Some(_), Some(_), _) => {
                    return Err(ParseError {
                        msg: "alignment and field_alignment are both specified".to_string(),
                        span,
                    })
                }
                (Some(_), None, Some(_)) => {
                    return Err(ParseError {
                        msg: "alignment and pointer_alignment are both specified".to_string(),
                        span,
                    })
                }
                (None, None, None) => {
                    return Err(ParseError {
                        msg: "Missing alignment specification".to_string(),
                        span,
                    })
                }
            };
        Ok((
            size,
            field_alignment,
            pointer_alignment,
            required_alignment,
            span,
        ))
    }

    fn parse_token(&mut self, token: Token) -> ParseResult<Spanned<Token>> {
        let cur = self.next()?;
        match (cur.val, token) {
            (Token::Ident(_), Token::Ident(_)) => Ok(cur),
            (Token::Number(_), Token::Number(_)) => Ok(cur),
            (_, _) if cur.val == token => Ok(cur),
            _ => Err(ParseError {
                msg: format!("Unexpected {}. Expected {}.", cur.val, token),
                span: cur.span,
            }),
        }
    }

    fn parse_ident(&mut self) -> ParseResult<(String, Span)> {
        let cur = self.parse_token(Token::Ident(0))?;
        match cur.val {
            Token::Ident(id) => Ok((self.strings[id].clone(), cur.span)),
            _ => unreachable!(),
        }
    }

    fn next(&mut self) -> ParseResult<Spanned<Token>> {
        let t = self.peek()?;
        self.pos += 1;
        Ok(t)
    }

    fn peek(&self) -> ParseResult<Spanned<Token>> {
        if self.pos < self.tokens.len() {
            Ok(self.tokens[self.pos])
        } else {
            Err(ParseError {
                msg: "Unexpected end of input".to_string(),
                span: self.tokens.last().unwrap().span,
            })
        }
    }
}

fn precedence(token: Token) -> usize {
    match token {
        Token::Star | Token::Div | Token::Mod => 90,
        Token::Plus | Token::Minus => 80,
        Token::EqEq | Token::NotEq | Token::Le | Token::Lt | Token::Ge | Token::Gt => 70,
        Token::AndAnd => 60,
        Token::OrOr => 50,
        _ => 0,
    }
}
