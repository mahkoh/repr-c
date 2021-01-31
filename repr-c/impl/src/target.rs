use Target::*;

macro_rules! targets {
    ($($name:ident,)*) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum Target {
            $($name),*
        }

        pub const TARGETS: &[Target] = &[
            $($name),*
        ];
    }
}

targets! {
    Aarch64Fuchsia,
    Aarch64LinuxAndroid,
    Aarch64PcWindowsMsvc,
    Aarch64UnknownFreebsd,
    Aarch64UnknownHermit,
    Aarch64UnknownLinuxGnu,
    Aarch64UnknownLinuxMusl,
    Aarch64UnknownNetbsd,
    Aarch64UnknownNone,
    Aarch64UnknownOpenbsd,
    Aarch64UnknownRedox,
    Arm64AppleIos,
    Arm64AppleIosMacabi,
    Arm64AppleTvos,
    Armebv7rUnknownNoneEabi,
    Armebv7rUnknownNoneEabihf,
    ArmLinuxAndroideabi,
    ArmUnknownLinuxGnueabi,
    ArmUnknownLinuxGnueabihf,
    Armv4tUnknownLinuxGnueabi,
    Armv5teUnknownLinuxGnueabi,
    Armv5teUnknownLinuxUclibcgnueabi,
    Armv6UnknownFreebsdGnueabihf,
    Armv6UnknownNetbsdelfEabihf,
    Armv7aNoneEabi,
    Armv7aNoneEabihf,
    Armv7AppleIos,
    Armv7NoneLinuxAndroid,
    Armv7rUnknownNoneEabi,
    Armv7rUnknownNoneEabihf,
    Armv7sAppleIos,
    Armv7UnknownFreebsdGnueabihf,
    Armv7UnknownLinuxGnueabi,
    Armv7UnknownLinuxGnueabihf,
    Armv7UnknownNetbsdelfEabihf,
    AvrUnknownUnknown,
    HexagonUnknownLinuxMusl,
    I386AppleIos,
    I586PcWindowsMsvc,
    I586UnknownLinuxGnu,
    I586UnknownLinuxMusl,
    I686LinuxAndroid,
    I686PcWindowsGnu,
    I686PcWindowsMsvc,
    I686UnknownFreebsd,
    I686UnknownHaiku,
    I686UnknownLinuxGnu,
    I686UnknownLinuxMusl,
    I686UnknownNetbsdelf,
    I686UnknownOpenbsd,
    I686UnknownWindows,
    Mips64elUnknownLinuxGnuabi64,
    Mips64elUnknownLinuxMusl,
    Mips64UnknownLinuxGnuabi64,
    Mips64UnknownLinuxMusl,
    MipselSonyPsp,
    MipselUnknownLinuxGnu,
    MipselUnknownLinuxMusl,
    MipselUnknownLinuxUclibc,
    MipselUnknownNone,
    Mipsisa32r6elUnknownLinuxGnu,
    Mipsisa32r6UnknownLinuxGnu,
    Mipsisa64r6elUnknownLinuxGnuabi64,
    Mipsisa64r6UnknownLinuxGnuabi64,
    MipsUnknownLinuxGnu,
    MipsUnknownLinuxMusl,
    MipsUnknownLinuxUclibc,
    Msp430NoneElf,
    Powerpc64leUnknownLinuxGnu,
    Powerpc64leUnknownLinuxMusl,
    Powerpc64UnknownFreebsd,
    Powerpc64UnknownLinuxGnu,
    Powerpc64UnknownLinuxMusl,
    PowerpcUnknownLinuxGnu,
    PowerpcUnknownLinuxGnuspe,
    PowerpcUnknownLinuxMusl,
    PowerpcUnknownNetbsd,
    Riscv32,
    Riscv32UnknownLinuxGnu,
    Riscv64,
    Riscv64UnknownLinuxGnu,
    S390xUnknownLinuxGnu,
    Sparc64UnknownLinuxGnu,
    Sparc64UnknownNetbsd,
    Sparc64UnknownOpenbsd,
    SparcUnknownLinuxGnu,
    Sparcv9SunSolaris,
    Thumbv4tNoneEabi,
    Thumbv6mNoneEabi,
    Thumbv7aPcWindowsMsvc,
    Thumbv7emNoneEabi,
    Thumbv7emNoneEabihf,
    Thumbv7mNoneEabi,
    Thumbv8mBaseNoneEabi,
    Thumbv8mMainNoneEabi,
    Thumbv8mMainNoneEabihf,
    Wasm32UnknownEmscripten,
    Wasm32UnknownUnknown,
    Wasm32Wasi,
    X86_64AppleIos,
    X86_64AppleIos13_0Macabi,
    X86_64AppleTvos,
    X86_64Elf,
    X86_64Fuchsia,
    X86_64LinuxAndroid,
    X86_64PcSolaris,
    X86_64PcWindowsGnu,
    X86_64PcWindowsMsvc,
    X86_64RumprunNetbsd,
    X86_64UnknownDragonfly,
    X86_64UnknownFreebsd,
    X86_64UnknownHaiku,
    X86_64UnknownHermit,
    X86_64UnknownL4reUclibc,
    X86_64UnknownLinuxGnu,
    X86_64UnknownLinuxGnux32,
    X86_64UnknownLinuxMusl,
    X86_64UnknownNetbsd,
    X86_64UnknownOpenbsd,
    X86_64UnknownRedox,
    X86_64UnknownWindows,
}

