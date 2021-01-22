// SPDX-License-Identifier: GPL-3.0-or-later
use anyhow::bail;
use anyhow::Result;
use cly_impl::ast::{
    Annotation, Array, BinaryExprType, BuiltinExpr, Declaration, DeclarationType, Expr, ExprType,
    Record, RecordField, Type, TypeExprType, TypeVariant, UnaryExprType,
};
use repc_impl::layout::{BuiltinType, RecordKind};
use repc_impl::target::Compiler;
use std::collections::HashMap;
use std::fmt::Write;
use std::mem;

pub(crate) fn generate(
    i: &[Declaration],
    compiler: Compiler,
) -> Result<(String, HashMap<usize, String>)> {
    let mut g = Generator {
        next: 1,
        ids: Default::default(),
        output: "".to_string(),
        current: "".to_string(),
        stack: vec![],
        compiler,
    };
    g.generate(i)?;
    Ok((g.output, g.ids))
}

struct Generator {
    next: usize,
    ids: HashMap<usize, String>,
    output: String,
    current: String,
    stack: Vec<String>,
    compiler: Compiler,
}

impl Generator {
    fn generate(&mut self, d: &[Declaration]) -> Result<()> {
        for d in d {
            match &d.ty {
                DeclarationType::Type(t) => self.emit_type_decl(&d.name, t)?,
                DeclarationType::Const(c) => self.emit_const(&d.name, c)?,
            }
        }
        Ok(())
    }

    fn emit_const(&mut self, name: &str, c: &Expr) -> Result<()> {
        self.stack
            .push(mem::replace(&mut self.current, String::new()));
        write!(self.current, "#define {} ", name)?;
        self.emit_expr(c)?;
        writeln!(self.output, "{}", self.current)?;
        self.current = self.stack.pop().unwrap();
        Ok(())
    }

    fn generate_id(&mut self) -> usize {
        let next = self.next;
        self.next += 1;
        next
    }

    fn emit_type_decl(&mut self, name: &str, t: &Type) -> Result<()> {
        assert!(self.ids.insert(t.id, name.to_string()).is_none());
        self.stack
            .push(mem::replace(&mut self.current, String::new()));
        let annotations = get_unique_annotations(&t.annotations)?;
        if let Some(p) = annotations.pragma_pack {
            write!(self.current, "#pragma pack(")?;
            self.emit_expr(p)?;
            writeln!(self.current, ")")?;
        }
        if self.compiler == Compiler::Msvc {
            if let Some(p) = annotations.align {
                match p {
                    Some(p) => self.emit_declspec_align(p)?,
                    _ => bail!("MSVC does not support @align without a value"),
                }
            }
        }
        match &t.variant {
            TypeVariant::Builtin(_) => bail!("builtin types cannot be declared"),
            TypeVariant::Record(r) => self.emit_record(name, &annotations, r)?,
            TypeVariant::Typedef(r) => self.emit_typedef(name, &annotations, r)?,
            TypeVariant::Array(a) => self.emit_array(name, &annotations, a)?,
            TypeVariant::Opaque(_) => bail!("opaque types cannot be declared"),
            TypeVariant::Name(..) => bail!("names cannot be declared"),
            TypeVariant::Enum(r) => self.emit_enum(name, &annotations, r)?,
        }
        let id = self.generate_id();
        writeln!(self.current, "{} var{};", name, id)?;
        if annotations.pragma_pack.is_some() {
            writeln!(self.current, "#pragma pack()")?;
        }

        writeln!(self.current, "struct {}_alignment {{", name)?;
        if self.compiler == Compiler::Msvc {
            writeln!(self.current, "    char a[_Alignof({})];", name)?;
            writeln!(self.current, "    char b;")?;
        } else {
            writeln!(self.current, "    char a;")?;
            writeln!(self.current, "    {} b;", name)?;
        }
        writeln!(self.current, "}};")?;
        let id = self.generate_id();
        writeln!(self.current, "struct {}_alignment var{};", name, id)?;

        writeln!(self.current, "#pragma pack(1)")?;
        writeln!(self.current, "struct {}_packed {{", name)?;
        writeln!(self.current, "    {} a;", name)?;
        writeln!(self.current, "}};")?;
        writeln!(self.current, "#pragma pack()")?;
        writeln!(self.current, "struct {}_required_alignment {{", name)?;
        if self.compiler == Compiler::Msvc {
            writeln!(
                self.current,
                "    char a[_Alignof(struct {}_packed)];",
                name
            )?;
            writeln!(self.current, "    char b;")?;
        } else {
            writeln!(self.current, "    char a;")?;
            writeln!(self.current, "    struct {}_packed b;", name)?;
        }
        writeln!(self.current, "}};")?;
        let id = self.generate_id();
        writeln!(
            self.current,
            "struct {}_required_alignment var{};",
            name, id
        )?;

        writeln!(self.current, "struct {}_size {{", name)?;
        writeln!(self.current, "    char a[sizeof({})+1];", name)?;
        writeln!(self.current, "    char b;")?;
        writeln!(self.current, "}};")?;
        let id = self.generate_id();
        writeln!(self.current, "struct {}_size var{};", name, id)?;

        writeln!(self.output, "{}", self.current)?;
        self.current = self.stack.pop().unwrap();
        Ok(())
    }

