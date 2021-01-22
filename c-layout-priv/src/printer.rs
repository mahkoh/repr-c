use crate::ast::{
    Annotation, Array, Declaration, DeclarationType, Expr, ExprType, Record, RecordField, Type,
    TypeVariant,
};
use std::fmt::{Display, Formatter, Result};

pub struct Printer<'a> {
    input: &'a str,
    d: &'a [Declaration],
}

pub fn printer<'a>(input: &'a str, d: &'a [Declaration]) -> Printer<'a> {
    Printer { input, d }
}

impl<'a> Display for Printer<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut pos = 0;
        for d in self.d {
            let mut printer = Printer_ {
                input: self.input,
                pos,
                f,
            };
            printer.print_decl(d)?;
            pos = printer.pos;
        }
        f.write_str(&self.input[pos..])
    }
}

struct Printer_<'a, 'b> {
    input: &'a str,
    pos: usize,
    f: &'a mut Formatter<'b>,
}

impl<'a, 'b> Printer_<'a, 'b> {
    fn print_decl(&mut self, d: &Declaration) -> Result {
        match &d.ty {
            DeclarationType::Type(ty) => self.print_type(ty),
            DeclarationType::Const(c) => self.print_top_level_expr(c),
        }
    }

    fn print_type(&mut self, t: &Type) -> Result {
        self.set_pos(t.lo)?;
        if let Some(l) = t.layout {
            write!(
                self.f,
                "{{ size: {}, alignment: {}",
                l.size_bits, l.alignment_bits
            )?;
            if l.required_alignment_bits != 8 {
                write!(self.f, ", required: {}", l.required_alignment_bits)?;
            }
            write!(self.f, " }}")?;
        }
        self.pos = t.layout_hi;
        self.print_annotations(&t.annotations)?;
        match &t.variant {
            TypeVariant::Record(r) => self.print_record(r),
            TypeVariant::Typedef(t) => self.print_type(t),
            TypeVariant::Array(a) => self.print_array(a),
            TypeVariant::Enum(a) => self.print_enum(a),
            _ => Ok(()),
        }
    }

    fn print_annotations(&mut self, a: &[Annotation]) -> Result {
        for a in a {
            self.print_annotation(a)?;
        }
        Ok(())
    }

    fn print_enum(&mut self, a: &[Expr]) -> Result {
        for a in a {
            self.print_top_level_expr(a)?;
        }
        Ok(())
    }

    fn print_annotation(&mut self, a: &Annotation) -> Result {
        match a {
            Annotation::PragmaPack(e) => self.print_top_level_expr(e),
            Annotation::AttrPacked => Ok(()),
            Annotation::Aligned(e) => self.print_top_level_expr(e),
        }
    }

    fn print_array(&mut self, a: &Array) -> Result {
        if let Some(n) = &a.num_elements {
            self.print_top_level_expr(n)?;
        }
        self.print_type(&a.element_type)
    }

    fn print_record(&mut self, r: &Record) -> Result {
        for f in &r.fields {
            self.print_record_field(f)?;
        }
        Ok(())
    }

    fn print_record_field(&mut self, f: &RecordField) -> Result {
        self.set_pos(f.lo)?;
        if let Some(l) = f.layout {
            write!(
                self.f,
                "{{ offset: {}, size: {} }}",
                l.offset_bits, l.size_bits
            )?;
        }
        self.pos = f.layout_hi;
        self.print_annotations(&f.annotations)?;
        self.print_type(&f.ty)?;
        if let Some(bw) = &f.bit_width {
            self.print_top_level_expr(bw)?;
        }
        Ok(())
    }

    fn print_top_level_expr(&mut self, e: &Expr) -> Result {
        if let ExprType::Lit(_) = e.ty {
            return Ok(());
        }
        self.set_pos(e.span.0)?;
        if let Some(l) = e.value {
            write!(self.f, "{{{}}}", l)?;
        }
        self.pos = e.value_hi;
        Ok(())
    }

    fn set_pos(&mut self, pos: usize) -> Result {
        self.f.write_str(&self.input[self.pos..pos])?;
        self.pos = pos;
        Ok(())
    }
}
