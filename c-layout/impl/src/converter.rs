use crate::ast::{
    BinaryExprType, BuiltinExpr, DeclarationType, ExprType, Index, IndexType, OffsetofType, Span,
    TypeExprType, UnaryExprType,
};
use crate::{ast, to_span, S};
use anyhow::{anyhow, Result};
use repr_c_impl::layout::{
    Annotation, Array, FieldLayout, LayoutInfo, Record, RecordField, Type, TypeLayout, TypeVariant,
};
use repr_c_impl::target::Target;
use repr_c_impl::util::BITS_PER_BYTE;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::ops::Not;

#[derive(Eq, PartialEq)]
pub struct ConversionResult {
    pub types: HashMap<String, Type<TypeLayout>>,
    pub consts: HashMap<String, i128>,
}

pub fn extract_layouts(input: &str, d: &[ast::Declaration]) -> Result<ConversionResult> {
    struct Converter<'a>(&'a str);
    impl<'a> Convert for Converter<'a> {
        type Src = TypeLayout;

        fn convert(&self, ty: Type<Self::Src>) -> Result<Type<TypeLayout>> {
            Ok(ty)
        }

        fn extract_type(&self, t: &ast::Type) -> Result<Self::Src> {
            match t.layout {
                None => Err(anyhow!(
                    "At {}: Missing type layout",
                    to_span(self.0, Span(t.lo, t.lo))
                )),
                Some(l) => Ok(l),
            }
        }

        fn extract_field(&self, t: &ast::RecordField, _pos: usize) -> Result<FieldLayout> {
            match t.layout {
                None => Err(anyhow!(
                    "At {}: Missing field layout",
                    to_span(self.0, Span(t.lo, t.lo))
                )),
                Some(l) => Ok(l),
            }
        }
    }
    Computer::new(input, d, Converter(input))?.compute_layouts()
}

pub fn compute_layouts(
    input: &str,
    d: &[ast::Declaration],
    target: Target,
) -> Result<ConversionResult> {
    struct Converter(Target);
    impl Convert for Converter {
        type Src = ();

        fn convert(&self, ty: Type<Self::Src>) -> Result<Type<TypeLayout>> {
            Ok(repr_c_impl::builder::compute_layout(self.0, &ty)?)
        }

        fn extract_type(&self, _: &ast::Type) -> Result<Self::Src> {
            Ok(())
        }

        fn extract_field(&self, _: &ast::RecordField, _: usize) -> Result<()> {
            Ok(())
        }
    }
    Computer::new(input, d, Converter(target))?.compute_layouts()
}

pub trait Convert {
    type Src: LayoutInfo<OpaqueLayout = TypeLayout>;

    fn convert(&self, ty: Type<Self::Src>) -> Result<Type<TypeLayout>>;
    fn extract_type(&self, ty: &ast::Type) -> Result<Self::Src>;
    fn extract_field(
        &self,
        field: &ast::RecordField,
        pos: usize,
    ) -> Result<<Self::Src as LayoutInfo>::FieldLayout>;
}

pub struct Computer<'a, C> {
    input: &'a str,
    d: &'a [ast::Declaration],
    declarations: HashMap<&'a str, &'a ast::Declaration>,
    type_layouts: HashMap<String, Type<TypeLayout>>,
    constants: HashMap<String, i128>,
    converting: HashSet<&'a str>,
    converter: C,
}

impl<'a, C: Convert> Computer<'a, C> {
    pub fn new(input: &'a str, d: &'a [ast::Declaration], converter: C) -> Result<Self> {
        let mut declarations = HashMap::new();
        for d in d {
            if let Some(old) = declarations.insert(&*d.name, d) {
                return Err(anyhow!(
                    "At {}: Type {} is declared multiple times. Previous declaration at {}",
                    to_span(input, d.span),
                    d.name,
                    to_span(input, old.span)
                ));
            }
        }
        Ok(Computer {
            input,
            d,
            declarations,
            converter,
            type_layouts: Default::default(),
            converting: Default::default(),
            constants: Default::default(),
        })
    }

