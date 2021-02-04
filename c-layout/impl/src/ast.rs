use repr_c_impl::layout::{BuiltinType, FieldLayout, RecordKind, TypeLayout};

pub struct State {
    pub next_id: usize,
}

/// A type declaration.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Span(pub usize, pub usize);

/// A type declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub span: Span,
    pub ty: DeclarationType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclarationType {
    Type(Type),
    Const(Expr),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Annotation {
    PragmaPack(Box<Expr>),
    AttrPacked,
    Aligned(Option<Box<Expr>>),
}

/// A type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type {
    pub id: usize,
    pub lo: usize,
    pub layout: Option<TypeLayout>,
    pub layout_hi: usize,
    pub annotations: Vec<Annotation>,
    pub variant: TypeVariant,
}

/// A struct or union.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub kind: RecordKind,
    pub fields: Vec<RecordField>,
}

/// A struct or union field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordField {
    pub parent_id: usize,
    pub pos: Option<usize>,
    pub lo: usize,
    pub layout: Option<FieldLayout>,
    pub layout_hi: usize,
    pub annotations: Vec<Annotation>,
    pub name: Option<String>,
    pub bit_width: Option<Box<Expr>>,
    pub ty: Type,
}

/// A type without its annotations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeVariant {
    Builtin(BuiltinType),
    Record(Record),
    Typedef(Box<Type>),
    Array(Array),
    Opaque(OpaqueTypeLayout),
    Name(String, Span),
    Enum(Vec<Expr>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpaqueTypeLayout {
    pub size_bits: Box<Expr>,
    pub pointer_alignment_bits: Box<Expr>,
    pub field_alignment_bits: Box<Expr>,
    pub required_alignment_bits: Box<Expr>,
}

/// An array.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Array {
    pub element_type: Box<Type>,
    pub num_elements: Option<Box<Expr>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expr {
    pub span: Span,
    pub value: Option<i128>,
    pub value_hi: usize,
    pub ty: ExprType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExprType {
    Lit(i128),
    Unary(UnaryExprType, Box<Expr>),
    Binary(BinaryExprType, Box<Expr>, Box<Expr>),
    TypeExpr(TypeExprType, Box<Type>),
    Builtin(BuiltinExpr),
    Name(String),
    Offsetof(OffsetofType, Type, Vec<Index>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BuiltinExpr {
    BitsPerByte,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UnaryExprType {
    Neg,
    Not,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TypeExprType {
    Sizeof,
    Alignof,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinaryExprType {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LogicalAnd,
    LogicalOr,
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
    pub span: Span,
    pub ty: IndexType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IndexType {
    Field(String),
    Array(Box<Expr>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OffsetofType {
    Bytes,
    Bits,
}
