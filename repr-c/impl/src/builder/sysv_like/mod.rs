use crate::builder::common::{
    apply_alignment_override, builtin_type_layout, compute_builtin_type_layout,
    compute_opaque_type_layout, short_enums,
};
use crate::layout::{Annotation, Array, BuiltinType, Type, TypeLayout, TypeVariant};
use crate::result::{err, ErrorKind, Result};
use crate::target::{system_compiler, Compiler, Target};
use crate::util::{align_to, annotation_alignment, is_attr_packed, size_mul, BITS_PER_BYTE};

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
            if dialect == Dialect::Mingw && !is_attr_packed(&ty.annotations) {
                mingw::compute_record_layout(target, r.kind, &ty.annotations, &r.fields)
            } else {
                sysv::compute_record_layout(target, r.kind, &ty.annotations, &r.fields, dialect)
            }
        }
        TypeVariant::Enum(v) => compute_enum_layout(target, v, &ty.annotations),
        TypeVariant::Typedef(dst) => {
            // Pre-validation ensures that typedefs do not have packing annotations.
            let dst_ty = compute_layout(target, dst, dialect)?;
            let max_alignment = annotation_alignment(target, &ty.annotations);
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
            let ety = compute_layout(target, &a.element_type, dialect)?;
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
    let mut required_size = match is_attr_packed(annotations) || short_enums(target) {
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
                Compiler::Clang => annotation_alignment(target, annotations),
                _ => None,
            };
            return Ok(Type {
                layout: apply_alignment_override(layout, max_alignment),
                annotations: annotations.to_vec(),
                variant: TypeVariant::Enum(v.to_vec()),
            });
        }
    }
    Err(err(ErrorKind::EnumOverflow))
}
