// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ast;
use crate::ast::DeclarationType;
use crate::converter::ConversionResult;
use repc_impl::layout::{Annotation, Array, Record, RecordField, Type, TypeLayout, TypeVariant};
use repc_impl::util::BITS_PER_BYTE;

pub fn enhance_declarations(d: &[ast::Declaration], m: &ConversionResult) -> Vec<ast::Declaration> {
    let mut res = vec![];
    for d in d {
        res.push(ast::Declaration {
            name: d.name.clone(),
            span: d.span,
            ty: match &d.ty {
                DeclarationType::Type(t) => {
                    DeclarationType::Type(enhance_type(t, m.types.get(&d.name).unwrap()))
                }
                DeclarationType::Const(c) => DeclarationType::Const(enhance_top_level_expr(
                    c,
                    *m.consts.get(&d.name).unwrap(),
                )),
            },
        });
    }
    res
}

fn enhance_type(t: &ast::Type, ty: &Type<TypeLayout>) -> ast::Type {
    let variant = match (&t.variant, &ty.variant) {
        (ast::TypeVariant::Record(r), TypeVariant::Record(rc)) => {
            ast::TypeVariant::Record(enhance_record(r, rc))
        }
        (ast::TypeVariant::Typedef(t), TypeVariant::Typedef(td)) => {
            ast::TypeVariant::Typedef(Box::new(enhance_type(t, td)))
        }
        (ast::TypeVariant::Array(a), TypeVariant::Array(ar)) => {
            ast::TypeVariant::Array(enhance_array(a, ar))
        }
        (ast::TypeVariant::Enum(a), TypeVariant::Enum(ar)) => {
            ast::TypeVariant::Enum(enhance_enum(a, ar))
        }
        (_, _) => t.variant.clone(),
    };
    ast::Type {
        id: t.id,
        lo: t.lo,
        layout: Some(ty.layout),
        layout_hi: t.layout_hi,
        annotations: enhance_annotations(&t.annotations, &ty.annotations),
        variant,
    }
}

fn enhance_record(r: &ast::Record, rc: &Record<TypeLayout>) -> ast::Record {
    ast::Record {
        kind: r.kind,
        fields: r
            .fields
            .iter()
            .zip(rc.fields.iter())
            .map(|(f, fc)| enhance_record_field(f, fc))
            .collect(),
    }
}

fn enhance_record_field(f: &ast::RecordField, fc: &RecordField<TypeLayout>) -> ast::RecordField {
    let mut bit_width = f.bit_width.clone();
    if let (Some(bit_width), Some(bw)) = (&mut bit_width, fc.bit_width) {
        bit_width.value = Some(bw as i128);
    }
    ast::RecordField {
        parent_id: f.parent_id,
        pos: f.pos,
        lo: f.lo,
        layout: fc.layout,
        layout_hi: f.layout_hi,
        annotations: enhance_annotations(&f.annotations, &fc.annotations),
        name: f.name.clone(),
        bit_width,
        ty: enhance_type(&f.ty, &fc.ty),
    }
}

fn enhance_array(a: &ast::Array, ar: &Array<TypeLayout>) -> ast::Array {
    ast::Array {
        element_type: Box::new(enhance_type(&a.element_type, &ar.element_type)),
        num_elements: match (&a.num_elements, ar.num_elements) {
            (Some(n), Some(ne)) => Some(Box::new(enhance_top_level_expr(n, ne as i128))),
            _ => None,
        },
    }
}

fn enhance_enum(a: &[ast::Expr], ar: &[i128]) -> Vec<ast::Expr> {
    a.iter()
        .zip(ar.iter())
        .map(|(l, r)| enhance_top_level_expr(l, *r))
        .collect()
}

fn enhance_annotations(a: &[ast::Annotation], an: &[Annotation]) -> Vec<ast::Annotation> {
    a.iter()
        .zip(an.iter())
        .map(|(l, r)| enhance_annotation(l, r))
        .collect()
}

fn enhance_annotation(a: &ast::Annotation, an: &Annotation) -> ast::Annotation {
    match (a, an) {
        (ast::Annotation::Aligned(Some(l)), Annotation::Align(Some(r))) => {
            let mut a = l.clone();
            a.value = Some((*r / BITS_PER_BYTE) as i128);
            ast::Annotation::Aligned(Some(a))
        }
        (ast::Annotation::PragmaPack(l), Annotation::PragmaPack(r)) => {
            let mut a = l.clone();
            a.value = Some((*r / BITS_PER_BYTE) as i128);
            ast::Annotation::PragmaPack(a)
        }
        _ => a.clone(),
    }
}

fn enhance_top_level_expr(e: &ast::Expr, v: i128) -> ast::Expr {
    let mut e = e.clone();
    e.value = Some(v);
    e
}
