use crate::layout::{
    Annotation, Array, BuiltinType, LayoutInfo, Record, RecordField, Type, TypeLayout, TypeVariant,
};

pub trait Visitor<I: LayoutInfo> {
    fn visit_type(&mut self, ty: &Type<I>) {
        visit_type(self, ty);
    }

    fn visit_annotations(&mut self, a: &[Annotation]) {
        visit_annotations(self, a);
    }

    fn visit_builtin_type(&mut self, bi: BuiltinType, ty: &Type<I>) {
        visit_builtin_type(self, bi, ty);
    }

    fn visit_record(&mut self, rt: &Record<I>, ty: &Type<I>) {
        visit_record(self, rt, ty);
    }

    fn visit_record_field(&mut self, field: &RecordField<I>, rt: &Record<I>, ty: &Type<I>) {
        visit_record_field(self, field, rt, ty);
    }

    fn visit_typedef(&mut self, dst: &Type<I>, ty: &Type<I>) {
        visit_typedef(self, dst, ty);
    }

    fn visit_array(&mut self, array: &Array<I>, ty: &Type<I>) {
        visit_array(self, array, ty);
    }

    fn visit_opaque_type(&mut self, layout: TypeLayout, ty: &Type<I>) {
        visit_opaque_type(self, layout, ty);
    }

    fn visit_enum(&mut self, v: &[i128], ty: &Type<I>) {
        visit_enum(self, v, ty);
    }
}

pub fn visit_type<I: LayoutInfo>(visitor: &mut (impl Visitor<I> + ?Sized), ty: &Type<I>) {
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

pub fn visit_record<I: LayoutInfo>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    rt: &Record<I>,
    ty: &Type<I>,
) {
    for f in &rt.fields {
        visitor.visit_record_field(f, rt, ty);
    }
}

pub fn visit_annotations<I: LayoutInfo>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    a: &[Annotation],
) {
    let _ = visitor;
    let _ = a;
    // nothing
}

pub fn visit_builtin_type<I: LayoutInfo>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    bi: BuiltinType,
    ty: &Type<I>,
) {
    let _ = visitor;
    let _ = bi;
    let _ = ty;
    // nothing
}

pub fn visit_typedef<I: LayoutInfo>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    dst: &Type<I>,
    ty: &Type<I>,
) {
    let _ = visitor;
    let _ = ty;
    visitor.visit_type(dst);
}

pub fn visit_record_field<I: LayoutInfo>(
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

pub fn visit_array<I: LayoutInfo>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    array: &Array<I>,
    ty: &Type<I>,
) {
    let _ = ty;
    visitor.visit_type(&array.element_type);
}

pub fn visit_opaque_type<I: LayoutInfo>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    layout: TypeLayout,
    ty: &Type<I>,
) {
    let _ = visitor;
    let _ = layout;
    let _ = ty;
    // nothing
}

pub fn visit_enum<I: LayoutInfo>(
    visitor: &mut (impl Visitor<I> + ?Sized),
    v: &[i128],
    ty: &Type<I>,
) {
    let _ = visitor;
    let _ = v;
    let _ = ty;
    // nothing
}
