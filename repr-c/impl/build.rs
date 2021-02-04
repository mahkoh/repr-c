use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::{env, io};

const CLANG: &str = "clang";
const MSVC: &str = "msvc";
const GCC: &str = "gcc";

const AARCH64_APPLE_DARWIN: &str = "aarch64-apple-darwin";
const AARCH64_APPLE_IOS_MACABI: &str = "aarch64-apple-ios-macabi";
const AARCH64_APPLE_IOS: &str = "aarch64-apple-ios";
const AARCH64_APPLE_MACOSX: &str = "aarch64-apple-macosx";
const AARCH64_APPLE_TVOS: &str = "aarch64-apple-tvos";
const AARCH64_FUCHSIA: &str = "aarch64-fuchsia";
const AARCH64_LINUX_ANDROID: &str = "aarch64-linux-android";
const AARCH64_PC_WINDOWS_MSVC: &str = "aarch64-pc-windows-msvc";
const AARCH64_UNKNOWN_FREEBSD: &str = "aarch64-unknown-freebsd";
const AARCH64_UNKNOWN_HERMIT: &str = "aarch64-unknown-hermit";
const AARCH64_UNKNOWN_LINUX_GNU: &str = "aarch64-unknown-linux-gnu";
const AARCH64_UNKNOWN_LINUX_MUSL: &str = "aarch64-unknown-linux-musl";
const AARCH64_UNKNOWN_NETBSD: &str = "aarch64-unknown-netbsd";
const AARCH64_UNKNOWN_NONE_SOFTFLOAT: &str = "aarch64-unknown-none-softfloat";
const AARCH64_UNKNOWN_NONE: &str = "aarch64-unknown-none";
const AARCH64_UNKNOWN_OPENBSD: &str = "aarch64-unknown-openbsd";
const AARCH64_UNKNOWN_REDOX: &str = "aarch64-unknown-redox";
const AARCH64_UWP_WINDOWS_MSVC: &str = "aarch64-uwp-windows-msvc";
const AARCH64_WRS_VXWORKS: &str = "aarch64-wrs-vxworks";
const ARM64_APPLE_IOS_MACABI: &str = "arm64-apple-ios-macabi";
const ARM64_APPLE_IOS: &str = "arm64-apple-ios";
const ARM64_APPLE_TVOS: &str = "arm64-apple-tvos";
const ARMEBV7R_NONE_EABIHF: &str = "armebv7r-none-eabihf";
const ARMEBV7R_NONE_EABI: &str = "armebv7r-none-eabi";
const ARMEBV7R_UNKNOWN_NONE_EABIHF: &str = "armebv7r-unknown-none-eabihf";
const ARMEBV7R_UNKNOWN_NONE_EABI: &str = "armebv7r-unknown-none-eabi";
const ARM_LINUX_ANDROIDEABI: &str = "arm-linux-androideabi";
const ARM_UNKNOWN_LINUX_GNUEABIHF: &str = "arm-unknown-linux-gnueabihf";
const ARM_UNKNOWN_LINUX_GNUEABI: &str = "arm-unknown-linux-gnueabi";
const ARM_UNKNOWN_LINUX_MUSLEABIHF: &str = "arm-unknown-linux-musleabihf";
const ARM_UNKNOWN_LINUX_MUSLEABI: &str = "arm-unknown-linux-musleabi";
const ARMV4T_UNKNOWN_LINUX_GNUEABI: &str = "armv4t-unknown-linux-gnueabi";
const ARMV5TE_UNKNOWN_LINUX_GNUEABI: &str = "armv5te-unknown-linux-gnueabi";
const ARMV5TE_UNKNOWN_LINUX_MUSLEABI: &str = "armv5te-unknown-linux-musleabi";
const ARMV5TE_UNKNOWN_LINUX_UCLIBCEABI: &str = "armv5te-unknown-linux-uclibceabi";
const ARMV5TE_UNKNOWN_LINUX_UCLIBCGNUEABI: &str = "armv5te-unknown-linux-uclibcgnueabi";
const ARMV6_UNKNOWN_FREEBSD_GNUEABIHF: &str = "armv6-unknown-freebsd-gnueabihf";
const ARMV6_UNKNOWN_FREEBSD: &str = "armv6-unknown-freebsd";
const ARMV6_UNKNOWN_NETBSD_EABIHF: &str = "armv6-unknown-netbsd-eabihf";
const ARMV6_UNKNOWN_NETBSDELF_EABIHF: &str = "armv6-unknown-netbsdelf-eabihf";
const ARMV7A_NONE_EABIHF: &str = "armv7a-none-eabihf";
const ARMV7A_NONE_EABI: &str = "armv7a-none-eabi";
const ARMV7_APPLE_IOS: &str = "armv7-apple-ios";
const ARMV7_LINUX_ANDROIDEABI: &str = "armv7-linux-androideabi";
const ARMV7_NONE_LINUX_ANDROID: &str = "armv7-none-linux-android";
const ARMV7R_NONE_EABIHF: &str = "armv7r-none-eabihf";
const ARMV7R_NONE_EABI: &str = "armv7r-none-eabi";
const ARMV7R_UNKNOWN_NONE_EABIHF: &str = "armv7r-unknown-none-eabihf";
const ARMV7R_UNKNOWN_NONE_EABI: &str = "armv7r-unknown-none-eabi";
const ARMV7S_APPLE_IOS: &str = "armv7s-apple-ios";
const ARMV7_UNKNOWN_FREEBSD_GNUEABIHF: &str = "armv7-unknown-freebsd-gnueabihf";
const ARMV7_UNKNOWN_FREEBSD: &str = "armv7-unknown-freebsd";
const ARMV7_UNKNOWN_LINUX_GNUEABIHF: &str = "armv7-unknown-linux-gnueabihf";
const ARMV7_UNKNOWN_LINUX_GNUEABI: &str = "armv7-unknown-linux-gnueabi";
const ARMV7_UNKNOWN_LINUX_MUSLEABIHF: &str = "armv7-unknown-linux-musleabihf";
const ARMV7_UNKNOWN_LINUX_MUSLEABI: &str = "armv7-unknown-linux-musleabi";
const ARMV7_UNKNOWN_NETBSD_EABIHF: &str = "armv7-unknown-netbsd-eabihf";
const ARMV7_UNKNOWN_NETBSDELF_EABIHF: &str = "armv7-unknown-netbsdelf-eabihf";
const ARMV7_WRS_VXWORKS_EABIHF: &str = "armv7-wrs-vxworks-eabihf";
const ASMJS_UNKNOWN_EMSCRIPTEN: &str = "asmjs-unknown-emscripten";
const AVR_UNKNOWN_GNU_ATMEGA328: &str = "avr-unknown-gnu-atmega328";
const AVR_UNKNOWN_UNKNOWN: &str = "avr-unknown-unknown";
const HEXAGON_UNKNOWN_LINUX_MUSL: &str = "hexagon-unknown-linux-musl";
const I386_APPLE_IOS: &str = "i386-apple-ios";
const I586_PC_WINDOWS_MSVC: &str = "i586-pc-windows-msvc";
const I586_UNKNOWN_LINUX_GNU: &str = "i586-unknown-linux-gnu";
const I586_UNKNOWN_LINUX_MUSL: &str = "i586-unknown-linux-musl";
const I686_APPLE_DARWIN: &str = "i686-apple-darwin";
const I686_APPLE_MACOSX: &str = "i686-apple-macosx";
const I686_LINUX_ANDROID: &str = "i686-linux-android";
const I686_PC_WINDOWS_GNU: &str = "i686-pc-windows-gnu";
const I686_PC_WINDOWS_MSVC: &str = "i686-pc-windows-msvc";
const I686_UNKNOWN_FREEBSD: &str = "i686-unknown-freebsd";
const I686_UNKNOWN_HAIKU: &str = "i686-unknown-haiku";
const I686_UNKNOWN_LINUX_GNU: &str = "i686-unknown-linux-gnu";
const I686_UNKNOWN_LINUX_MUSL: &str = "i686-unknown-linux-musl";
const I686_UNKNOWN_NETBSDELF: &str = "i686-unknown-netbsdelf";
const I686_UNKNOWN_NETBSD: &str = "i686-unknown-netbsd";
const I686_UNKNOWN_OPENBSD: &str = "i686-unknown-openbsd";
const I686_UNKNOWN_UEFI: &str = "i686-unknown-uefi";
const I686_UNKNOWN_WINDOWS: &str = "i686-unknown-windows";
const I686_UWP_WINDOWS_GNU: &str = "i686-uwp-windows-gnu";
const I686_UWP_WINDOWS_MSVC: &str = "i686-uwp-windows-msvc";
const I686_WRS_VXWORKS: &str = "i686-wrs-vxworks";
const MIPS64EL_UNKNOWN_LINUX_GNUABI64: &str = "mips64el-unknown-linux-gnuabi64";
const MIPS64EL_UNKNOWN_LINUX_MUSLABI64: &str = "mips64el-unknown-linux-muslabi64";
const MIPS64EL_UNKNOWN_LINUX_MUSL: &str = "mips64el-unknown-linux-musl";
const MIPS64_UNKNOWN_LINUX_GNUABI64: &str = "mips64-unknown-linux-gnuabi64";
const MIPS64_UNKNOWN_LINUX_MUSLABI64: &str = "mips64-unknown-linux-muslabi64";
const MIPS64_UNKNOWN_LINUX_MUSL: &str = "mips64-unknown-linux-musl";
const MIPSEL_SONY_PSP: &str = "mipsel-sony-psp";
const MIPSEL_UNKNOWN_LINUX_GNU: &str = "mipsel-unknown-linux-gnu";
const MIPSEL_UNKNOWN_LINUX_MUSL: &str = "mipsel-unknown-linux-musl";
const MIPSEL_UNKNOWN_LINUX_UCLIBC: &str = "mipsel-unknown-linux-uclibc";
const MIPSEL_UNKNOWN_NONE: &str = "mipsel-unknown-none";
const MIPSISA32R6EL_UNKNOWN_LINUX_GNU: &str = "mipsisa32r6el-unknown-linux-gnu";
const MIPSISA32R6_UNKNOWN_LINUX_GNU: &str = "mipsisa32r6-unknown-linux-gnu";
const MIPSISA64R6EL_UNKNOWN_LINUX_GNUABI64: &str = "mipsisa64r6el-unknown-linux-gnuabi64";
const MIPSISA64R6_UNKNOWN_LINUX_GNUABI64: &str = "mipsisa64r6-unknown-linux-gnuabi64";
const MIPS_UNKNOWN_LINUX_GNU: &str = "mips-unknown-linux-gnu";
const MIPS_UNKNOWN_LINUX_MUSL: &str = "mips-unknown-linux-musl";
const MIPS_UNKNOWN_LINUX_UCLIBC: &str = "mips-unknown-linux-uclibc";
const MSP430_NONE_ELF: &str = "msp430-none-elf";
const POWERPC64LE_UNKNOWN_LINUX_GNU: &str = "powerpc64le-unknown-linux-gnu";
const POWERPC64LE_UNKNOWN_LINUX_MUSL: &str = "powerpc64le-unknown-linux-musl";
const POWERPC64_UNKNOWN_FREEBSD: &str = "powerpc64-unknown-freebsd";
const POWERPC64_UNKNOWN_LINUX_GNU: &str = "powerpc64-unknown-linux-gnu";
const POWERPC64_UNKNOWN_LINUX_MUSL: &str = "powerpc64-unknown-linux-musl";
const POWERPC64_WRS_VXWORKS: &str = "powerpc64-wrs-vxworks";
const POWERPC_UNKNOWN_LINUX_GNUSPE: &str = "powerpc-unknown-linux-gnuspe";
const POWERPC_UNKNOWN_LINUX_GNU: &str = "powerpc-unknown-linux-gnu";
const POWERPC_UNKNOWN_LINUX_MUSL: &str = "powerpc-unknown-linux-musl";
const POWERPC_UNKNOWN_NETBSD: &str = "powerpc-unknown-netbsd";
const POWERPC_WRS_VXWORKS_SPE: &str = "powerpc-wrs-vxworks-spe";
const POWERPC_WRS_VXWORKS: &str = "powerpc-wrs-vxworks";
const RISCV32GC_UNKNOWN_LINUX_GNU: &str = "riscv32gc-unknown-linux-gnu";
const RISCV32IMAC_UNKNOWN_NONE_ELF: &str = "riscv32imac-unknown-none-elf";
const RISCV32IMC_UNKNOWN_NONE_ELF: &str = "riscv32imc-unknown-none-elf";
const RISCV32I_UNKNOWN_NONE_ELF: &str = "riscv32i-unknown-none-elf";
const RISCV32: &str = "riscv32";
const RISCV32_UNKNOWN_LINUX_GNU: &str = "riscv32-unknown-linux-gnu";
const RISCV64GC_UNKNOWN_LINUX_GNU: &str = "riscv64gc-unknown-linux-gnu";
const RISCV64GC_UNKNOWN_NONE_ELF: &str = "riscv64gc-unknown-none-elf";
const RISCV64IMAC_UNKNOWN_NONE_ELF: &str = "riscv64imac-unknown-none-elf";
const RISCV64: &str = "riscv64";
const RISCV64_UNKNOWN_LINUX_GNU: &str = "riscv64-unknown-linux-gnu";
const S390X_UNKNOWN_LINUX_GNU: &str = "s390x-unknown-linux-gnu";
const SPARC64_UNKNOWN_LINUX_GNU: &str = "sparc64-unknown-linux-gnu";
const SPARC64_UNKNOWN_NETBSD: &str = "sparc64-unknown-netbsd";
const SPARC64_UNKNOWN_OPENBSD: &str = "sparc64-unknown-openbsd";
const SPARC_UNKNOWN_LINUX_GNU: &str = "sparc-unknown-linux-gnu";
const SPARCV9_SUN_SOLARIS: &str = "sparcv9-sun-solaris";
const THUMBV4T_NONE_EABI: &str = "thumbv4t-none-eabi";
const THUMBV6M_NONE_EABI: &str = "thumbv6m-none-eabi";
const THUMBV7A_PC_WINDOWS_MSVC: &str = "thumbv7a-pc-windows-msvc";
const THUMBV7A_UWP_WINDOWS_MSVC: &str = "thumbv7a-uwp-windows-msvc";
const THUMBV7EM_NONE_EABIHF: &str = "thumbv7em-none-eabihf";
const THUMBV7EM_NONE_EABI: &str = "thumbv7em-none-eabi";
const THUMBV7M_NONE_EABI: &str = "thumbv7m-none-eabi";
const THUMBV7NEON_LINUX_ANDROIDEABI: &str = "thumbv7neon-linux-androideabi";
const THUMBV7NEON_UNKNOWN_LINUX_GNUEABIHF: &str = "thumbv7neon-unknown-linux-gnueabihf";
const THUMBV7NEON_UNKNOWN_LINUX_MUSLEABIHF: &str = "thumbv7neon-unknown-linux-musleabihf";
const THUMBV8M_BASE_NONE_EABI: &str = "thumbv8m.base-none-eabi";
const THUMBV8M_MAIN_NONE_EABIHF: &str = "thumbv8m.main-none-eabihf";
const THUMBV8M_MAIN_NONE_EABI: &str = "thumbv8m.main-none-eabi";
const WASM32_UNKNOWN_EMSCRIPTEN: &str = "wasm32-unknown-emscripten";
const WASM32_UNKNOWN_UNKNOWN: &str = "wasm32-unknown-unknown";
const WASM32_WASI: &str = "wasm32-wasi";
const X86_64_APPLE_DARWIN: &str = "x86_64-apple-darwin";
const X86_64_APPLE_IOS_MACABI: &str = "x86_64-apple-ios-macabi";
const X86_64_APPLE_IOS: &str = "x86_64-apple-ios";
const X86_64_APPLE_MACOSX: &str = "x86_64-apple-macosx";
const X86_64_APPLE_TVOS: &str = "x86_64-apple-tvos";
const X86_64_ELF: &str = "x86_64-elf";
const X86_64_FORTANIX_UNKNOWN_SGX: &str = "x86_64-fortanix-unknown-sgx";
const X86_64_FUCHSIA: &str = "x86_64-fuchsia";
const X86_64_LINUX_ANDROID: &str = "x86_64-linux-android";
const X86_64_LINUX_KERNEL: &str = "x86_64-linux-kernel";
const X86_64_PC_SOLARIS: &str = "x86_64-pc-solaris";
const X86_64_PC_WINDOWS_GNU: &str = "x86_64-pc-windows-gnu";
const X86_64_PC_WINDOWS_MSVC: &str = "x86_64-pc-windows-msvc";
const X86_64_RUMPRUN_NETBSD: &str = "x86_64-rumprun-netbsd";
const X86_64_SUN_SOLARIS: &str = "x86_64-sun-solaris";
const X86_64_UNKNOWN_DRAGONFLY: &str = "x86_64-unknown-dragonfly";
const X86_64_UNKNOWN_FREEBSD: &str = "x86_64-unknown-freebsd";
const X86_64_UNKNOWN_HAIKU: &str = "x86_64-unknown-haiku";
const X86_64_UNKNOWN_HERMIT_KERNEL: &str = "x86_64-unknown-hermit-kernel";
const X86_64_UNKNOWN_HERMIT: &str = "x86_64-unknown-hermit";
const X86_64_UNKNOWN_ILLUMOS: &str = "x86_64-unknown-illumos";
const X86_64_UNKNOWN_L4RE_UCLIBC: &str = "x86_64-unknown-l4re-uclibc";
const X86_64_UNKNOWN_LINUX_GNU: &str = "x86_64-unknown-linux-gnu";
const X86_64_UNKNOWN_LINUX_GNUX32: &str = "x86_64-unknown-linux-gnux32";
const X86_64_UNKNOWN_LINUX_MUSL: &str = "x86_64-unknown-linux-musl";
const X86_64_UNKNOWN_NETBSD: &str = "x86_64-unknown-netbsd";
const X86_64_UNKNOWN_OPENBSD: &str = "x86_64-unknown-openbsd";
const X86_64_UNKNOWN_REDOX: &str = "x86_64-unknown-redox";
const X86_64_UNKNOWN_UEFI: &str = "x86_64-unknown-uefi";
const X86_64_UNKNOWN_WINDOWS: &str = "x86_64-unknown-windows";
const X86_64_UWP_WINDOWS_GNU: &str = "x86_64-uwp-windows-gnu";
const X86_64_UWP_WINDOWS_MSVC: &str = "x86_64-uwp-windows-msvc";
const X86_64_WRS_VXWORKS: &str = "x86_64-wrs-vxworks";

