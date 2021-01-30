use anyhow::bail;
use anyhow::Result;
use c_layout_impl::ast::{
    Annotation, Array, BinaryExprType, BuiltinExpr, Declaration, DeclarationType, Expr, ExprType,
    Record, RecordField, Type, TypeExprType, TypeVariant, UnaryExprType,
};
use repr_c_impl::layout::{BuiltinType, RecordKind};
use std::collections::HashMap;
use std::fmt::Write;
use std::mem;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Dialect {
    Msvc,
    Gcc,
}

pub(crate) fn generate(
    i: &[Declaration],
    dialect: Dialect,
) -> Result<(String, HashMap<usize, String>)> {
    let mut g = Generator {
        next: 1,
        ids: Default::default(),
        output: "".to_string(),
        current: "".to_string(),
        stack: vec![],
        dialect,
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
    dialect: Dialect,
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
        write!(&mut self.current, "#define {} ", name)?;
        self.emit_expr(c)?;
        writeln!(&mut self.output, "{}", self.current)?;
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
            write!(&mut self.current, "#pragma pack(")?;
            self.emit_expr(p)?;
            writeln!(&mut self.current, ")")?;
        }
        if self.dialect == Dialect::Msvc {
            if let Some(p) = annotations.align {
                self.emit_declspec_align(p)?;
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
        writeln!(&mut self.current, "{} var{};", name, id)?;
        if annotations.pragma_pack.is_some() {
            writeln!(&mut self.current, "#pragma pack()")?;
        }

        writeln!(&mut self.current, "struct {}_alignment {{", name)?;
        writeln!(&mut self.current, "    char a[_Alignof({})];", name)?;
        writeln!(&mut self.current, "    char b;")?;
        writeln!(&mut self.current, "}};")?;
        let id = self.generate_id();
        writeln!(&mut self.current, "struct {}_alignment var{};", name, id)?;

        writeln!(&mut self.current, "#pragma pack(1)")?;
        writeln!(&mut self.current, "struct {}_packed {{", name)?;
        writeln!(&mut self.current, "    {} a;", name)?;
        writeln!(&mut self.current, "}};")?;
        writeln!(&mut self.current, "#pragma pack()")?;
        writeln!(&mut self.current, "struct {}_required_alignment {{", name)?;
        writeln!(
            &mut self.current,
            "    char a[_Alignof(struct {}_packed)];",
            name
        )?;
        writeln!(&mut self.current, "    char b;")?;
        writeln!(&mut self.current, "}};")?;
        let id = self.generate_id();
        writeln!(
            &mut self.current,
            "struct {}_required_alignment var{};",
            name, id
        )?;

        writeln!(&mut self.current, "struct {}_size {{", name)?;
        writeln!(&mut self.current, "    char a[sizeof({})+1];", name)?;
        writeln!(&mut self.current, "    char b;")?;
        writeln!(&mut self.current, "}};")?;
        let id = self.generate_id();
        writeln!(&mut self.current, "struct {}_size var{};", name, id)?;

        writeln!(&mut self.output, "{}", self.current)?;
        self.current = self.stack.pop().unwrap();
        Ok(())
    }

    fn emit_record(&mut self, n: &str, a: &Annotations, r: &Record) -> Result<()> {
        let kind = match r.kind {
            RecordKind::Struct => "struct",
            RecordKind::Union => "union",
        };
        writeln!(&mut self.current, "typedef {} {{", kind)?;
        for f in &r.fields {
            self.emit_record_field(f)?;
        }
        write!(&mut self.current, "}}")?;
        self.emit_gcc_attributes(a)?;
        writeln!(&mut self.current, " {};", n)?;
        Ok(())
    }

    fn emit_enum(&mut self, n: &str, a: &Annotations, e: &[Expr]) -> Result<()> {
        writeln!(&mut self.current, "typedef enum {{")?;
        for (idx, e) in e.iter().enumerate() {
            write!(&mut self.current, "    F{} = ", idx)?;
            self.emit_expr(e)?;
            writeln!(&mut self.current, ",")?;
        }
        write!(&mut self.current, "}}")?;
        self.emit_gcc_attributes(a)?;
        writeln!(&mut self.current, " {};", n)?;
        Ok(())
    }

    fn emit_array(&mut self, n: &str, an: &Annotations, a: &Array) -> Result<()> {
        write!(&mut self.current, "typedef ")?;
        self.emit_type_name(&a.element_type)?;
        write!(&mut self.current, " {}[", n)?;
        if let Some(v) = &a.num_elements {
            self.emit_expr(v)?;
        } else if self.dialect != Dialect::Msvc {
            write!(self.current, "0")?;
        }
        write!(&mut self.current, "]")?;
        self.emit_gcc_attributes(an)?;
        writeln!(&mut self.current, ";")?;
        Ok(())
    }

    fn emit_typedef(&mut self, n: &str, a: &Annotations, u: &Type) -> Result<()> {
        write!(&mut self.current, "typedef ")?;
        self.emit_type_name(u)?;
        writeln!(&mut self.current, " {}", n)?;
        self.emit_gcc_attributes(a)?;
        writeln!(&mut self.current, ";")?;
        Ok(())
    }

    fn emit_gcc_attributes(&mut self, a: &Annotations) -> Result<()> {
        if self.dialect == Dialect::Gcc {
            if let Some(a) = a.align {
                write!(&mut self.current, " __attribute__((aligned(")?;
                self.emit_expr(a)?;
                write!(&mut self.current, ")))")?;
            }
        }
        if a.attr_packed {
            write!(&mut self.current, " __attribute__((packed))")?;
        }
        Ok(())
    }

    fn emit_builtin_type(&mut self, bi: BuiltinType) -> Result<()> {
        use BuiltinType::*;
        let s = match bi {
            Unit | U8 | U16 | U32 | U64 | U128 | I8 | I16 | I32 | I64 | I128 | F32 | F64 => {
                bail!("type {:?} cannot be used", bi)
            }
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
        write!(&mut self.current, "{}", s)?;
        Ok(())
    }

    fn emit_record_field(&mut self, f: &RecordField) -> Result<()> {
        let annotations = get_unique_annotations(&f.annotations)?;
        if annotations.pragma_pack.is_some() {
            bail!("pragma pack cannot be used on fields");
        }
        write!(&mut self.current, "    ")?;
        if self.dialect == Dialect::Msvc {
            if let Some(a) = annotations.align {
                self.emit_declspec_align(a)?;
            }
        }
        self.emit_type_name(&f.ty)?;
        if let Some(n) = &f.name {
            write!(&mut self.current, " {}", n)?;
        }
        if let Some(bw) = &f.bit_width {
            write!(&mut self.current, ":")?;
            self.emit_expr(bw)?;
        }
        self.emit_gcc_attributes(&annotations)?;
        writeln!(&mut self.current, ";")?;
        Ok(())
    }

    fn emit_expr(&mut self, e: &Expr) -> Result<()> {
        match &e.ty {
            ExprType::Lit(v) => write!(&mut self.current, "{}", v)?,
            ExprType::Name(n) => write!(&mut self.current, "{}", n)?,
            ExprType::Unary(k, v) => {
                let s = match k {
                    UnaryExprType::Neg => "-",
                    UnaryExprType::Not => "!",
                };
                write!(&mut self.current, "{}", s)?;
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
                write!(&mut self.current, " {} ", s)?;
                self.emit_expr(r)?;
            }
            ExprType::TypeExpr(k, t) => {
                let s = match k {
                    TypeExprType::Sizeof => "sizeof",
                    TypeExprType::Alignof => "alignof",
                };
                write!(&mut self.current, "{}(", s)?;
                self.emit_type_name(t)?;
                write!(&mut self.current, ")")?;
            }
            ExprType::Builtin(bi) => match bi {
                BuiltinExpr::BitsPerByte => write!(&mut self.current, "8")?,
            },
            ExprType::Offsetof(_, _, _) => bail!("cannot emit offsetof"),
        }
        Ok(())
    }

    fn emit_type_name(&mut self, ty: &Type) -> Result<()> {
        use TypeVariant::*;
        match &ty.variant {
            Name(n, _) => write!(&mut self.current, "{}", n)?,
            Builtin(bi) => self.emit_builtin_type(*bi)?,
            _ => {
                let name = format!("unnamed_type_{}", self.generate_id());
                self.emit_type_decl(&name, &ty)?;
                write!(&mut self.current, "{}", name)?;
            }
        }
        Ok(())
    }

    fn emit_declspec_align(&mut self, e: &Expr) -> Result<()> {
        write!(&mut self.current, "__declspec(align(")?;
        self.emit_expr(e)?;
        write!(&mut self.current, ")) ")?;
        Ok(())
    }
}

struct Annotations<'a> {
    align: Option<&'a Expr>,
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
                align = Some(&**n);
            }
        }
    }
    Ok(Annotations {
        align,
        attr_packed,
        pragma_pack,
    })
}
