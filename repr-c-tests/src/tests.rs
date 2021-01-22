use anyhow::{bail, Context, Result};
use c_layout_priv::ast::Declaration;
use c_layout_priv::TEST_TARGETS;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use repr_c::target::Target;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

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
    let has_errors = AtomicBool::new(false);
    let r: Result<()> = dirs.par_iter().try_for_each(|dir| {
        let input_path = dir.join("input.txt");
        let input = std::fs::read_to_string(&input_path)?;
        let declarations = c_layout_priv::parse(&input).context("Parsing failed")?;
        TEST_TARGETS.par_iter().try_for_each(|target| {
            if !process_target(&dir, &input, &declarations, *target)? {
                has_errors.store(true, Ordering::Relaxed);
                eprintln!("{}/{} failed", dir.display(), target.name());
            }
            Ok(())
        })
    });
    r?;
    if has_errors.load(Ordering::Relaxed) {
        bail!("some tests failed");
    }
    Ok(())
}

fn process_target(
    dir: &Path,
    input: &str,
    declarations: &[Declaration],
    target: &dyn Target,
) -> Result<bool> {
    let actual_layout = c_layout_priv::compute_layouts(input, declarations, target)?;

    let expected_file = dir.join(target.name()).with_extension("txt");
    let expected = std::fs::read_to_string(expected_file)?;
    let expected_declarations = c_layout_priv::parse(&expected).context("Parsing failed")?;
    let expected_layout = c_layout_priv::extract_layouts(&expected, &expected_declarations)?;

    if actual_layout == expected_layout {
        return Ok(true);
    }

    let actual_file = dir.join(target.name()).with_extension("actual.txt");
    let enhanced = c_layout_priv::enhance_declarations(declarations, &actual_layout);
    std::fs::write(
        actual_file,
        c_layout_priv::printer(input, &enhanced).to_string(),
    )?;
    Ok(false)
}