const TARGETS: &[(&str, &str)] = &[
    (AARCH64_APPLE_MACOSX, CLANG),
    (AARCH64_FUCHSIA, CLANG),
    (AARCH64_LINUX_ANDROID, CLANG),
    (AARCH64_PC_WINDOWS_MSVC, MSVC),
    (AARCH64_UNKNOWN_FREEBSD, CLANG),
    (AARCH64_UNKNOWN_HERMIT, CLANG),
    (AARCH64_UNKNOWN_LINUX_GNU, GCC),
    (AARCH64_UNKNOWN_LINUX_MUSL, GCC),
    (AARCH64_UNKNOWN_NETBSD, CLANG),
    (AARCH64_UNKNOWN_NONE, CLANG),
    (AARCH64_UNKNOWN_OPENBSD, CLANG),
    (AARCH64_UNKNOWN_REDOX, CLANG),
    (ARM64_APPLE_IOS, CLANG),
    (ARM64_APPLE_IOS_MACABI, CLANG),
    (ARM64_APPLE_TVOS, CLANG),
    (ARMEBV7R_UNKNOWN_NONE_EABI, CLANG),
    (ARMEBV7R_UNKNOWN_NONE_EABIHF, CLANG),
    (ARM_LINUX_ANDROIDEABI, CLANG),
    (ARM_UNKNOWN_LINUX_GNUEABI, GCC),
    (ARM_UNKNOWN_LINUX_GNUEABIHF, GCC),
    (ARMV4T_UNKNOWN_LINUX_GNUEABI, GCC),
    (ARMV5TE_UNKNOWN_LINUX_GNUEABI, GCC),
    (ARMV5TE_UNKNOWN_LINUX_UCLIBCGNUEABI, GCC),
    (ARMV6_UNKNOWN_FREEBSD_GNUEABIHF, CLANG),
    (ARMV6_UNKNOWN_NETBSDELF_EABIHF, CLANG),
    (ARMV7A_NONE_EABI, CLANG),
    (ARMV7A_NONE_EABIHF, CLANG),
    (ARMV7_APPLE_IOS, CLANG),
    (ARMV7_NONE_LINUX_ANDROID, CLANG),
    (ARMV7R_UNKNOWN_NONE_EABI, CLANG),
    (ARMV7R_UNKNOWN_NONE_EABIHF, CLANG),
    (ARMV7S_APPLE_IOS, CLANG),
    (ARMV7_UNKNOWN_FREEBSD_GNUEABIHF, CLANG),
    (ARMV7_UNKNOWN_LINUX_GNUEABI, GCC),
    (ARMV7_UNKNOWN_LINUX_GNUEABIHF, GCC),
    (ARMV7_UNKNOWN_NETBSDELF_EABIHF, CLANG),
    (AVR_UNKNOWN_UNKNOWN, GCC),
    (HEXAGON_UNKNOWN_LINUX_MUSL, CLANG),
    (I386_APPLE_IOS, CLANG),
    (I586_PC_WINDOWS_MSVC, MSVC),
    (I586_UNKNOWN_LINUX_GNU, GCC),
    (I586_UNKNOWN_LINUX_MUSL, GCC),
    (I686_APPLE_MACOSX, CLANG),
    (I686_LINUX_ANDROID, CLANG),
    (I686_PC_WINDOWS_GNU, GCC),
    (I686_PC_WINDOWS_MSVC, MSVC),
    (I686_UNKNOWN_FREEBSD, CLANG),
    (I686_UNKNOWN_HAIKU, CLANG),
    (I686_UNKNOWN_LINUX_GNU, GCC),
    (I686_UNKNOWN_LINUX_MUSL, GCC),
    (I686_UNKNOWN_NETBSDELF, CLANG),
    (I686_UNKNOWN_OPENBSD, CLANG),
    (I686_UNKNOWN_WINDOWS, MSVC),
    (MIPS64EL_UNKNOWN_LINUX_GNUABI64, GCC),
    (MIPS64EL_UNKNOWN_LINUX_MUSL, GCC),
    (MIPS64_UNKNOWN_LINUX_GNUABI64, GCC),
    (MIPS64_UNKNOWN_LINUX_MUSL, GCC),
    (MIPSEL_SONY_PSP, CLANG),
    (MIPSEL_UNKNOWN_LINUX_GNU, GCC),
    (MIPSEL_UNKNOWN_LINUX_MUSL, GCC),
    (MIPSEL_UNKNOWN_LINUX_UCLIBC, GCC),
    (MIPSEL_UNKNOWN_NONE, CLANG),
    (MIPSISA32R6EL_UNKNOWN_LINUX_GNU, GCC),
    (MIPSISA32R6_UNKNOWN_LINUX_GNU, GCC),
    (MIPSISA64R6EL_UNKNOWN_LINUX_GNUABI64, GCC),
    (MIPSISA64R6_UNKNOWN_LINUX_GNUABI64, GCC),
    (MIPS_UNKNOWN_LINUX_GNU, GCC),
    (MIPS_UNKNOWN_LINUX_MUSL, GCC),
    (MIPS_UNKNOWN_LINUX_UCLIBC, GCC),
    (MSP430_NONE_ELF, CLANG),
    (POWERPC64LE_UNKNOWN_LINUX_GNU, GCC),
    (POWERPC64LE_UNKNOWN_LINUX_MUSL, GCC),
    (POWERPC64_UNKNOWN_FREEBSD, CLANG),
    (POWERPC64_UNKNOWN_LINUX_GNU, GCC),
    (POWERPC64_UNKNOWN_LINUX_MUSL, GCC),
    (POWERPC_UNKNOWN_LINUX_GNU, GCC),
    (POWERPC_UNKNOWN_LINUX_GNUSPE, CLANG),
    (POWERPC_UNKNOWN_LINUX_MUSL, GCC),
    (POWERPC_UNKNOWN_NETBSD, CLANG),
    (RISCV32, CLANG),
    (RISCV32_UNKNOWN_LINUX_GNU, GCC),
    (RISCV64, CLANG),
    (RISCV64_UNKNOWN_LINUX_GNU, GCC),
    (S390X_UNKNOWN_LINUX_GNU, GCC),
    (SPARC64_UNKNOWN_LINUX_GNU, GCC),
    (SPARC64_UNKNOWN_NETBSD, CLANG),
    (SPARC64_UNKNOWN_OPENBSD, CLANG),
    (SPARC_UNKNOWN_LINUX_GNU, GCC),
    (SPARCV9_SUN_SOLARIS, CLANG),
    (THUMBV4T_NONE_EABI, CLANG),
    (THUMBV6M_NONE_EABI, CLANG),
    (THUMBV7A_PC_WINDOWS_MSVC, MSVC),
    (THUMBV7EM_NONE_EABIHF, CLANG),
    (THUMBV7EM_NONE_EABI, CLANG),
    (THUMBV7M_NONE_EABI, CLANG),
    (THUMBV8M_BASE_NONE_EABI, CLANG),
    (THUMBV8M_MAIN_NONE_EABIHF, CLANG),
    (THUMBV8M_MAIN_NONE_EABI, CLANG),
    (WASM32_UNKNOWN_EMSCRIPTEN, CLANG),
    (WASM32_UNKNOWN_UNKNOWN, CLANG),
    (WASM32_WASI, CLANG),
    (X86_64_APPLE_IOS_MACABI, CLANG),
    (X86_64_APPLE_IOS, CLANG),
    (X86_64_APPLE_MACOSX, CLANG),
    (X86_64_APPLE_TVOS, CLANG),
    (X86_64_ELF, CLANG),
    (X86_64_FUCHSIA, CLANG),
    (X86_64_LINUX_ANDROID, CLANG),
    (X86_64_PC_SOLARIS, CLANG),
    (X86_64_PC_WINDOWS_GNU, GCC),
    (X86_64_PC_WINDOWS_MSVC, MSVC),
    (X86_64_RUMPRUN_NETBSD, CLANG),
    (X86_64_UNKNOWN_DRAGONFLY, CLANG),
    (X86_64_UNKNOWN_FREEBSD, CLANG),
    (X86_64_UNKNOWN_HAIKU, CLANG),
    (X86_64_UNKNOWN_HERMIT, CLANG),
    (X86_64_UNKNOWN_L4RE_UCLIBC, CLANG),
    (X86_64_UNKNOWN_LINUX_GNUX32, GCC),
    (X86_64_UNKNOWN_LINUX_GNU, GCC),
    (X86_64_UNKNOWN_LINUX_MUSL, GCC),
    (X86_64_UNKNOWN_NETBSD, CLANG),
    (X86_64_UNKNOWN_OPENBSD, CLANG),
    (X86_64_UNKNOWN_REDOX, CLANG),
    (X86_64_UNKNOWN_WINDOWS, MSVC),
];

