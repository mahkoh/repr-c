use std::fmt::Debug;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type<I: LayoutInfo> {
    pub layout: I,
    pub annotations: Vec<Annotation>,
    pub variant: TypeVariant<I>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Annotation {
    PragmaPack(u64),
    AttrPacked,
    Aligned(u64),
}

impl From<Type<TypeLayout>> for Type<()> {
    fn from(src: Type<TypeLayout>) -> Self {
        Type {
            layout: (),
            annotations: src.annotations,
            variant: src.variant.into(),
        }
    }
}

pub trait LayoutInfo: Copy + Default + Debug + Eq + PartialEq {
    type FieldLayout: Copy + Default + Debug + Eq + PartialEq;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct TypeLayout {
    pub size_bits: u64,
    pub alignment_bits: u64,
    pub required_alignment_bits: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct FieldLayout {
    pub offset_bits: u64,
    pub size_bits: u64,
}

impl LayoutInfo for TypeLayout {
    type FieldLayout = FieldLayout;
}

impl LayoutInfo for () {
    type FieldLayout = ();
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeVariant<I: LayoutInfo> {
    Builtin(BuiltinType),
    Record(Record<I>),
    Typedef(Box<Type<I>>),
    Array(Array<I>),
    Enum(Vec<i128>),
    Opaque(TypeLayout),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record<I: LayoutInfo> {
    pub kind: RecordKind,
    pub fields: Vec<RecordField<I>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Array<I: LayoutInfo> {
    pub element_type: Box<Type<I>>,
    pub num_elements: u64,
}

impl From<TypeVariant<TypeLayout>> for TypeVariant<()> {
    fn from(src: TypeVariant<TypeLayout>) -> Self {
        match src {
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
            TypeVariant::Opaque(l) => TypeVariant::Opaque(l),
            TypeVariant::Enum(v) => TypeVariant::Enum(v),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordField<I: LayoutInfo> {
    pub layout: I::FieldLayout,
    pub annotations: Vec<Annotation>,
    pub name: Option<String>,
    pub bit_width: Option<u64>,
    pub ty: Type<I>,
}

impl From<RecordField<TypeLayout>> for RecordField<()> {
    fn from(src: RecordField<TypeLayout>) -> Self {
        RecordField {
            layout: (),
            annotations: src.annotations,
            name: src.name,
            bit_width: src.bit_width,
            ty: src.ty.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RecordKind {
    Struct,
    Union,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
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
    /// `*const T`,`*mut T`, `&T, `&mut T` for sized `T`; `fn()` etc.
    Pointer,
}
