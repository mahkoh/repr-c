use anyhow::Result;
use repr_c_impl::target::{system_compiler, Compiler, Target};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::io::ErrorKind;
use std::path::Path;

#[cfg(test)]
mod tests;

#[derive(Clone, Deserialize, Debug, Default)]
#[serde(default)]
pub struct InputConfig {
    #[serde(deserialize_with = "deserialize_compilers")]
    pub include_compilers: Option<Vec<Compiler>>,
    #[serde(deserialize_with = "deserialize_compilers")]
    pub exclude_compilers: Option<Vec<Compiler>>,
    pub include_targets: Option<Vec<String>>,
    pub exclude_targets: Option<Vec<String>>,
}

fn deserialize_compilers<'de, D>(d: D) -> Result<Option<Vec<Compiler>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct XCompiler(Compiler);
    impl<'a> Deserialize<'a> for XCompiler {
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'a>>::Error>
        where
            D: Deserializer<'a>,
        {
            let s = String::deserialize(deserializer)?;
            let c = match &*s {
                "msvc" => Compiler::Msvc,
                "gcc" => Compiler::Gcc,
                "clang" => Compiler::Clang,
                _ => return Err(D::Error::unknown_variant(&s, &["msvc", "gcc", "clang"])),
            };
            Ok(XCompiler(c))
        }
    }
    <Option<Vec<XCompiler>>>::deserialize(d)
        .map(|o| o.map(|v| v.into_iter().map(|c| c.0).collect()))
}

impl InputConfig {
    pub fn test_target(&self, target: Target) -> bool {
        if let Some(c) = &self.include_compilers {
            if !c.contains(&system_compiler(target)) {
                return false;
            }
        }
        if let Some(c) = &self.exclude_compilers {
            if c.contains(&system_compiler(target)) {
                return false;
            }
        }
        let name = target.name();
        if let Some(i) = &self.include_targets {
            if !i.iter().any(|n| n == name) {
                return false;
            }
        }
        if let Some(i) = &self.exclude_targets {
            if i.iter().any(|n| n == name) {
                return false;
            }
        }
        true
    }
}

pub fn read_input_config(dir: &Path) -> Result<(String, InputConfig)> {
    let contents = match std::fs::read_to_string(dir.join("config.toml")) {
        Ok(c) => c,
        Err(e) if e.kind() == ErrorKind::NotFound => "".to_string(),
        Err(e) => return Err(e.into()),
    };
    let config = toml::from_str(&contents)?;
    Ok((contents, config))
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct GlobalConfig {
    pub compiler: String,
    pub include_tests: Option<Vec<String>>,
    pub exclude_tests: Option<Vec<String>>,
    pub include_targets: Option<Vec<String>>,
    pub exclude_targets: Option<Vec<String>>,
}

impl GlobalConfig {
    pub fn test_test(&self, test: &str) -> bool {
        if let Some(i) = &self.include_tests {
            if !i.iter().any(|n| n == test) {
                return false;
            }
        }
        if let Some(i) = &self.exclude_tests {
            if i.iter().any(|n| n == test) {
                return false;
            }
        }
        true
    }

    pub fn test_target(&self, target: Target) -> bool {
        let name = target.name();
        if let Some(i) = &self.include_targets {
            if !i.iter().any(|n| n == name) {
                return false;
            }
        }
        if let Some(i) = &self.exclude_targets {
            if i.iter().any(|n| n == name) {
                return false;
            }
        }
        true
    }
}