const RUST_TARGET_MAP: &[(&str, &str)] = &[
    (AARCH64_APPLE_DARWIN, AARCH64_APPLE_MACOSX),
    (AARCH64_APPLE_IOS, ARM64_APPLE_IOS),
    (AARCH64_APPLE_IOS_MACABI, ARM64_APPLE_IOS_MACABI),
    (AARCH64_APPLE_TVOS, ARM64_APPLE_TVOS),
    (AARCH64_FUCHSIA, AARCH64_FUCHSIA),
    (AARCH64_LINUX_ANDROID, AARCH64_LINUX_ANDROID),
    (AARCH64_PC_WINDOWS_MSVC, AARCH64_PC_WINDOWS_MSVC),
    (AARCH64_UNKNOWN_FREEBSD, AARCH64_UNKNOWN_FREEBSD),
    (AARCH64_UNKNOWN_HERMIT, AARCH64_UNKNOWN_HERMIT),
    (AARCH64_UNKNOWN_LINUX_GNU, AARCH64_UNKNOWN_LINUX_GNU),
    (AARCH64_UNKNOWN_LINUX_MUSL, AARCH64_UNKNOWN_LINUX_MUSL),
    (AARCH64_UNKNOWN_NETBSD, AARCH64_UNKNOWN_NETBSD),
    (AARCH64_UNKNOWN_NONE, AARCH64_UNKNOWN_NONE),
    (AARCH64_UNKNOWN_NONE_SOFTFLOAT, AARCH64_UNKNOWN_NONE),
    (AARCH64_UNKNOWN_OPENBSD, AARCH64_UNKNOWN_OPENBSD),
    (AARCH64_UNKNOWN_REDOX, AARCH64_UNKNOWN_REDOX),
    (AARCH64_UWP_WINDOWS_MSVC, AARCH64_PC_WINDOWS_MSVC),
    (AARCH64_WRS_VXWORKS, AARCH64_UNKNOWN_LINUX_GNU),
    (ARMEBV7R_NONE_EABI, ARMEBV7R_UNKNOWN_NONE_EABI),
    (ARMEBV7R_NONE_EABIHF, ARMEBV7R_UNKNOWN_NONE_EABIHF),
    (ARM_LINUX_ANDROIDEABI, ARM_LINUX_ANDROIDEABI),
    (ARM_UNKNOWN_LINUX_GNUEABI, ARM_UNKNOWN_LINUX_GNUEABI),
    (ARM_UNKNOWN_LINUX_GNUEABIHF, ARM_UNKNOWN_LINUX_GNUEABIHF),
    (ARM_UNKNOWN_LINUX_MUSLEABI, ARM_UNKNOWN_LINUX_GNUEABI),
    (ARM_UNKNOWN_LINUX_MUSLEABIHF, ARM_UNKNOWN_LINUX_GNUEABIHF),
    (ARMV4T_UNKNOWN_LINUX_GNUEABI, ARMV4T_UNKNOWN_LINUX_GNUEABI),
    (ARMV5TE_UNKNOWN_LINUX_GNUEABI, ARMV5TE_UNKNOWN_LINUX_GNUEABI),
    (
        ARMV5TE_UNKNOWN_LINUX_MUSLEABI,
        ARMV5TE_UNKNOWN_LINUX_GNUEABI,
    ),
    (
        ARMV5TE_UNKNOWN_LINUX_UCLIBCEABI,
        ARMV5TE_UNKNOWN_LINUX_UCLIBCGNUEABI,
    ),
    (ARMV6_UNKNOWN_FREEBSD, ARMV6_UNKNOWN_FREEBSD_GNUEABIHF),
    (ARMV6_UNKNOWN_NETBSD_EABIHF, ARMV6_UNKNOWN_NETBSDELF_EABIHF),
    (ARMV7A_NONE_EABI, ARMV7A_NONE_EABI),
    (ARMV7A_NONE_EABIHF, ARMV7A_NONE_EABIHF),
    (ARMV7_APPLE_IOS, ARMV7_APPLE_IOS),
    (ARMV7_LINUX_ANDROIDEABI, ARMV7_NONE_LINUX_ANDROID),
    (ARMV7R_NONE_EABI, ARMV7R_UNKNOWN_NONE_EABI),
    (ARMV7R_NONE_EABIHF, ARMV7R_UNKNOWN_NONE_EABIHF),
    (ARMV7S_APPLE_IOS, ARMV7S_APPLE_IOS),
    (ARMV7_UNKNOWN_FREEBSD, ARMV7_UNKNOWN_FREEBSD_GNUEABIHF),
    (ARMV7_UNKNOWN_LINUX_GNUEABI, ARMV7_UNKNOWN_LINUX_GNUEABI),
    (ARMV7_UNKNOWN_LINUX_GNUEABIHF, ARMV7_UNKNOWN_LINUX_GNUEABIHF),
    (ARMV7_UNKNOWN_LINUX_MUSLEABI, ARMV7_UNKNOWN_LINUX_GNUEABI),
    (
        ARMV7_UNKNOWN_LINUX_MUSLEABIHF,
        ARMV7_UNKNOWN_LINUX_GNUEABIHF,
    ),
    (ARMV7_UNKNOWN_NETBSD_EABIHF, ARMV7_UNKNOWN_NETBSDELF_EABIHF),
    (ARMV7_WRS_VXWORKS_EABIHF, ARMV7_UNKNOWN_LINUX_GNUEABIHF),
    (ASMJS_UNKNOWN_EMSCRIPTEN, WASM32_UNKNOWN_EMSCRIPTEN),
    (AVR_UNKNOWN_GNU_ATMEGA328, AVR_UNKNOWN_UNKNOWN),
    (HEXAGON_UNKNOWN_LINUX_MUSL, HEXAGON_UNKNOWN_LINUX_MUSL),
    (I386_APPLE_IOS, I386_APPLE_IOS),
    (I586_PC_WINDOWS_MSVC, I586_PC_WINDOWS_MSVC),
    (I586_UNKNOWN_LINUX_GNU, I586_UNKNOWN_LINUX_GNU),
    (I586_UNKNOWN_LINUX_MUSL, I586_UNKNOWN_LINUX_MUSL),
    (I686_APPLE_DARWIN, I686_APPLE_MACOSX),
    (I686_LINUX_ANDROID, I686_LINUX_ANDROID),
    (I686_PC_WINDOWS_GNU, I686_PC_WINDOWS_GNU),
    (I686_PC_WINDOWS_MSVC, I686_PC_WINDOWS_MSVC),
    (I686_UNKNOWN_FREEBSD, I686_UNKNOWN_FREEBSD),
    (I686_UNKNOWN_HAIKU, I686_UNKNOWN_HAIKU),
    (I686_UNKNOWN_LINUX_GNU, I686_UNKNOWN_LINUX_GNU),
    (I686_UNKNOWN_LINUX_MUSL, I686_UNKNOWN_LINUX_MUSL),
    (I686_UNKNOWN_NETBSD, I686_UNKNOWN_NETBSDELF),
    (I686_UNKNOWN_OPENBSD, I686_UNKNOWN_OPENBSD),
    (I686_UNKNOWN_UEFI, I686_UNKNOWN_WINDOWS),
    (I686_UWP_WINDOWS_GNU, I686_PC_WINDOWS_GNU),
    (I686_UWP_WINDOWS_MSVC, I686_PC_WINDOWS_MSVC),
    (I686_WRS_VXWORKS, I686_UNKNOWN_LINUX_GNU),
    (
        MIPS64EL_UNKNOWN_LINUX_GNUABI64,
        MIPS64EL_UNKNOWN_LINUX_GNUABI64,
    ),
    (
        MIPS64EL_UNKNOWN_LINUX_MUSLABI64,
        MIPS64EL_UNKNOWN_LINUX_MUSL,
    ),
    (MIPS64_UNKNOWN_LINUX_GNUABI64, MIPS64_UNKNOWN_LINUX_GNUABI64),
    (MIPS64_UNKNOWN_LINUX_MUSLABI64, MIPS64_UNKNOWN_LINUX_MUSL),
    (MIPSEL_SONY_PSP, MIPSEL_SONY_PSP),
    (MIPSEL_UNKNOWN_LINUX_GNU, MIPSEL_UNKNOWN_LINUX_GNU),
    (MIPSEL_UNKNOWN_LINUX_MUSL, MIPSEL_UNKNOWN_LINUX_MUSL),
    (MIPSEL_UNKNOWN_LINUX_UCLIBC, MIPSEL_UNKNOWN_LINUX_UCLIBC),
    (MIPSEL_UNKNOWN_NONE, MIPSEL_UNKNOWN_NONE),
    (
        MIPSISA32R6EL_UNKNOWN_LINUX_GNU,
        MIPSISA32R6EL_UNKNOWN_LINUX_GNU,
    ),
    (MIPSISA32R6_UNKNOWN_LINUX_GNU, MIPSISA32R6_UNKNOWN_LINUX_GNU),
    (
        MIPSISA64R6EL_UNKNOWN_LINUX_GNUABI64,
        MIPSISA64R6EL_UNKNOWN_LINUX_GNUABI64,
    ),
    (
        MIPSISA64R6_UNKNOWN_LINUX_GNUABI64,
        MIPSISA64R6_UNKNOWN_LINUX_GNUABI64,
    ),
    (MIPS_UNKNOWN_LINUX_GNU, MIPS_UNKNOWN_LINUX_GNU),
    (MIPS_UNKNOWN_LINUX_MUSL, MIPS_UNKNOWN_LINUX_MUSL),
    (MIPS_UNKNOWN_LINUX_UCLIBC, MIPS_UNKNOWN_LINUX_UCLIBC),
    (MSP430_NONE_ELF, MSP430_NONE_ELF),
    (POWERPC64LE_UNKNOWN_LINUX_GNU, POWERPC64LE_UNKNOWN_LINUX_GNU),
    (
        POWERPC64LE_UNKNOWN_LINUX_MUSL,
        POWERPC64LE_UNKNOWN_LINUX_MUSL,
    ),
    (POWERPC64_UNKNOWN_FREEBSD, POWERPC64_UNKNOWN_FREEBSD),
    (POWERPC64_UNKNOWN_LINUX_GNU, POWERPC64_UNKNOWN_LINUX_GNU),
    (POWERPC64_UNKNOWN_LINUX_MUSL, POWERPC64_UNKNOWN_LINUX_MUSL),
    (POWERPC64_WRS_VXWORKS, POWERPC64_UNKNOWN_LINUX_GNU),
    (POWERPC_UNKNOWN_LINUX_GNU, POWERPC_UNKNOWN_LINUX_GNU),
    (POWERPC_UNKNOWN_LINUX_GNUSPE, POWERPC_UNKNOWN_LINUX_GNUSPE),
    (POWERPC_UNKNOWN_LINUX_MUSL, POWERPC_UNKNOWN_LINUX_MUSL),
    (POWERPC_UNKNOWN_NETBSD, POWERPC_UNKNOWN_NETBSD),
    (POWERPC_WRS_VXWORKS, POWERPC_UNKNOWN_LINUX_GNU),
    (POWERPC_WRS_VXWORKS_SPE, POWERPC_UNKNOWN_LINUX_GNUSPE),
    (RISCV32GC_UNKNOWN_LINUX_GNU, RISCV32_UNKNOWN_LINUX_GNU),
    (RISCV32IMAC_UNKNOWN_NONE_ELF, RISCV32),
    (RISCV32IMC_UNKNOWN_NONE_ELF, RISCV32),
    (RISCV32I_UNKNOWN_NONE_ELF, RISCV32),
    (RISCV64GC_UNKNOWN_LINUX_GNU, RISCV64_UNKNOWN_LINUX_GNU),
    (RISCV64GC_UNKNOWN_NONE_ELF, RISCV64),
    (RISCV64IMAC_UNKNOWN_NONE_ELF, RISCV64),
    (S390X_UNKNOWN_LINUX_GNU, S390X_UNKNOWN_LINUX_GNU),
    (SPARC64_UNKNOWN_LINUX_GNU, SPARC64_UNKNOWN_LINUX_GNU),
    (SPARC64_UNKNOWN_NETBSD, SPARC64_UNKNOWN_NETBSD),
    (SPARC64_UNKNOWN_OPENBSD, SPARC64_UNKNOWN_OPENBSD),
    (SPARC_UNKNOWN_LINUX_GNU, SPARC_UNKNOWN_LINUX_GNU),
    (SPARCV9_SUN_SOLARIS, SPARCV9_SUN_SOLARIS),
    (THUMBV4T_NONE_EABI, THUMBV4T_NONE_EABI),
    (THUMBV6M_NONE_EABI, THUMBV6M_NONE_EABI),
    (THUMBV7A_PC_WINDOWS_MSVC, THUMBV7A_PC_WINDOWS_MSVC),
    (THUMBV7A_UWP_WINDOWS_MSVC, THUMBV7A_PC_WINDOWS_MSVC),
    (THUMBV7EM_NONE_EABI, THUMBV7EM_NONE_EABI),
    (THUMBV7EM_NONE_EABIHF, THUMBV7EM_NONE_EABIHF),
    (THUMBV7M_NONE_EABI, THUMBV7M_NONE_EABI),
    (THUMBV7NEON_LINUX_ANDROIDEABI, ARMV7_NONE_LINUX_ANDROID),
    (
        THUMBV7NEON_UNKNOWN_LINUX_GNUEABIHF,
        ARMV7_UNKNOWN_LINUX_GNUEABIHF,
    ),
    (
        THUMBV7NEON_UNKNOWN_LINUX_MUSLEABIHF,
        ARMV7_UNKNOWN_LINUX_GNUEABIHF,
    ),
    (THUMBV8M_BASE_NONE_EABI, THUMBV8M_BASE_NONE_EABI),
    (THUMBV8M_MAIN_NONE_EABI, THUMBV8M_MAIN_NONE_EABI),
    (THUMBV8M_MAIN_NONE_EABIHF, THUMBV8M_MAIN_NONE_EABIHF),
    (WASM32_UNKNOWN_EMSCRIPTEN, WASM32_UNKNOWN_EMSCRIPTEN),
    (WASM32_UNKNOWN_UNKNOWN, WASM32_UNKNOWN_UNKNOWN),
    (WASM32_WASI, WASM32_WASI),
    (X86_64_APPLE_DARWIN, X86_64_APPLE_MACOSX),
    (X86_64_APPLE_IOS, X86_64_APPLE_IOS),
    (X86_64_APPLE_IOS_MACABI, X86_64_APPLE_IOS_MACABI),
    (X86_64_APPLE_TVOS, X86_64_APPLE_TVOS),
    (X86_64_FORTANIX_UNKNOWN_SGX, X86_64_ELF),
    (X86_64_FUCHSIA, X86_64_FUCHSIA),
    (X86_64_LINUX_ANDROID, X86_64_LINUX_ANDROID),
    (X86_64_LINUX_KERNEL, X86_64_ELF),
    (X86_64_PC_SOLARIS, X86_64_PC_SOLARIS),
    (X86_64_PC_WINDOWS_GNU, X86_64_PC_WINDOWS_GNU),
    (X86_64_PC_WINDOWS_MSVC, X86_64_PC_WINDOWS_MSVC),
    (X86_64_RUMPRUN_NETBSD, X86_64_RUMPRUN_NETBSD),
    (X86_64_SUN_SOLARIS, X86_64_PC_SOLARIS),
    (X86_64_UNKNOWN_DRAGONFLY, X86_64_UNKNOWN_DRAGONFLY),
    (X86_64_UNKNOWN_FREEBSD, X86_64_UNKNOWN_FREEBSD),
    (X86_64_UNKNOWN_HAIKU, X86_64_UNKNOWN_HAIKU),
    (X86_64_UNKNOWN_HERMIT, X86_64_UNKNOWN_HERMIT),
    (X86_64_UNKNOWN_HERMIT_KERNEL, X86_64_UNKNOWN_HERMIT),
    (X86_64_UNKNOWN_ILLUMOS, X86_64_PC_SOLARIS),
    (X86_64_UNKNOWN_L4RE_UCLIBC, X86_64_UNKNOWN_L4RE_UCLIBC),
    (X86_64_UNKNOWN_LINUX_GNU, X86_64_UNKNOWN_LINUX_GNU),
    (X86_64_UNKNOWN_LINUX_GNUX32, X86_64_UNKNOWN_LINUX_GNUX32),
    (X86_64_UNKNOWN_LINUX_MUSL, X86_64_UNKNOWN_LINUX_MUSL),
    (X86_64_UNKNOWN_NETBSD, X86_64_UNKNOWN_NETBSD),
    (X86_64_UNKNOWN_OPENBSD, X86_64_UNKNOWN_OPENBSD),
    (X86_64_UNKNOWN_REDOX, X86_64_UNKNOWN_REDOX),
    (X86_64_UNKNOWN_UEFI, X86_64_UNKNOWN_WINDOWS),
    (X86_64_UWP_WINDOWS_GNU, X86_64_PC_WINDOWS_GNU),
    (X86_64_UWP_WINDOWS_MSVC, X86_64_PC_WINDOWS_MSVC),
    (X86_64_WRS_VXWORKS, X86_64_UNKNOWN_LINUX_GNU),
];

