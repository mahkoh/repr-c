// SPDX-License-Identifier: MIT OR Apache-2.0
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Compiler {
    Msvc,
    Gcc,
    Clang,
}

include!(concat!(env!("OUT_DIR"), "/targets.rs"));

include!(concat!(env!("OUT_DIR"), "/host.rs"));

include!(concat!(env!("OUT_DIR"), "/target_map.rs"));
