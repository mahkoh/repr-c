#!/bin/bash
# SPDX-License-Identifier: GPL-3.0-or-later

set -xe

function error() {
  printf "$@" >&2
  exit 1
}

case "$1" in
  *-msvc | *-windows)
    if [[ -v USE_CLANG_FOR_MSVC ]]; then
      PATH=~/bin/custom-clang/bin
      clang -gdwarf-5 -glldb -target "$1" -c -o "$3" "$2"
    else
      case "$1" in
        i[56]86-pc-windows-msvc| i686-unknown-windows)
          cl x86 "$2" "$3"
          ;;
        x86_64-pc-windows-msvc | x86_64-unknown-windows)
          cl x64 "$2" "$3"
          ;;
        thumbv7a-pc-windows-msvc)
          cl arm "$2" "$3"
          ;;
        aarch64-pc-windows-msvc)
          cl arm64 "$2" "$3"
          ;;
        *)
          error "Unknown msvc target %s" "$1"
      esac
    fi
    ;;
  x86_64-pc-windows-gnu | \
  i686-pc-windows-gnu | \
  aarch64-unknown-linux-gnu | \
  aarch64-unknown-linux-musl | \
  arm-unknown-linux-gnueabi | \
  arm-unknown-linux-gnueabihf | \
  armv4t-unknown-linux-gnueabi | \
  armv5te-unknown-linux-gnueabi | \
  armv5te-unknown-linux-uclibcgnueabi | \
  armv7-unknown-linux-gnueabi | \
  armv7-unknown-linux-gnueabihf | \
  i586-unknown-linux-gnu | \
  i586-unknown-linux-musl | \
  i686-unknown-linux-gnu | \
  i686-unknown-linux-musl | \
  mipsel-unknown-linux-gnu | \
  mipsel-unknown-linux-musl | \
  mipsel-unknown-linux-uclibc | \
  mipsisa32r6el-unknown-linux-gnu | \
  mipsisa32r6-unknown-linux-gnu | \
  mips-unknown-linux-gnu | \
  mips-unknown-linux-musl | \
  mips-unknown-linux-uclibc | \
  powerpc64le-unknown-linux-gnu | \
  powerpc64le-unknown-linux-musl | \
  powerpc64-unknown-linux-gnu | \
  powerpc64-unknown-linux-musl | \
  powerpc-unknown-linux-gnu | \
  powerpc-unknown-linux-musl | \
  riscv32-unknown-linux-gnu | \
  riscv64-unknown-linux-gnu | \
  s390x-unknown-linux-gnu | \
  sparc64-unknown-linux-gnu | \
  sparc-unknown-linux-gnu | \
  x86_64-unknown-linux-gnu | \
  x86_64-unknown-linux-gnux32 | \
  x86_64-unknown-linux-musl | \
  avr-unknown-unknown)
    cd "$HOME/bin/gcc-cross/$1/bin"
    PATH=$(pwd) gcc -gdwarf-5 -c -o "$3" "$2"
    ;;
  mips64el-unknown-linux-gnuabi64 | \
  mips64el-unknown-linux-musl | \
  mips64-unknown-linux-gnuabi64 | \
  mips64-unknown-linux-musl | \
  mipsisa64r6el-unknown-linux-gnuabi64 | \
  mipsisa64r6-unknown-linux-gnuabi64)
    cd "$HOME/bin/gcc-cross/$1/bin"
    PATH=$(pwd) gcc -mabi=64 -gdwarf-5 -c -o "$3" "$2"
    ;;
  aarch64-apple-macosx | \
  aarch64-fuchsia | \
  aarch64-linux-android | \
  aarch64-unknown-freebsd | \
  aarch64-unknown-hermit | \
  aarch64-unknown-netbsd | \
  aarch64-unknown-none | \
  aarch64-unknown-openbsd | \
  aarch64-unknown-redox | \
  arm64-apple-ios | \
  arm64-apple-ios-macabi | \
  arm64-apple-tvos | \
  armebv7r-unknown-none-eabi | \
  armebv7r-unknown-none-eabihf | \
  arm-linux-androideabi | \
  armv6-unknown-freebsd-gnueabihf | \
  armv6-unknown-netbsdelf-eabihf | \
  armv7a-none-eabi | \
  armv7a-none-eabihf | \
  armv7-apple-ios | \
  armv7-none-linux-android | \
  armv7r-unknown-none-eabi | \
  armv7r-unknown-none-eabihf | \
  armv7s-apple-ios | \
  armv7-unknown-freebsd-gnueabihf | \
  armv7-unknown-netbsdelf-eabihf | \
  i386-apple-ios | \
  i686-apple-macosx | \
  i686-linux-android | \
  i686-unknown-freebsd | \
  i686-unknown-haiku | \
  i686-unknown-netbsdelf | \
  i686-unknown-openbsd | \
  mipsel-sony-psp | \
  mipsel-unknown-none | \
  msp430-none-elf | \
  powerpc64-unknown-freebsd | \
  powerpc-unknown-netbsd | \
  riscv32 | \
  riscv64 | \
  sparc64-unknown-netbsd | \
  sparc64-unknown-openbsd | \
  sparcv9-sun-solaris | \
  thumbv4t-none-eabi | \
  thumbv6m-none-eabi | \
  thumbv7em-none-eabi | \
  thumbv7em-none-eabihf | \
  thumbv7m-none-eabi | \
  thumbv8m.base-none-eabi | \
  thumbv8m.main-none-eabi | \
  thumbv8m.main-none-eabihf | \
  wasm32-unknown-emscripten | \
  wasm32-unknown-unknown | \
  wasm32-wasi | \
  x86_64-apple-ios | \
  x86_64-apple-ios-macabi | \
  x86_64-apple-macosx | \
  x86_64-apple-tvos | \
  x86_64-elf | \
  x86_64-fuchsia | \
  x86_64-linux-android | \
  x86_64-pc-solaris | \
  x86_64-rumprun-netbsd | \
  x86_64-unknown-dragonfly | \
  x86_64-unknown-freebsd | \
  x86_64-unknown-haiku | \
  x86_64-unknown-hermit | \
  x86_64-unknown-l4re-uclibc | \
  x86_64-unknown-netbsd | \
  x86_64-unknown-openbsd | \
  hexagon-unknown-linux-musl | \
  powerpc-unknown-linux-gnuspe | \
  x86_64-unknown-redox)
    PATH=~/bin/custom-clang/bin
    clang -gdwarf-5 -glldb -target "$1" -integrated-as -c -o "$3" "$2"
    ;;
  *)
    error "Unknown triple %s" "$1"
    ;;
esac
