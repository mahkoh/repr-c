use crate::layout::{Annotation, Array, BuiltinType, Record, RecordField, Type, TypeLayout};
use crate::result::{Error, Result};
use crate::target::{LayoutAlgorithm, Target};
use crate::util::BITS_PER_BYTE;
use crate::visitor::{visit_array, visit_builtin_type, visit_opaque_type, visit_record_field, Visitor, visit_typedef};
use std::ops::Not;

mod common;
mod msvc;

pub fn compute_layout(target: &dyn Target, ty: &Type<()>) -> Result<Type<TypeLayout>> {
    pre_validate(ty)?;
    let ty = match target.layout_algorithm() {
        LayoutAlgorithm::Msvc => msvc::compute_layout(target, ty),
        LayoutAlgorithm::SysV => unimplemented!(),
        LayoutAlgorithm::MinGw => unimplemented!(),
    }?;
    post_validate(&ty)?;
    Ok(ty)
}

fn pre_validate(ty: &Type<()>) -> Result<()> {
    let mut pv = PreValidator(vec![]);
    pv.visit_type(ty);
    match pv.0.pop() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

fn post_validate(ty: &Type<TypeLayout>) -> Result<()> {
    let mut pv = PostValidator(vec![]);
    pv.visit_type(ty);
    match pv.0.pop() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

struct PreValidator(Vec<Error>);

impl Visitor<()> for PreValidator {
    fn visit_annotations(&mut self, a: &[Annotation]) {
        let mut num_align = 0;
        let mut num_packed = 0;
        let mut num_pragma_packed = 0;
        for a in a {
            match a {
                Annotation::PragmaPack(n) => {
                    num_pragma_packed += 1;
                    self.validate_alignment(*n);
                }
                Annotation::AttrPacked => num_packed += 1,
                Annotation::Aligned(n) => {
                    num_align += 1;
                    self.validate_alignment(*n);
                }
            }
        }
        if num_align > 1 {
            self.0.push(Error::MultipleAlignmentAnnotations);
        }
        if num_packed > 1 || num_pragma_packed > 1 {
            self.0.push(Error::MultiplePackedAnnotations);
        }
    }

    fn visit_builtin_type(&mut self, bi: BuiltinType, ty: &Type<()>) {
        if ty.annotations.is_empty().not() {
            self.0.push(Error::AnnotatedBuiltinType);
        }
        visit_builtin_type(self, bi, ty);
    }

    fn visit_record_field(&mut self, field: &RecordField<()>, rt: &Record<()>, ty: &Type<()>) {
        match field.bit_width {
            Some(0) => {
                if field.name.is_some() {
                    self.0.push(Error::NamedZeroSizeBitField);
                }
            }
            None => {
                if field.name.is_none() {
                    self.0.push(Error::UnnamedRegularField);
                }
            }
            _ => {}
        }
        visit_record_field(self, field, rt, ty);
    }

    fn visit_typedef(&mut self, dst: &Type<()>, ty: &Type<()>) {
        for a in &dst.annotations {
            match a {
                Annotation::Aligned(_) => {},
                Annotation::PragmaPack(_) => self.0.push(Error::PackedTypedef),
                Annotation::AttrPacked => self.0.push(Error::PackedTypedef),
            }
        }
        visit_typedef(self, dst, ty);
    }

    fn visit_array(&mut self, at: &Array<()>, ty: &Type<()>) {
        if ty.annotations.is_empty().not() {
            self.0.push(Error::AnnotatedArray);
        }
        visit_array(self, at, ty);
    }

    fn visit_opaque_type(&mut self, layout: TypeLayout, ty: &Type<()>) {
        if ty.annotations.is_empty().not() {
            self.0.push(Error::AnnotatedOpaqueType);
        }
        if layout.size_bits % BITS_PER_BYTE != 0 {
            self.0.push(Error::SubByteSize);
        }
        self.validate_alignment(layout.alignment_bits);
        self.validate_alignment(layout.required_alignment_bits);
        visit_opaque_type(self, layout, ty);
    }
}

impl PreValidator {
    fn validate_alignment(&mut self, a: u64) {
        if a < BITS_PER_BYTE {
            self.0.push(Error::SubByteAlignment);
        }
        if a.is_power_of_two().not() {
            self.0.push(Error::PowerOfTwoAlignment);
        }
    }
}

struct PostValidator(Vec<Error>);

impl Visitor<TypeLayout> for PostValidator {
    fn visit_record_field(
        &mut self,
        field: &RecordField<TypeLayout>,
        rt: &Record<TypeLayout>,
        ty: &Type<TypeLayout>,
    ) {
        if let Some(n) = field.bit_width {
            if n > field.ty.layout.size_bits {
                self.0.push(Error::OversizedBitfield);
            }
        }
        visit_record_field(self, field, rt, ty);
    }
}