    fn emit_record(&mut self, n: &str, a: &Annotations, r: &Record) -> Result<()> {
        let kind = match r.kind {
            RecordKind::Struct => "struct",
            RecordKind::Union => "union",
        };
        writeln!(self.current, "typedef {} {{", kind)?;
        for f in &r.fields {
            self.emit_record_field(f)?;
        }
        write!(self.current, "}}")?;
        self.emit_gcc_attributes(a)?;
        writeln!(self.current, " {};", n)?;
        Ok(())
    }

    fn emit_enum(&mut self, n: &str, a: &Annotations, e: &[Expr]) -> Result<()> {
        writeln!(self.current, "typedef enum {{")?;
        for e in e.iter() {
            let idx = self.generate_id();
            write!(self.current, "    F{} = ", idx)?;
            self.emit_expr(e)?;
            writeln!(self.current, ",")?;
        }
        write!(self.current, "}}")?;
        self.emit_gcc_attributes(a)?;
        writeln!(self.current, " {};", n)?;
        Ok(())
    }

    fn emit_array(&mut self, n: &str, an: &Annotations, a: &Array) -> Result<()> {
        write!(self.current, "typedef ")?;
        self.emit_type_name(&a.element_type)?;
        write!(self.current, " {}[", n)?;
        if let Some(v) = &a.num_elements {
            self.emit_expr(v)?;
        } else if self.compiler != Compiler::Msvc {
            write!(self.current, "0")?;
        }
        write!(self.current, "]")?;
        self.emit_gcc_attributes(an)?;
        writeln!(self.current, ";")?;
        Ok(())
    }

    fn emit_typedef(&mut self, n: &str, a: &Annotations, u: &Type) -> Result<()> {
        write!(self.current, "typedef ")?;
        self.emit_type_name(u)?;
        write!(self.current, " {}", n)?;
        self.emit_gcc_attributes(a)?;
        writeln!(self.current, ";")?;
        Ok(())
    }

    fn emit_gcc_attributes(&mut self, a: &Annotations) -> Result<()> {
        if self.compiler != Compiler::Msvc {
            match a.align {
                Some(Some(a)) => {
                    write!(self.current, " __attribute__((aligned(")?;
                    self.emit_expr(a)?;
                    write!(self.current, ")))")?;
                }
                Some(None) => write!(self.current, " __attribute__((aligned))")?,
                _ => {}
            }
        }
        if a.attr_packed {
            write!(self.current, " __attribute__((packed))")?;
        }
        Ok(())
    }

    fn emit_builtin_type(&mut self, bi: BuiltinType) -> Result<()> {
        use BuiltinType::*;
        let s = match bi {
            Unit | U8 | U16 | U32 | U64 | I8 | I16 | I32 | I64 | F32 | F64 => {
                bail!("type {:?} cannot be used", bi)
            }
            I128 => "__int128",
            U128 => "unsigned __int128",
            Bool => "_Bool",
            Char => "char",
            SignedChar => "signed char",
            UnsignedChar => "unsigned char",
            Short => "short",
            UnsignedShort => "unsigned short",
            Int => "int",
            UnsignedInt => "unsigned int",
            Long => "long",
            UnsignedLong => "unsigned long",
            LongLong => "long long",
            UnsignedLongLong => "unsigned long long",
            Float => "float",
            Double => "double",
            Pointer => "void*",
        };
        write!(self.current, "{}", s)?;
        Ok(())
    }

