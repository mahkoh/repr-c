use crate::layout::{BuiltinType, TypeLayout};
use crate::util::BITS_PER_BYTE;

pub trait Target: 'static + Send + Sync {
    fn layout_algorithm(&self) -> LayoutAlgorithm;
    fn builtin_type_layout(&self, b: BuiltinType) -> TypeLayout;
    fn is_64_bit(&self) -> bool;
    fn endianness(&self) -> Endianness;
    fn name(&self) -> &str;
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LayoutAlgorithm {
    Msvc,
    SysV,
    MinGw,
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Endianness {
    Little,
    Big,
}

pub const TARGETS: &[&'static dyn Target] = &[
    &X8664PcWindowsMsvc,
    &X8664UwpWindowsMsvc,
    &I586PcWindowsMsvc,
    &I686PcWindowsMsvc,
    &I686UwpWindowsMsvc,
    &AArch64PcWindowsMsvc,
    &AArch64UwpWindowsMsvc,
    &Thumbv7aPcWindowsMsvc,
    &Thumbv7aUwpWindowsMsvc,
];

macro_rules! windows_target {
    ($(#[$meta:meta])* $id:ident, $name:expr, $wide:expr, $ptr_size:expr) => {
        $(#[$meta])*
        pub struct $id;

        impl Target for $id {
            fn layout_algorithm(&self) -> LayoutAlgorithm {
                LayoutAlgorithm::Msvc
            }

            fn builtin_type_layout(&self, b: BuiltinType) -> TypeLayout {
                let (size_bytes, alignment_bytes) = match b {
                    BuiltinType::Unit => (0, 1),
                    BuiltinType::Bool => (1, 1),
                    BuiltinType::U8 => (1, 1),
                    BuiltinType::U16 => (2, 2),
                    BuiltinType::U32 => (4, 4),
                    BuiltinType::U64 => (8, 8),
                    BuiltinType::U128 => (16, 16),
                    BuiltinType::I8 => (1, 1),
                    BuiltinType::I16 => (2, 2),
                    BuiltinType::I32 => (4, 4),
                    BuiltinType::I64 => (8, 8),
                    BuiltinType::I128 => (16, 16),
                    BuiltinType::Char => (1, 1),
                    BuiltinType::SignedChar => (1, 1),
                    BuiltinType::UnsignedChar => (1, 1),
                    BuiltinType::Short => (2, 2),
                    BuiltinType::UnsignedShort => (2, 2),
                    BuiltinType::Int => (4, 4),
                    BuiltinType::UnsignedInt => (4, 4),
                    BuiltinType::Long => (4, 4),
                    BuiltinType::UnsignedLong => (4, 4),
                    BuiltinType::LongLong => (8, 8),
                    BuiltinType::UnsignedLongLong => (8, 8),
                    BuiltinType::F32 => (4, 4),
                    BuiltinType::F64 => (8, 8),
                    BuiltinType::Float => (4, 4),
                    BuiltinType::Double => (8, 8),
                    BuiltinType::Pointer => ($ptr_size, $ptr_size),
                };
                TypeLayout {
                    size_bits: size_bytes * BITS_PER_BYTE,
                    alignment_bits: alignment_bytes * BITS_PER_BYTE,
                    required_alignment_bits: BITS_PER_BYTE,
                }
            }

            fn is_64_bit(&self) -> bool {
                $wide
            }

            fn endianness(&self) -> Endianness {
                Endianness::Little
            }

            fn name(&self) -> &str {
                $name
            }
        }
    }
}

macro_rules! windows_64_bit_target {
    ($($tt:tt)*) => {
        windows_target!($($tt)*, true, 8);
    }
}

macro_rules! windows_32_bit_target {
    ($($tt:tt)*) => {
        windows_target!($($tt)*, false, 4);
    }
}

windows_32_bit_target! {
    /// The `i586-pc-windows-msvc` target
    I586PcWindowsMsvc, "i586-pc-windows-msvc"
}
windows_32_bit_target! {
    /// The `i686-pc-windows-msvc` target
    I686PcWindowsMsvc, "i686-pc-windows-msvc"
}
windows_32_bit_target! {
    /// The `i686-uwp-windows-msvc` target
    I686UwpWindowsMsvc, "i686-uwp-windows-msvc"
}
windows_32_bit_target! {
    /// The `thumbv7a-pc-windows-msvc` target
    Thumbv7aPcWindowsMsvc, "thumbv7a-pc-windows-msvc"
}
windows_32_bit_target! {
    /// The `thumbv7a-uwp-windows-msvc` target
    Thumbv7aUwpWindowsMsvc, "thumbv7a-uwp-windows-msvc"
}

windows_64_bit_target! {
    /// The `x86_64-pc-windows-msvc` target
    X8664PcWindowsMsvc, "x86_64-pc-windows-msvc"
}
windows_64_bit_target! {
    /// The `x86_64-uwp-windows-msvc` target
    X8664UwpWindowsMsvc, "x86_64-uwp-windows-msvc"
}
windows_64_bit_target! {
    /// The `aarch64-pc-windows-msvc` target
    AArch64PcWindowsMsvc, "aarch64-pc-windows-msvc"
}
windows_64_bit_target! {
    /// The `aarch64-uwp-windows-msvc` target
    AArch64UwpWindowsMsvc, "aarch64-uwp-windows-msvc"
}
