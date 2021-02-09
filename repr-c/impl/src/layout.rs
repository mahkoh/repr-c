// SPDX-License-Identifier: MIT OR Apache-2.0
use std::fmt::Debug;

/// A C type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type<I: Layout> {
    /// The layout of the type.
    pub layout: I::TypeLayout,
    /// The annotations on this type.
    pub annotations: Vec<Annotation>,
    /// The variant of the type.
    pub variant: TypeVariant<I>,
}

impl<I: Layout> Type<I> {
    /// Returns the identical type with the [`Layout`] converted.
    pub fn into<J: Layout>(self) -> Type<J>
    where
        I::TypeLayout: Into<J::TypeLayout>,
        I::FieldLayout: Into<J::FieldLayout>,
        I::OpaqueLayout: Into<J::OpaqueLayout>,
    {
        Type {
            layout: self.layout.into(),
            annotations: self.annotations,
            variant: self.variant.into(),
        }
    }
}

/// An annotation of a type or field.
///
/// Builtin types, arrays, and opaque types cannot be annotated.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Annotation {
    /// The `PragmaPack` annotation.
    ///
    /// This cannot be used on typedefs or fields. At most one of these can be used on a
    /// type.
    ///
    /// If the argument is `n`, it corresponds to `#pragma pack(n/8)` in C.
    /// If `n` is not a multiple of 8, this annotation will be ignored.
    PragmaPack(u64),
    /// The `AttrPacked` annotation.
    ///
    /// This cannot be used on typedefs.
    ///
    /// This corresponds to `__attribute__((packed))` in C. On MSVC targets, the behavior
    /// is the behavior of Clang.
    AttrPacked,
    /// The `Aligned` annotation.
    ///
    /// If the argument is `Some(n)`, `n` must be a power of two and at least 8. It is the
    /// corresponding C argument but in bits instead of bytes.
    ///
    /// If the argument is `Some(n)`, it corresponds to `__declspec(align(n/8))` on MSVC
    /// targets and `__attribute__((aligned(n/8)))` otherwise.
    ///
    /// If the argument is `None`, it corresponds to `__attribute__((aligned))`. On MSVC
    /// targets, the behavior is the behavior of Clang.
    Align(Option<u64>),
}

/// A collection of types encoding the layout of a type.
pub trait Layout {
    /// The type used to encode the layout of the type itself.
    type TypeLayout: Copy + Default + Debug + Eq + PartialEq;
    /// The type used to encode the layout of a field in a record.
    type FieldLayout: Copy + Default + Debug + Eq + PartialEq;
    /// The type used to encode the layout of an opaque type.
    type OpaqueLayout: Copy + Default + Debug + Eq + PartialEq;
}

/// The computed layout of a type.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct TypeLayout {
    /// The size of the type in bits.
    ///
    /// This is the value returned by `sizeof` and C and `std::mem::size_of` in Rust
    /// (but in bits instead of bytes). This is a multiple of `pointer_alignment_bits`.
    pub size_bits: u64,
    /// The alignment of the type, in bits, when used as a field in a record.
    pub field_alignment_bits: u64,
    /// The alignment, in bits, of valid pointers to this type.
    ///
    /// This is the value returned by `std::mem::align_of` in Rust
    /// (but in bits instead of bytes). `size_bits` is a multiple of this value.
    pub pointer_alignment_bits: u64,
    /// The required alignment of the type in bits.
    ///
    /// This value is only used by MSVC targets. It is 8 on all other
    /// targets. On MSVC targets, this value restricts the effects of `#pragma pack` except
    /// in some cases involving bit-fields.
    pub required_alignment_bits: u64,
}

/// The layout of a field.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct FieldLayout {
    /// The offset of the struct, in bits, from the start of the struct.
    pub offset_bits: u64,
    /// The size, in bits, of the field.
    ///
    /// For bit-fields, this is the width of the field.
    pub size_bits: u64,
}

impl Layout for TypeLayout {
    type TypeLayout = TypeLayout;
    type FieldLayout = FieldLayout;
    type OpaqueLayout = TypeLayout;
}

impl Layout for () {
    type TypeLayout = ();
    type FieldLayout = ();
    type OpaqueLayout = TypeLayout;
}

impl Into<()> for TypeLayout {
    fn into(self) {}
}

impl Into<()> for FieldLayout {
    fn into(self) {}
}

/// An enum of all available types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeVariant<I: Layout> {
    /// A builtin type.
    Builtin(BuiltinType),
    /// A record. Struct or union.
    Record(Record<I>),
    /// A typedef.
    ///
    /// The box contains the target type.
    Typedef(Box<Type<I>>),
    /// An array.
    Array(Array<I>),
    /// An enum.
    ///
    /// The vector contains the values of the variants.
    Enum(Vec<i128>),
    /// An opaque type.
    ///
    /// This does not correspond to anything in C. It is useful if the layout of a nested
    /// type is already known and should not be recomputed. On all supported targets,
    /// substituting a type for the opaque type with the same layout leads to the same
    /// layout for the containing type.
    Opaque(I::OpaqueLayout),
}

