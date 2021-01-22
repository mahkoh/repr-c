use crate::builder::common::{compute_builtin_type_layout, compute_opaque_type_layout};
use crate::layout::{
    Annotation, Array, BuiltinType, FieldLayout, Record, RecordField, RecordKind, Type, TypeLayout,
    TypeVariant,
};
use crate::result::{Error, Result};
use crate::target::Target;
use crate::util::{
    align_to, is_packed, max_alignment, size_add, size_mul, MaxAssign, MinAssign, MinExt,
    BITS_PER_BYTE,
};

pub fn compute_layout(target: &dyn Target, ty: &Type<()>) -> Result<Type<TypeLayout>> {
    match &ty.variant {
        TypeVariant::Builtin(bi) => compute_builtin_type_layout(target, *bi),
        TypeVariant::Opaque(layout) => compute_opaque_type_layout(*layout),
        TypeVariant::Record(r) => compute_record_layout(target, r.kind, &ty.annotations, &r.fields),
        TypeVariant::Typedef(dst) => {
            // Pre-validation ensures that typedefs do not have packing annotations.
            let dst_ty = compute_layout(target, dst)?;
            let max_alignment = max_alignment(&ty.annotations);
            // __declspec(align) increases both the required and the regular alignment but
            // never decreases it. It does not affect the size so that the size of the
            // type can be smaller than its alignment.
            //
            // ```c,msvc,msvc-tc-001
            // __declspec(align(2)) typedef int A;
            // __declspec(align(8)) typedef int B;
            //
            // #pragma pack(1)
            //
            // struct X {
            //         A a;
            // };
            //
            // struct Y {
            //         B b;
            // };
            //
            // static void f(void) {
            //         static_assert(sizeof(A) == 4, "");
            //         static_assert(_Alignof(A) == 4, "");
            //
            //         static_assert(sizeof(struct X) == 4, "");
            //         static_assert(_Alignof(struct X) == 2, "");
            //
            //         static_assert(sizeof(B) == 4, "");
            //         static_assert(_Alignof(B) == 8, "");
            //
            //         static_assert(sizeof(struct Y) == 8, "");
            //         static_assert(_Alignof(struct Y) == 8, "");
            // }
            // ```
            Ok(Type {
                layout: TypeLayout {
                    size_bits: dst_ty.layout.size_bits,
                    alignment_bits: dst_ty.layout.alignment_bits.max(max_alignment),
                    required_alignment_bits: dst_ty
                        .layout
                        .required_alignment_bits
                        .max(max_alignment),
                },
                annotations: ty.annotations.clone(),
                variant: TypeVariant::Typedef(Box::new(dst_ty)),
            })
        }
        TypeVariant::Array(a) => {
            let ety = compute_layout(target, &a.element_type)?;
            Ok(Type {
                layout: TypeLayout {
                    size_bits: size_mul(ety.layout.size_bits, a.num_elements)?,
                    alignment_bits: ety.layout.alignment_bits,
                    required_alignment_bits: ety.layout.required_alignment_bits,
                },
                // Pre-validation ensures that arrays do not have annotations.
                annotations: vec![],
                variant: TypeVariant::Array(Array {
                    element_type: Box::new(ety),
                    num_elements: a.num_elements,
                }),
            })
        }
        TypeVariant::Enum(v) => {
            let mut layout = target.builtin_type_layout(BuiltinType::Int);
            let alignment = max_alignment(&ty.annotations);
            layout.required_alignment_bits.assign_max(alignment);
            layout.alignment_bits.assign_max(alignment);
            Ok(Type {
                layout,
                annotations: ty.annotations.clone(),
                variant: TypeVariant::Enum(v.clone()),
            })
        }
    }
}

fn compute_record_layout(
    target: &dyn Target,
    ty: RecordKind,
    annotations: &[Annotation],
    u: &[RecordField<()>],
) -> Result<Type<TypeLayout>> {
    RecordLayoutBuilder::new(target, ty, annotations)?.compute(u)
}

