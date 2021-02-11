// SPDX-License-Identifier: GPL-3.0-or-later
use anyhow::{anyhow, bail, Context, Result};
use isnt::std_1::vec::IsntVecExt;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use repr_c_impl::target::Target;
use std::path::Path;
use std::sync::Mutex;

const TARGET: Target = Target::X86_64PcWindowsMsvc;

#[test]
fn test() -> Result<()> {
    let mut dirs = vec![];
    for dir in std::fs::read_dir("testfiles")? {
        let dir = dir?;
        if dir.file_type()?.is_dir() {
            dirs.push(dir.path());
        }
    }
    dirs.sort();
    let failed = Mutex::new(vec![]);
    let r: Result<()> = dirs.par_iter().try_for_each(|dir| {
        if !process_dir(dir).with_context(|| anyhow!("processing {} failed", dir.display()))? {
            failed.lock().unwrap().push(format!("{}", dir.display()));
        }
        Ok(())
    });
    r?;
    let mut failed = failed.lock().unwrap();
    if failed.is_not_empty() {
        failed.sort();
        for s in &*failed {
            eprintln!("{} failed", s);
        }
        bail!("some tests failed");
    }
    Ok(())
}

fn process_dir(dir: &Path) -> Result<bool> {
    let input_path = dir.join("input.txt");
    let input = std::fs::read_to_string(&input_path)?;
    let declarations = crate::parse(&input).context("Parsing failed")?;

    let actual_conversion_result = crate::compute_layouts(&input, &declarations, TARGET)?;

    let expected_file = dir.join("expected.txt");
    if expected_file.exists() {
        let expected = std::fs::read_to_string(&expected_file)?;
        let expected_declarations = crate::parse(&expected)?;
        let expected_conversion_result = crate::extract_layouts(&expected, &expected_declarations)?;

        if actual_conversion_result == expected_conversion_result {
            return Ok(true);
        }
    }

    let actual_file = dir.join("actual.txt");
    let enhanced = crate::enhance_declarations(&declarations, &actual_conversion_result);
    std::fs::write(actual_file, crate::printer(&input, &enhanced).to_string())?;
    Ok(false)
}