impl<I: Layout> TypeVariant<I> {
    /// Returns the identical type variant with the [`Layout`] converted.
    pub fn into<J: Layout>(self) -> TypeVariant<J>
    where
        I::TypeLayout: Into<J::TypeLayout>,
        I::FieldLayout: Into<J::FieldLayout>,
        I::OpaqueLayout: Into<J::OpaqueLayout>,
    {
        match self {
            TypeVariant::Builtin(bi) => TypeVariant::Builtin(bi),
            TypeVariant::Record(rt) => TypeVariant::Record(Record {
                kind: rt.kind,
                fields: rt.fields.into_iter().map(|v| v.into()).collect(),
            }),
            TypeVariant::Typedef(td) => TypeVariant::Typedef(Box::new((*td).into())),
            TypeVariant::Array(at) => TypeVariant::Array(Array {
                element_type: Box::new((*at.element_type).into()),
                num_elements: at.num_elements,
            }),
            TypeVariant::Opaque(l) => TypeVariant::Opaque(l.into()),
            TypeVariant::Enum(v) => TypeVariant::Enum(v),
        }
    }
}

/// A record.
///
/// This corresponds to a struct or a union in C.
///
/// # Example
///
/// ```c
/// #pragma pack(8)
/// struct {
///     int i __attribute__((aligned(16)));
///     char :1;
/// };
/// ```
///
/// ```
/// # use repr_c_impl::layout::{Record, RecordKind, RecordField, Annotation, Type, TypeVariant, BuiltinType};
/// Type::<()> {
///     layout: (),
///     annotations: vec!(Annotation::PragmaPack(64)),
///     variant: TypeVariant::Record(Record::<()> {
///         kind: RecordKind::Struct,
///         fields: vec![
///             RecordField {
///                 layout: None,
///                 annotations: vec!(Annotation::Align(Some(128))),
///                 named: true,
///                 bit_width: None,
///                 ty: Type {
///                     layout: (),
///                     annotations: vec!(),
///                     variant: TypeVariant::Builtin(BuiltinType::Int),
///                 },
///             },
///             RecordField {
///                 layout: None,
///                 annotations: vec!(),
///                 named: false,
///                 bit_width: Some(1),
///                 ty: Type {
///                     layout: (),
///                     annotations: vec!(),
///                     variant: TypeVariant::Builtin(BuiltinType::Char),
///                 },
///             },
///         ],
///     }),
/// };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record<I: Layout> {
    /// The type of the record. Struct or union.
    pub kind: RecordKind,
    /// The fields of the record.
    pub fields: Vec<RecordField<I>>,
}

/// An array.
///
/// This corresponds to the type of `T var[N]` in C.
///
/// # Example
///
/// ```c
/// int i[1];
/// ```
///
/// ```
/// # use repr_c_impl::layout::{Type, TypeVariant, Array, BuiltinType};
/// Array::<()> {
///     element_type: Box::new(Type {
///         layout: (),
///         annotations: vec!(),
///         variant: TypeVariant::Builtin(BuiltinType::Int),
///     }),
///     num_elements: Some(1),
/// };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Array<I: Layout> {
    /// The type of elements of the array.
    pub element_type: Box<Type<I>>,
    /// The number of elements in the array.
    ///
    /// If this is `None`, it corresponds to a flexible array in C.
    pub num_elements: Option<u64>,
}

/// A field of a record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordField<I: Layout> {
    /// The layout of the field.
    ///
    /// This is `None` for unnamed fields.
    pub layout: Option<I::FieldLayout>,
    /// The annotations on this field.
    pub annotations: Vec<Annotation>,
    /// Whether the field is named.
    ///
    /// An unnamed field in C is always a bit-field and is declared like `T : N` where
    /// `T` is the type of the field and `N` is the width of the bit-field.
    pub named: bool,
    /// If this is a bit-field, the width of the field.
    ///
    /// The field is recognized as a bit-field if and only if this is `Some`.
    pub bit_width: Option<u64>,
    /// The type of the field.
    pub ty: Type<I>,
}

impl<I: Layout> RecordField<I> {
    /// Returns the identical record field with the [`Layout`] converted.
    pub fn into<J: Layout>(self) -> RecordField<J>
    where
        I::TypeLayout: Into<J::TypeLayout>,
        I::FieldLayout: Into<J::FieldLayout>,
        I::OpaqueLayout: Into<J::OpaqueLayout>,
    {
        RecordField {
            layout: self.layout.map(|v| v.into()),
            annotations: self.annotations,
            named: self.named,
            bit_width: self.bit_width,
            ty: self.ty.into(),
        }
    }
}

/// The type of a record. Either a struct or a union.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RecordKind {
    /// A struct.
    Struct,
    /// A union.
    Union,
}

/// A builtin type.
///
/// This includes both builtin Rust and C types. The Rust types will be treated like the
/// corresponding C types if possible. If the type does not exist on the target, the
/// results might not be meaningful.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BuiltinType {
    /// `()`
    Unit,
    /// `bool`
    Bool,
    /// `u8`
    U8,
    /// `u16`
    U16,
    /// `u32`
    U32,
    /// `u64`
    U64,
    /// `u128`
    U128,
    /// `i8`
    I8,
    /// `i16`
    I16,
    /// `i32`
    I32,
    /// `i64`
    I64,
    /// `i128`
    I128,
    /// `c_char`
    Char,
    /// `c_schar`
    SignedChar,
    /// `c_uchar`
    UnsignedChar,
    /// `c_short`
    Short,
    /// `c_ushort`
    UnsignedShort,
    /// `c_int`
    Int,
    /// `c_uint`
    UnsignedInt,
    /// `c_long`
    Long,
    /// `c_ulong`
    UnsignedLong,
    /// `c_longlong`
    LongLong,
    /// `c_ulonglong`
    UnsignedLongLong,
    /// `f32`
    F32,
    /// `f64`
    F64,
    /// `c_float`
    Float,
    /// `c_double`
    Double,
    /// `*const T`,`*mut T`, `&T`, `&mut T` for sized `T`; `fn()`, `Option<fn()>`, etc.
    Pointer,
}
