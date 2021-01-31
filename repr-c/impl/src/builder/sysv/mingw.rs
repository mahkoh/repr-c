use crate::builder::sysv::{sysv, Dialect};
use crate::layout::{
    Annotation, FieldLayout, Record, RecordField, RecordKind, Type, TypeLayout, TypeVariant,
};
use crate::result::{Error, Result};
use crate::target::Target;
use crate::util::{
    align_to, annotation_alignment, is_attr_packed, pragma_pack_value, size_add, MaxAssign,
    MinAssign, BITS_PER_BYTE,
};

pub(crate) fn compute_layout(target: Target, ty: &Type<()>) -> Result<Type<TypeLayout>> {
    super::compute_layout(target, ty, Dialect::Mingw)
}

pub(super) fn compute_record_layout(
    target: Target,
    ty: RecordKind,
    annotations: &[Annotation],
    u: &[RecordField<()>],
) -> Result<Type<TypeLayout>> {
    if is_attr_packed(annotations) {
        sysv::compute_record_layout(target, ty, annotations, u, Dialect::Mingw)
    } else {
        RecordLayoutBuilder::new(target, ty, annotations)?.compute(u)
    }
}

struct RecordLayoutBuilder<'a> {
    target: Target,
    annotations: &'a [Annotation],
    alignment_bits: u64,
    size_bits: u64,
    max_field_alignment_bits: Option<u64>,
    record_fields: Vec<RecordField<TypeLayout>>,
    kind: RecordKind,
    ongoing_bitfield: Option<OngoingBitfield>,
}

struct OngoingBitfield {
    // The size of the storage unit of the previous bitfield. This is the size of the underlying
    // type, e.g., `int`.
    ty_size_bits: u64,
    // The number of bits that remain unused in the storage unit. This can be 0 if all of the bits
    // have been used.
    unused_size_bits: u64,
}

impl<'a> RecordLayoutBuilder<'a> {
    fn new(target: Target, kind: RecordKind, annotations: &'a [Annotation]) -> Result<Self> {
        let pragma_pack_value = pragma_pack_value(annotations);
        let max_field_alignment_bits = match pragma_pack_value {
            Some(8) | Some(16) | Some(32) | Some(64) | Some(128) => pragma_pack_value,
            _ => None,
        };
        let alignment_bits = annotation_alignment(annotations).unwrap_or(BITS_PER_BYTE);
        Ok(Self {
            target,
            annotations,
            alignment_bits,
            size_bits: 0,
            max_field_alignment_bits,
            record_fields: vec![],
            kind,
            ongoing_bitfield: None,
        })
    }

    fn compute(mut self, fields: &[RecordField<()>]) -> Result<Type<TypeLayout>> {
        for f in fields {
            self.layout_field(f)?;
        }
        self.size_bits = align_to(self.size_bits, self.alignment_bits)?;
        Ok(Type {
            layout: TypeLayout {
                size_bits: self.size_bits,
                field_alignment_bits: self.alignment_bits,
                pointer_alignment_bits: self.alignment_bits,
                required_alignment_bits: BITS_PER_BYTE,
            },
            annotations: self.annotations.to_vec(),
            variant: TypeVariant::Record(Record {
                kind: self.kind,
                fields: self.record_fields,
            }),
        })
    }

    fn layout_field(&mut self, field: &RecordField<()>) -> Result<()> {
        let ty = compute_layout(self.target, &field.ty)?;
        // The field alignment is calculated using the usual sysv algorithm:
        // - Start with the alignment of the type
        // - Apply the packed attributes of the field
        // - Apply the alignment attributes of the field
        // - Apply the pragma pack attributes
        let field_alignment_bits = {
            let mut field_alignment_bits = ty.layout.field_alignment_bits;
            if is_attr_packed(&field.annotations) {
                field_alignment_bits = BITS_PER_BYTE;
            }
            field_alignment_bits.assign_max(annotation_alignment(&field.annotations));
            field_alignment_bits.assign_min(self.max_field_alignment_bits);
            field_alignment_bits
        };
        // The field affects the alignment of the record if it's not a zero-sized bit-field or if
        // the previous field was a non-zero-sized bit-field.

        if field.bit_width != Some(0) || self.ongoing_bitfield.is_some() {
            let mut ty_alignment_bits = ty.layout.field_alignment_bits;
            if is_attr_packed(&field.annotations) && field.bit_width.is_none() {
                ty_alignment_bits = BITS_PER_BYTE;
            }
            ty_alignment_bits.assign_max(annotation_alignment(&field.annotations));
            ty_alignment_bits.assign_min(self.max_field_alignment_bits);
            self.alignment_bits.assign_max(ty_alignment_bits);
        }
        let layout = match field.bit_width {
            Some(width) => self.layout_bit_field(
                ty.layout.size_bits,
                field_alignment_bits,
                field.named,
                width,
            ),
            None => self.layout_regular_field(ty.layout.size_bits, field_alignment_bits),
        }?;
        self.record_fields.push(RecordField {
            layout,
            annotations: field.annotations.clone(),
            named: field.named,
            bit_width: field.bit_width,
            ty,
        });
        Ok(())
    }

    fn layout_bit_field(
        &mut self,
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
            return Err(Error::OversizedBitfield);
        }
        if self.kind == RecordKind::Union {
            self.size_bits.assign_max(width);
            return ok!(0);
        }
        match width {
            0 => self.ongoing_bitfield = None,
            _ => {
                if let Some(ref mut p) = &mut self.ongoing_bitfield {
                    if p.ty_size_bits == ty_size_bits && p.unused_size_bits >= width {
                        let offset_bits = self.size_bits - p.unused_size_bits;
                        p.unused_size_bits -= width;
                        return ok!(offset_bits);
                    }
                }
                self.ongoing_bitfield = Some(OngoingBitfield {
                    ty_size_bits,
                    unused_size_bits: ty_size_bits - width,
                });
            }
        }
        let offset_bits = align_to(self.size_bits, field_alignment_bits)?;
        self.size_bits = match width {
            0 => offset_bits,
            _ => size_add(offset_bits, ty_size_bits)?,
        };
        ok!(offset_bits)
    }

    fn layout_regular_field(
        &mut self,
        ty_size_bits: u64,
        field_alignment_bits: u64,
    ) -> Result<Option<FieldLayout>> {
        self.ongoing_bitfield = None;
        let offset_bits = match self.kind {
            RecordKind::Struct => align_to(self.size_bits, field_alignment_bits)?,
            RecordKind::Union => 0,
        };
        self.size_bits
            .assign_max(size_add(offset_bits, ty_size_bits)?);
        Ok(Some(FieldLayout {
            offset_bits,
            size_bits: ty_size_bits,
        }))
    }
}
