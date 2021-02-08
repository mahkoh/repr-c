#![allow(clippy::match_like_matches_macro)]

use crate::layout::{BuiltinType, Type, TypeLayout, TypeVariant};
use crate::result::Result;
use crate::target::Target;
use crate::util::{MinExt, BITS_PER_BYTE};

pub fn compute_builtin_type_layout(target: Target, bi: BuiltinType) -> Result<Type<TypeLayout>> {
    Ok(Type {
        layout: builtin_type_layout(target, bi),
        // Pre-validation ensures that builtin types do not have annotations.
        annotations: vec![],
        variant: TypeVariant::Builtin(bi),
    })
}

pub fn compute_opaque_type_layout(layout: TypeLayout) -> Result<Type<TypeLayout>> {
    Ok(Type {
        layout,
        // Pre-validation ensures that opaque types do not have annotations.
        annotations: vec![],
        variant: TypeVariant::Opaque(layout),
    })
}

pub fn apply_alignment_override(layout: TypeLayout, alignment: Option<u64>) -> TypeLayout {
    TypeLayout {
        field_alignment_bits: alignment.unwrap_or(layout.field_alignment_bits),
        pointer_alignment_bits: alignment.min2(layout.pointer_alignment_bits),
        ..layout
    }
}

pub fn unnamed_field_affects_record_alignment(target: Target) -> bool {
    use Target::*;
    match target {
        | Aarch64Fuchsia
        | Aarch64LinuxAndroid
        | Aarch64UnknownFreebsd
        | Aarch64UnknownHermit
        | Aarch64UnknownLinuxGnu
        | Aarch64UnknownLinuxMusl
        | Aarch64UnknownNetbsd
        | Aarch64UnknownNone
        | Aarch64UnknownOpenbsd
        | Aarch64UnknownRedox
        | Armebv7rUnknownNoneEabi
        | Armebv7rUnknownNoneEabihf
        | ArmLinuxAndroideabi
        | ArmUnknownLinuxGnueabi
        | ArmUnknownLinuxGnueabihf
        | Armv4tUnknownLinuxGnueabi
        | Armv5teUnknownLinuxGnueabi
        | Armv5teUnknownLinuxUclibcgnueabi
        | Armv6UnknownFreebsdGnueabihf
        | Armv6UnknownNetbsdelfEabihf
        | Armv7aNoneEabi
        | Armv7aNoneEabihf
        | Armv7AppleIos
        | Armv7NoneLinuxAndroid
        | Armv7rUnknownNoneEabi
        | Armv7rUnknownNoneEabihf
        | Armv7sAppleIos
        | Armv7UnknownFreebsdGnueabihf
        | Armv7UnknownLinuxGnueabi
        | Armv7UnknownLinuxGnueabihf
        | Armv7UnknownNetbsdelfEabihf
        | AvrUnknownUnknown
        | Thumbv4tNoneEabi
        | Thumbv6mNoneEabi
        | Thumbv7emNoneEabi
        | Thumbv7emNoneEabihf
        | Thumbv7mNoneEabi
        | Thumbv8mBaseNoneEabi
        | Thumbv8mMainNoneEabi
        | Thumbv8mMainNoneEabihf => true,
        _ => false,
    }
}

pub fn min_zero_width_bitfield_alignment(target: Target) -> Option<u64> {
    use Target::*;
    match target {
        AvrUnknownUnknown => Some(8),
        Armv7AppleIos | Armv7sAppleIos => Some(32),
        _ => None,
    }
}

