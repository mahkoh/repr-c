use gimli::{Reader, ReaderOffset};
use object::{Object, ObjectSection, ObjectSymbol, Relocation, RelocationKind, RelocationTarget};
use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;
use repr_c_impl::target::Target;

pub fn add_relocations(
    relocations: &mut HashMap<usize, Relocation>,
    file: &object::File,
    section: &object::Section,
    target: Target,
) {
    for (offset64, mut relocation) in section.relocations() {
        let offset = offset64 as usize;
        let mut kind = relocation.kind();
        if let RelocationKind::Elf(e) = kind {
            use Target::*;
            match target {
                AvrUnknownUnknown => {
                    const R_AVR_32: u32 = 1;
                    const R_AVR_16: u32 = 4;
                    match e {
                        R_AVR_16 | R_AVR_32 => kind = RelocationKind::Absolute,
                        _ => panic!("unsupported avr relocation kind {:?}", e),
                    }
                }
                MipselSonyPsp | MipselUnknownNone => {
                    const R_MIPS_16: u32 = 1;
                    const R_MIPS_32: u32 = 2;
                    match e {
                        R_MIPS_16 | R_MIPS_32 => kind = RelocationKind::Absolute,
                        _ => panic!("unsupported mips relocation kind {:?}", e),
                    }
                }
                Msp430NoneElf => {
                    const R_MSP430_32: u32 = 1;
                    const R_MSP430_16_BYTE: u32 = 5;
                    match e {
                        R_MSP430_32 | R_MSP430_16_BYTE => kind = RelocationKind::Absolute,
                        _ => panic!("unsupported msp430 relocation kind {:?}", e),
                    }
                }
                PowerpcUnknownNetbsd => {
                    const R_PPC_ADDR32: u32 = 1;
                    match e {
                        R_PPC_ADDR32 => kind = RelocationKind::Absolute,
                        _ => panic!("unsupported powerpc relocation kind {:?}", e),
                    }
                }
                Riscv32 => {
                    const R_RISCV_32: u32 = 1;
                    match e {
                        R_RISCV_32 => kind = RelocationKind::Absolute,
                        _ => panic!("unsupported riscv relocation kind {:?}", e),
                    }
                }
                Sparcv9SunSolaris => {
                    const R_SPARC_32: u32 = 3;
                    const R_SPARC_UA32: u32 = 23;
                    const R_SPARC_64: u32 = 32;
                    match e {
                        R_SPARC_32 | R_SPARC_UA32 | R_SPARC_64 => kind = RelocationKind::Absolute,
                        _ => panic!("unsupported sparc relocation kind {:?}", e),
                    }
                }
                _ => { },
            }
        }
        if kind != RelocationKind::Absolute {
            panic!("unsupported relocation kind {:?}", relocation.kind());
        }
        if let RelocationTarget::Symbol(symbol_idx) = relocation.target() {
            match file.symbol_by_index(symbol_idx) {
                Ok(symbol) => {
                    let addend = symbol.address().wrapping_add(relocation.addend() as u64);
                    relocation.set_addend(addend as i64);
                }
                Err(_) => panic!(),
            }
        }
        if relocations.insert(offset, relocation).is_some() {
            panic!();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Relocate<R: Reader<Offset = usize>> {
    pub relocations: Rc<HashMap<usize, Relocation>>,
    pub section: R,
    pub reader: R,
}

impl<R: Reader<Offset = usize>> Relocate<R> {
    fn relocate(&self, offset: usize, value: u64) -> u64 {
        if let Some(relocation) = self.relocations.get(&offset) {
            if relocation.has_implicit_addend() {
                // Use the explicit addend too, because it may have the symbol value.
                return value.wrapping_add(relocation.addend() as u64);
            } else {
                return relocation.addend() as u64;
            }
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
