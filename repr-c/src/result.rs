use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    AnnotatedBuiltinType,
    AnnotatedOpaqueType,
    AnnotatedArray,
    SizeOverflow,
    AlignmentOverflow,
    PowerOfTwoAlignment,
    SubByteAlignment,
    SubByteSize,
    MultipleAlignmentAnnotations,
    MultiplePackedAnnotations,
    NamedZeroSizeBitField,
    UnnamedRegularField,
    OversizedBitfield,
    PackedTypedef,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Error::*;
        let s = match self {
            AnnotatedBuiltinType => "Builtin types cannot have annotations",
            AnnotatedOpaqueType => "Opaque types cannot have annotations",
            AnnotatedArray => "Arrays cannot have annotations",
            SizeOverflow => "The object size in bits overflows u64",
            AlignmentOverflow => "The object alignment in bits overflows u64",
            PowerOfTwoAlignment => "Alignments must be a power of two",
            SubByteAlignment => "Alignments must be at least 8",
            SubByteSize => "Sizes must be a multiple of 8",
            MultipleAlignmentAnnotations => {
                "A type/field can have at most one alignment annotation"
            }
            MultiplePackedAnnotations => "A type/field can have at most one packed annotation",
            NamedZeroSizeBitField => "A zero-sized bit-field cannot be named",
            UnnamedRegularField => "Regular fields must be named",
            OversizedBitfield => {
                "The width of a bit-field cannot be larger than the width of the underlying type"
            }
            PackedTypedef => "Typedefs cannot have packing annotations",
        };
        f.write_str(s)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
