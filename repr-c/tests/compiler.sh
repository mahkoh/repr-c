#!/bin/bash
# SPDX-License-Identifier: GPL-3.0-or-later

set -xe

function error() {
  # shellcheck disable=SC2059
  printf "$@" >&2
  exit 1
}

case "$COMPILER" in
  clang)
    PATH=~/bin/custom-clang/bin
    clang -gdwarf-5 -glldb -target "$TARGET" -integrated-as -c -o "$OUTPUT" "$INPUT"
    ;;
  gcc)
    mabi=""
    case "$TARGET" in
      mips64* | mipsisa64*)
        mabi="-mabi=64"
        ;;
    esac
    PATH=~/bin/gcc-cross/$TARGET/bin
    gcc $mabi -gdwarf-5 -c -o "$OUTPUT" "$INPUT"
    ;;
  msvc)
    case "$TARGET" in
      i[56]86*)
        ARCH=x86
        ;;
      x86_64*)
        ARCH=x64
        ;;
      thumbv7a*)
        ARCH=arm
        ;;
      aarch64*)
        ARCH=arm64
        ;;
      *)
        error "Unknown msvc target %s" "$TARGET"
    esac
    cd "$(dirname "$INPUT")"
    ~/opt/msvc/bin/$ARCH/cl /c /Zi "$INPUT"
    mv vc140.pdb "$OUTPUT"
    ;;
  *)
    error "Unknown compiler %s" "$COMPILER"
    ;;
esac
