mod msvc {
    include!(concat!(env!("OUT_DIR"), "/msvc.rs"));
}

use crate::layout::{BuiltinType, TypeLayout};
use crate::target::msvc::{
    Aarch64PcWindowsMsvc, Aarch64UwpWindowsMsvc, I586PcWindowsMsvc, I686PcWindowsMsvc,
    I686UwpWindowsMsvc, Thumbv7aPcWindowsMsvc, Thumbv7aUwpWindowsMsvc, X86_64PcWindowsMsvc,
    X86_64UwpWindowsMsvc,
};

pub trait Target: 'static + Send + Sync {
    fn layout_algorithm(&self) -> LayoutAlgorithm;
    fn builtin_type_layout(&self, b: BuiltinType) -> TypeLayout;
    fn endianness(&self) -> Endianness;
    fn name(&self) -> &str;
    fn effective_pragma_pack(&self, pack_bits: Option<u64>) -> Option<u64>;
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
    &X86_64PcWindowsMsvc,
    &X86_64UwpWindowsMsvc,
    &I586PcWindowsMsvc,
    &I686PcWindowsMsvc,
    &I686UwpWindowsMsvc,
    &Aarch64PcWindowsMsvc,
    &Aarch64UwpWindowsMsvc,
    &Thumbv7aPcWindowsMsvc,
    &Thumbv7aUwpWindowsMsvc,
];
