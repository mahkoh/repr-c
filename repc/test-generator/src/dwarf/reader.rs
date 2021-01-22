// SPDX-License-Identifier: GPL-3.0-or-later
use gimli::{Reader, ReaderOffset};
use object::{Object, ObjectSection, ObjectSymbol, RelocationKind, RelocationTarget};
use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

pub fn add_relocations(
    relocations: &mut HashMap<usize, (bool, i64)>,
    file: &object::File,
    section: &object::Section,
) {
    for (offset64, relocation) in section.relocations() {
        let offset = offset64 as usize;
        let mut addend = match relocation.kind() {
            RelocationKind::Absolute => relocation.addend(),
            RelocationKind::SectionOffset => relocation.addend() - section.address() as i64,
            _ => panic!("unsupported relocation kind {:#?}", relocation),
        };
        if let RelocationTarget::Symbol(symbol_idx) = relocation.target() {
            if let Ok(symbol) = file.symbol_by_index(symbol_idx) {
                addend = symbol.address().wrapping_add(addend as u64) as i64;
            } else {
                panic!("could not find symbol of relocation {:#?}", relocation);
            }
        }
        if relocations
            .insert(offset, (relocation.has_implicit_addend(), addend))
            .is_some()
        {
            panic!();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Relocate<R: Reader<Offset = usize>> {
    pub relocations: Rc<HashMap<usize, (bool, i64)>>,
    pub section: R,
    pub reader: R,
}

impl<R: Reader<Offset = usize>> Relocate<R> {
    fn relocate(&self, offset: usize, value: u64) -> u64 {
        if let Some(&(has_implicit_addend, addend)) = self.relocations.get(&offset) {
            return match has_implicit_addend {
                true => value.wrapping_add(addend as u64),
                false => addend as u64,
            };
        }
        value
    }
}

impl<R: Reader<Offset = usize>> Reader for Relocate<R> {
    type Endian = R::Endian;
    type Offset = R::Offset;

    #[inline]
    fn endian(&self) -> Self::Endian {
        self.reader.endian()
    }

    #[inline]
    fn len(&self) -> Self::Offset {
        self.reader.len()
    }

    #[inline]
    fn empty(&mut self) {
        self.reader.empty()
    }

    #[inline]
    fn truncate(&mut self, len: Self::Offset) -> gimli::Result<()> {
        self.reader.truncate(len)
    }

    #[inline]
    fn offset_from(&self, base: &Self) -> Self::Offset {
        self.reader.offset_from(&base.reader)
    }

    #[inline]
    fn offset_id(&self) -> gimli::ReaderOffsetId {
        self.reader.offset_id()
    }

    #[inline]
    fn lookup_offset_id(&self, id: gimli::ReaderOffsetId) -> Option<Self::Offset> {
        self.reader.lookup_offset_id(id)
    }

    #[inline]
    fn find(&self, byte: u8) -> gimli::Result<Self::Offset> {
        self.reader.find(byte)
    }

    #[inline]
    fn skip(&mut self, len: Self::Offset) -> gimli::Result<()> {
        self.reader.skip(len)
    }

    #[inline]
    fn split(&mut self, len: Self::Offset) -> gimli::Result<Self> {
        let mut other = self.clone();
        other.reader.truncate(len)?;
        self.reader.skip(len)?;
        Ok(other)
    }

    #[inline]
    fn to_slice(&self) -> gimli::Result<Cow<[u8]>> {
        self.reader.to_slice()
    }

    #[inline]
    fn to_string(&self) -> gimli::Result<Cow<str>> {
        self.reader.to_string()
    }

    #[inline]
    fn to_string_lossy(&self) -> gimli::Result<Cow<str>> {
        self.reader.to_string_lossy()
    }

    #[inline]
    fn read_slice(&mut self, buf: &mut [u8]) -> gimli::Result<()> {
        self.reader.read_slice(buf)
    }

    fn read_address(&mut self, address_size: u8) -> gimli::Result<u64> {
        let offset = self.reader.offset_from(&self.section);
        let value = self.reader.read_address(address_size)?;
        Ok(self.relocate(offset, value))
    }

    fn read_length(&mut self, format: gimli::Format) -> gimli::Result<usize> {
        let offset = self.reader.offset_from(&self.section);
        let value = self.reader.read_length(format)?;
        ReaderOffset::from_u64(self.relocate(offset, value as u64))
    }

    fn read_offset(&mut self, format: gimli::Format) -> gimli::Result<usize> {
        let offset = self.reader.offset_from(&self.section);
        let value = self.reader.read_offset(format)?;
        ReaderOffset::from_u64(self.relocate(offset, value as u64))
    }

    fn read_sized_offset(&mut self, size: u8) -> gimli::Result<usize> {
        let offset = self.reader.offset_from(&self.section);
        let value = self.reader.read_sized_offset(size)?;
        ReaderOffset::from_u64(self.relocate(offset, value as u64))
    }
}
