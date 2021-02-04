use std::fmt;
use std::fmt::{Display, Formatter};

/// The result type of this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// An error produced by this crate.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    /// Returns the type of the error.
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

impl std::error::Error for Error {}

/// The type of an error produced by this crate.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A builtin type was annotated.
    ///
    /// Builtin types cannot be annotated. You probably want to annotate a typedef of the
    /// builtin type.
    AnnotatedBuiltinType,
    /// An opaque type was annotated.
    AnnotatedOpaqueType,
    /// An array was annotated.
    ///
    /// Arrays cannot be annotated. You probably want to annotate a typedef of the array.
    AnnotatedArray,
    /// The size of the type cannot be represented in `u64`.
    SizeOverflow,
    /// One of the alignments given in the input is not a power of two.
    PowerOfTwoAlignment,
    /// One of the alignments given in the input is not at least 8.
    SubByteAlignment,
    /// The size of an opaque type is not at least c.
    SubByteSize,
    /// A type has multiple `PragmaPack` annotations.
    MultiplePragmaPackedAnnotations,
    /// A zero-sized bit-field is named.
    ///
    /// Zero-sized bitfields must be unnamed.
    NamedZeroSizeBitField,
    /// A regular field is unnamed.
    ///
    /// Only bit-fields can be unnamed.
    UnnamedRegularField,
    /// One of the bit-fields in the input has a width larger than the size of its type.
    OversizedBitfield,
    /// A typedef has a `PragmaPack` or `AttrPacked` annotation.
    ///
    /// Typedefs can only have `Aligned` annotations.
    PackedTypedef,
    /// A field has a `PragmaPack` annotation.
    ///
    /// Fields cannot have `PragmaPack` annotations.
    PragmaPackedField,
    /// One of the enums in the input contains a variant that is too large for the target.
    EnumOverflow,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ErrorKind::*;
        let s = match self {
            AnnotatedBuiltinType => "Builtin types cannot have annotations",
            AnnotatedOpaqueType => "Opaque types cannot have annotations",
            AnnotatedArray => "Arrays cannot have annotations",
            SizeOverflow => "The object size in bits overflows u64",
            PowerOfTwoAlignment => "Alignments must be a power of two",
            SubByteAlignment => "Alignments must be at least 8",
            SubByteSize => "Sizes must be a multiple of 8",
            PragmaPackedField => "Fields cannot have pragma_pack annotations",
            MultiplePragmaPackedAnnotations => {
                "A type/field can have at most one packed annotation"
            }
            NamedZeroSizeBitField => "A zero-sized bit-field cannot be named",
            UnnamedRegularField => "Regular fields must be named",
            OversizedBitfield => {
                "The width of a bit-field cannot be larger than the width of the underlying type"
            }
            PackedTypedef => "Typedefs cannot have packing annotations",
            EnumOverflow => "Enum constant cannot be represented in any integer type",
        };
        f.write_str(s)
    }
}

pub(crate) fn err(kind: ErrorKind) -> Error {
    Error { kind }
}
