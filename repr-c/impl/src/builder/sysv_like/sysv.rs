#![allow(clippy::if_same_then_else)]

use crate::builder::common::{
    ignore_non_zero_sized_bitfield_type_alignment, ignore_zero_sized_bitfield_type_alignmont,
    min_zero_width_bitfield_alignment, unnamed_field_affects_record_alignment,
};
use crate::builder::sysv_like::{Dialect, RecordLayoutBuilder};
use crate::layout::{FieldLayout, RecordField, RecordKind, Type, TypeLayout};
use crate::result::{err, ErrorKind, Result};
use crate::target::{system_compiler, Compiler, Target};
use crate::util::{
    align_to, annotation_alignment, is_attr_packed, size_add, MaxAssign, MinAssign, MinExt,
    BITS_PER_BYTE,
};

pub(crate) fn compute_layout(target: Target, ty: &Type<()>) -> Result<Type<TypeLayout>> {
    super::compute_layout(target, ty, Dialect::Sysv)
}

pub(super) fn layout_fields(
    rlb: &mut RecordLayoutBuilder,
    fields: &[RecordField<()>],
) -> Result<()> {
    for f in fields {
        layout_field(rlb, f)?;
    }
    Ok(())
}

fn layout_field(rlb: &mut RecordLayoutBuilder, field: &RecordField<()>) -> Result<()> {
    let ty = super::compute_layout(rlb.target, &field.ty, Dialect::Sysv)?;
    let layout = match field.bit_width {
        Some(size_bits) => layout_bit_field(
            rlb,
            ty.layout.size_bits,
            ty.layout.field_alignment_bits,
            field,
            size_bits,
        ),
        None => layout_regular_field(rlb, ty.layout, field),
    }?;
    rlb.record_fields.push(RecordField {
        layout,
        annotations: field.annotations.clone(),
        named: field.named,
        bit_width: field.bit_width,
        ty,
    });
    Ok(())
}

fn layout_bit_field(
    rlb: &mut RecordLayoutBuilder,
    ty_size_bits: u64,
    mut ty_field_alignment_bits: u64,
    field: &RecordField<()>,
    width: u64,
) -> Result<Option<FieldLayout>> {
    if width > 0 {
        if width > ty_size_bits {
            return Err(err(ErrorKind::OversizedBitfield));
        }
        // Some targets ignore the alignment of the underlying type when laying out
        // non-zero-sized bit-fields. See test case 0072. On such targets, bit-fields never
        // cross a storage boundary. See test case 0081.
        if ignore_non_zero_sized_bitfield_type_alignment(rlb.target) {
            ty_field_alignment_bits = 1;
        }
    } else {
        // Some targets ignore the alignment of the underlying type when laying out
        // zero-sized bit-fields. See test case 0073.
        if ignore_zero_sized_bitfield_type_alignmont(rlb.target) {
            ty_field_alignment_bits = 1;
        }
        // Some targets have a minimum alignment of zero-sized bit-fields. See test case
        // 0074.
        ty_field_alignment_bits.assign_max(min_zero_width_bitfield_alignment(rlb.target));
    }
    // __attribute__((packed)) on the record is identical to __attribute__((packed)) on each
    // field. See test case 0067.
    let attr_packed = rlb.attr_packed || is_attr_packed(&field.annotations);
    let annotation_alignment = annotation_alignment(rlb.target, &field.annotations).unwrap_or(1);
    let first_unused_bit = match rlb.kind {
        RecordKind::Union => 0,
        RecordKind::Struct => rlb.size_bits,
    };
    let mut field_alignment_bits;
    if rlb.compiler == Compiler::Gcc {
        field_alignment_bits = annotation_alignment;
        if width == 0 {
            field_alignment_bits.assign_max(ty_field_alignment_bits);
        } else {
            field_alignment_bits.assign_min(rlb.max_field_alignment_bits);
            if !attr_packed && rlb.max_field_alignment_bits.is_none() {
                if ty_field_alignment_bits > ty_size_bits {
                    field_alignment_bits.assign_max(ty_field_alignment_bits);
                } else {
                    let start_bit = align_to(first_unused_bit, field_alignment_bits)?;
                    if start_bit % ty_field_alignment_bits + width > ty_size_bits {
                        field_alignment_bits.assign_max(ty_field_alignment_bits);
                    }
                }
            }
        }
    } else {
        let field_crosses_storage_boundary = first_unused_bit % ty_field_alignment_bits + width > ty_size_bits;
    }
    let field_alignment_bits;
    if width == 0 {
        // A zero-sized bit-field aligns the starting position for subsequent fields
        // to the field alignment of the type unless it was explicitly increased.
        // __attribute__((packed)) and #pragma pack are ignored. See test case 0082.
        field_alignment_bits = ty_field_alignment_bits.max(annotation_alignment);
    } else if rlb.max_field_alignment_bits.is_none()
        && !attr_packed
        && field_crosses_storage_boundary
    {
        // Otherwise, if there are no #pragma pack and __attribute__((packed)) in play and
        // the field would otherwise cross a storage boundary, the field starts at the field
        // alignment of the type unless it was explicitly increased. See test case 0083.
        field_alignment_bits = ty_field_alignment_bits.max(annotation_alignment);
    } else if rlb.max_field_alignment_bits.is_none() {
        // Otherwise, if there is no #pragma pack in play, the field starts at the explicitly
        // requested alignment.
        field_alignment_bits = annotation_alignment;
    } else if annotation_alignment <= rlb.max_field_alignment_bits.unwrap() {
        // Otherwise, if the field has an annotation alignment
        field_alignment_bits = annotation_alignment.min2(rlb.max_field_alignment_bits);
    } else {
        field_alignment_bits = 1;
    }
    let offset_bits = align_to(first_unused_bit, field_alignment_bits)?;
    rlb.size_bits.assign_max(size_add(offset_bits, width)?);
    // Unnamed fields do not contribute to the record alignment except on a few targets.
    // See test case 0079.
    if field.named || unnamed_field_affects_record_alignment(rlb.target) {
        let inherited_alignment_bits;
        if width == 0 {
            // If the width is 0, #pragma pack and __attribute__((packed)) are ignored.
            // See test case 0075.
            inherited_alignment_bits = ty_field_alignment_bits.max(annotation_alignment);
        } else if let Some(max_field_alignment_bits) = rlb.max_field_alignment_bits {
            // Otherwise, if a #pragma pack is in effect, __attribute__((packed)) on the field or
            // record is ignored. See test case 0076.
            inherited_alignment_bits = ty_field_alignment_bits
                .max(annotation_alignment)
                .min(max_field_alignment_bits);
        } else if attr_packed {
            // Otherwise, if the field or the record is packed, the field alignment is 1 bit unless
            // it is explicitly increased with __attribute__((aligned)). See test case 0077.
            inherited_alignment_bits = annotation_alignment;
        } else {
            // Otherwise, the field alignment is the field alignment of the underlying type unless
            // it is explicitly increased with __attribute__((aligned)). See test case 0078.
            inherited_alignment_bits = ty_field_alignment_bits.max(annotation_alignment);
        }
        rlb.alignment_bits.assign_max(inherited_alignment_bits);
    }
    match field.named {
        true => Ok(Some(FieldLayout {
            offset_bits,
            size_bits: width,
        })),
        false => Ok(None),
    }
}

