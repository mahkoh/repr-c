use crate::c::Dialect;
use anyhow::{bail, Context, Result};
use c_layout_priv::ast::Declaration;
use c_layout_priv::TEST_TARGETS;
use repr_c::target::Target;
use serde::Deserialize;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process::Command;

mod c;
mod pdb;

fn main() {
    if let Err(e) = main_() {
        eprintln!("{:?}", e);
    }
}

#[derive(Deserialize, Debug)]
struct Userconfig {
    compilers: HashMap<String, String>,
}

fn main_() -> Result<()> {
    let userconfig: Userconfig = toml::from_str(&std::fs::read_to_string("userconfig.toml")?)?;

    let mut dirs = vec![];
    for dir in std::fs::read_dir(".")? {
        let dir = dir?;
        if dir.file_type()?.is_dir() {
            dirs.push(dir.path());
        }
    }
    dirs.sort();
    for dir in dirs {
        eprintln!("processing {}", dir.display());
        let input_path = dir.join("input.txt");
        let input = std::fs::read_to_string(&input_path)?;
        let hash = {
            let mut hash = DefaultHasher::new();
            input.hash(&mut hash);
            hash.finish()
        };
        let declarations = c_layout_priv::parse(&input).context("Parsing failed")?;
        for target in TEST_TARGETS.iter().copied() {
            process_target(&dir, &input, &declarations, hash, target, &userconfig)?;
        }
    }
    Ok(())
}

fn up_to_date(dir: &Path, hash: u64, target: &dyn Target) -> Result<bool> {
    let path = dir.join(target.name()).with_extension("txt");
    let input = match std::fs::read_to_string(&path) {
        Ok(i) => i,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.into()),
    };
    let first = match input.lines().last() {
        Some(l) => l,
        None => return Ok(false),
    };
    let suffix = match first.strip_prefix("// hash: ") {
        Some(s) => s,
        None => return Ok(false),
    };
    match u64::from_str_radix(suffix, 16) {
        Ok(n) if n == hash => Ok(true),
        _ => return Ok(false),
    }
}

fn process_target(
    dir: &Path,
    input: &str,
    declarations: &[Declaration],
    hash: u64,
    target: &dyn Target,
    userconfig: &Userconfig,
) -> Result<()> {
    if up_to_date(dir, hash, target)? {
        return Ok(());
    }
    eprintln!("processing target {}", target.name());
    let (code, ids) = c::generate(&declarations, Dialect::Msvc)?;
    let tmpdir = tempdir::TempDir::new("")?;
    let c_file = tmpdir.path().join("test.c");
    std::fs::write(&c_file, code)?;
    let status = Command::new(userconfig.compilers.get(target.name()).unwrap())
        .arg(&c_file)
        .status()?;
    if status.code() != Some(0) {
        bail!("msvc-pdb did not exit successfully");
    }
    let output = std::fs::read(tmpdir.path().join("test.c.pdb"))?;
    let conversion_result = pdb::convert(target, &input, &declarations, &output, &ids)?;
    let decls = c_layout_priv::enhance_declarations(&declarations, &conversion_result);
    let output = c_layout_priv::printer(&input, &decls).to_string();
    let output_file = dir.join(target.name()).with_extension("txt");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)?;
    file.write_all(output.as_bytes())?;
    if output.as_bytes().last().copied() != Some(b'\n') {
        writeln!(file)?;
    }
    writeln!(file, "// hash: {:08x}", hash)?;
    Ok(())
}
