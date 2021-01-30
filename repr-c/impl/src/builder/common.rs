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

pub fn resolve_typedefs(mut ty: &Type<()>) -> &Type<()> {
    while let TypeVariant::Typedef(t) = &ty.variant {
        ty = t;
    }
    ty
}

pub fn unnamed_field_affects_record_alignment(target: Target) -> bool {
    use Target::*;
    match target {
        Aarch64Fuchsia
        | Aarch64LinuxAndroid
        | Aarch64UnknownFreebsd
        | Aarch64UnknownHermit
        | Aarch64UnknownNetbsd
        | Aarch64UnknownNone
        | Aarch64UnknownOpenbsd
        | AvrUnknownUnknown
        | Aarch64UnknownRedox => true,
        _ => false,
    }
}

pub fn builtin_type_layout(target: Target, b: BuiltinType) -> TypeLayout {
    use BuiltinType::*;
    use Target::*;

    match target {
        AvrUnknownUnknown => return avr_builtin_type_layout(b),
        Msp430NoneElf => return msp430_builtin_type_layout(b),
        _ => { },
    }

    let (size_bytes, alignment_bytes) = match b {
        Unit => (0, 1),
        Char | SignedChar | UnsignedChar | Bool | U8 | I8 => (1, 1),
        Short | UnsignedShort | U16 | I16 => (2, 2),
        Float | F32 | Int | UnsignedInt | U32 | I32 => (4, 4),
        Double | F64 | LongLong | UnsignedLongLong | U64 | I64 => (8, 8),
        U128 | I128 => (16, 16),
        Long | UnsignedLong => match target {
            Aarch64PcWindowsMsvc
            | I586PcWindowsMsvc
            | I686PcWindowsMsvc
            | Thumbv7aPcWindowsMsvc
            | AvrUnknownUnknown
            | I386AppleIos
            | I686LinuxAndroid
            | I686UnknownFreebsd
            | I686UnknownHaiku
            | I686UnknownNetbsdelf
            | I686UnknownOpenbsd
            | MipselSonyPsp
            | MipselUnknownNone
            | Msp430NoneElf
            | PowerpcUnknownNetbsd
            | Riscv32
            | Sparcv9SunSolaris
            | Thumbv4tNoneEabi
            | Thumbv6mNoneEabi
            | Thumbv7emNoneEabi
            | Thumbv7emNoneEabihf
            | Thumbv7mNoneEabi
            | Thumbv8mBaseNoneEabi
            | Thumbv8mMainNoneEabi
            | Thumbv8mMainNoneEabihf
            | Wasm32UnknownEmscripten
            | Wasm32UnknownUnknown
            | Wasm32Wasi
            | X86_64PcWindowsMsvc => (4, 4),
            X86_64AppleIos
            | X86_64AppleIos13_0Macabi
            | X86_64AppleTvos
            | X86_64Elf
            | X86_64Fuchsia
            | X86_64LinuxAndroid
            | X86_64PcSolaris
            | X86_64RumprunNetbsd
            | X86_64UnknownDragonfly
            | X86_64UnknownFreebsd
            | X86_64UnknownHaiku
            | X86_64UnknownHermit
            | X86_64UnknownL4reUclibc
            | X86_64UnknownNetbsd
            | X86_64UnknownOpenbsd
            | X86_64UnknownRedox
            | Aarch64Fuchsia
            | Aarch64LinuxAndroid
            | Aarch64UnknownFreebsd
            | Aarch64UnknownHermit
            | Aarch64UnknownNetbsd
            | Aarch64UnknownNone
            | Aarch64UnknownOpenbsd
            | Aarch64UnknownRedox
            | Arm64AppleIos
            | Arm64AppleIosMacabi
            | Arm64AppleTvos
            | X86_64UnknownLinuxGnu => (8, 8),
        },
        Pointer => match target {
            I586PcWindowsMsvc
            | I686PcWindowsMsvc
            | Thumbv7aPcWindowsMsvc
            | AvrUnknownUnknown
            | I386AppleIos
            | I686LinuxAndroid
            | I686UnknownFreebsd
            | I686UnknownHaiku
            | I686UnknownNetbsdelf
            | I686UnknownOpenbsd
            | MipselSonyPsp
            | MipselUnknownNone
            | Msp430NoneElf
            | PowerpcUnknownNetbsd
            | Riscv32
            | Sparcv9SunSolaris
            | Thumbv4tNoneEabi
            | Thumbv6mNoneEabi
            | Thumbv7emNoneEabi
            | Thumbv7emNoneEabihf
            | Thumbv7mNoneEabi
            | Thumbv8mBaseNoneEabi
            | Thumbv8mMainNoneEabi
            | Thumbv8mMainNoneEabihf
            | Wasm32UnknownEmscripten
            | Wasm32UnknownUnknown
            | Wasm32Wasi => (4, 4),
            X86_64AppleIos
            | X86_64AppleIos13_0Macabi
            | X86_64AppleTvos
            | X86_64Elf
            | X86_64Fuchsia
            | X86_64LinuxAndroid
            | X86_64PcSolaris
            | X86_64RumprunNetbsd
            | X86_64UnknownDragonfly
            | X86_64UnknownFreebsd
            | X86_64UnknownHaiku
            | X86_64UnknownHermit
            | X86_64UnknownL4reUclibc
            | X86_64UnknownNetbsd
            | X86_64UnknownOpenbsd
            | X86_64UnknownRedox
            | Aarch64PcWindowsMsvc
            | X86_64PcWindowsMsvc
            | Aarch64Fuchsia
            | Aarch64LinuxAndroid
            | Aarch64UnknownFreebsd
            | Aarch64UnknownHermit
            | Aarch64UnknownNetbsd
            | Aarch64UnknownNone
            | Aarch64UnknownOpenbsd
            | Aarch64UnknownRedox
            | Arm64AppleIos
            | Arm64AppleIosMacabi
            | Arm64AppleTvos
            | X86_64UnknownLinuxGnu => (8, 8),
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
        Short | UnsignedShort | U16 | I16 | Int | UnsignedInt => 2,
        Long | UnsignedLong | Double | Float | F32 | U32 | I32 => 4,
        F64 | LongLong | UnsignedLongLong | U64 | I64 => 8,
        U128 | I128 => 16,
        Pointer => 2,
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
        },
        Short | UnsignedShort | U16 | I16 | Int | UnsignedInt => 2,
        Long | UnsignedLong | Float | F32 | U32 | I32 => 4,
        Double | F64 | LongLong | UnsignedLongLong | U64 | I64 => 8,
        U128 | I128 => 16,
        Pointer => 2,
    };
    TypeLayout {
        size_bits: size_bytes * BITS_PER_BYTE,
        pointer_alignment_bits: alignment * BITS_PER_BYTE,
        field_alignment_bits: alignment * BITS_PER_BYTE,
        required_alignment_bits: BITS_PER_BYTE,
    }
}

pub fn bitfield_type_alignment_matters(target: Target) -> bool {
    use Target::*;
    match target {
        AvrUnknownUnknown => false,
        _ => true,
    }
}