pub fn builtin_type_layout(target: Target, b: BuiltinType) -> TypeLayout {
    use BuiltinType::*;
    use Target::*;

    match target {
        AvrUnknownUnknown => return avr_builtin_type_layout(b),
        Msp430NoneElf => return msp430_builtin_type_layout(b),
        _ => {}
    }

    // See test case 0013.
    let (size_bytes, alignment_bytes) = match b {
        Unit => (0, 1),
        Char | SignedChar | UnsignedChar | Bool | U8 | I8 => (1, 1),
        Short | UnsignedShort | U16 | I16 => (2, 2),
        Float | F32 | Int | UnsignedInt | U32 | I32 => (4, 4),
        Double | F64 | LongLong | UnsignedLongLong | U64 | I64 => match target {
            I386AppleIos | I686LinuxAndroid | I686UnknownFreebsd | I686UnknownHaiku
            | I686UnknownNetbsdelf | I686UnknownOpenbsd | Armv7AppleIos | Armv7sAppleIos
            | I586UnknownLinuxGnu | I586UnknownLinuxMusl | I686UnknownLinuxGnu
            | I686UnknownLinuxMusl | I686AppleMacosx => (8, 4),
            _ => (8, 8),
        },
        // See test case 0050.
        U128 | I128 => match target {
            S390xUnknownLinuxGnu => (16, 8),
            _ => (16, 16),
        },
        Long | UnsignedLong => match target {
            | Aarch64PcWindowsMsvc
            | Armebv7rUnknownNoneEabi
            | Armebv7rUnknownNoneEabihf
            | ArmLinuxAndroideabi
            | ArmUnknownLinuxGnueabi
            | ArmUnknownLinuxGnueabihf
            | Armv4tUnknownLinuxGnueabi
            | Armv5teUnknownLinuxGnueabi
            | Armv5teUnknownLinuxUclibcgnueabi
            | Armv6UnknownFreebsdGnueabihf
            | Armv6UnknownNetbsdelfEabihf
            | Armv7aNoneEabi
            | Armv7aNoneEabihf
            | Armv7AppleIos
            | Armv7NoneLinuxAndroid
            | Armv7rUnknownNoneEabi
            | Armv7rUnknownNoneEabihf
            | Armv7sAppleIos
            | Armv7UnknownFreebsdGnueabihf
            | Armv7UnknownLinuxGnueabi
            | Armv7UnknownLinuxGnueabihf
            | Armv7UnknownNetbsdelfEabihf
            | AvrUnknownUnknown
            | HexagonUnknownLinuxMusl
            | I386AppleIos
            | I586PcWindowsMsvc
            | I586UnknownLinuxGnu
            | I586UnknownLinuxMusl
            | I686AppleMacosx
            | I686LinuxAndroid
            | I686PcWindowsGnu
            | I686PcWindowsMsvc
            | I686UnknownFreebsd
            | I686UnknownHaiku
            | I686UnknownLinuxGnu
            | I686UnknownLinuxMusl
            | I686UnknownNetbsdelf
            | I686UnknownOpenbsd
            | I686UnknownWindows
            | MipselSonyPsp
            | MipselUnknownLinuxGnu
            | MipselUnknownLinuxMusl
            | MipselUnknownLinuxUclibc
            | MipselUnknownNone
            | Mipsisa32r6elUnknownLinuxGnu
            | Mipsisa32r6UnknownLinuxGnu
            | MipsUnknownLinuxGnu
            | MipsUnknownLinuxMusl
            | MipsUnknownLinuxUclibc
            | Msp430NoneElf
            | PowerpcUnknownLinuxGnu
            | PowerpcUnknownLinuxGnuspe
            | PowerpcUnknownLinuxMusl
            | PowerpcUnknownNetbsd
            | Riscv32
            | Riscv32UnknownLinuxGnu
            | SparcUnknownLinuxGnu
            | Thumbv4tNoneEabi
            | Thumbv6mNoneEabi
            | Thumbv7aPcWindowsMsvc
            | Thumbv7emNoneEabi
            | Thumbv7emNoneEabihf
            | Thumbv7mNoneEabi
            | Thumbv8mBaseNoneEabi
            | Thumbv8mMainNoneEabi
            | Thumbv8mMainNoneEabihf
            | Wasm32UnknownEmscripten
            | Wasm32UnknownUnknown
            | Wasm32Wasi
            | X86_64PcWindowsGnu
            | X86_64PcWindowsMsvc
            | X86_64UnknownWindows => (4, 4),
            _ => (8, 8),
        },
        Pointer => match target {
            | Armebv7rUnknownNoneEabi
            | Armebv7rUnknownNoneEabihf
            | ArmLinuxAndroideabi
            | ArmUnknownLinuxGnueabi
            | ArmUnknownLinuxGnueabihf
            | Armv4tUnknownLinuxGnueabi
            | Armv5teUnknownLinuxGnueabi
            | Armv5teUnknownLinuxUclibcgnueabi
            | Armv6UnknownFreebsdGnueabihf
            | Armv6UnknownNetbsdelfEabihf
            | Armv7aNoneEabi
            | Armv7aNoneEabihf
            | Armv7AppleIos
            | Armv7NoneLinuxAndroid
            | Armv7rUnknownNoneEabi
            | Armv7rUnknownNoneEabihf
            | Armv7sAppleIos
            | Armv7UnknownFreebsdGnueabihf
            | Armv7UnknownLinuxGnueabi
            | Armv7UnknownLinuxGnueabihf
            | Armv7UnknownNetbsdelfEabihf
            | AvrUnknownUnknown
            | HexagonUnknownLinuxMusl
            | I386AppleIos
            | I586PcWindowsMsvc
            | I586UnknownLinuxGnu
            | I586UnknownLinuxMusl
            | I686AppleMacosx
            | I686LinuxAndroid
            | I686PcWindowsGnu
            | I686PcWindowsMsvc
            | I686UnknownFreebsd
            | I686UnknownHaiku
            | I686UnknownLinuxGnu
            | I686UnknownLinuxMusl
            | I686UnknownNetbsdelf
            | I686UnknownOpenbsd
            | I686UnknownWindows
            | MipselSonyPsp
            | MipselUnknownLinuxGnu
            | MipselUnknownLinuxMusl
            | MipselUnknownLinuxUclibc
            | MipselUnknownNone
            | Mipsisa32r6elUnknownLinuxGnu
            | Mipsisa32r6UnknownLinuxGnu
            | MipsUnknownLinuxGnu
            | MipsUnknownLinuxMusl
            | MipsUnknownLinuxUclibc
            | Msp430NoneElf
            | PowerpcUnknownLinuxGnu
            | PowerpcUnknownLinuxGnuspe
            | PowerpcUnknownLinuxMusl
            | PowerpcUnknownNetbsd
            | Riscv32
            | Riscv32UnknownLinuxGnu
            | SparcUnknownLinuxGnu
            | Thumbv4tNoneEabi
            | Thumbv6mNoneEabi
            | Thumbv7aPcWindowsMsvc
            | Thumbv7emNoneEabi
            | Thumbv7emNoneEabihf
            | Thumbv7mNoneEabi
            | Thumbv8mBaseNoneEabi
            | Thumbv8mMainNoneEabi
            | Thumbv8mMainNoneEabihf
            | Wasm32UnknownEmscripten
            | Wasm32UnknownUnknown
            | Wasm32Wasi => (4, 4),
            _ => (8, 8),
        },
    };
    TypeLayout {
        size_bits: size_bytes * BITS_PER_BYTE,
        pointer_alignment_bits: alignment_bytes * BITS_PER_BYTE,
        field_alignment_bits: alignment_bytes * BITS_PER_BYTE,
        required_alignment_bits: BITS_PER_BYTE,
    }
}