    pub fn compute_layouts(mut self) -> Result<ConversionResult> {
        for d in self.d {
            match d.ty {
                DeclarationType::Type(_) => {
                    self.compute_decl_ty_layout(d, d.span)?;
                }
                DeclarationType::Const(_) => {
                    self.compute_decl_const(d, d.span)?;
                }
            }
        }
        Ok(ConversionResult {
            types: self.type_layouts,
            consts: self.constants,
        })
    }

    fn span(&self, span: Span) -> S {
        to_span(self.input, span)
    }

    fn compute_decl_const(&mut self, d: &'a ast::Declaration, site: Span) -> Result<i128> {
        if let Some(value) = self.constants.get(&d.name) {
            return Ok(*value);
        }
        if self.converting.insert(&d.name).not() {
            return Err(anyhow!(
                "At {}: The value of {} depends on itself",
                self.span(d.span),
                d.name
            ));
        }
        let e = match &d.ty {
            DeclarationType::Type(_) => {
                return Err(anyhow!(
                    "At {}: {} is declared as a type but must be a constant at {}",
                    self.span(d.span),
                    d.name,
                    self.span(site),
                ));
            }
            DeclarationType::Const(e) => e,
        };
        let res = self.eval_expr(e);
        self.converting.remove(&*d.name);
        let res = res?;
        self.constants.insert(d.name.clone(), res);
        Ok(res)
    }

    fn compute_decl_ty_layout(
        &mut self,
        d: &'a ast::Declaration,
        site: Span,
    ) -> Result<TypeLayout> {
        if let Some(layout) = self.type_layouts.get(&d.name) {
            return Ok(layout.layout);
        }
        let ty = match &d.ty {
            DeclarationType::Type(ty) => ty,
            DeclarationType::Const(_) => {
                return Err(anyhow!(
                    "At {}: {} is declared as a constant but must be a type at {}",
                    self.span(d.span),
                    d.name,
                    self.span(site),
                ));
            }
        };
        if self.converting.insert(&d.name).not() {
            return Err(anyhow!(
                "At {}: The layout of {} depends on itself",
                self.span(d.span),
                d.name
            ));
        }
        let res = self.compute_type_layout(ty);
        self.converting.remove(&*d.name);
        let res = res?;
        let layout = res.layout;
        self.type_layouts.insert(d.name.clone(), res);
        Ok(layout)
    }

    fn compute_type_layout(&mut self, t: &'a ast::Type) -> Result<Type<TypeLayout>> {
        let t = self.convert_type(t)?;
        Ok(self.converter.convert(t)?)
    }

    fn convert_type(&mut self, t: &'a ast::Type) -> Result<Type<C::Src>> {
        let variant = match &t.variant {
            ast::TypeVariant::Opaque(l) => TypeVariant::Opaque(TypeLayout {
                size_bits: self.eval_u64_expr(&l.size_bits)?,
                pointer_alignment_bits: self.eval_u64_expr(&l.pointer_alignment_bits)?,
                field_alignment_bits: self.eval_u64_expr(&l.field_alignment_bits)?,
                required_alignment_bits: self.eval_u64_expr(&l.required_alignment_bits)?,
            }),
            ast::TypeVariant::Builtin(bi) => TypeVariant::Builtin(*bi),
            ast::TypeVariant::Record(r) => TypeVariant::Record(self.convert_record(r)?),
            ast::TypeVariant::Array(a) => TypeVariant::Array(self.convert_array(a)?),
            ast::TypeVariant::Name(n, span) => match self.declarations.get(&**n) {
                None => {
                    return Err(anyhow!(
                        "At {}: The referenced type {} is not declared",
                        self.span(*span),
                        n
                    ))
                }
                Some(&d) => TypeVariant::Opaque(self.compute_decl_ty_layout(d, *span)?),
            },
            ast::TypeVariant::Typedef(td) => TypeVariant::Typedef(Box::new(self.convert_type(td)?)),
            ast::TypeVariant::Enum(e) => {
                let mut res = vec![];
                for e in e {
                    res.push(self.eval_expr(e)?);
                }
                TypeVariant::Enum(res)
            }
        };
        Ok(Type {
            layout: self.converter.extract_type(t)?,
            annotations: self.convert_annotations(&t.annotations)?,
            variant,
        })
    }

