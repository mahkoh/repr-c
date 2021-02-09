// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::builder::common::{
    apply_alignment_override, builtin_type_layout, compute_builtin_type_layout,
    compute_opaque_type_layout, pack_all_enums,
};
use crate::builder::sysv_like::mingw::OngoingBitfield;
use crate::layout::{
    Annotation, Array, BuiltinType, Record, RecordField, RecordKind, Type, TypeLayout, TypeVariant,
};
use crate::result::Result;
use crate::target::{system_compiler, Compiler, Target};
use crate::util::{
    align_to, annotation_alignment, is_attr_packed, pragma_pack_value, size_mul, BITS_PER_BYTE,
};

pub mod mingw;
pub mod sysv;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Dialect {
    Sysv,
    Mingw,
}

fn compute_layout(target: Target, ty: &Type<()>, dialect: Dialect) -> Result<Type<TypeLayout>> {
    match &ty.variant {
        TypeVariant::Builtin(bi) => compute_builtin_type_layout(target, *bi),
        TypeVariant::Opaque(layout) => compute_opaque_type_layout(*layout),
        TypeVariant::Record(r) => {
            compute_record_layout(dialect, target, r.kind, &ty.annotations, &r.fields)
        }
        TypeVariant::Enum(v) => compute_enum_layout(target, v, &ty.annotations),
        TypeVariant::Typedef(dst) => {
            // #pragma pack and __attribute__((packed)) are ignored on typedefs.
            // See test case 0088.
            let dst_ty = compute_layout(target, dst, dialect)?;
            let max_alignment = annotation_alignment(target, &ty.annotations);
            // __attribute__((aligned(N))) sets the field alignment to N even if N is smaller
            // than the alignment of the underlying type. See test case 0046.
            Ok(Type {
                layout: apply_alignment_override(dst_ty.layout, max_alignment),
                annotations: ty.annotations.clone(),
                variant: TypeVariant::Typedef(Box::new(dst_ty)),
            })
        }
        TypeVariant::Array(a) => {
            let ety = compute_layout(target, &a.element_type, dialect)?;
            Ok(Type {
                layout: TypeLayout {
                    // The size of an array is the size of the underlying type multiplied by the
                    // number of elements rounded up to the alignment. Since the element size might
                    // not be a multiple of the field alignment, the address of the second element
                    // might not be properly aligned for the field alignment. See test case 0045.
                    size_bits: align_to(
                        size_mul(ety.layout.size_bits, a.num_elements.unwrap_or(0))?,
                        ety.layout.field_alignment_bits,
                    )?,
                    // Since the size is now a multiple of the field alignment, we know
                    // that all pointers will be aligned to the field alignment.
                    pointer_alignment_bits: ety.layout.field_alignment_bits,
                    // The other alignments are inherited from the underlying type.
                    ..ety.layout
                },
                // Pre-validation ensures that arrays do not have annotations.
                annotations: vec![],
                variant: TypeVariant::Array(Array {
                    element_type: Box::new(ety),
                    num_elements: a.num_elements,
                }),
            })
        }
    }
}

struct RecordLayoutBuilder {
    target: Target,
    // The alignment of this record.
    alignment_bits: u64,
    // The size of the record. This might not be a multiple of 8 if the record contains bit-fields.
    // For structs, this is also the offset of the first bit after the last field.
    size_bits: u64,
    // Whether the record has an __attribute__((packed)) annotation.
    attr_packed: bool,
    // The value of #pragma pack(N) at the type level if any.
    max_field_alignment_bits: Option<u64>,
    // The fields in this record.
    record_fields: Vec<RecordField<TypeLayout>>,
    // The kind of this record. Struct or Union.
    kind: RecordKind,
    // `Some` if the previous field was a non-zero-sized bit-field. Only used by MinGW.
    ongoing_bitfield: Option<OngoingBitfield>,
}

fn compute_record_layout(
    dialect: Dialect,
    target: Target,
    kind: RecordKind,
    annotations: &[Annotation],
    fields: &[RecordField<()>],
) -> Result<Type<TypeLayout>> {
    let attr_packed = is_attr_packed(annotations);
    // Pre-validation ensures that there is at most one #pragma pack annotation.
    let pragma_pack_value = pragma_pack_value(annotations);
    // #pragma pack(N) is ignored if N is not one of {1,2,4,8,16}. See test case 0064.
    let max_field_alignment_bits = match pragma_pack_value {
        Some(8) | Some(16) | Some(32) | Some(64) | Some(128) => pragma_pack_value,
        _ => None,
    };
    // An alignment annotation on the record increases the overall alignment of the record.
    // See test case 0065.
    let alignment_bits = annotation_alignment(target, annotations).unwrap_or(BITS_PER_BYTE);
    let mut rlb = RecordLayoutBuilder {
        target,
        alignment_bits,
        size_bits: 0,
        attr_packed,
        max_field_alignment_bits,
        record_fields: vec![],
        kind,
        ongoing_bitfield: None,
    };
    match dialect {
        Dialect::Mingw => mingw::layout_fields(&mut rlb, fields)?,
        Dialect::Sysv => sysv::layout_fields(&mut rlb, fields)?,
    }
    // The size of a record is always a multiple of its alignment. See test case 0066.
    rlb.size_bits = align_to(rlb.size_bits, rlb.alignment_bits)?;
    Ok(Type {
        layout: TypeLayout {
            size_bits: rlb.size_bits,
            field_alignment_bits: rlb.alignment_bits,
            pointer_alignment_bits: rlb.alignment_bits,
            required_alignment_bits: BITS_PER_BYTE,
        },
        annotations: annotations.to_vec(),
        variant: TypeVariant::Record(Record {
            kind,
            fields: rlb.record_fields,
        }),
    })
}

fn compute_enum_layout(
    target: Target,
    v: &[i128],
    annotations: &[Annotation],
) -> Result<Type<TypeLayout>> {
    // #pragma pack is ignored on enums. See test case 0061.

    // A packed enum has minimum size 1 byte. On some targets, all enums have an implicit
    // packed attribute. Otherwise the minimum size is the size of `int`. See test case 0060.
    let mut required_size = match is_attr_packed(annotations) || pack_all_enums(target) {
        true => BITS_PER_BYTE,
        false => builtin_type_layout(target, BuiltinType::Int).size_bits,
    };
    // The size of the enum is the size of the smallest integer type whose size is at least
    // as large as the minimum size and which can represent all variants. See test case 0062.
    for &v in v {
        let (v, offset) = if v < 0 { (!v, 1) } else { (v, 0) };
        let required = 128 - v.leading_zeros() as u64 + offset;
        while required > required_size {
            required_size *= 2;
        }
    }
    let candidates = [
        BuiltinType::Char,
        BuiltinType::Short,
        BuiltinType::Int,
        BuiltinType::Long,
        BuiltinType::LongLong,
    ];
    let layout = candidates
        .iter()
        .map(|ty| builtin_type_layout(target, *ty))
        .find(|l| l.size_bits >= required_size)
        .unwrap_or_else(|| builtin_type_layout(target, BuiltinType::I128));
    // Clang respects __attribute__((aligned)) on enums. The behavior is the same
    // as the behavior on typedefs. See test case 0063.
    let max_alignment = match system_compiler(target) {
        Compiler::Clang => annotation_alignment(target, annotations),
        _ => None,
    };
    Ok(Type {
        layout: apply_alignment_override(layout, max_alignment),
        annotations: annotations.to_vec(),
        variant: TypeVariant::Enum(v.to_vec()),
    })
}