struct RecordLayoutBuilder<'a> {
    target: &'a dyn Target,
    annotations: &'a [Annotation],
    alignment_bits: u64,
    required_alignment_bits: u64,
    rounding_alignment_bits: u64,
    size_bits: u64,
    max_field_alignment_bits: Option<u64>,
    struct_fields: Vec<RecordField<TypeLayout>>,
    kind: RecordKind,
    ongoing_bitfield: Option<PendingBitfield>,
}

struct PendingBitfield {
    ty_size_bits: u64,
    unused_size_bits: u64,
}

impl<'a> RecordLayoutBuilder<'a> {
    fn new(
        target: &'a dyn Target,
        kind: RecordKind,
        annotations: &'a [Annotation],
    ) -> Result<Self> {
        let max_field_alignment_bits = annotations
            .iter()
            .flat_map(|a| {
                match a {
                    Annotation::PragmaPack(n) => Some(*n),
                    // __attribute__((packed)) behaves like #pragma pack(1)
                    Annotation::AttrPacked => Some(1),
                    _ => None,
                }
                .into_iter()
            })
            .filter(|&n| {
                // #pragma pack(N) with N > sizeof(ptr) is ignored
                n <= target.builtin_type_layout(BuiltinType::Pointer).size_bits
            })
            .min();
        let required_alignment_bits = max_alignment(annotations).max(BITS_PER_BYTE);
        Ok(Self {
            target,
            annotations,
            rounding_alignment_bits: required_alignment_bits,
            alignment_bits: required_alignment_bits,
            size_bits: 0,
            required_alignment_bits,
            max_field_alignment_bits,
            struct_fields: vec![],
            kind,
            ongoing_bitfield: None,
        })
    }

    fn compute(mut self, fields: &[RecordField<()>]) -> Result<Type<TypeLayout>> {
        for f in fields {
            self.layout_field(f)?;
        }
        // Round up the struct size to the alignment.
        // TODO
        self.size_bits = align_to(self.size_bits, self.rounding_alignment_bits)?;
        // self.size_bits = align_to(self.size_bits, self.alignment_bits)?;
        if self.size_bits == 0 {
            // A struct has size at least 4 bytes.
            const MIN_STRUCT_SIZE_BITS: u64 = 4 * BITS_PER_BYTE;
            self.size_bits = MIN_STRUCT_SIZE_BITS;
            // If the required (!) alignment is at least 4 bytes, round up to the next
            // multiple of the struct alignment.
            //
            // ```c
            // struct X {
            //         long long b[0];
            // };
            //
            // struct Y {
            //         char __attribute__((aligned(4))) a[0];
            //         long long b[0];
            // };
            //
            // static void f(void) {
            //         _Static_assert(sizeof(struct X)   == 4, "");
            //         _Static_assert(_Alignof(struct X) == 8, "");
            //
            //         _Static_assert(sizeof(struct Y)   == 8, "");
            //         _Static_assert(_Alignof(struct Y) == 8, "");
            // }
            // ```
            if self.required_alignment_bits >= self.size_bits {
                self.size_bits = self.alignment_bits;
            }
        }
        Ok(Type {
            layout: TypeLayout {
                size_bits: self.size_bits,
                alignment_bits: self.alignment_bits,
                required_alignment_bits: self.required_alignment_bits,
            },
            annotations: self.annotations.to_vec(),
            variant: TypeVariant::Record(Record {
                kind: self.kind,
                fields: self.struct_fields,
            }),
        })
    }

    fn layout_field(&mut self, field: &RecordField<()>) -> Result<()> {
        // Compute layout of the underlying type
        let field_ty = compute_layout(self.target, &field.ty)?;
        let (ty_size_bits, alignment_bits) = {
            let layout = field_ty.layout;
            let mut alignment_bits = layout.alignment_bits;
            let required_alignment_bits =
                max_alignment(&field.annotations).max(layout.required_alignment_bits);
            // Update the overall required alignment of this struct. If this is a
            // bit-field, then the required alignment of the field does not contribute
            // to the overall required alignment.
            if field.bit_width.is_none() {
                self.required_alignment_bits
                    .assign_max(required_alignment_bits);
            }
            // If the field or struct is packed, reduce the alignment of the field ...
            if is_packed(&field.annotations) {
                alignment_bits = BITS_PER_BYTE;
            } else {
                alignment_bits.assign_min(self.max_field_alignment_bits);
            }
            // ... but the required alignment still takes precedence.
            alignment_bits.assign_max(required_alignment_bits);
            (layout.size_bits, alignment_bits)
        };
        let layout = match field.bit_width {
            Some(n) => self.layout_bit_field(ty_size_bits, alignment_bits, n),
            None => self.layout_regular_field(ty_size_bits, alignment_bits),
        }?;
        self.struct_fields.push(RecordField {
            layout,
            annotations: field.annotations.clone(),
            name: field.name.clone(),
            bit_width: field.bit_width,
            ty: field_ty,
        });
        Ok(())
    }

