use crate::layout::Annotation;
use crate::result::{Error, Result};

pub const BITS_PER_BYTE: u64 = 8;

pub(crate) trait MinAssign<T> {
    fn assign_min(&mut self, other: T);
}

impl MinAssign<u64> for u64 {
    fn assign_min(&mut self, other: u64) {
        *self = (*self).min(other);
    }
}

impl MinAssign<Option<u64>> for u64 {
    fn assign_min(&mut self, other: Option<u64>) {
        *self = (*self).min2(other)
    }
}

pub(crate) trait MaxAssign<T> {
    fn assign_max(&mut self, other: T);
}

impl MaxAssign<Option<u64>> for Option<u64> {
    fn assign_max(&mut self, other: Option<u64>) {
        *self = (*self).max2(other);
    }
}

impl MaxAssign<Option<u64>> for u64 {
    fn assign_max(&mut self, other: Option<u64>) {
        *self = (*self).max2(other);
    }
}

impl MaxAssign<u64> for u64 {
    fn assign_max(&mut self, other: u64) {
        *self = (*self).max(other);
    }
}

pub(crate) trait MinExt<T> {
    type Output;

    fn min2(self, other: T) -> Self::Output;
}

impl MinExt<Option<u64>> for Option<u64> {
    type Output = Option<u64>;

    fn min2(self, other: Option<u64>) -> Option<u64> {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (None, _) => other,
            _ => self,
        }
    }
}

impl MinExt<Option<u64>> for u64 {
    type Output = u64;

    fn min2(self, other: Option<u64>) -> u64 {
        match other {
            Some(b) => self.min(b),
            _ => self,
        }
    }
}

impl MinExt<u64> for Option<u64> {
    type Output = u64;

    fn min2(self, other: u64) -> u64 {
        match self {
            Some(b) => other.min(b),
            _ => other,
        }
    }
}

pub(crate) trait MaxExt<T> {
    type Output;

    fn max2(self, other: T) -> Self::Output;
}

impl MaxExt<Option<u64>> for Option<u64> {
    type Output = Option<u64>;

    fn max2(self, other: Option<u64>) -> Option<u64> {
        match (self, other) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (None, _) => other,
            _ => self,
        }
    }
}

impl MaxExt<Option<u64>> for u64 {
    type Output = u64;

    fn max2(self, other: Option<u64>) -> u64 {
        match other {
            Some(b) => self.max(b),
            _ => self,
        }
    }
}

impl MaxExt<u64> for Option<u64> {
    type Output = u64;

    fn max2(self, other: u64) -> u64 {
        match self {
            Some(b) => other.max(b),
            _ => other,
        }
    }
}

pub(crate) fn align_to(n: u64, m: u64) -> Result<u64> {
    assert!(m.is_power_of_two());
    let mask = m - 1;
    match n.checked_add(mask) {
        Some(n) => Ok(n & !mask),
        _ => Err(Error::SizeOverflow),
    }
}

pub(crate) fn is_attr_packed(a: &[Annotation]) -> bool {
    a.iter().any(|a| matches!(a, Annotation::AttrPacked))
}

pub(crate) fn annotation_alignment(a: &[Annotation]) -> Option<u64> {
    for a in a {
        if let Annotation::Aligned(n) = a {
            return Some(*n);
        }
    }
    None
}

pub(crate) fn pragma_pack_value(a: &[Annotation]) -> Option<u64> {
    for a in a {
        if let Annotation::PragmaPack(n) = a {
            return Some(*n);
        }
    }
    None
}

pub(crate) fn size_mul(a: u64, b: u64) -> Result<u64> {
    match a.checked_mul(b) {
        Some(v) => Ok(v),
        None => Err(Error::SizeOverflow),
    }
}

pub(crate) fn size_add(a: u64, b: u64) -> Result<u64> {
    match a.checked_add(b) {
        Some(v) => Ok(v),
        None => Err(Error::SizeOverflow),
    }
}

// pub(crate) fn align_mul(a: u64, b: u64) -> Result<u64> {
//     match a.checked_mul(b) {
//         Some(v) => Ok(v),
//         None => Err(Error::AlignmentOverflow),
//     }
// }
