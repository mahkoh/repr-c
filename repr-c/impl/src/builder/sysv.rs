use crate::builder::common::{apply_alignment_override, builtin_type_layout, compute_builtin_type_layout, compute_opaque_type_layout, resolve_typedefs, unnamed_field_affects_record_alignment, bitfield_type_alignment_matters};
use crate::layout::{
    Annotation, Array, BuiltinType, FieldLayout, Record, RecordField, RecordKind, Type, TypeLayout,
    TypeVariant,
};
use crate::result::{Error, Result};
use crate::target::{system_compiler, Compiler, Target};
use crate::util::{
    align_to, annotation_alignment, is_attr_packed, size_add, size_mul, MaxAssign, MinAssign,
    MinExt, BITS_PER_BYTE,
};

pub fn compute_layout(target: Target, ty: &Type<()>) -> Result<Type<TypeLayout>> {
    match &ty.variant {
        TypeVariant::Builtin(bi) => compute_builtin_type_layout(target, *bi),
        TypeVariant::Opaque(layout) => compute_opaque_type_layout(*layout),
        TypeVariant::Record(r) => compute_record_layout(target, r.kind, &ty.annotations, &r.fields),
        TypeVariant::Enum(v) => compute_enum_layout(target, v, &ty.annotations),
        TypeVariant::Typedef(dst) => {
            // Pre-validation ensures that typedefs do not have packing annotations.
            let dst_ty = compute_layout(target, dst)?;
            let max_alignment = annotation_alignment(&ty.annotations);
            // __attribute__((aligned(N))) sets the field alignment to N even if N is smaller
            // than the alignment of the underlying type.
            //
            // ```c,gcc,sysv-tc-0046
            // typedef int Int __attribute__((aligned(1)));
            //
            // static void f(void) {
            //         _Static_assert(sizeof(Int) == 4, "");
            //         _Static_assert(_Alignof(Int) == 1, "");
            // }
            // ```
            Ok(Type {
                layout: apply_alignment_override(dst_ty.layout, max_alignment),
                annotations: ty.annotations.clone(),
                variant: TypeVariant::Typedef(Box::new(dst_ty)),
            })
        }
        TypeVariant::Array(a) => {
            let ety = compute_layout(target, &a.element_type)?;
            Ok(Type {
                layout: TypeLayout {
                    // The size of an array is the size of the underlying type multiplied by the
                    // number of elements rounded up to the alignment. Since the element size might
                    // not be a multiple of the field alignment, the address of the second element
                    // might not be properly aligned for the field alignment.
                    //
                    // ```c,gcc,sysv-tc-0045
                    // typedef char Char[3] __attribute__((aligned(2)));
                    //
                    // typedef Char X[3];
                    //
                    // struct Y {
                    //         X x;
                    // };
                    //
                    // static void f(void) {
                    //         _Static_assert(sizeof(X) == 10, "");
                    //         _Static_assert(_Alignof(X) == 2, "");
                    //         _Static_assert(__builtin_offsetof(struct Y, x[1]) == 3, "");
                    // }
                    // ```
                    size_bits: align_to(
                        size_mul(ety.layout.size_bits, a.num_elements)?,
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

fn compute_enum_layout(
    target: Target,
    v: &[i128],
    annotations: &[Annotation],
) -> Result<Type<TypeLayout>> {
    // A packed enum has minimum size 1 byte. An unpacked enum is as least as large as `int`. Given
    // this minimum size, the size of the enum is the size of the smallest integer type that fits
    // all enum constants.
    //
    // ```c,gcc
    // enum __attribute__((packed)) E {
    //         E = 1,
    // };
    //
    // enum __attribute__((packed)) F {
    //         F = 1111,
    // };
    //
    // enum G {
    //         G = 1,
    // };
    //
    // enum H {
    //         H = 11111111111111111,
    // };
    //
    // static void f(void) {
    //         _Static_assert(sizeof(enum E) == 1, "");
    //         _Static_assert(sizeof(enum F) == 2, "");
    //         _Static_assert(sizeof(enum G) == 4, "");
    //         _Static_assert(sizeof(enum H) == 8, "");
    // }
    // ```
    let mut required_size = match is_attr_packed(annotations) {
        true => BITS_PER_BYTE,
        false => builtin_type_layout(target, BuiltinType::Int).size_bits,
    };
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
    for &candidate in candidates.iter() {
        let layout = builtin_type_layout(target, candidate);
        if layout.size_bits >= required_size {
            // Clang respects __attribute__((aligned)) on enums. The behavior is the same
            // as the behavior on typedefs.
            //
            // ```c,clang
            // enum __attribute__((aligned(8))) E {
            //         E = 1,
            // };
            //
            // enum __attribute__((aligned(1))) F {
            //         F = 1,
            // };
            //
            // static void f(void) {
            //         _Static_assert(sizeof(enum E) == 4, "");
            //         _Static_assert(_Alignof(enum E) == 8, "");
            //
            //         _Static_assert(sizeof(enum F) == 4, "");
            //         _Static_assert(_Alignof(enum F) == 1, "");
            // }
            // ```
            let max_alignment = match system_compiler(target) {
                Compiler::Clang => annotation_alignment(annotations),
                _ => None,
            };
            return Ok(Type {
                layout: apply_alignment_override(layout, max_alignment),
                annotations: annotations.to_vec(),
                variant: TypeVariant::Enum(v.to_vec()),
            });
        }
    }
    Err(Error::EnumOverflow)
}

fn compute_record_layout(
    target: Target,
    ty: RecordKind,
    annotations: &[Annotation],
    u: &[RecordField<()>],
) -> Result<Type<TypeLayout>> {
    RecordLayoutBuilder::new(target, ty, annotations)?.compute(u)
}

struct RecordLayoutBuilder<'a> {
    target: Target,
    // The annotations of this type.
    annotations: &'a [Annotation],
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
}

impl<'a> RecordLayoutBuilder<'a> {
    fn new(target: Target, kind: RecordKind, annotations: &'a [Annotation]) -> Result<Self> {
        // Pre-validation ensures that there is at most one #pragma pack annotation.
        let pragma_pack_value = annotations
            .iter()
            .flat_map(|a| match a {
                Annotation::PragmaPack(n) => Some(*n).into_iter(),
                _ => None.into_iter(),
            })
            .next();
        // #pragma pack(N) is ignored if N is not one of {1,2,4,8,16}.
        //
        // ```c,gcc
        // #pragma pack(32)
        // struct X {
        //         __attribute__((aligned(128))) int x;
        // };
        //
        // #pragma pack(16)
        // struct Y {
        //         __attribute__((aligned(128))) int x;
        // };
        //
        // static void f(void) {
        //         _Static_assert(_Alignof(struct X) == 128, "");
        //         _Static_assert(_Alignof(struct Y) == 16, "");
        // }
        // ```
        let max_field_alignment_bits = match pragma_pack_value {
            Some(8) | Some(16) | Some(32) | Some(64) | Some(128) => pragma_pack_value,
            _ => None,
        };
        let packed = is_attr_packed(annotations);
        // An alignment annotation on the record increases the overall alignment of the record.
        //
        // ```c,gcc
        // struct __attribute__((aligned(128))) X {
        //         int x;
        // };
        //
        // static void f(void) {
        //         _Static_assert(_Alignof(struct X) == 128, "");
        // }
        // ```
        let alignment_bits = annotation_alignment(annotations).unwrap_or(BITS_PER_BYTE);
        Ok(Self {
            target,
            annotations,
            alignment_bits,
            size_bits: 0,
            attr_packed: packed,
            max_field_alignment_bits,
            record_fields: vec![],
            kind,
        })
    }

    fn compute(mut self, fields: &[RecordField<()>]) -> Result<Type<TypeLayout>> {
        for f in fields {
            self.layout_field(f)?;
        }
        // The size is always a multiple of the alignment.
        //
        // ```c,gcc
        // struct __attribute__((aligned(128))) X {
        //         int x;
        // };
        //
        // static void f(void) {
        //         _Static_assert(sizeof(struct X) == 128, "");
        // }
        // ```
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
        let layout = match field.bit_width {
            Some(size_bits) => self.layout_bit_field(
                ty.layout.size_bits,
                ty.layout.field_alignment_bits,
                field,
                size_bits,
            ),
            None => self.layout_regular_field(ty.layout, field),
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
        mut ty_field_alignment_bits: u64,
        field: &RecordField<()>,
        size_bits: u64,
    ) -> Result<Option<FieldLayout>> {
        // Bit-fields cannot have a width larger than the size of the underlying type.
        if size_bits > ty_size_bits {
            return Err(Error::OversizedBitfield);
        }
        // Bit-fields of type _Bool must have width 0 or 1.
        if size_bits > 1
            && resolve_typedefs(&field.ty).variant == TypeVariant::Builtin(BuiltinType::Bool)
        {
            return Err(Error::OversizedBitfield);
        }
        if !bitfield_type_alignment_matters(self.target) {
            ty_field_alignment_bits = BITS_PER_BYTE;
        }
        // In the following, `annotation_alignment == 0` means that there was no
        // __attribute__((aligned)) on the field.
        let annotation_alignment = annotation_alignment(&field.annotations).unwrap_or(0);
        let attr_packed = self.attr_packed || is_attr_packed(&field.annotations);
        // The field alignment is based on the alignment of the underlying type, #pragma pack,
        // __attribute__((aligned)) on the field, and __attribute__((packed)) on the field or record.
        let field_alignment_bits = if size_bits == 0 {
            // If the width is 0, #pragma pack and __attribute__((packed)) are ignored. The field
            // alignment is the alignment of the underlying type unless it is explicitly increased
            // with __attribute__((aligned)).
            //
            // ```c,gcc
            // #pragma pack(2)
            //
            // struct X {
            //         char c;
            //         __attribute__((packed)) int :0;
            //         char d;
            // };
            //
            // struct Y {
            //         char c;
            //         __attribute__((aligned(8))) int :0;
            //         char d;
            // };
            //
            // static void f(void) {
            //         _Static_assert(__builtin_offsetof(struct X, d) == 4, "");
            //         _Static_assert(__builtin_offsetof(struct Y, d) == 8, "");
            // }
            // ```
            ty_field_alignment_bits.max(annotation_alignment)
        } else if let Some(max_field_alignment_bits) = self.max_field_alignment_bits {
            // Otherwise, if a #pragma pack is in effect, __attribute__((packed)) on the field or
            // record is ignored.
            //
            // ```c,gcc
            // #pragma pack(16)
            //
            // struct __attribute__((packed)) X {
            //         __attribute__((packed)) int i:1;
            // };
            //
            // struct Y {
            //         __attribute__((aligned(8))) int i:1;
            // };
            //
            // static void f(void) {
            //         _Static_assert(_Alignof(struct X) == 4, "");
            //         _Static_assert(_Alignof(struct Y) == 8, "");
            // }
            // ```
            ty_field_alignment_bits
                .max(annotation_alignment)
                .min(max_field_alignment_bits)
        } else if attr_packed {
            // Otherwise, if the field or the record is packed, the field alignment is 1 bit unless
            // it is explicitly increased with __attribute__((aligned)).
            //
            // ```c,gcc
            // struct __attribute__((packed)) X {
            //         __attribute__((aligned(2))) int i:1;
            // };
            //
            // struct Y {
            //         __attribute__((packed, aligned(2))) int i:1;
            // };
            //
            // static void f(void) {
            //         _Static_assert(_Alignof(struct X) == 2, "");
            //         _Static_assert(_Alignof(struct Y) == 2, "");
            // }
            // ```
            annotation_alignment.max(1)
        } else {
            // Otherwise, the field alignment is the field alignment of the underlying type unless
            // it is explicitly increased with __attribute__((aligned)).
            //
            // ```c,gcc
            // struct X {
            //         __attribute__((aligned(8))) int i:1;
            // };
            //
            // struct Y {
            //         __attribute__((aligned(1))) int i:1;
            // };
            //
            // static void f(void) {
            //         _Static_assert(_Alignof(struct X) == 8, "");
            //         _Static_assert(_Alignof(struct Y) == 4, "");
            // }
            // ```
            ty_field_alignment_bits.max(annotation_alignment)
        };
        // Unnamed fields do not contribute to the record alignment except on a few targets.
        //
        // ```c,clang
        // struct X {
        //         int :1;
        // };
        //
        // static void f(void) {
        // #if !defined(__APPLE__) && (defined(__arm__) || defined(__aarch64__))
        //         _Static_assert(_Alignof(struct X) == 4, "");
        // #else
        //         _Static_assert(_Alignof(struct X) == 1, "");
        // #endif
        // }
        // ```
        if field.named || unnamed_field_affects_record_alignment(self.target) {
            self.alignment_bits.assign_max(field_alignment_bits);
        }
        let first_unused_bit = match self.kind {
            RecordKind::Union => 0,
            RecordKind::Struct => self.size_bits,
        };
        let field_crosses_storage_boundary = match system_compiler(self.target) {
            Compiler::Gcc if ty_field_alignment_bits > ty_size_bits => true,
            _ => first_unused_bit % ty_field_alignment_bits + size_bits > ty_size_bits,
        };
        let offset_bits = if size_bits == 0 {
            align_to(first_unused_bit, field_alignment_bits)?
        } else if self.max_field_alignment_bits.is_none()
            && !attr_packed
            && field_crosses_storage_boundary
        {
            align_to(first_unused_bit, field_alignment_bits)?
        } else if annotation_alignment != 0
            && (system_compiler(self.target) == Compiler::Gcc
                || self.max_field_alignment_bits.is_none()
                || annotation_alignment <= self.max_field_alignment_bits.unwrap())
        {
            align_to(
                first_unused_bit,
                annotation_alignment.min2(self.max_field_alignment_bits),
            )?
        } else {
            first_unused_bit
        };
        self.size_bits.assign_max(size_add(offset_bits, size_bits)?);
        match field.named {
            true => Ok(Some(FieldLayout {
                offset_bits,
                size_bits,
            })),
            false => Ok(None),
        }
    }

    fn layout_regular_field(
        &mut self,
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
        if self.attr_packed || is_attr_packed(&field.annotations) {
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
        field_alignment_bits.assign_max(annotation_alignment(&field.annotations));
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
        field_alignment_bits.assign_min(self.max_field_alignment_bits);
        let offset_bits = match self.kind {
            // A struct field starts at the next offset in the struct that is properly
            // aligned with respect to the start of the struct.
            RecordKind::Struct => align_to(self.size_bits, field_alignment_bits)?,
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
        self.size_bits.assign_max(size_add(offset_bits, size_bits)?);
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
        self.alignment_bits.assign_max(field_alignment_bits);
        Ok(Some(FieldLayout {
            offset_bits,
            size_bits,
        }))
    }
}
