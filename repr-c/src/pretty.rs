use crate::layout::{Array, Record, RecordField, RecordKind, Type, TypeLayout};
use crate::target::{Endianness, Target};
use crate::visitor::{visit_record, visit_record_field, visit_type, Visitor};
use std::fmt::{Display, Error, Formatter, Result};
use std::mem;

pub fn pretty<'a>(ty: &'a Type<TypeLayout>, target: &'a dyn Target) -> impl Display + 'a {
    Pretty { ty, target }
}

struct Pretty<'a> {
    ty: &'a Type<TypeLayout>,
    target: &'a dyn Target,
}

impl Display for Pretty<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let (ty, first, last) = match self.target.endianness() {
            Endianness::Little => ("little", "least", "most"),
            Endianness::Big => ("big", "most", "least"),
        };
        writeln!(
            f,
            "Target {} is {}-endian.\nBits in each byte are numbered from {} to {} significant.\n",
            self.target.name(),
            ty,
            first,
            last
        )?;
        write!(f, "Size: {}", self.ty.layout.size_bits)?;
        if self.ty.layout.size_bits % self.ty.layout.field_alignment_bits != 0 {
            write!(f, " (not a multiple of the alignment)")?;
        }
        writeln!(f)?;
        writeln!(f, "Alignment: {}", self.ty.layout.field_alignment_bits)?;
        writeln!(
            f,
            "Required alignment: {}\n",
            self.ty.layout.required_alignment_bits
        )?;
        let mut printer = Printer {
            f,
            bits_width: self.ty.layout.size_bits.to_string().len().max(3),
            position: 0,
            errors: vec![],
            prefix: vec!["self".to_string()],
            record_start: 0,
        };
        printer.visit_type(self.ty)?;
        match printer.errors.pop() {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }
}

struct Printer<'a, 'b> {
    f: &'a mut Formatter<'b>,
    bits_width: usize,
    position: u64,
    errors: Vec<Error>,
    prefix: Vec<String>,
    record_start: u64,
}

impl<'a, 'b> Printer<'a, 'b> {
    fn visit_type(&mut self, ty: &Type<TypeLayout>) -> Result {
        let type_start = self.position;
        self.write_start()?;
        visit_type(self, ty);
        self.set_position(type_start + ty.layout.size_bits, false)?;
        self.write_stop()?;
        Ok(())
    }

    fn visit_record_field(
        &mut self,
        field: &RecordField<TypeLayout>,
        rt: &Record<TypeLayout>,
        ty: &Type<TypeLayout>,
    ) -> Result {
        if let Some(name) = &field.name {
            self.set_position(self.record_start + field.layout.offset_bits, true)?;
            self.prefix.push(name.clone());
            if let Some(n) = field.bit_width {
                self.write_start()?;
                self.set_position(self.position + n, false)?;
                self.write_stop()?;
            } else {
                visit_record_field(self, field, rt, ty);
            }
            self.prefix.pop();
        }
        Ok(())
    }

    fn visit_array(&mut self, array: &Array<TypeLayout>, _ty: &Type<TypeLayout>) -> Result {
        for i in 0..array.num_elements {
            self.prefix.push(format!("[{}]", i));
            visit_type(self, &array.element_type);
            self.prefix.pop();
        }
        Ok(())
    }

    fn visit_record(&mut self, rt: &Record<TypeLayout>, ty: &Type<TypeLayout>) -> Result {
        if rt.kind == RecordKind::Union {
            self.set_position(self.position + ty.layout.size_bits, false)?;
        } else {
            let parent_record_start = mem::replace(&mut self.record_start, self.position);
            visit_record(self, rt, ty);
            self.set_position(self.record_start + ty.layout.size_bits, true)?;
            self.record_start = parent_record_start;
        }
        Ok(())
    }

    fn set_position(&mut self, pos: u64, padding: bool) -> Result {
        if pos == self.position {
            return Ok(());
        }
        if padding {
            self.prefix.push("<padding>".to_string());
            self.write_start()?;
        }
        let bars = "│ ".repeat(self.prefix.len());
        if pos - self.position > 3 {
            writeln!(
                self.f,
                "{:>bits_width$}  {}",
                self.position,
                bars,
                bits_width = self.bits_width
            )?;
            writeln!(
                self.f,
                "{:>bits_width$}  {}",
                "…",
                bars,
                bits_width = self.bits_width
            )?;
            writeln!(
                self.f,
                "{:>bits_width$}  {}",
                pos - 1,
                bars,
                bits_width = self.bits_width
            )?;
        } else {
            for i in self.position..pos {
                writeln!(
                    self.f,
                    "{:>bits_width$}  {}",
                    i,
                    bars,
                    bits_width = self.bits_width
                )?;
            }
        }
        self.position = pos;
        if padding {
            self.write_stop()?;
            self.prefix.pop();
        }
        Ok(())
    }

    fn write_start(&mut self) -> Result {
        self.write_start_stop(true)
    }

    fn write_stop(&mut self) -> Result {
        self.write_start_stop(false)
    }

    fn write_start_stop(&mut self, start: bool) -> Result {
        let edge = match start {
            true => '┐',
            false => '┘',
        };
        write!(self.f, "{:bits_width$} ─", "", bits_width = self.bits_width)?;
        for _ in 0..self.prefix.len() - 1 {
            write!(self.f, "┼─")?;
        }
        write!(self.f, "{}", edge)?;
        if start {
            write!(self.f, " {}", self.prefix.last().unwrap())?;
        }
        writeln!(self.f)
    }

    fn handle_result(&mut self, r: Result) {
        if let Err(e) = r {
            self.errors.push(e);
        }
    }
}

impl<'a, 'b> Visitor<TypeLayout> for Printer<'a, 'b> {
    fn visit_type(&mut self, ty: &Type<TypeLayout>) {
        let result = self.visit_type(ty);
        self.handle_result(result);
    }

    fn visit_record(&mut self, rt: &Record<TypeLayout>, ty: &Type<TypeLayout>) {
        let result = self.visit_record(rt, ty);
        self.handle_result(result);
    }

    fn visit_record_field(
        &mut self,
        field: &RecordField<TypeLayout>,
        rt: &Record<TypeLayout>,
        ty: &Type<TypeLayout>,
    ) {
        let result = self.visit_record_field(field, rt, ty);
        self.handle_result(result);
    }

    fn visit_array(&mut self, array: &Array<TypeLayout>, ty: &Type<TypeLayout>) {
        let result = self.visit_array(array, ty);
        self.handle_result(result);
    }
}