fn emit_host_target() -> io::Result<()> {
    let rust_target = env::var("TARGET").unwrap();
    let llvm_target = RUST_TARGET_MAP.iter().find(|t| t.0 == rust_target);

    let mut path = PathBuf::from(env::var("OUT_DIR").unwrap());
    path.push("host.rs");
    let mut file = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap(),
    );
    writeln!(file, "/// The target that this crate was compiled for.")?;
    writeln!(file, "///")?;
    writeln!(file, "/// `None` if that target is not implemented.")?;
    write!(file, "pub const HOST_TARGET: Option<Target> = ")?;
    match llvm_target {
        Some(t) => write!(file, "Some(Target::{})", to_camel_case(t.1))?,
        _ => write!(file, "None")?,
    }
    writeln!(file, ";")?;
    Ok(())
}

fn open(s: &str) -> io::Result<BufWriter<File>> {
    let mut path = PathBuf::from(env::var("OUT_DIR").unwrap());
    path.push(s);
    Ok(BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?,
    ))
}

fn to_camel_case(s: &str) -> String {
    let mut res = String::new();
    let mut uppercase = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            if uppercase {
                uppercase = false;
                res.push(c.to_ascii_uppercase());
            } else {
                res.push(c);
            }
        }
        if c == '.' || c == '-' {
            uppercase = true;
        }
    }
    res
}

