#!/bin/bash

set -xe

case $1 in
  *-msvc)
    case $1 in
      i[56]86-*)
        arch=x86
        ;;
      x86_64-*)
        arch=x64
        ;;
      thumbv7a-*)
        arch=arm
        ;;
      aarch64-*)
        arch=arm64
        ;;
      *)
        printf "Unknown msvc triple %s" "$1" >&2
        exit 1
        ;;
    esac
    cl $arch "$2" "$3"
    ;;
  x86_64-unknown-linux-gnu)
    gcc -gdwarf-5 -c -o "$3" "$2"
    ;;
  avr-unknown-unknown)
    cd "$HOME/bin/gcc-cross/$1/bin"
    PATH=$(pwd) gcc -gdwarf-5 -c -o "$3" "$2"
    ;;
  *)
    clang -gdwarf-5 -glldb -target "$1" -c -o "$3" "$2"
    ;;
esac
