use crate::c::Dialect;
use anyhow::{anyhow, bail, Context, Result};
use c_layout_priv::ast::Declaration;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use repr_c::target::{Target, TARGETS};
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

#[derive(Copy, Clone, Deserialize, Debug, Default)]
struct InputConfig {}

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
    dirs.par_iter().try_for_each(|dir| {
        let config =
            read_config(dir).with_context(|| anyhow!("cannot read config in {}", dir.display()))?;
        let input_path = dir.join("input.txt");
        let input = std::fs::read_to_string(&input_path)?;
        let hash = {
            let mut hash = DefaultHasher::new();
            input.hash(&mut hash);
            config.0.hash(&mut hash);
            hash.finish()
        };
        let declarations = c_layout_priv::parse(&input)
            .with_context(|| anyhow!("Parsing of {} failed", input_path.display()))?;
        TARGETS.par_iter().try_for_each(|target| {
            process_target(
                &dir,
                &input,
                &declarations,
                hash,
                *target,
                &userconfig,
                &config.1,
            )
        })
    })
}

fn read_config(dir: &Path) -> Result<(String, InputConfig)> {
    let contents = match std::fs::read_to_string(dir.join("config.toml")) {
        Ok(c) => c,
        Err(e) if e.kind() == ErrorKind::NotFound => "".to_string(),
        Err(e) => return Err(e.into()),
    };
    let config = toml::from_str(&contents)?;
    Ok((contents, config))
}

fn up_to_date(hash: u64, expected: &Path) -> Result<bool> {
    let input = match std::fs::read_to_string(expected) {
        Ok(i) => i,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.into()),
    };
    let last = match input.lines().last() {
        Some(l) => l,
        None => return Ok(false),
    };
    let suffix = match last.strip_prefix("// hash: ") {
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
    config: &InputConfig,
) -> Result<()> {
    let _ = config;
    let output_file = dir.join(target.name()).with_extension("expected.txt");
    if up_to_date(hash, &output_file)? {
        return Ok(());
    }
    eprintln!("generating {}", output_file.display());
    let (code, ids) = c::generate(&declarations, Dialect::Msvc)?;
    let tmpdir = tempdir::TempDir::new("")?;
    let c_file = tmpdir.path().join("test.c");
    let pdb_file = tmpdir.path().join("test.pdb");
    let compiler = userconfig.compilers.get(target.name()).unwrap();
    std::fs::write(&c_file, code)?;
    let cmd = format!(
        "set -x; {} '{}' '{}'",
        compiler,
        c_file.display(),
        pdb_file.display(),
    );
    let output = Command::new("bash").arg("-c").arg(cmd).output()?;
    if output.status.code() != Some(0) {
        bail!(
            "{} did not exit successfully:\nstdout: {}\nstderr: {}",
            compiler,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = std::fs::read(pdb_file)?;
    let conversion_result = pdb::convert(target, &input, &declarations, &output, &ids)?;
    let decls = c_layout_priv::enhance_declarations(&declarations, &conversion_result);
    let output = c_layout_priv::printer(&input, &decls).to_string();
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