pub fn avr_builtin_type_layout(b: BuiltinType) -> TypeLayout {
    use BuiltinType::*;

    let size_bytes = match b {
        Unit => 0,
        Char | SignedChar | UnsignedChar | Bool | U8 | I8 => 1,
        Pointer | Short | UnsignedShort | U16 | I16 | Int | UnsignedInt => 2,
        Long | UnsignedLong | Double | Float | F32 | U32 | I32 => 4,
        F64 | LongLong | UnsignedLongLong | U64 | I64 => 8,
        U128 | I128 => 16,
    };
    TypeLayout {
        size_bits: size_bytes * BITS_PER_BYTE,
        pointer_alignment_bits: BITS_PER_BYTE,
        field_alignment_bits: BITS_PER_BYTE,
        required_alignment_bits: BITS_PER_BYTE,
    }
}

pub fn msp430_builtin_type_layout(b: BuiltinType) -> TypeLayout {
    use BuiltinType::*;

    let mut alignment = 2;

    let size_bytes = match b {
        Unit => 0,
        Char | SignedChar | UnsignedChar | Bool | U8 | I8 => {
            alignment = 1;
            1
        }
        Pointer | Short | UnsignedShort | U16 | I16 | Int | UnsignedInt => 2,
        Long | UnsignedLong | Float | F32 | U32 | I32 => 4,
        Double | F64 | LongLong | UnsignedLongLong | U64 | I64 => 8,
        U128 | I128 => 16,
    };
    TypeLayout {
        size_bits: size_bytes * BITS_PER_BYTE,
        pointer_alignment_bits: alignment * BITS_PER_BYTE,
        field_alignment_bits: alignment * BITS_PER_BYTE,
        required_alignment_bits: BITS_PER_BYTE,
    }
}

