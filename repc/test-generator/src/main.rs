// SPDX-License-Identifier: GPL-3.0-or-later
use anyhow::{anyhow, bail, Context, Result};
use cly_impl::ast::Declaration;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use repc_impl::target::{system_compiler, Compiler, Target, TARGETS};
use repc_tests::{read_input_config, GlobalConfig, InputConfig};
use std::collections::hash_map::DefaultHasher;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::process;
use std::process::Command;

mod c;
mod dwarf;
mod pdb;

fn main() {
    if let Err(e) = main_() {
        eprintln!("{:?}", e);
        process::exit(1);
    }
}

fn main_() -> Result<()> {
    let userconfig: GlobalConfig = toml::from_str(&std::fs::read_to_string("config.toml")?)?;

    let mut dirs = vec![];
    for dir in std::fs::read_dir("testfiles")? {
        let dir = dir?;
        if dir.file_type()?.is_dir() && userconfig.test_test(dir.file_name().to_str().unwrap()) {
            dirs.push(dir.path());
        }
    }
    dirs.sort();
    dirs.par_iter().try_for_each(|dir| {
        let config = read_input_config(dir)
            .with_context(|| anyhow!("cannot read config in {}", dir.display()))?;
        let input_path = dir.join("input.txt");
        let input = std::fs::read_to_string(&input_path)?;
        let hash = {
            let mut hash = DefaultHasher::new();
            input.hash(&mut hash);
            config.0.hash(&mut hash);
            hash.finish()
        };
        let declarations = cly_impl::parse(&input)
            .with_context(|| anyhow!("Parsing of {} failed", input_path.display()))?;
        std::fs::create_dir_all(dir.join("output"))?;
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
        _ => Ok(false),
    }
}

fn process_target(
    dir: &Path,
    input: &str,
    declarations: &[Declaration],
    hash: u64,
    target: Target,
    userconfig: &GlobalConfig,
    config: &InputConfig,
) -> Result<()> {
    if !config.test_target(target) {
        return Ok(());
    }
    if !userconfig.test_target(target) {
        return Ok(());
    }
    let expected_file = dir
        .join("output")
        .join(format!("{}.expected.txt", target.name()));
    if up_to_date(hash, &expected_file)? {
        return Ok(());
    }
    eprintln!("generating {}", expected_file.display());
    let (code, ids) = c::generate(&declarations, system_compiler(target))?;
    let tmpdir = tempdir::TempDir::new("")?;
    let c_file = tmpdir.path().join("test.c");
    let output_file = tmpdir.path().join("test.output");
    std::fs::write(&c_file, code)?;
    let compiler = match system_compiler(target) {
        Compiler::Msvc if config.use_clang_for_msvc_targets => "clang",
        Compiler::Msvc => "msvc",
        Compiler::Gcc => "gcc",
        Compiler::Clang => "clang",
    };
    let mut cmd = Command::new(&userconfig.compiler);
    cmd.env("COMPILER", compiler);
    cmd.env("TARGET", target.name());
    cmd.env("INPUT", &c_file);
    cmd.env("OUTPUT", &output_file);
    let output = cmd.output()?;
    if output.status.code() != Some(0) {
        bail!(
            "{} did not exit successfully:\nstdout: {}\nstderr: {}",
            userconfig.compiler,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = std::fs::read(output_file)?;
    let conversion_result = match system_compiler(target) {
        Compiler::Msvc if !config.use_clang_for_msvc_targets => {
            pdb::convert(target, &input, &declarations, &output, &ids)
        }
        _ => dwarf::convert(target, &input, &declarations, &output, &ids),
    }?;
    let decls = cly_impl::enhance_declarations(&declarations, &conversion_result);
    let output = cly_impl::printer(&input, &decls).to_string();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(expected_file)?;
    file.write_all(output.as_bytes())?;
    if output.as_bytes().last().copied() != Some(b'\n') {
        writeln!(file)?;
    }
    writeln!(file, "// hash: {:08x}", hash)?;
    Ok(())
}
