// SPDX-License-Identifier: GPL-3.0-or-later
mod reader;

use crate::dwarf::reader::{add_relocations, Relocate};
use anyhow::Result;
use borrow::Cow;
use cly_impl::ast;
use cly_impl::ast::Declaration;
use cly_impl::converter::{Computer, ConversionResult, Convert};
use gimli::{
    Attribute, AttributeValue, DW_AT_bit_size, DW_AT_byte_size, DW_AT_data_bit_offset,
    DW_AT_data_member_location, DW_AT_name, DW_AT_type, DW_TAG_member, DW_TAG_pointer_type,
    DW_TAG_structure_type, DW_TAG_typedef, DW_TAG_union_type, DebuggingInformationEntry,
    EndianRcSlice, EntriesTree, EvaluationResult, Location, RunTimeEndian, SectionId,
};
use object::{Object, ObjectSection};
use repr_c_impl::builder::common::builtin_type_layout;
use repr_c_impl::layout::{BuiltinType, FieldLayout, Type, TypeLayout};
use repr_c_impl::target::Target;
use repr_c_impl::util::BITS_PER_BYTE;
use std::borrow;
use std::collections::HashMap;
use std::rc::Rc;

struct Field {
    offset_bits: u64,
    size_bits: Option<u64>,
    type_offset: usize,
}

struct Converter<'a> {
    target: Target,
    offset_fields: HashMap<usize, Vec<Field>>,
    typedefs: HashMap<usize, usize>,
    name_offsets: HashMap<String, usize>,
    offset_names: HashMap<usize, String>,
    offset_sizes: HashMap<usize, u64>,
    type_id_names: &'a HashMap<usize, String>,
}

impl<'a> Converter<'a> {
    fn traverse_typedefs(&self, mut offset: usize) -> usize {
        while let Some(next) = self.typedefs.get(&offset) {
            offset = *next;
        }
        offset
    }

    fn get_record_fields(&self, name: &str) -> &[Field] {
        let offset = *self.name_offsets.get(name).unwrap();
        let offset = self.traverse_typedefs(offset);
        self.offset_fields.get(&offset).unwrap()
    }

    fn get_second_field_offset(&self, name: &str, ty: &str) -> u64 {
        let struct_name = format!("{}_{}", name, ty);
        let fields = self.get_record_fields(&struct_name);
        fields[1].offset_bits
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
            Record(_) | Typedef(_) | Array(_) | Enum(_) => self.type_id_names.get(&ty.id).unwrap(),
            Builtin(bi) => return Ok(builtin_type_layout(self.target, *bi)),
            Opaque(_) => unreachable!(),
        };
        let required_alignment = self.get_second_field_offset(name, "required_alignment");
        let alignment = self.get_second_field_offset(name, "alignment");
        let size_bits = self.get_second_field_offset(name, "size") - 8;
        Ok(TypeLayout {
            size_bits,
            // We have no way to extract the pointer alignment. Set it to the same value
            // as the field alignment so that it does not get printed.
            pointer_alignment_bits: alignment,
            field_alignment_bits: alignment,
            required_alignment_bits: required_alignment,
        })
    }

    fn extract_field(&self, field: &ast::RecordField, fpos: usize) -> Result<FieldLayout> {
        let name = self.type_id_names.get(&field.parent_id).unwrap();
        let dwarf_field = &self.get_record_fields(name)[fpos];
        let size_bits = match (&field.bit_width, dwarf_field.size_bits) {
            (Some(_), Some(b)) => b,
            _ => {
                let offset = self.traverse_typedefs(dwarf_field.type_offset);
                match self.offset_sizes.get(&offset) {
                    Some(v) => *v,
                    _ => {
                        let name = self.offset_names.get(&dwarf_field.type_offset).unwrap();
                        self.get_second_field_offset(name, "size") - 8
                    }
                }
            }
        };
        Ok(FieldLayout {
            offset_bits: dwarf_field.offset_bits,
            size_bits,
        })
    }
}

