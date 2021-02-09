// SPDX-License-Identifier: MIT OR Apache-2.0
#![deny(unreachable_patterns)]
#![deny(non_snake_case)]

pub mod builder;
pub mod layout;
pub mod result;
pub mod target;
pub mod util;
pub mod visitor;
#[cfg(test)]
mod tests;
