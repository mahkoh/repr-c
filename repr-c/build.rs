use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;

fn generate_msvc_targets() {
    let x86_epp = r#"
        match pack_bits {
            Some(8) | Some(16) | Some(32) => pack_bits,
            _ => None,
        }"#;

    let arm_epp = r#"
        match pack_bits {
            Some(8) | Some(16) | Some(32) | Some(64) | Some(128) => pack_bits,
            _ => Some(64),
        }"#;

    let x86_64_epp = r#"
        match pack_bits {
            Some(8) | Some(16) | Some(32) | Some(64) => pack_bits,
            _ => None,
        }"#;

    let aarch64_epp = r#"
        match pack_bits {
            Some(8) | Some(16) | Some(32) | Some(64) => pack_bits,
            Some(128) => None,
            _ => Some(64),
        }"#;

    let targets = [
        ("I586PcWindowsMsvc", "i586-pc-windows-msvc", 4, x86_epp),
        ("I686PcWindowsMsvc", "i686-pc-windows-msvc", 4, x86_epp),
        ("I686UwpWindowsMsvc", "i686-pc-windows-msvc", 4, x86_epp),
        (
            "Thumbv7aPcWindowsMsvc",
            "thumbv7a-pc-windows-msvc",
            4,
            arm_epp,
        ),
        (
            "Thumbv7aUwpWindowsMsvc",
            "thumbv7a-uwp-windows-msvc",
            4,
            arm_epp,
        ),
        (
            "X86_64PcWindowsMsvc",
            "x86_64-pc-windows-msvc",
            8,
            x86_64_epp,
        ),
        (
            "X86_64UwpWindowsMsvc",
            "x86_64-uwp-windows-msvc",
            8,
            x86_64_epp,
        ),
        (
            "Aarch64PcWindowsMsvc",
            "aarch64-pc-windows-msvc",
            8,
            aarch64_epp,
        ),
        (
            "Aarch64UwpWindowsMsvc",
            "aarch64-uwp-windows-msvc",
            8,
            aarch64_epp,
        ),
    ];

    let mut out_file = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("msvc.rs"))
            .unwrap(),
    );

    writeln!(
        out_file,
        "use super::{{Target, LayoutAlgorithm, Endianness}};"
    )
    .unwrap();
    writeln!(out_file, "use crate::layout::{{BuiltinType, TypeLayout}};").unwrap();
    writeln!(out_file, "use crate::{{BITS_PER_BYTE}};").unwrap();
    writeln!(out_file).unwrap();

    for (id, name, ptr_width, epp) in targets.iter().copied() {
        write!(
            out_file,
            r#"
pub struct {id};

impl Target for {id} {{

    fn layout_algorithm(&self) -> LayoutAlgorithm {{
        LayoutAlgorithm::Msvc
    }}

    fn builtin_type_layout(&self, b: BuiltinType) -> TypeLayout {{
        let (size_bytes, alignment_bytes) = match b {{
            BuiltinType::Unit => (0, 1),
            BuiltinType::Bool => (1, 1),
            BuiltinType::U8 => (1, 1),
            BuiltinType::U16 => (2, 2),
            BuiltinType::U32 => (4, 4),
            BuiltinType::U64 => (8, 8),
            BuiltinType::U128 => (16, 16),
            BuiltinType::I8 => (1, 1),
            BuiltinType::I16 => (2, 2),
            BuiltinType::I32 => (4, 4),
            BuiltinType::I64 => (8, 8),
            BuiltinType::I128 => (16, 16),
            BuiltinType::Char => (1, 1),
            BuiltinType::SignedChar => (1, 1),
            BuiltinType::UnsignedChar => (1, 1),
            BuiltinType::Short => (2, 2),
            BuiltinType::UnsignedShort => (2, 2),
            BuiltinType::Int => (4, 4),
            BuiltinType::UnsignedInt => (4, 4),
            BuiltinType::Long => (4, 4),
            BuiltinType::UnsignedLong => (4, 4),
            BuiltinType::LongLong => (8, 8),
            BuiltinType::UnsignedLongLong => (8, 8),
            BuiltinType::F32 => (4, 4),
            BuiltinType::F64 => (8, 8),
            BuiltinType::Float => (4, 4),
            BuiltinType::Double => (8, 8),
            BuiltinType::Pointer => ({ptr_width}, {ptr_width}),
        }};
        TypeLayout {{
            size_bits: size_bytes * BITS_PER_BYTE,
            pointer_alignment_bits: alignment_bytes * BITS_PER_BYTE,
            field_alignment_bits: alignment_bytes * BITS_PER_BYTE,
            required_alignment_bits: BITS_PER_BYTE,
        }}
    }}

    fn endianness(&self) -> Endianness {{
        Endianness::Little
    }}

    fn name(&self) -> &str {{
        "{name}"
    }}

    fn effective_pragma_pack(&self, pack_bits: Option<u64>) -> Option<u64> {{
{epp}
    }}
}}
"#,
            id = id,
            ptr_width = ptr_width,
            name = name,
            epp = epp
        )
        .unwrap();
    }
}

fn main() {
    generate_msvc_targets();
    println!("cargo:rerun-if-changed=build.rs");
}