    fn layout_regular_field(&mut self, size_bits: u64, alignment_bits: u64) -> Result<FieldLayout> {
        self.ongoing_bitfield = None;
        self.rounding_alignment_bits.assign_max(alignment_bits);
        self.alignment_bits.assign_max(alignment_bits);
        let offset_bits = match self.kind {
            RecordKind::Struct => align_to(self.size_bits, alignment_bits)?,
            RecordKind::Union => 0,
        };
        self.size_bits.assign_max(offset_bits + size_bits);
        Ok(FieldLayout {
            offset_bits,
            size_bits,
        })
    }

    fn layout_bit_field(
        &mut self,
        ty_size_bits: u64,
        field_alignment_bits: u64,
        width: u64,
    ) -> Result<FieldLayout> {
        if width == 0 {
            // A zero-sized bit-field that does not follow a non-zero-sized bit-field does not affect
            // the overall layout of the record. Even in a union where the order would otherwise
            // not matter.
            //
            // ```c,msvc
            // union X {
            //         int :0;
            //         char :1;
            // };
            //
            // union Y {
            //         char :1;
            //         int :0;
            // };
            //
            // static void f(void) {
            //         static_assert(sizeof(union X) == 1, "");
            //         static_assert(sizeof(union Y) == 4, "");
            // }
            // ```
            if self.ongoing_bitfield.is_none() {
                return Ok(FieldLayout {
                    size_bits: 0,
                    offset_bits: match self.kind {
                        RecordKind::Struct => self.size_bits,
                        RecordKind::Union => 0,
                    },
                });
            }
            self.ongoing_bitfield = None;
        } else {
            // Even _Bool allows bitfields up to its type size.
            if width > ty_size_bits {
                return Err(Error::OversizedBitfield);
            }
            // If there is an ongoing bit-field in a struct whose underlying type has the same size and
            // if there is enough space left to place this bit-field, then this bit-field is placed in
            // the ongoing bit-field and the overall layout of the struct is not affected by this
            // bit-field.
            if let (RecordKind::Struct, Some(ref mut p)) = (self.kind, &mut self.ongoing_bitfield) {
                if p.ty_size_bits == ty_size_bits && p.unused_size_bits >= width {
                    let offset_bits = self.size_bits - p.unused_size_bits;
                    p.unused_size_bits -= width;
                    return Ok(FieldLayout {
                        offset_bits,
                        size_bits: width,
                    });
                }
            }
            self.ongoing_bitfield = Some(PendingBitfield {
                ty_size_bits,
                unused_size_bits: ty_size_bits - width,
            });
        }
        let offset_bits = match self.kind {
            RecordKind::Struct => {
                // TODO
                self.rounding_alignment_bits
                    .assign_max(field_alignment_bits.min2(self.max_field_alignment_bits));
                self.alignment_bits.assign_max(field_alignment_bits);
                let offset_bits = align_to(self.size_bits, field_alignment_bits)?;
                self.size_bits = match width {
                    0 => offset_bits,
                    _ => size_add(offset_bits, ty_size_bits)?,
                };
                offset_bits
            }
            RecordKind::Union => {
                // Bit-fields do not affect the alignment of a union.
                self.size_bits.assign_max(ty_size_bits);
                0
            }
        };
        Ok(FieldLayout {
            offset_bits,
            size_bits: width,
        })
    }
}

#[cfg(test)]
mod test {}
