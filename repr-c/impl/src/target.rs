use Target::*;

macro_rules! targets {
    ($($name:ident,)*) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum Target {
            $($name),*
        }

        pub const TARGETS: &[Target] = &[
            $(Target::$name),*
        ];
    }
}

targets! {
    Aarch64PcWindowsMsvc,
    I586PcWindowsMsvc,
    I686PcWindowsMsvc,
    Thumbv7aPcWindowsMsvc,
    X86_64PcWindowsMsvc,
    X86_64UnknownLinuxGnu,
    X86_64AppleIos,
    X86_64AppleIos13_0Macabi,
    X86_64AppleTvos,
    X86_64Elf,
    X86_64Fuchsia,
    X86_64LinuxAndroid,
    X86_64PcSolaris,
    X86_64RumprunNetbsd,
    X86_64UnknownDragonfly,
    X86_64UnknownFreebsd,
    X86_64UnknownHaiku,
    X86_64UnknownHermit,
    X86_64UnknownL4reUclibc,
    X86_64UnknownNetbsd,
    X86_64UnknownOpenbsd,
    X86_64UnknownRedox,
    Aarch64Fuchsia,
    Aarch64LinuxAndroid,
    Aarch64UnknownFreebsd,
    Aarch64UnknownHermit,
    Aarch64UnknownNetbsd,
    Aarch64UnknownNone,
    Aarch64UnknownOpenbsd,
    Aarch64UnknownRedox,
    Arm64AppleIos,
    Arm64AppleIosMacabi,
    Arm64AppleTvos,
    AvrUnknownUnknown,
    I386AppleIos,
    I686LinuxAndroid,
    I686UnknownFreebsd,
    I686UnknownHaiku,
    I686UnknownNetbsdelf,
    I686UnknownOpenbsd,
    MipselSonyPsp,
    MipselUnknownNone,
    Msp430NoneElf,
    PowerpcUnknownNetbsd,
    Riscv32,
    Sparcv9SunSolaris,
    Thumbv4tNoneEabi,
    Thumbv6mNoneEabi,
    Thumbv7emNoneEabi,
    Thumbv7emNoneEabihf,
    Thumbv7mNoneEabi,
    Thumbv8mBaseNoneEabi,
    Thumbv8mMainNoneEabi,
    Thumbv8mMainNoneEabihf,
    Wasm32UnknownEmscripten,
    Wasm32UnknownUnknown,
    Wasm32Wasi,
}

impl Target {
    pub fn name(self) -> &'static str {
        match self {
            Aarch64PcWindowsMsvc => "aarch64-pc-windows-msvc",
            I586PcWindowsMsvc => "i586-pc-windows-msvc",
            I686PcWindowsMsvc => "i686-pc-windows-msvc",
            Thumbv7aPcWindowsMsvc => "thumbv7a-pc-windows-msvc",
            X86_64PcWindowsMsvc => "x86_64-pc-windows-msvc",
            X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu",
            X86_64AppleIos => "x86_64-apple-ios",
            X86_64AppleIos13_0Macabi => "x86_64-apple-ios13.0-macabi",
            X86_64AppleTvos => "x86_64-apple-tvos",
            X86_64Elf => "x86_64-elf",
            X86_64Fuchsia => "x86_64-fuchsia",
            X86_64LinuxAndroid => "x86_64-linux-android",
            X86_64PcSolaris => "x86_64-pc-solaris",
            X86_64RumprunNetbsd => "x86_64-rumprun-netbsd",
            X86_64UnknownDragonfly => "x86_64-unknown-dragonfly",
            X86_64UnknownFreebsd => "x86_64-unknown-freebsd",
            X86_64UnknownHaiku => "x86_64-unknown-haiku",
            X86_64UnknownHermit => "x86_64-unknown-hermit",
            X86_64UnknownL4reUclibc => "x86_64-unknown-l4re-uclibc",
            X86_64UnknownNetbsd => "x86_64-unknown-netbsd",
            X86_64UnknownOpenbsd => "x86_64-unknown-openbsd",
            X86_64UnknownRedox => "x86_64-unknown-redox",
            Aarch64Fuchsia => "aarch64-fuchsia",
            Aarch64LinuxAndroid => "aarch64-linux-android",
            Aarch64UnknownFreebsd => "aarch64-unknown-freebsd",
            Aarch64UnknownHermit => "aarch64-unknown-hermit",
            Aarch64UnknownNetbsd => "aarch64-unknown-netbsd",
            Aarch64UnknownNone => "aarch64-unknown-none",
            Aarch64UnknownOpenbsd => "aarch64-unknown-openbsd",
            Aarch64UnknownRedox => "aarch64-unknown-redox",
            Arm64AppleIos => "arm64-apple-ios",
            Arm64AppleIosMacabi => "arm64-apple-ios-macabi",
            Arm64AppleTvos => "arm64-apple-tvos",
            AvrUnknownUnknown => "avr-unknown-unknown",
            I386AppleIos => "i386-apple-ios",
            I686LinuxAndroid => "i686-linux-android",
            I686UnknownFreebsd => "i686-unknown-freebsd",
            I686UnknownHaiku => "i686-unknown-haiku",
            I686UnknownNetbsdelf => "i686-unknown-netbsdelf",
            I686UnknownOpenbsd => "i686-unknown-openbsd",
            MipselSonyPsp => "mipsel-sony-psp",
            MipselUnknownNone => "mipsel-unknown-none",
            Msp430NoneElf => "msp430-none-elf",
            PowerpcUnknownNetbsd => "powerpc-unknown-netbsd",
            Riscv32 => "riscv32",
            Sparcv9SunSolaris => "sparcv9-sun-solaris",
            Thumbv4tNoneEabi => "thumbv4t-none-eabi",
            Thumbv6mNoneEabi => "thumbv6m-none-eabi",
            Thumbv7emNoneEabi => "thumbv7em-none-eabi",
            Thumbv7emNoneEabihf => "thumbv7em-none-eabihf",
            Thumbv7mNoneEabi => "thumbv7m-none-eabi",
            Thumbv8mBaseNoneEabi => "thumbv8m.base-none-eabi",
            Thumbv8mMainNoneEabi => "thumbv8m.main-none-eabi",
            Thumbv8mMainNoneEabihf => "thumbv8m.main-none-eabihf",
            Wasm32UnknownEmscripten => "wasm32-unknown-emscripten",
            Wasm32UnknownUnknown => "wasm32-unknown-unknown",
            Wasm32Wasi => "wasm32-wasi",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Compiler {
    Msvc,
    Gcc,
    Clang,
}

pub fn system_compiler(target: Target) -> Compiler {
    match target {
        Aarch64PcWindowsMsvc
        | I586PcWindowsMsvc
        | I686PcWindowsMsvc
        | Thumbv7aPcWindowsMsvc
        | X86_64PcWindowsMsvc => Compiler::Msvc,
        AvrUnknownUnknown
        | X86_64UnknownLinuxGnu => Compiler::Gcc,
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
        | X86_64UnknownRedox => Compiler::Clang,
    }
}
