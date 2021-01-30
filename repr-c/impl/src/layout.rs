use std::fmt::Debug;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type<I: LayoutInfo> {
    pub layout: I,
    pub annotations: Vec<Annotation>,
    pub variant: TypeVariant<I>,
}

impl<I: LayoutInfo> Type<I> {
    pub fn into<J: LayoutInfo>(self) -> Type<J>
    where
        I: Into<J>,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Annotation {
    PragmaPack(u64),
    AttrPacked,
    Aligned(u64),
}

pub trait LayoutInfo: Copy + Default + Debug + Eq + PartialEq {
    type FieldLayout: Copy + Default + Debug + Eq + PartialEq;
    type OpaqueLayout: Copy + Default + Debug + Eq + PartialEq;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct TypeLayout {
    pub size_bits: u64,
    pub field_alignment_bits: u64,
    pub pointer_alignment_bits: u64,
    pub required_alignment_bits: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct FieldLayout {
    pub offset_bits: u64,
    pub size_bits: u64,
}

impl LayoutInfo for TypeLayout {
    type FieldLayout = FieldLayout;
    type OpaqueLayout = TypeLayout;
}

impl LayoutInfo for () {
    type FieldLayout = ();
    type OpaqueLayout = TypeLayout;
}

impl Into<()> for TypeLayout {
    fn into(self) {}
}

impl Into<()> for FieldLayout {
    fn into(self) {}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeVariant<I: LayoutInfo> {
    Builtin(BuiltinType),
    Record(Record<I>),
    Typedef(Box<Type<I>>),
    Array(Array<I>),
    Enum(Vec<i128>),
    Opaque(I::OpaqueLayout),
}

impl<I: LayoutInfo> TypeVariant<I> {
    pub fn into<J: LayoutInfo>(self) -> TypeVariant<J>
    where
        I: Into<J>,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordField<I: LayoutInfo> {
    pub layout: Option<I::FieldLayout>,
    pub annotations: Vec<Annotation>,
    pub named: bool,
    pub bit_width: Option<u64>,
    pub ty: Type<I>,
}

impl<I: LayoutInfo> RecordField<I> {
    pub fn into<J: LayoutInfo>(self) -> RecordField<J>
    where
        I: Into<J>,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RecordKind {
    Struct,
    Union,
}

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
    /// `*const T`,`*mut T`, `&T, `&mut T` for sized `T`; `fn()` etc.
    Pointer,
}
