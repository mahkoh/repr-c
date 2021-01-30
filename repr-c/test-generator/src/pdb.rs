use anyhow::{bail, Result};
use c_layout_impl::ast;
use c_layout_impl::ast::Declaration;
use c_layout_impl::converter::{Computer, ConversionResult, Convert};
use pdb::{BitfieldType, FallibleIterator, FieldList, MemberType, TypeData};
use repr_c_impl::builder::common::builtin_type_layout;
use repr_c_impl::layout::{FieldLayout, Type, TypeLayout};
use repr_c_impl::target::Target;
use std::collections::HashMap;
use std::io::Cursor;

struct Converter<'a> {
    target: Target,
    records: HashMap<String, u32>,
    bitfields: HashMap<u32, BitfieldType>,
    fields: HashMap<u32, FieldList<'a>>,
    ids: &'a HashMap<usize, String>,
    pdb_index_names: HashMap<u32, String>,
    sizes: HashMap<String, u64>,
}

impl<'a> Converter<'a> {
    fn get_b_offset(&self, name: &str, ty: &str) -> u64 {
        let fl = self.records.get(&format!("{}_{}", name, ty)).unwrap();
        let mut res = None;
        self.for_each_member(*fl, |m| {
            if m.name.to_string() == "b" {
                res = Some(m.offset as u64 * 8);
            }
        });
        match res {
            Some(n) => n,
            _ => panic!(),
        }
    }

    fn for_each_member<F: FnMut(&MemberType)>(&self, fl: u32, mut f: F) {
        let mut fields = self.fields.get(&fl).unwrap();
        loop {
            for field in &fields.fields {
                match field {
                    TypeData::Member(m) => f(m),
                    _ => panic!(),
                }
            }
            match fields.continuation {
                Some(f) => {
                    fields = self.fields.get(&f.0).unwrap();
                }
                None => break,
            }
        }
    }
}

impl<'a> Convert for Converter<'a> {
    type Src = TypeLayout;

    fn convert(&self, ty: Type<Self::Src>) -> Result<Type<TypeLayout>> {
        Ok(ty)
    }

    fn extract_type(&self, ty: &ast::Type) -> Result<TypeLayout> {
        use ast::TypeVariant::*;
        let name = match &ty.variant {
            Name(n, _) => n,
            Record(_) | Typedef(_) | Array(_) | Enum(_) => self.ids.get(&ty.id).unwrap(),
            Builtin(bi) => return Ok(builtin_type_layout(self.target, *bi)),
            Opaque(_) => unreachable!(),
        };
        let alignment = self.get_b_offset(name, "alignment");
        let size_bits = self.get_b_offset(name, "size") - 8;
        Ok(TypeLayout {
            size_bits,
            // We have no way to extract the pointer alignment. Set it to the same value
            // as the field alignment so that it does not get printed.
            pointer_alignment_bits: alignment,
            field_alignment_bits: alignment,
            required_alignment_bits: self.get_b_offset(name, "required_alignment"),
        })
    }

    fn extract_field(&self, field: &ast::RecordField, fpos: usize) -> Result<FieldLayout> {
        let name = self.ids.get(&field.parent_id).unwrap();
        let fields = self.records.get(name).unwrap();
        let mut pos = 0;
        let mut offset = None;
        let mut size = None;
        self.for_each_member(*fields, |m| {
            if pos == fpos {
                match field.bit_width {
                    Some(_) => {
                        let bf = self.bitfields.get(&m.field_type.0).unwrap();
                        offset = Some(m.offset as u64 * 8 + bf.position as u64);
                        size = Some(bf.length as u64);
                    }
                    None => {
                        offset = Some(m.offset as u64 * 8);
                        size = Some(if m.field_type.0 < 0x1000 {
                            match m.field_type.0 {
                                // https://github.com/Microsoft/microsoft-pdb/blob/082c5290e5aff028ae84e43affa8be717aa7af73/include/cvinfo.h#L326-L750
                                0x0010 | 0x0020 | 0x0068 | 0x0069 | 0x0070 | 0x0030 => 8,
                                0x0011 | 0x0021 | 0x0072 | 0x0073 => 16,
                                0x0012 | 0x0022 | 0x0074 | 0x0075 | 0x0403 | 0x0040 => 32,
                                0x0013 | 0x0023 | 0x0076 | 0x0077 | 0x0603 | 0x0041 => 64,
                                _ => unreachable!("0x{:04x}", m.field_type.0),
                            }
                        } else {
                            let name = self.pdb_index_names.get(&m.field_type.0).unwrap();
                            self.sizes.get(name).copied().unwrap() * 8
                        });
                    }
                }
            }
            pos += 1;
        });
        match (offset, size) {
            (Some(offset_bits), Some(size_bits)) => Ok(FieldLayout {
                offset_bits,
                size_bits,
            }),
            _ => panic!(),
        }
    }
}

pub(crate) fn convert(
    target: Target,
    input: &str,
    d: &[Declaration],
    pdb: &[u8],
    ids: &HashMap<usize, String>,
) -> Result<ConversionResult> {
    let mut records = HashMap::new();
    let mut bitfields = HashMap::new();
    let mut fields = HashMap::new();
    let mut sizes = HashMap::new();
    let mut pdb_index_names = HashMap::new();

    let mut pdb = pdb::PDB::open(Cursor::new(pdb))?;
    let ti = pdb.type_information()?;
    let mut ti = ti.iter();
    while let Some(ti1) = ti.next()? {
        let idx = ti1.index().0;
        let ti = ti1.parse()?;
        match ti {
            TypeData::Class(c) => {
                let name = c.name.to_string().to_string();
                pdb_index_names.insert(idx, name.clone());
                if let Some(f) = c.fields {
                    records.insert(name.clone(), f.0);
                    sizes.insert(name, c.size as u64);
                }
            }
            TypeData::Union(u) => {
                let name = u.name.to_string().to_string();
                pdb_index_names.insert(idx, name.clone());
                if u.fields.0 != 0 {
                    records.insert(name.clone(), u.fields.0);
                    sizes.insert(name, u.size as u64);
                }
            }
            TypeData::Bitfield(bf) => {
                bitfields.insert(idx, bf);
            }
            TypeData::FieldList(fl) => {
                fields.insert(idx, fl);
            }
            TypeData::Array(a) => {
                let name = format!("__array_name_{}", idx);
                pdb_index_names.insert(idx, name.clone());
                sizes.insert(name, a.dimensions.last().copied().unwrap() as u64);
            }
            TypeData::Enumeration(_) => {}
            _ => bail!("unexpected type info {:?}", ti),
        }
    }

    let converter = Converter {
        target,
        records,
        bitfields,
        fields,
        ids,
        pdb_index_names,
        sizes,
    };

    Computer::new(input, d, converter)
        .unwrap()
        .compute_layouts()
}