impl Target {
    pub fn name(self) -> &'static str {
        match self {
            Aarch64Fuchsia => "aarch64-fuchsia",
            Aarch64LinuxAndroid => "aarch64-linux-android",
            Aarch64PcWindowsMsvc => "aarch64-pc-windows-msvc",
            Aarch64UnknownFreebsd => "aarch64-unknown-freebsd",
            Aarch64UnknownHermit => "aarch64-unknown-hermit",
            Aarch64UnknownLinuxGnu => "aarch64-unknown-linux-gnu",
            Aarch64UnknownLinuxMusl => "aarch64-unknown-linux-musl",
            Aarch64UnknownNetbsd => "aarch64-unknown-netbsd",
            Aarch64UnknownNone => "aarch64-unknown-none",
            Aarch64UnknownOpenbsd => "aarch64-unknown-openbsd",
            Aarch64UnknownRedox => "aarch64-unknown-redox",
            Arm64AppleIos => "arm64-apple-ios",
            Arm64AppleIosMacabi => "arm64-apple-ios-macabi",
            Arm64AppleTvos => "arm64-apple-tvos",
            Armebv7rUnknownNoneEabi => "armebv7r-unknown-none-eabi",
            Armebv7rUnknownNoneEabihf => "armebv7r-unknown-none-eabihf",
            ArmLinuxAndroideabi => "arm-linux-androideabi",
            ArmUnknownLinuxGnueabi => "arm-unknown-linux-gnueabi",
            ArmUnknownLinuxGnueabihf => "arm-unknown-linux-gnueabihf",
            Armv4tUnknownLinuxGnueabi => "armv4t-unknown-linux-gnueabi",
            Armv5teUnknownLinuxGnueabi => "armv5te-unknown-linux-gnueabi",
            Armv5teUnknownLinuxUclibcgnueabi => "armv5te-unknown-linux-uclibcgnueabi",
            Armv6UnknownFreebsdGnueabihf => "armv6-unknown-freebsd-gnueabihf",
            Armv6UnknownNetbsdelfEabihf => "armv6-unknown-netbsdelf-eabihf",
            Armv7aNoneEabi => "armv7a-none-eabi",
            Armv7aNoneEabihf => "armv7a-none-eabihf",
            Armv7AppleIos => "armv7-apple-ios",
            Armv7NoneLinuxAndroid => "armv7-none-linux-android",
            Armv7rUnknownNoneEabi => "armv7r-unknown-none-eabi",
            Armv7rUnknownNoneEabihf => "armv7r-unknown-none-eabihf",
            Armv7sAppleIos => "armv7s-apple-ios",
            Armv7UnknownFreebsdGnueabihf => "armv7-unknown-freebsd-gnueabihf",
            Armv7UnknownLinuxGnueabi => "armv7-unknown-linux-gnueabi",
            Armv7UnknownLinuxGnueabihf => "armv7-unknown-linux-gnueabihf",
            Armv7UnknownNetbsdelfEabihf => "armv7-unknown-netbsdelf-eabihf",
            AvrUnknownUnknown => "avr-unknown-unknown",
            HexagonUnknownLinuxMusl => "hexagon-unknown-linux-musl",
            I386AppleIos => "i386-apple-ios",
            I586PcWindowsMsvc => "i586-pc-windows-msvc",
            I586UnknownLinuxGnu => "i586-unknown-linux-gnu",
            I586UnknownLinuxMusl => "i586-unknown-linux-musl",
            I686LinuxAndroid => "i686-linux-android",
            I686PcWindowsGnu => "i686-pc-windows-gnu",
            I686PcWindowsMsvc => "i686-pc-windows-msvc",
            I686UnknownFreebsd => "i686-unknown-freebsd",
            I686UnknownHaiku => "i686-unknown-haiku",
            I686UnknownLinuxGnu => "i686-unknown-linux-gnu",
            I686UnknownLinuxMusl => "i686-unknown-linux-musl",
            I686UnknownNetbsdelf => "i686-unknown-netbsdelf",
            I686UnknownOpenbsd => "i686-unknown-openbsd",
            I686UnknownWindows => "i686-unknown-windows",
            Mips64elUnknownLinuxGnuabi64 => "mips64el-unknown-linux-gnuabi64",
            Mips64elUnknownLinuxMusl => "mips64el-unknown-linux-musl",
            Mips64UnknownLinuxGnuabi64 => "mips64-unknown-linux-gnuabi64",
            Mips64UnknownLinuxMusl => "mips64-unknown-linux-musl",
            MipselSonyPsp => "mipsel-sony-psp",
            MipselUnknownLinuxGnu => "mipsel-unknown-linux-gnu",
            MipselUnknownLinuxMusl => "mipsel-unknown-linux-musl",
            MipselUnknownLinuxUclibc => "mipsel-unknown-linux-uclibc",
            MipselUnknownNone => "mipsel-unknown-none",
            Mipsisa32r6elUnknownLinuxGnu => "mipsisa32r6el-unknown-linux-gnu",
            Mipsisa32r6UnknownLinuxGnu => "mipsisa32r6-unknown-linux-gnu",
            Mipsisa64r6elUnknownLinuxGnuabi64 => "mipsisa64r6el-unknown-linux-gnuabi64",
            Mipsisa64r6UnknownLinuxGnuabi64 => "mipsisa64r6-unknown-linux-gnuabi64",
            MipsUnknownLinuxGnu => "mips-unknown-linux-gnu",
            MipsUnknownLinuxMusl => "mips-unknown-linux-musl",
            MipsUnknownLinuxUclibc => "mips-unknown-linux-uclibc",
            Msp430NoneElf => "msp430-none-elf",
            Powerpc64leUnknownLinuxGnu => "powerpc64le-unknown-linux-gnu",
            Powerpc64leUnknownLinuxMusl => "powerpc64le-unknown-linux-musl",
            Powerpc64UnknownFreebsd => "powerpc64-unknown-freebsd",
            Powerpc64UnknownLinuxGnu => "powerpc64-unknown-linux-gnu",
            Powerpc64UnknownLinuxMusl => "powerpc64-unknown-linux-musl",
            PowerpcUnknownLinuxGnu => "powerpc-unknown-linux-gnu",
            PowerpcUnknownLinuxGnuspe => "powerpc-unknown-linux-gnuspe",
            PowerpcUnknownLinuxMusl => "powerpc-unknown-linux-musl",
            PowerpcUnknownNetbsd => "powerpc-unknown-netbsd",
            Riscv32 => "riscv32",
            Riscv32UnknownLinuxGnu => "riscv32-unknown-linux-gnu",
            Riscv64 => "riscv64",
            Riscv64UnknownLinuxGnu => "riscv64-unknown-linux-gnu",
            S390xUnknownLinuxGnu => "s390x-unknown-linux-gnu",
            Sparc64UnknownLinuxGnu => "sparc64-unknown-linux-gnu",
            Sparc64UnknownNetbsd => "sparc64-unknown-netbsd",
            Sparc64UnknownOpenbsd => "sparc64-unknown-openbsd",
            SparcUnknownLinuxGnu => "sparc-unknown-linux-gnu",
            Sparcv9SunSolaris => "sparcv9-sun-solaris",
            Thumbv4tNoneEabi => "thumbv4t-none-eabi",
            Thumbv6mNoneEabi => "thumbv6m-none-eabi",
            Thumbv7aPcWindowsMsvc => "thumbv7a-pc-windows-msvc",
            Thumbv7emNoneEabihf => "thumbv7em-none-eabihf",
            Thumbv7emNoneEabi => "thumbv7em-none-eabi",
            Thumbv7mNoneEabi => "thumbv7m-none-eabi",
            Thumbv8mBaseNoneEabi => "thumbv8m.base-none-eabi",
            Thumbv8mMainNoneEabihf => "thumbv8m.main-none-eabihf",
            Thumbv8mMainNoneEabi => "thumbv8m.main-none-eabi",
            Wasm32UnknownEmscripten => "wasm32-unknown-emscripten",
            Wasm32UnknownUnknown => "wasm32-unknown-unknown",
            Wasm32Wasi => "wasm32-wasi",
            X86_64AppleIos13_0Macabi => "x86_64-apple-ios13.0-macabi",
            X86_64AppleIos => "x86_64-apple-ios",
            X86_64AppleTvos => "x86_64-apple-tvos",
            X86_64Elf => "x86_64-elf",
            X86_64Fuchsia => "x86_64-fuchsia",
            X86_64LinuxAndroid => "x86_64-linux-android",
            X86_64PcSolaris => "x86_64-pc-solaris",
            X86_64PcWindowsGnu => "x86_64-pc-windows-gnu",
            X86_64PcWindowsMsvc => "x86_64-pc-windows-msvc",
            X86_64RumprunNetbsd => "x86_64-rumprun-netbsd",
            X86_64UnknownDragonfly => "x86_64-unknown-dragonfly",
            X86_64UnknownFreebsd => "x86_64-unknown-freebsd",
            X86_64UnknownHaiku => "x86_64-unknown-haiku",
            X86_64UnknownHermit => "x86_64-unknown-hermit",
            X86_64UnknownL4reUclibc => "x86_64-unknown-l4re-uclibc",
            X86_64UnknownLinuxGnux32 => "x86_64-unknown-linux-gnux32",
            X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu",
            X86_64UnknownLinuxMusl => "x86_64-unknown-linux-musl",
            X86_64UnknownNetbsd => "x86_64-unknown-netbsd",
            X86_64UnknownOpenbsd => "x86_64-unknown-openbsd",
            X86_64UnknownRedox => "x86_64-unknown-redox",
            X86_64UnknownWindows => "x86_64-unknown-windows",
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
        | Aarch64PcWindowsMsvc
        | I586PcWindowsMsvc
        | I686PcWindowsMsvc
        | I686UnknownWindows
        | Thumbv7aPcWindowsMsvc
        | X86_64PcWindowsMsvc
        | X86_64UnknownWindows => Compiler::Msvc,
        | Aarch64UnknownLinuxGnu
        | Aarch64UnknownLinuxMusl
        | ArmUnknownLinuxGnueabi
        | ArmUnknownLinuxGnueabihf
        | Armv4tUnknownLinuxGnueabi
        | Armv5teUnknownLinuxGnueabi
        | Armv5teUnknownLinuxUclibcgnueabi
        | Armv7UnknownLinuxGnueabi
        | Armv7UnknownLinuxGnueabihf
        | AvrUnknownUnknown
        | I586UnknownLinuxGnu
        | I586UnknownLinuxMusl
        | I686PcWindowsGnu
        | I686UnknownLinuxGnu
        | I686UnknownLinuxMusl
        | Mips64elUnknownLinuxGnuabi64
        | Mips64elUnknownLinuxMusl
        | Mips64UnknownLinuxGnuabi64
        | Mips64UnknownLinuxMusl
        | MipselUnknownLinuxGnu
        | MipselUnknownLinuxMusl
        | MipselUnknownLinuxUclibc
        | Mipsisa32r6elUnknownLinuxGnu
        | Mipsisa32r6UnknownLinuxGnu
        | Mipsisa64r6elUnknownLinuxGnuabi64
        | Mipsisa64r6UnknownLinuxGnuabi64
        | MipsUnknownLinuxGnu
        | MipsUnknownLinuxMusl
        | MipsUnknownLinuxUclibc
        | Powerpc64leUnknownLinuxGnu
        | Powerpc64leUnknownLinuxMusl
        | Powerpc64UnknownLinuxGnu
        | Powerpc64UnknownLinuxMusl
        | PowerpcUnknownLinuxGnu
        | PowerpcUnknownLinuxMusl
        | Riscv32UnknownLinuxGnu
        | Riscv64UnknownLinuxGnu
        | S390xUnknownLinuxGnu
        | Sparc64UnknownLinuxGnu
        | SparcUnknownLinuxGnu
        | X86_64PcWindowsGnu
        | X86_64UnknownLinuxGnu
        | X86_64UnknownLinuxGnux32
        | X86_64UnknownLinuxMusl => Compiler::Gcc,
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
        | Armebv7rUnknownNoneEabi
        | Armebv7rUnknownNoneEabihf
        | ArmLinuxAndroideabi
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
        | Armv7UnknownNetbsdelfEabihf
        | HexagonUnknownLinuxMusl
        | I386AppleIos
        | I686LinuxAndroid
        | I686UnknownFreebsd
        | I686UnknownHaiku
        | I686UnknownNetbsdelf
        | I686UnknownOpenbsd
        | MipselSonyPsp
        | MipselUnknownNone
        | Msp430NoneElf
        | Powerpc64UnknownFreebsd
        | PowerpcUnknownLinuxGnuspe
        | PowerpcUnknownNetbsd
        | Riscv32
        | Riscv64
        | Sparc64UnknownNetbsd
        | Sparc64UnknownOpenbsd
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
        | X86_64AppleIos
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
        | X86_64UnknownRedox => Compiler::Clang,
    }
}
