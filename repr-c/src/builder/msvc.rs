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
            // __declspec(align) increases both the required and the field alignment but
            // never decreases them. It does not affect the size or the pointer alignment.
            //
            // ```c,msvc,msvc-tc-0014
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
                    field_alignment_bits: dst_ty.layout.field_alignment_bits.max(max_alignment),
                    required_alignment_bits: dst_ty
                        .layout
                        .required_alignment_bits
                        .max(max_alignment),
                    ..dst_ty.layout
                },
                annotations: ty.annotations.clone(),
                variant: TypeVariant::Typedef(Box::new(dst_ty)),
            })
        }
        TypeVariant::Array(a) => {
            let ety = compute_layout(target, &a.element_type)?;
            Ok(Type {
                layout: TypeLayout {
                    // The size of an array is the size of the underlying type multiplied by the
                    // number of elements. Since the size might not be a multiple of the field
                    // alignment, the address of the second element might not be properly aligned
                    // for the field alignment.
                    //
                    // ```c,msvc,msvc-tc-0018
                    // __declspec(align(4)) typedef char Char;
                    //
                    // typedef Char X[3];
                    //
                    // static void f(void) {
                    //         static_assert(sizeof(X) == 3, "");
                    //         static_assert(_Alignof(X) == 4, "");
                    // }
                    // ```
                    size_bits: size_mul(ety.layout.size_bits, a.num_elements)?,
                    // The alignments are inherited from the underlying type.
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
        TypeVariant::Enum(v) => {
            // #pragma pack is ignored on enums.
            let alignment = max_alignment(&ty.annotations);
            // Enums always have the base type int even if the values do not fit into int. The
            // values are silently truncated if necessary.
            //
            // ```c,msvc,msvc-tc-0019
            // #pragma pack(1)
            //
            // enum E {
            //         A = 1,
            //         B = 0xffff0fffffff,
            // };
            //
            // static void f(void) {
            //         static_assert(sizeof(enum E) == 4, "");
            //         static_assert(B == 0x0fffffff, "");
            // }
            // ```
            let mut layout = target.builtin_type_layout(BuiltinType::Int);
            layout.required_alignment_bits.assign_max(alignment);
            layout.field_alignment_bits.assign_max(alignment);
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
    // The annotations of this type.
    annotations: &'a [Annotation],
    // The required alignment of the type.
    required_alignment_bits: u64,
    // The alignment of pointers that point to an object of this type. This is greater to or equal
    // to the required alignment. Once all fields have been laid out, the size of the record will be
    // rounded up to this value.
    pointer_alignment_bits: u64,
    // The alignment of this type when it is used as a record field. This is greater to or equal to
    // the pointer alignment.
    field_alignment_bits: u64,
    // The size of the record.
    size_bits: u64,
    // The minimum value of all __attribute__((packed)) and #pragma pack(N) at the type level.
    max_field_alignment_bits: Option<u64>,
    // The fields in this record.
    record_fields: Vec<RecordField<TypeLayout>>,
    // The kind of this record. Struct or Union.
    kind: RecordKind,
    // Set to `Some` if and only if the previous field was a non-zero-sized bitfield.
    ongoing_bitfield: Option<OngoingBitfield>,
    // Set to `true` if and only if the record contains at least on non-bitfield field.
    contains_non_bitfield: bool,
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
    fn new(
        target: &'a dyn Target,
        kind: RecordKind,
        annotations: &'a [Annotation],
    ) -> Result<Self> {
        let min_pack_value = annotations
            .iter()
            .flat_map(|a| {
                match a {
                    Annotation::PragmaPack(n) => Some(*n),
                    // __attribute__((packed)) behaves like #pragma pack(1) in clang.
                    //
                    // ```c,clang
                    // struct __attribute__((packed)) Y {
                    //         int i;
                    // };
                    //
                    // static void f(void) {
                    //         _Static_assert(_Alignof(struct Y) == 1, "");
                    // }
                    // ```
                    Annotation::AttrPacked => Some(1),
                    _ => None,
                }
                .into_iter()
            })
            .min();
        // The effect of #pragma pack(N) depends on the target.
        //
        // x86: By default, there is no maximum field alignment. N={1,2,4} set the maximum field
        //      alignment to that value. All other N activate the default.
        // x64: By default, there is no maximum field alignment. N={1,2,4,8} set the maximum field
        //      alignment to that value. All other N activate the default.
        // arm: By default, the maximum field alignment is 8. N={1,2,4,8,16} set the maximum field
        //      alignment to that value. All other N activate the default.
        // arm64: By default, the maximum field alignment is 8. N={1,2,4,8} set the maximum field
        //        alignment to that value. N=16 disables the maximum field alignment. All other N
        //        activate the default.
        //
        // ```c,msvc,msvc-tc-0020
        // struct A {
        //         __declspec(align(128)) int i:1;
        // };
        //
        // struct B {
        //         struct A x;
        // };
        //
        // #pragma pack(4)
        // struct C {
        //         struct A x;
        // };
        // #pragma pack()
        //
        // #pragma pack(8)
        // struct D {
        //         struct A x;
        // };
        // #pragma pack()
        //
        // #pragma pack(16)
        // struct E {
        //         struct A x;
        // };
        // #pragma pack()
        //
        // #pragma pack(32)
        // struct F {
        //         struct A x;
        // };
        // #pragma pack()
        //
        // static void f(void) {
        // #if defined(_M_IX86)
        //         static_assert(_Alignof(struct B) == 128, "");
        //         static_assert(_Alignof(struct C) == 4, "");
        //         static_assert(_Alignof(struct D) == 128, "");
        //         static_assert(_Alignof(struct E) == 128, "");
        //         static_assert(_Alignof(struct F) == 128, "");
        // #elif defined(_M_X64)
        //         static_assert(_Alignof(struct B) == 128, "");
        //         static_assert(_Alignof(struct C) == 4, "");
        //         static_assert(_Alignof(struct D) == 8, "");
        //         static_assert(_Alignof(struct E) == 128, "");
        //         static_assert(_Alignof(struct F) == 128, "");
        // #elif defined(_M_ARM)
        //         static_assert(_Alignof(struct B) == 8, "");
        //         static_assert(_Alignof(struct C) == 4, "");
        //         static_assert(_Alignof(struct D) == 8, "");
        //         static_assert(_Alignof(struct E) == 16, "");
        //         static_assert(_Alignof(struct F) == 8, "");
        // #elif defined(_M_ARM64)
        //         static_assert(_Alignof(struct B) == 8, "");
        //         static_assert(_Alignof(struct C) == 4, "");
        //         static_assert(_Alignof(struct D) == 8, "");
        //         static_assert(_Alignof(struct E) == 128, "");
        //         static_assert(_Alignof(struct F) == 8, "");
        // #else
        //         static_assert(0, "unknown target");
        // #endif
        // }
        // ```
        let max_field_alignment_bits = target.effective_pragma_pack(min_pack_value);
        // The required alignment can be increased by adding a __declspec(align)
        // annotation.
        //
        // ```c,msvc,msvc-tc-0023
        // __declspec(align(8)) struct X {
        //         char c;
        // };
        //
        // #pragma pack(1)
        //
        // struct Y {
        //         struct X x;
        // };
        //
        // static void f(void) {
        //         static_assert(_Alignof(struct Y) == 8, "");
        // }
        // ```
        let required_alignment_bits = max_alignment(annotations);
        Ok(Self {
            target,
            annotations,
            required_alignment_bits,
            // pointer and field alignment are at least as strict as the required
            // alignment
            pointer_alignment_bits: required_alignment_bits,
            field_alignment_bits: required_alignment_bits,
            size_bits: 0,
            max_field_alignment_bits,
            record_fields: vec![],
            kind,
            ongoing_bitfield: None,
            contains_non_bitfield: false,
        })
    }

    fn compute(mut self, fields: &[RecordField<()>]) -> Result<Type<TypeLayout>> {
        for f in fields {
            self.layout_field(f)?;
        }
        if self.size_bits == 0 {
            // As an extension, MSVC allows records that only contain zero-sized bitfields and empty
            // arrays. Such records would be zero-sized but this case is handled here separately to
            // ensure that there are no zero-sized records.
            self.handle_zero_sized_record();
        }
        // The size is always a multiple of the pointer alignment.
        self.size_bits = align_to(self.size_bits, self.pointer_alignment_bits)?;
        Ok(Type {
            layout: TypeLayout {
                size_bits: self.size_bits,
                field_alignment_bits: self.field_alignment_bits,
                pointer_alignment_bits: self.pointer_alignment_bits,
                required_alignment_bits: self.required_alignment_bits,
            },
            annotations: self.annotations.to_vec(),
            variant: TypeVariant::Record(Record {
                kind: self.kind,
                fields: self.record_fields,
            }),
        })
    }

    fn handle_zero_sized_record(&mut self) {
        match self.kind {
            RecordKind::Union => {
                // If all fields in a union have size 0, the size of the whole enum is set to ...
                if self.contains_non_bitfield {
                    // ... its alignment if it contains at least one non-bitfield.
                    //
                    // ```c,msvc,msvc-tc-0024
                    // union X {
                    //         long long b[];
                    // };
                    //
                    // union Y {
                    //         long long :0;
                    //         char b[];
                    // };
                    //
                    // static void f(void) {
                    //         static_assert(sizeof(union X) == 8, "");
                    //         static_assert(sizeof(union Y) == 1, "");
                    // }
                    // ```
                    self.size_bits = self.field_alignment_bits;
                } else {
                    // ... 4 bytes if it contains only bitfields.
                    //
                    // ```c,msvc,msvc-tc-0025
                    // union X {
                    //         long long :0;
                    // };
                    //
                    // union Y {
                    //         char :0;
                    // };
                    //
                    // static void f(void) {
                    //         static_assert(sizeof(union X) == 4, "");
                    //         static_assert(sizeof(union Y) == 4, "");
                    // }
                    // ```
                    self.size_bits = 4 * BITS_PER_BYTE;
                }
            }
            RecordKind::Struct => {
                // If all fields in a struct have size 0, its size is set to its required alignment
                // but at least to 4 bytes.
                //
                // ```c,msvc,msvc-tc-0026
                // struct X {
                //         __declspec(align(2)) long long b[];
                // };
                //
                // struct Y {
                //         __declspec(align(8)) long long b[];
                // };
                //
                // static void f(void) {
                //         static_assert(sizeof(struct X) == 4, "");
                //         static_assert(_Alignof(struct X) == 8, "");
                //
                //         static_assert(sizeof(struct Y) == 8, "");
                // }
                // ```
                self.size_bits = self.required_alignment_bits.max(4 * BITS_PER_BYTE);
                self.pointer_alignment_bits.assign_min(self.size_bits);
            }
        }
    }

    fn layout_field(&mut self, field: &RecordField<()>) -> Result<()> {
        // The offset and the size of the field is based on the layout of the underlying type.
        let field_ty = compute_layout(self.target, &field.ty)?;
        let (ty_size_bits, field_alignment_bits) = {
            let layout = field_ty.layout;
            // The offset of the field is based on the alignment of the underlying type.
            //
            // ```c,msvc,msvc-tc-0027
            // #include <stdlib.h>
            //
            // struct Y {
            //         char c;
            //         int i;
            // };
            //
            // static void f(void) {
            //         static_assert(offsetof(struct Y, i) == 4, "");
            // }
            // ```
            let mut field_alignment_bits = layout.field_alignment_bits;
            // The required alignment of the field is the maximum of the required alignment of the
            // underlying type and the __declspec(align) annotation on the field itself.
            //
            // ```c,msvc,msvc-tc-0028
            // __declspec(align(4)) typedef char Char;
            //
            // #pragma pack(1)
            //
            // struct A {
            //         Char a;
            // };
            //
            // struct B {
            //         __declspec(align(4)) char a;
            // };
            //
            // struct C {
            //         __declspec(align(8)) Char a;
            // };
            //
            // struct D {
            //         __declspec(align(2)) Char a;
            // };
            //
            // static void f(void) {
            //         static_assert(_Alignof(struct A) == 4, "");
            //         static_assert(_Alignof(struct B) == 4, "");
            //         static_assert(_Alignof(struct C) == 8, "");
            //         static_assert(_Alignof(struct D) == 4, "");
            // }
            // ```
            let required_alignment_bits =
                max_alignment(&field.annotations).max(layout.required_alignment_bits);
            // The required alignment of a record is the maximum of the required alignments of its
            // fields except that the required alignment of bitfields is ignored.
            //
            // ```c,msvc,msvc-tc-0029
            // __declspec(align(4)) typedef char Char;
            //
            // struct A {
            //         Char a;
            // };
            //
            // struct B {
            //         Char a:1;
            // };
            //
            // #pragma pack(1)
            //
            // struct C {
            //         struct A a;
            // };
            //
            // struct D {
            //         struct B a;
            // };
            //
            // static void f(void) {
            //         static_assert(_Alignof(struct C) == 4, "");
            //         static_assert(_Alignof(struct D) == 1, "");
            // }
            // ```
            if field.bit_width.is_none() {
                self.required_alignment_bits
                    .assign_max(required_alignment_bits);
            }
            // If the field or struct is packed, reduce the alignment of the field ...
            if is_packed(&field.annotations) {
                // __attribute__((packed)) on a field is a clang extension. It behaves as if #pragma
                // pack(1) had been applied only to this field.
                //
                // ```c,clang
                // struct A {
                //         int a __attribute__((packed));
                //         short b;
                // };
                //
                // static void f(void) {
                //         _Static_assert(_Alignof(struct A) == 2, "");
                // }
                // ```
                field_alignment_bits = BITS_PER_BYTE;
            } else {
                // ```c,msvc,msvc-tc-0030
                // #pragma pack(2)
                //
                // struct A {
                //         int a;
                // };
                //
                // static void f(void) {
                //         static_assert(_Alignof(struct A) == 2, "");
                // }
                // ```
                field_alignment_bits.assign_min(self.max_field_alignment_bits);
            }
            // ... but the required alignment still takes precedence.
            //
            // ```c,msvc,msvc-tc-0031
            // __declspec(align(4)) typedef char Char;
            //
            // #pragma pack(2)
            //
            // struct A {
            //         Char a;
            // };
            //
            // static void f(void) {
            //         static_assert(_Alignof(struct A) == 4, "");
            // }
            // ```
            field_alignment_bits.assign_max(required_alignment_bits);
            (layout.size_bits, field_alignment_bits)
        };
        // These functions return `None` if and only if the field is unnamed.
        let layout = match field.bit_width {
            Some(n) => self.layout_bit_field(ty_size_bits, field_alignment_bits, field.named, n),
            None => self.layout_regular_field(ty_size_bits, field_alignment_bits),
        }?;
        self.record_fields.push(RecordField {
            layout,
            annotations: field.annotations.clone(),
            named: field.named,
            bit_width: field.bit_width,
            ty: field_ty,
        });
        Ok(())
    }

    fn layout_regular_field(
        &mut self,
        size_bits: u64,
        field_alignment_bits: u64,
    ) -> Result<Option<FieldLayout>> {
        self.contains_non_bitfield = true;
        self.ongoing_bitfield = None;
        // The alignment of the field affects both the pointer alignment and the field
        // alignment of the record.
        //
        // ```c,msvc,msvc-tc-0032
        // struct A {
        //         long a;
        //         char c;
        // };
        //
        // static void f(void) {
        //         static_assert(_Alignof(struct A) == 4, "");
        //         static_assert(sizeof(struct A) == 8, "");
        // }
        // ```
        self.pointer_alignment_bits.assign_max(field_alignment_bits);
        self.field_alignment_bits.assign_max(field_alignment_bits);
        let offset_bits = match self.kind {
            // A struct field starts at the next offset in the struct that is properly
            // aligned with respect to the start of the struct.
            //
            // ```c,msvc,msvc-tc-0033
            // #include <stdlib.h>
            //
            // struct A {
            //         char c;
            //         long a;
            // };
            //
            // static void f(void) {
            //         static_assert(offsetof(struct A, a) == 4, "");
            // }
            // ```
            RecordKind::Struct => align_to(self.size_bits, field_alignment_bits)?,
            // A union field always starts at offset 0.
            RecordKind::Union => 0,
        };
        // Set the size of the record to the maximum of the current size and the end of
        // the field.
        //
        // ```c,msvc,msvc-tc-0034
        // union U {
        //         int l;
        //         char c;
        // };
        //
        // static void f(void) {
        //         static_assert(sizeof(union U) == 4, "");
        // }
        // ```
        self.size_bits.assign_max(offset_bits + size_bits);
        Ok(Some(FieldLayout {
            offset_bits,
            size_bits,
        }))
    }

    fn layout_bit_field(
        &mut self,
        ty_size_bits: u64,
        field_alignment_bits: u64,
        named: bool,
        width: u64,
    ) -> Result<Option<FieldLayout>> {
        if width == 0 {
            // A zero-sized bit-field that does not follow a non-zero-sized bit-field does not affect
            // the overall layout of the record. Even in a union where the order would otherwise
            // not matter.
            //
            // ```c,msvc,msvc-tc-0035
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
                return Ok(None);
            }
            self.ongoing_bitfield = None;
        } else {
            // Even _Bool allows bitfields up to its type size.
            //
            // ```c,msvc,msvc-tc-0036
            // struct S {
            //         _Bool v:8;
            // };
            //
            // static void f(void) {
            //         static_assert(sizeof(struct S) == 1, "");
            // }
            // ```
            if width > ty_size_bits {
                return Err(Error::OversizedBitfield);
            }
            // If there is an ongoing bit-field in a struct whose underlying type has the same size and
            // if there is enough space left to place this bit-field, then this bit-field is placed in
            // the ongoing bit-field and the overall layout of the struct is not affected by this
            // bit-field.
            //
            // ```c,msvc,msvc-tc-0037
            // struct S {
            //         unsigned int i:1;
            //         __declspec(align(128)) long j:1;
            // };
            //
            // static void f(void) {
            //         static_assert(sizeof(struct S) == 4, "");
            //         static_assert(_Alignof(struct S) == 4, "");
            // }
            // ```
            if let (RecordKind::Struct, Some(ref mut p)) = (self.kind, &mut self.ongoing_bitfield) {
                if p.ty_size_bits == ty_size_bits && p.unused_size_bits >= width {
                    let offset_bits = self.size_bits - p.unused_size_bits;
                    p.unused_size_bits -= width;
                    return Ok(match named {
                        true => Some(FieldLayout {
                            offset_bits,
                            size_bits: width,
                        }),
                        false => None,
                    });
                }
            }
            self.ongoing_bitfield = Some(OngoingBitfield {
                ty_size_bits,
                unused_size_bits: ty_size_bits - width,
            });
        }
        let offset_bits = match self.kind {
            RecordKind::Struct => {
                // This is the one place in the layout of a record where the pointer alignment might
                // get assigned a smaller value than the field alignment. This can only happen if
                // the field or the type of the field has a required alignment. Otherwise the value
                // of field_alignment_bits is already bound by max_field_alignment_bits.
                //
                // ```c,msvc,msvc-tc-0038
                // #pragma pack(1)
                //
                // struct S {
                //         __declspec(align(4)) char c:1;
                // };
                //
                // typedef struct S X[3];
                //
                // static void f(void) {
                //         static_assert(sizeof(X) == 3, "");
                //         static_assert(_Alignof(X) == 4, "");
                // }
                // ```
                self.pointer_alignment_bits
                    .assign_max(field_alignment_bits.min2(self.max_field_alignment_bits));
                self.field_alignment_bits.assign_max(field_alignment_bits);
                let offset_bits = align_to(self.size_bits, field_alignment_bits)?;
                self.size_bits = match width {
                    // A zero-width bitfield only increases to size of the struct to the
                    // offset a non-zero-width bitfield with the same alignment would
                    // start.
                    //
                    // ```c,msvc,msvc-tc-0039
                    // #include <stdlib.h>
                    //
                    // struct S {
                    //         char c:1;
                    //         int :0;
                    //         char d;
                    // };
                    //
                    // static void f(void) {
                    //         static_assert(offsetof(struct S, d) == 4, "");
                    // }
                    // ```
                    0 => offset_bits,
                    // A non-zero-width bitfield always increases the size by the full
                    // size of the underlying type. Even if we are in a packed context.
                    //
                    // ```c,msvc,msvc-tc-0040
                    // #pragma pack(1)
                    //
                    // struct S {
                    //         char c;
                    //         int :1;
                    // };
                    //
                    // static void f(void) {
                    //         static_assert(sizeof(struct S) == 5, "");
                    // }
                    // ```
                    _ => size_add(offset_bits, ty_size_bits)?,
                };
                offset_bits
            }
            RecordKind::Union => {
                // Bit-fields do not affect the alignment of a union.
                //
                // ```c,msvc,msvc-tc-0041
                // union U {
                //         int a:1;
                // };
                //
                // static void f(void) {
                //         static_assert(_Alignof(union U) == 1, "");
                // }
                // ```
                self.size_bits.assign_max(ty_size_bits);
                0
            }
        };
        Ok(match named {
            true => Some(FieldLayout {
                offset_bits,
                size_bits: width,
            }),
            false => None,
        })
    }
}