pub(crate) fn convert(
    target: Target,
    input: &str,
    d: &[Declaration],
    dwarf_bytes: &[u8],
    type_id_names: &HashMap<usize, String>,
) -> Result<ConversionResult> {
    let object = object::File::parse(dwarf_bytes).unwrap();
    let endian = if object.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };

    let load_section = |id: gimli::SectionId| -> Result<_> {
        let mut relocations = HashMap::new();
        let data = {
            (|| {
                let section = match object.section_by_name(id.name()) {
                    Some(section) => section,
                    _ if id == SectionId::DebugStrOffsets => {
                        match object.section_by_name(".debug_str_offs") {
                            Some(section) => section,
                            _ => return Ok(Cow::Owned(Vec::with_capacity(1))),
                        }
                    }
                    _ => return Ok(Cow::Owned(Vec::with_capacity(1))),
                };
                add_relocations(&mut relocations, &object, &section);
                section.uncompressed_data()
            })()?
        };
        let data_ref = data.into_owned().into_boxed_slice().into();
        let reader = gimli::EndianRcSlice::new(data_ref, endian);
        let section = reader.clone();
        Ok(Relocate {
            relocations: Rc::new(relocations),
            section,
            reader,
        })
    };

    let load_section_sup = |_| {
        Ok(Relocate {
            relocations: Rc::new(HashMap::new()),
            section: EndianRcSlice::new(vec![].into_boxed_slice().into(), endian),
            reader: EndianRcSlice::new(vec![].into_boxed_slice().into(), endian),
        })
    };
    let dwarf = gimli::Dwarf::load(load_section, load_section_sup)?;

    let type_offset = |entry: &DebuggingInformationEntry<_, _>| match entry
        .attr(DW_AT_type)
        .unwrap()
        .unwrap()
        .raw_value()
    {
        AttributeValue::UnitRef(v) => v.0,
        _ => unreachable!(),
    };

    let mut offset_fields = HashMap::new();
    let mut typedefs = HashMap::new();
    let mut name_offsets = HashMap::new();
    let mut offset_names = HashMap::new();
    let mut offset_sizes = HashMap::new();

    let eval_udata = |tag: &Attribute<_>| {
        if let Some(v) = tag.udata_value() {
            return v;
        }
        let encoding = dwarf.debug_info.units().next().unwrap().unwrap().encoding();
        let mut eval = tag.exprloc_value().unwrap().evaluation(encoding);
        eval.set_initial_value(0);
        assert!(matches!(
            eval.evaluate().unwrap(),
            EvaluationResult::Complete
        ));
        let res = eval.result();
        assert_eq!(res.len(), 1);
        match res[0].location {
            Location::Address { address } => address,
            _ => panic!(),
        }
    };

    let mut units = dwarf.units();
    while let Some(header) = units.next()? {
        let unit = dwarf.unit(header)?;
        let mut tree: EntriesTree<_> = unit.entries_tree(None)?;
        let root = tree.root()?;
        let mut top_level = root.children();
        while let Some(node) = top_level.next()? {
            let entry = node.entry();
            let entry_tag = entry.tag();
            let offset = entry.offset().0;
            if let Some(tag) = entry.attr(DW_AT_byte_size)? {
                offset_sizes.insert(offset, eval_udata(&tag) * BITS_PER_BYTE);
            } else if entry_tag == DW_TAG_pointer_type {
                offset_sizes.insert(
                    offset,
                    builtin_type_layout(target, BuiltinType::Pointer).size_bits,
                );
            }
            if let Some(name) = entry.attr(DW_AT_name)? {
                let name = {
                    let name = dwarf.attr_string(&unit, name.raw_value())?;
                    std::str::from_utf8(name.reader.bytes())
                        .unwrap()
                        .to_string()
                };
                name_offsets.insert(name.clone(), offset);
                offset_names.insert(offset, name);
            }
            if entry_tag == DW_TAG_typedef {
                typedefs.insert(offset, type_offset(&entry));
            }
            if entry_tag == DW_TAG_structure_type || entry_tag == DW_TAG_union_type {
                let mut children = node.children();
                let mut fields = vec![];
                while let Some(child_node) = children.next()? {
                    let child_entry = child_node.entry();
                    if child_entry.tag() != DW_TAG_member {
                        panic!();
                    }
                    let offset_bits =
                        if let Some(loc) = child_entry.attr(DW_AT_data_member_location)? {
                            BITS_PER_BYTE * eval_udata(&loc)
                        } else if let Some(loc) = child_entry.attr(DW_AT_data_bit_offset)? {
                            eval_udata(&loc)
                        } else if entry_tag == DW_TAG_union_type {
                            0
                        } else {
                            panic!();
                        };
                    let size_bits = child_entry.attr(DW_AT_bit_size)?.map(|bs| eval_udata(&bs));
                    let type_offset = type_offset(child_entry);
                    fields.push(Field {
                        offset_bits,
                        size_bits,
                        type_offset,
                    });
                }
                offset_fields.insert(offset, fields);
            }
        }
    }

    let converter = Converter {
        target,
        offset_fields,
        typedefs,
        name_offsets,
        offset_names,
        offset_sizes,
        type_id_names,
    };

    Computer::new(input, d, converter)
        .unwrap()
        .compute_layouts()
}
