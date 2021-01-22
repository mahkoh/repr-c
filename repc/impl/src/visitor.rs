// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::layout::{
    Annotation, Array, BuiltinType, Layout, Record, RecordField, Type, TypeVariant,
};

/// This trait represents a visitor that walks through a [`Type`].
///
/// Each method's default implementation is accessible through the function of the same name in the
/// `visitor` module.
///
/// # Example
///
/// ```
/// # use repc_impl::layout::{Layout, Type};
/// # use repc_impl::visitor::{Visitor, visit_enum};
/// struct Impl;
///
/// impl<I: Layout> Visitor<I> for Impl {
///     fn visit_enum(&mut self, variants: &[i128], ty: &Type<I>) {
///         println!("Variants: {:?}", variants);
///         visit_enum(self, variants, ty)
///     }
/// }
/// ```
pub trait Visitor<I: Layout> {
    /// Called for types.
    fn visit_type(&mut self, ty: &Type<I>) {
        visit_type(self, ty);
    }

    /// Called for the annotations on a type or field.
    fn visit_annotations(&mut self, annotations: &[Annotation]) {
        visit_annotations(self, annotations);
    }

    /// Called for builtin types.
    fn visit_builtin_type(&mut self, builtin_type: BuiltinType, ty: &Type<I>) {
        visit_builtin_type(self, builtin_type, ty);
    }

    /// Called for records.
    fn visit_record(&mut self, record: &Record<I>, ty: &Type<I>) {
        visit_record(self, record, ty);
    }

    /// Called for record fields.
    fn visit_record_field(&mut self, field: &RecordField<I>, record: &Record<I>, ty: &Type<I>) {
        visit_record_field(self, field, record, ty);
    }

    /// Called for typedefs.
    fn visit_typedef(&mut self, dst: &Type<I>, ty: &Type<I>) {
        visit_typedef(self, dst, ty);
    }

    /// Called for arrays.
    fn visit_array(&mut self, array: &Array<I>, ty: &Type<I>) {
        visit_array(self, array, ty);
    }

    /// Called for opaque types.
    fn visit_opaque_type(&mut self, layout: I::OpaqueLayout, ty: &Type<I>) {
        visit_opaque_type(self, layout, ty);
    }

    /// Called for enums.
    fn visit_enum(&mut self, variants: &[i128], ty: &Type<I>) {
        visit_enum(self, variants, ty);
    }
}

/// The default implementation of `Visitor::visit_type`.
pub fn visit_type<I: Layout>(visitor: &mut (impl Visitor<I> + ?Sized), ty: &Type<I>) {
    visitor.visit_annotations(&ty.annotations);
    match &ty.variant {
        TypeVariant::Builtin(bi) => visitor.visit_builtin_type(*bi, ty),
        TypeVariant::Record(rt) => visitor.visit_record(rt, ty),
        TypeVariant::Typedef(td) => visitor.visit_typedef(td, ty),
        TypeVariant::Array(at) => visitor.visit_array(at, ty),
        TypeVariant::Opaque(l) => visitor.visit_opaque_type(*l, ty),
        TypeVariant::Enum(l) => visitor.visit_enum(l, ty),
    }
}

/// The default implementation of `Visitor::visit_record`.
pub fn visit_record<I: Layout>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    record: &Record<I>,
    ty: &Type<I>,
) {
    for f in &record.fields {
        visitor.visit_record_field(f, record, ty);
    }
}

/// The default implementation of `Visitor::visit_annotations`.
pub fn visit_annotations<I: Layout>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    annotations: &[Annotation],
) {
    let _ = visitor;
    let _ = annotations;
    // nothing
}

/// The default implementation of `Visitor::visit_builtin_type`.
pub fn visit_builtin_type<I: Layout>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    builtin_type: BuiltinType,
    ty: &Type<I>,
) {
    let _ = visitor;
    let _ = builtin_type;
    let _ = ty;
    // nothing
}

/// The default implementation of `Visitor::visit_typedef`.
pub fn visit_typedef<I: Layout>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    dst: &Type<I>,
    ty: &Type<I>,
) {
    let _ = visitor;
    let _ = ty;
    visitor.visit_type(dst);
}

/// The default implementation of `Visitor::visit_record`.
pub fn visit_record_field<I: Layout>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    field: &RecordField<I>,
    record: &Record<I>,
    ty: &Type<I>,
) {
    let _ = record;
    let _ = ty;
    visitor.visit_annotations(&field.annotations);
    visitor.visit_type(&field.ty);
}

/// The default implementation of `Visitor::visit_array`.
pub fn visit_array<I: Layout>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    array: &Array<I>,
    ty: &Type<I>,
) {
    let _ = ty;
    visitor.visit_type(&array.element_type);
}

/// The default implementation of `Visitor::visit_opaque_type`.
pub fn visit_opaque_type<I: Layout>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    layout: I::OpaqueLayout,
    ty: &Type<I>,
) {
    let _ = visitor;
    let _ = layout;
    let _ = ty;
    // nothing
}

/// The default implementation of `Visitor::visit_enum`.
pub fn visit_enum<I: Layout>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    variants: &[i128],
    ty: &Type<I>,
) {
    let _ = visitor;
    let _ = variants;
    let _ = ty;
    // nothing
}