    fn convert_record(&mut self, r: &'a ast::Record) -> Result<Record<C::Src>> {
        let mut fields = vec![];
        for f in &r.fields {
            fields.push(self.convert_record_field(f)?);
        }
        Ok(Record {
            kind: r.kind,
            fields,
        })
    }

    fn convert_record_field(&mut self, f: &'a ast::RecordField) -> Result<RecordField<C::Src>> {
        Ok(RecordField {
            layout: match f.pos {
                Some(p) => Some(self.converter.extract_field(f, p)?),
                _ => None,
            },
            annotations: self.convert_annotations(&f.annotations)?,
            named: f.name.is_some(),
            bit_width: f
                .bit_width
                .as_ref()
                .map(|w| self.eval_u64_expr(w))
                .transpose()?,
            ty: self.convert_type(&f.ty)?,
        })
    }

    fn convert_annotations(&mut self, a: &'a [ast::Annotation]) -> Result<Vec<Annotation>> {
        let mut res = vec![];
        for a in a {
            res.push(match a {
                ast::Annotation::PragmaPack(e) => {
                    Annotation::PragmaPack(BITS_PER_BYTE * self.eval_u64_expr(e)?)
                }
                ast::Annotation::AttrPacked => Annotation::AttrPacked,
                ast::Annotation::Aligned(e) => {
                    Annotation::Aligned(BITS_PER_BYTE * self.eval_u64_expr(e)?)
                }
            });
        }
        Ok(res)
    }

    fn eval_u64_expr(&mut self, e: &'a ast::Expr) -> Result<u64> {
        let v = self.eval_expr(e)?.try_into().map_err(|_| {
            anyhow!(
                "At {}: Expression value does not fit into u64",
                self.span(e.span)
            )
        })?;
        Ok(v)
    }

    fn eval_expr(&mut self, e: &'a ast::Expr) -> Result<i128> {
        match &e.ty {
            ExprType::Lit(n) => Ok(*n),
            ExprType::Builtin(b) => match b {
                BuiltinExpr::BitsPerByte => Ok(BITS_PER_BYTE as i128),
            },
            ExprType::Unary(k, v) => {
                let v = self.eval_expr(v)?;
                match *k {
                    UnaryExprType::Neg => v
                        .checked_neg()
                        .ok_or_else(|| anyhow!("At {}: Expression overflow", self.span(e.span))),
                    UnaryExprType::Not => Ok(if v != 0 { 0 } else { 1 }),
                }
            }
            ExprType::Binary(k, le, re) => {
                use BinaryExprType::*;
                let l = self.eval_expr(le)?;
                let r = self.eval_expr(re)?;
                match *k {
                    Add | Sub | Mul => match *k {
                        Add => l.checked_add(r),
                        Sub => l.checked_sub(r),
                        Mul => l.checked_mul(r),
                        _ => unreachable!(),
                    }
                    .ok_or_else(|| anyhow!("At {}: Expression overflow", self.span(e.span))),
                    Div | Mod => {
                        if r == 0 {
                            return Err(anyhow!("At {}: Division by zero", self.span(re.span)));
                        }
                        Ok(match *k {
                            Div => l / r,
                            Mod => l % r,
                            _ => unreachable!(),
                        })
                    }
                    LogicalAnd | LogicalOr | Eq | NotEq | Lt | Le | Gt | Ge => {
                        let ll = l != 0;
                        let rr = r != 0;
                        Ok(match *k {
                            LogicalAnd => ll && rr,
                            LogicalOr => ll || rr,
                            Eq => l == r,
                            NotEq => l != r,
                            Lt => l < r,
                            Le => l <= r,
                            Gt => l > r,
                            Ge => l >= r,
                            _ => unreachable!(),
                        } as i128)
                    }
                }
            }
            ExprType::TypeExpr(k, t) => {
                let layout = self.compute_type_layout(t)?.layout;
                Ok(match k {
                    TypeExprType::Sizeof => (layout.size_bits / BITS_PER_BYTE) as i128,
                    TypeExprType::Alignof => (layout.field_alignment_bits / BITS_PER_BYTE) as i128,
                })
            }
            ExprType::Name(n) => match self.declarations.get(&**n) {
                None => {
                    Err(anyhow!(
                        "At {}: The referenced constant {} is not declared",
                        self.span(e.span),
                        n
                    ))
                }
                Some(&d) => self.compute_decl_const(d, e.span),
            },
            ExprType::Offsetof(k, aty, p) => {
                let ty = self.compute_type_layout(aty)?;
                let val = self.eval_offsetof(*k, aty, &ty, &p[0], &p[1..])?;
                match k {
                    OffsetofType::Bytes => Ok((val / BITS_PER_BYTE) as i128),
                    OffsetofType::Bits => Ok(val as i128),
                }
            }
        }
    }

