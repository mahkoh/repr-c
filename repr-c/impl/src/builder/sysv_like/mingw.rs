// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::builder::sysv_like::{Dialect, RecordLayoutBuilder};
use crate::layout::{FieldLayout, RecordField, RecordKind, Type, TypeLayout};
use crate::result::{err, ErrorType, Result};
use crate::target::Target;
use crate::util::{
    align_to, annotation_alignment, is_attr_packed, size_add, MaxAssign, MinAssign, BITS_PER_BYTE,
};

pub(crate) fn compute_layout(target: Target, ty: &Type<()>) -> Result<Type<TypeLayout>> {
    super::compute_layout(target, ty, Dialect::Mingw)
}

pub(super) struct OngoingBitfield {
    // The size of the storage unit of the previous bitfield. This is the size of the underlying
    // type, e.g., `int`.
    ty_size_bits: u64,
    // The number of bits that remain unused in the storage unit. This can be 0 if all of the bits
    // have been used.
    unused_size_bits: u64,
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
    let ty = compute_layout(rlb.target, &field.ty)?;
    let annotation_alignment_bits =
        annotation_alignment(rlb.target, &field.annotations).unwrap_or(BITS_PER_BYTE);
    // __attribute__((packed)) on the record is identical to __attribute__((packed)) on each
    // field. See test case 0067.
    let is_attr_packed = rlb.attr_packed || is_attr_packed(&field.annotations);
    // The alignment of the field is calculated in the usual way except that the alignment of
    // the underlying type is ignored in three cases
    // - the field is packed
    // - the field is a bit-field and the previous field was a non-zero-sized bit-field with the same type size
    // - the field is a zero-sized bit-field and the previous field was not a non-zero-sized bit-field
    // See test case 0068.
    let ignore_type_alignment = match (is_attr_packed, field.bit_width, &rlb.ongoing_bitfield) {
        (true, _, _) => true,
        (_, Some(_), Some(o)) if o.ty_size_bits == ty.layout.size_bits => true,
        (_, Some(0), None) => true,
        _ => false,
    };
    let mut field_alignment_bits = ty.layout.field_alignment_bits;
    if ignore_type_alignment {
        field_alignment_bits = BITS_PER_BYTE;
    }
    field_alignment_bits.assign_max(annotation_alignment_bits);
    field_alignment_bits.assign_min(rlb.max_field_alignment_bits);
    // The field affects the record alignment in one of three cases
    // - the field is a regular field
    // - the field is a zero-width bit-field following a non-zero-width bit-field
    // - the field is a non-zero-width bit-field and not packed.
    // See test case 0069.
    let update_record_alignment = field.bit_width == None
        || field.bit_width == Some(0) && rlb.ongoing_bitfield.is_some()
        || field.bit_width != Some(0) && !is_attr_packed;
    // If a field affects the alignment of a record, the alignment is calculated in the
    // usual way except that __attribute__((packed)) is ignored on a zero-width bit-field.
    // See test case 0068.
    if update_record_alignment {
        let mut ty_alignment_bits = ty.layout.field_alignment_bits;
        if is_attr_packed && field.bit_width != Some(0) {
            ty_alignment_bits = BITS_PER_BYTE;
        }
        ty_alignment_bits.assign_max(annotation_alignment_bits);
        ty_alignment_bits.assign_min(rlb.max_field_alignment_bits);
        rlb.alignment_bits.assign_max(ty_alignment_bits);
    }
    // NOTE: ty_alignment_bits and field_alignment_bits are different in the following case:
    // Y = { size: 64, alignment: 64 }struct {
    //     { offset: 0, size: 1 }c { size: 8, alignment: 8 }char:1,
    //     @attr_packed _ { size: 64, alignment: 64 }long long:0,
    //     { offset: 8, size: 8 }d { size: 8, alignment: 8 }char,
    // }

    // These functions return `None` if and only if the field is unnamed.
    let layout = match field.bit_width {
        Some(width) => layout_bit_field(
            rlb,
            ty.layout.size_bits,
            field_alignment_bits,
            field.named,
            width,
        ),
        None => layout_regular_field(rlb, ty.layout.size_bits, field_alignment_bits),
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
    field_alignment_bits: u64,
    named: bool,
    width: u64,
) -> Result<Option<FieldLayout>> {
    macro_rules! ok {
        ($offset:expr) => {
            Ok(match named {
                true => Some(FieldLayout {
                    offset_bits: $offset,
                    size_bits: width,
                }),
                false => None,
            })
        };
    }
    if width > ty_size_bits {
        return Err(err(ErrorType::OversizedBitfield));
    }
    // In a union, the size of the underlying type does not affect the size of the union.
    // See test case 0070.
    if rlb.kind == RecordKind::Union {
        rlb.size_bits.assign_max(width);
        return ok!(0);
    }
    match width {
        0 => rlb.ongoing_bitfield = None,
        _ => {
            // If there is an ongoing bit-field in a struct whose underlying type has the same size and
            // if there is enough space left to place this bit-field, then this bit-field is placed in
            // the ongoing bit-field and the size of the struct is not affected by this
            // bit-field. See test case 0037.
            if let Some(ref mut p) = &mut rlb.ongoing_bitfield {
                if p.ty_size_bits == ty_size_bits && p.unused_size_bits >= width {
                    let offset_bits = rlb.size_bits - p.unused_size_bits;
                    p.unused_size_bits -= width;
                    return ok!(offset_bits);
                }
            }
            // Otherwise this field is part of a new ongoing bit-field.
            rlb.ongoing_bitfield = Some(OngoingBitfield {
                ty_size_bits,
                unused_size_bits: ty_size_bits - width,
            });
        }
    }
    let offset_bits = align_to(rlb.size_bits, field_alignment_bits)?;
    rlb.size_bits = match width {
        // A zero-width bitfield only increases the size of the struct to the
        // offset a non-zero-width bitfield with the same alignment would
        // start. See test case 0039.
        0 => offset_bits,
        // A non-zero-width bitfield always increases the size by the full
        // size of the underlying type. Even if we are in a packed context.
        // See test cases 0040 and 0071.
        _ => size_add(offset_bits, ty_size_bits)?,
    };
    ok!(offset_bits)
}

fn layout_regular_field(
    rlb: &mut RecordLayoutBuilder,
    ty_size_bits: u64,
    field_alignment_bits: u64,
) -> Result<Option<FieldLayout>> {
    rlb.ongoing_bitfield = None;
    let offset_bits = match rlb.kind {
        // A struct field starts at the next offset in the struct that is properly
        // aligned with respect to the start of the struct. See test case 0033.
        RecordKind::Struct => align_to(rlb.size_bits, field_alignment_bits)?,
        // A union field always starts at offset 0.
        RecordKind::Union => 0,
    };
    // Set the size of the record to the maximum of the current size and the end of
    // the field. See test case 0034.
    rlb.size_bits
        .assign_max(size_add(offset_bits, ty_size_bits)?);
    Ok(Some(FieldLayout {
        offset_bits,
        size_bits: ty_size_bits,
    }))
}