pub fn ignore_non_zero_sized_bitfield_type_alignment(target: Target) -> bool {
    use Target::*;
    match target {
        AvrUnknownUnknown | Armv7AppleIos | Armv7sAppleIos => true,
        _ => false,
    }
}

pub fn ignore_zero_sized_bitfield_type_alignmont(target: Target) -> bool {
    use Target::*;
    match target {
        AvrUnknownUnknown => true,
        _ => false,
    }
}

pub fn pack_all_enums(target: Target) -> bool {
    use Target::*;
    match target {
        HexagonUnknownLinuxMusl => true,
        _ => false,
    }
}

pub fn default_aligned_alignment(target: Target) -> u64 {
    use Target::*;
    match target {
        | AvrUnknownUnknown => 8,
        | ArmUnknownLinuxGnueabi
        | ArmUnknownLinuxGnueabihf
        | Armebv7rUnknownNoneEabi
        | Armebv7rUnknownNoneEabihf
        | Armv4tUnknownLinuxGnueabi
        | Armv5teUnknownLinuxGnueabi
        | Armv5teUnknownLinuxUclibcgnueabi
        | Armv6UnknownFreebsdGnueabihf
        | Armv6UnknownNetbsdelfEabihf
        | Armv7UnknownFreebsdGnueabihf
        | Armv7UnknownLinuxGnueabi
        | Armv7UnknownLinuxGnueabihf
        | Armv7UnknownNetbsdelfEabihf
        | Armv7aNoneEabi
        | Armv7aNoneEabihf
        | Armv7rUnknownNoneEabi
        | Armv7rUnknownNoneEabihf
        | MipsUnknownLinuxGnu
        | MipsUnknownLinuxMusl
        | MipsUnknownLinuxUclibc
        | MipselUnknownLinuxGnu
        | MipselUnknownLinuxMusl
        | MipselUnknownLinuxUclibc
        | Mipsisa32r6UnknownLinuxGnu
        | Mipsisa32r6elUnknownLinuxGnu
        | S390xUnknownLinuxGnu
        | SparcUnknownLinuxGnu
        | Thumbv4tNoneEabi
        | Thumbv6mNoneEabi
        | Thumbv7aPcWindowsMsvc
        | Thumbv7emNoneEabi
        | Thumbv7emNoneEabihf
        | Thumbv7mNoneEabi
        | Thumbv8mBaseNoneEabi
        | Thumbv8mMainNoneEabi
        | Thumbv8mMainNoneEabihf => 64,
        _ => 128,
    }
}