fn layout_regular_field(
    rlb: &mut RecordLayoutBuilder,
    type_layout: TypeLayout,
    field: &RecordField<()>,
) -> Result<Option<FieldLayout>> {
    // The alignment of a field is based on the field alignment of the underlying type.
    //
    // ```c,gcc
    // struct X {
    //         char c;
    //         int i;
    // };
    //
    // static void f(void) {
    //         _Static_assert(__builtin_offsetof(struct X, i) == 4, "");
    // }
    // ```
    let mut field_alignment_bits = type_layout.field_alignment_bits;
    // If the struct or the field is packed, then the alignment of the underlying type is
    // ignored.
    //
    // ```c,gcc
    // struct __attribute__((packed)) X {
    //         char c;
    //         int i;
    // };
    //
    // struct Y {
    //         char c;
    //         __attribute__((packed)) int i;
    // };
    //
    // static void f(void) {
    //         _Static_assert(__builtin_offsetof(struct X, i) == 1, "");
    //         _Static_assert(__builtin_offsetof(struct Y, i) == 1, "");
    // }
    // ```
    if rlb.attr_packed || is_attr_packed(&field.annotations) {
        field_alignment_bits = BITS_PER_BYTE;
    }
    // The field alignment can be increased by __attribute__((aligned)) annotations on the
    // field.
    //
    // ```c,gcc
    // struct X {
    //         char c;
    //         __attribute__((aligned(8))) int i;
    // };
    //
    // struct Y {
    //         char c;
    //         __attribute__((packed, aligned(8))) int i;
    // };
    //
    // static void f(void) {
    //         _Static_assert(__builtin_offsetof(struct X, i) == 8, "");
    //         _Static_assert(__builtin_offsetof(struct Y, i) == 8, "");
    // }
    // ```
    field_alignment_bits.assign_max(annotation_alignment(rlb.target, &field.annotations));
    // #pragma pack takes precedence over all other attributes.
    //
    // ```c,gcc
    // #pragma pack(2)
    // struct X {
    //         char c;
    //         __attribute__((aligned(8))) int i;
    // };
    //
    // static void f(void) {
    //         _Static_assert(__builtin_offsetof(struct X, i) == 2, "");
    // }
    // ```
    field_alignment_bits.assign_min(rlb.max_field_alignment_bits);
    let offset_bits = match rlb.kind {
        // A struct field starts at the next offset in the struct that is properly
        // aligned with respect to the start of the struct.
        RecordKind::Struct => align_to(rlb.size_bits, field_alignment_bits)?,
        // A union field always starts at offset 0.
        RecordKind::Union => 0,
    };
    let size_bits = type_layout.size_bits;
    // Set the size of the record to the maximum of the current size and the end of
    // the field.
    //
    // ```c,gcc,tc-0034
    // union U {
    //         int l;
    //         char c;
    // };
    //
    // static void f(void) {
    //         static_assert(sizeof(union U) == 4, "");
    // }
    // ```
    rlb.size_bits.assign_max(size_add(offset_bits, size_bits)?);
    // The alignment of a record is the maximum of its field alignments.
    //
    // ```c,gcc,tc-0032
    // struct A {
    //         long a;
    //         char c;
    // };
    //
    // static void f(void) {
    //         _Static_assert(_Alignof(struct A) == 4, "");
    //         _Static_assert(sizeof(struct A) == 8, "");
    // }
    // ```
    rlb.alignment_bits.assign_max(field_alignment_bits);
    Ok(Some(FieldLayout {
        offset_bits,
        size_bits,
    }))
}
