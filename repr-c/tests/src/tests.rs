use crate::{read_input_config, GlobalConfig, InputConfig};
use anyhow::{anyhow, bail, Context, Result};
use c_layout_impl::ast::Declaration;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use repr_c_impl::layout::{FieldLayout, LayoutInfo, Type, TypeLayout};
use repr_c_impl::target::{Target, TARGETS};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn test() -> Result<()> {
    let userconfig: GlobalConfig = toml::from_str(&std::fs::read_to_string("config.toml")?)?;
    let mut dirs = vec![];
    for dir in std::fs::read_dir("testfiles")? {
        let dir = dir?;
        if dir.file_type()?.is_dir() {
            if userconfig.test_test(dir.file_name().to_str().unwrap()) {
                dirs.push(dir.path());
            }
        }
    }
    dirs.sort();
    let has_errors = AtomicBool::new(false);
    let r: Result<()> = dirs.par_iter().try_for_each(|dir| {
        process_dir(dir, &has_errors, &userconfig)
            .with_context(|| anyhow!("processing {} failed", dir.display()))
    });
    r?;
    if has_errors.load(Ordering::Relaxed) {
        bail!("some tests failed");
    }
    Ok(())
}

fn process_dir(dir: &Path, has_errors: &AtomicBool, global_config: &GlobalConfig) -> Result<()> {
    let config = read_input_config(dir)?.1;
    let input_path = dir.join("input.txt");
    let input = std::fs::read_to_string(&input_path)?;
    let declarations = c_layout_impl::parse(&input).context("Parsing failed")?;
    TARGETS.par_iter().try_for_each(|target| {
        if !process_target(&dir, &input, &declarations, *target, &config, global_config)? {
            has_errors.store(true, Ordering::Relaxed);
            eprintln!("{}/{} failed", dir.display(), target.name());
        }
        Ok(())
    })
}

fn process_target(
    dir: &Path,
    input: &str,
    declarations: &[Declaration],
    target: Target,
    input_config: &InputConfig,
    global_config: &GlobalConfig,
) -> Result<bool> {
    if !input_config.test_target(target) {
        return Ok(true);
    }
    if !global_config.test_target(target) {
        return Ok(true);
    }
    let mut actual_conversion_result = c_layout_impl::compute_layouts(input, declarations, target)?;
    actual_conversion_result.types = actual_conversion_result
        .types
        .into_iter()
        .map(|(l, r)| {
            let r: Type<TypeLayoutWithoutPointerAlignment> = r.into();
            (l, r.into())
        })
        .collect();

    let expected_file = dir.join(target.name()).with_extension("expected.txt");
    let expected = std::fs::read_to_string(&expected_file)
        .with_context(|| anyhow!("cannot read {}", expected_file.display()))?;
    let expected_declarations = c_layout_impl::parse(&expected)
        .with_context(|| anyhow!("Parsing {} failed", expected_file.display()))?;
    let expected_conversion_result =
        c_layout_impl::extract_layouts(&expected, &expected_declarations)?;

    if actual_conversion_result == expected_conversion_result {
        return Ok(true);
    }

    let actual_file = dir.join(target.name()).with_extension("actual.txt");
    let enhanced = c_layout_impl::enhance_declarations(declarations, &actual_conversion_result);
    std::fs::write(
        actual_file,
        c_layout_impl::printer(input, &enhanced).to_string(),
    )?;
    Ok(false)
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
struct TypeLayoutWithoutPointerAlignment {
    pub size_bits: u64,
    pub field_alignment_bits: u64,
    pub required_alignment_bits: u64,
}

impl From<TypeLayout> for TypeLayoutWithoutPointerAlignment {
    fn from(src: TypeLayout) -> Self {
        Self {
            size_bits: src.size_bits,
            field_alignment_bits: src.field_alignment_bits,
            required_alignment_bits: src.required_alignment_bits,
        }
    }
}

impl Into<TypeLayout> for TypeLayoutWithoutPointerAlignment {
    fn into(self) -> TypeLayout {
        TypeLayout {
            size_bits: self.size_bits,
            field_alignment_bits: self.field_alignment_bits,
            pointer_alignment_bits: self.field_alignment_bits,
            required_alignment_bits: self.required_alignment_bits,
        }
    }
}

impl LayoutInfo for TypeLayoutWithoutPointerAlignment {
    type FieldLayout = FieldLayout;
    type OpaqueLayout = TypeLayoutWithoutPointerAlignment;
}