    fn emit_record_field(&mut self, f: &RecordField) -> Result<()> {
        let annotations = get_unique_annotations(&f.annotations)?;
        if annotations.pragma_pack.is_some() {
            bail!("pragma pack cannot be used on fields");
        }
        write!(self.current, "    ")?;
        if self.compiler == Compiler::Msvc {
            if let Some(a) = annotations.align {
                match a {
                    Some(a) => self.emit_declspec_align(a)?,
                    _ => bail!("MSVC doesn't support @align without a value"),
                }
            }
        }
        self.emit_type_name(&f.ty)?;
        if let Some(n) = &f.name {
            write!(self.current, " {}", n)?;
        }
        if let Some(bw) = &f.bit_width {
            write!(self.current, ":")?;
            self.emit_expr(bw)?;
        }
        self.emit_gcc_attributes(&annotations)?;
        writeln!(self.current, ";")?;
        Ok(())
    }

    #[allow(clippy::many_single_char_names)]
    fn emit_expr(&mut self, e: &Expr) -> Result<()> {
        match &e.ty {
            ExprType::Lit(v) => write!(self.current, "{}", v)?,
            ExprType::Name(n) => write!(self.current, "{}", n)?,
            ExprType::Unary(k, v) => {
                let s = match k {
                    UnaryExprType::Neg => "-",
                    UnaryExprType::Not => "!",
                };
                write!(self.current, "{}", s)?;
                self.emit_expr(v)?;
            }
            ExprType::Binary(k, l, r) => {
                self.emit_expr(l)?;
                let s = match k {
                    BinaryExprType::Add => "+",
                    BinaryExprType::Sub => "-",
                    BinaryExprType::Mul => "*",
                    BinaryExprType::Div => "/",
                    BinaryExprType::Mod => "%",
                    BinaryExprType::LogicalAnd => "&&",
                    BinaryExprType::LogicalOr => "||",
                    BinaryExprType::Eq => "==",
                    BinaryExprType::NotEq => "!=",
                    BinaryExprType::Lt => "<",
                    BinaryExprType::Le => "<=",
                    BinaryExprType::Gt => ">",
                    BinaryExprType::Ge => ">=",
                };
                write!(self.current, " {} ", s)?;
                self.emit_expr(r)?;
            }
            ExprType::TypeExpr(k, t) => {
                write!(self.current, "(sizeof(")?;
                self.emit_type_name(t)?;
                write!(self.current, ")")?;
                if *k == TypeExprType::SizeofBits {
                    write!(self.current, "*8")?;
                }
                write!(self.current, ")")?;
            }
            ExprType::Builtin(bi) => match bi {
                BuiltinExpr::BitsPerByte => write!(self.current, "8")?,
            },
            ExprType::Offsetof(_, _, _) => bail!("cannot emit offsetof"),
        }
        Ok(())
    }

    fn emit_type_name(&mut self, ty: &Type) -> Result<()> {
        use TypeVariant::*;
        match &ty.variant {
            Name(n, _) => write!(self.current, "{}", n)?,
            Builtin(bi) => self.emit_builtin_type(*bi)?,
            _ => {
                let name = format!("unnamed_type_{}", self.generate_id());
                self.emit_type_decl(&name, &ty)?;
                write!(self.current, "{}", name)?;
            }
        }
        Ok(())
    }

    fn emit_declspec_align(&mut self, e: &Expr) -> Result<()> {
        write!(self.current, "__declspec(align(")?;
        self.emit_expr(e)?;
        write!(self.current, ")) ")?;
        Ok(())
    }
}

struct Annotations<'a> {
    align: Option<Option<&'a Expr>>,
    attr_packed: bool,
    pragma_pack: Option<&'a Expr>,
}

fn get_unique_annotations(a: &[Annotation]) -> Result<Annotations> {
    let mut align = None;
    let mut attr_packed = false;
    let mut pragma_pack = None;
    for a in a {
        match a {
            Annotation::PragmaPack(n) => {
                if pragma_pack.is_some() {
                    bail!("cannot used multiple pragma pack annotations");
                }
                pragma_pack = Some(&**n);
            }
            Annotation::AttrPacked => attr_packed = true,
            Annotation::Aligned(n) => {
                if align.is_some() {
                    bail!("cannot used multiple align annotations");
                }
                align = Some(n.as_ref().map(|n| &**n));
            }
        }
    }
    Ok(Annotations {
        align,
        attr_packed,
        pragma_pack,
    })
}