    fn eval_offsetof(
        &mut self,
        k: OffsetofType,
        aty: &'a ast::Type,
        ty: &Type<TypeLayout>,
        head: &'a Index,
        rest: &'a [Index],
    ) -> Result<u64> {
        let (aty, ty, base) = match (&aty.variant, &ty.variant, &head.ty) {
            (ast::TypeVariant::Record(ar), TypeVariant::Record(r), IndexType::Field(name)) => {
                let af = match ar
                    .fields
                    .iter()
                    .find(|f| f.name.as_ref() == Some(name))
                {
                    Some(f) => f,
                    None => {
                        return Err(anyhow!(
                            "At {}: Type has no field {}",
                            self.span(head.span),
                            name
                        ))
                    }
                };
                let pos = ar
                    .fields
                    .iter()
                    .position(|f| f.name.as_ref() == Some(name))
                    .unwrap();
                let f = &r.fields[pos];
                if f.bit_width.is_some() && k == OffsetofType::Bytes {
                    return Err(anyhow!(
                        "At {}: Cannot compute bytewise offset of bit field",
                        self.span(head.span)
                    ));
                }
                (&af.ty, &f.ty, f.layout.unwrap().offset_bits)
            }
            (ast::TypeVariant::Array(aa), TypeVariant::Array(a), IndexType::Array(pos)) => {
                let pos = self.eval_u64_expr(pos)?;
                if pos >= a.num_elements && aa.num_elements.is_some() {
                    return Err(anyhow!("At {}: Out of bounds", self.span(head.span)));
                }
                match a.element_type.layout.size_bits.checked_mul(pos) {
                    None => {
                        return Err(anyhow!("At {}: Offset overflow", self.span(head.span)));
                    }
                    Some(b) => (&*aa.element_type, &*a.element_type, b),
                }
            }
            (ast::TypeVariant::Name(n, span), _, _) => {
                let d = match self.declarations.get(&**n) {
                    None => {
                        return Err(anyhow!(
                            "At {}: The referenced type {} is not declared",
                            self.span(*span),
                            n
                        ))
                    }
                    Some(d) => *d,
                };
                self.compute_decl_ty_layout(d, head.span)?;
                let aty = match &d.ty {
                    DeclarationType::Type(aty) => aty,
                    DeclarationType::Const(_) => unreachable!(),
                };
                let ty = self.type_layouts.get(n).unwrap().clone();
                return self.eval_offsetof(k, aty, &ty, head, rest);
            }
            (_, _, IndexType::Field(_)) => {
                return Err(anyhow!("At {}: Type is not a record", self.span(head.span)));
            }
            (_, _, IndexType::Array(_)) => {
                return Err(anyhow!("At {}: Type is not an array", self.span(head.span)));
            }
        };
        Ok(base
            + match rest {
                [head, rest @ ..] => self.eval_offsetof(k, aty, ty, head, rest)?,
                _ => 0,
            })
    }

    fn convert_array(&mut self, a: &'a ast::Array) -> Result<Array<C::Src>> {
        Ok(Array {
            element_type: Box::new(self.convert_type(&a.element_type)?),
            num_elements: match &a.num_elements {
                None => 0,
                Some(n) => self.eval_u64_expr(n)?,
            },
        })
    }
}