fn emit_targets() -> io::Result<()> {
    let mut file = open("targets.rs")?;
    write!(
        file,
        "\
/// The target of a C compiler.
///
/// The names used here are the names used by LLVM and Clang. GCC uses the same target
/// names except that `x86_64-pc-windows-gnu` is called `x86_64-w64-mingw64` and
/// `i686-pc-windows-gnu` is called `i686-w64-mingw32`.
///
/// Each target documents its system compiler. The system compiler of a target is its
/// native compiler. For the `*-windows-gnu` targets this is GCC. For all other `*-windows*`
/// targets it is MSVC. For most `*-linux-*` targets it is GCC. For most other targets
/// it is Clang.
///
/// The notion of a target's system compiler matters because compilers generate slightly
/// different layouts even on the same target.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Target {{
"
    )?;
    for target in TARGETS {
        writeln!(file, "    /// The `{}` target.", target.0)?;
        writeln!(file, "    ///")?;
        writeln!(
            file,
            "    /// The system compiler of this target is {}.",
            match target.1 {
                CLANG => "Clang",
                MSVC => "MSVC",
                GCC => "GCC",
                _ => panic!(),
            }
        )?;
        writeln!(file, "    {},", to_camel_case(target.0))?;
    }
    writeln!(file, "}}")?;
    writeln!(file)?;
    writeln!(file, "impl Target {{")?;
    writeln!(file, "    /// Returns the name of the LLVM target.")?;
    writeln!(file, "    pub fn name(self) -> &'static str {{")?;
    writeln!(file, "        match self {{")?;
    for target in TARGETS {
        writeln!(
            file,
            r#"            Target::{} => "{}","#,
            to_camel_case(target.0),
            target.0
        )?;
    }
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    writeln!(file)?;
    writeln!(file, "/// A slice of all targets.")?;
    writeln!(file, "pub const TARGETS: &[Target] = &[")?;
    for target in TARGETS {
        writeln!(file, "    Target::{},", to_camel_case(target.0))?;
    }
    writeln!(file, "];")?;
    writeln!(file)?;
    writeln!(
        file,
        "pub fn system_compiler(target: Target) -> Compiler {{"
    )?;
    writeln!(file, "    match target {{")?;
    for target in TARGETS {
        writeln!(
            file,
            r#"        Target::{} => Compiler::{},"#,
            to_camel_case(target.0),
            to_camel_case(target.1)
        )?;
    }
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    Ok(())
}

fn main() -> io::Result<()> {
    emit_targets()?;
    emit_host_target()?;
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
