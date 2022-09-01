use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;
use regex::Regex;
use serde_yaml::{Mapping, Value};
use std::{
    error::Error,
    fs, io,
    path::Path,
    process::{Command, Output},
};

/// Wrapper on `conan create`
/// to create more than one package with a specific version
#[derive(clap::Parser, Clone)]
pub struct Create {
    /// Path to [package folder]/[pattern folder]
    path: String,

    /// Desired version TODO: bug with `--pattern`
    #[clap(long, default_value = "latest")]
    version: String,

    /// Profile to `create`
    #[clap(long, default_value = "default")]
    profile: String,

    /// Pattern to `create` some package from `path` with `pattern`
    #[clap(long)]
    pattern: Option<String>,

    /// Configure verbosity
    #[clap(long, short)]
    verbose: bool,
}

fn to_ver_folder(versions: &Value) -> Option<Vec<(&str, &str)>> {
    versions
        .as_mapping()?
        .iter()
        .map(|(ver, val)| Some((ver.as_str()?, val["folder"].as_str()?)))
        .collect()
}

fn package_pattern(package: &str, version: &str) -> String {
    format!("{package}/{version}@")
}

fn create_package(
    path: &Path,
    package: &str,
    version: &str,
    folder: &str,
    profile: &str,
) -> io::Result<Output> {
    let pattern = package_pattern(package, version);
    let binding = path.join(folder);
    Command::new("conan")
        .args(["create", &binding.to_string_lossy(), &pattern])
        .arg(format!("-pr={profile}"))
        .output()
}

fn creating_info(package: &str, version: &str) {
    let pattern = package_pattern(package, version);
    println!(
        "{} {}",
        "creating".bright_cyan().bold(),
        pattern.as_str().bright_green()
    );
}

fn at_create_info(package: &str, version: &str) {
    let pattern = package_pattern(package, version);
    println!(
        "{} - {}",
        pattern.as_str().bright_green().bold(),
        "created".bright_cyan()
    );
}

fn with_config(create: Create) -> Result<()> {
    let path: &Path = create.path.as_ref();

    let config = fs::read_to_string(path.join("config.yml"))?;
    let yaml: Value = serde_yaml::from_str(&config)?;

    let notation =
        to_ver_folder(&yaml["versions"]).ok_or(anyhow!("possible incorrect conan .yml config"))?;
    let (version, folder) = if create.version == "latest" {
        notation.into_iter().max()
    } else {
        notation.into_iter().find(|(ver, _)| ver == &create.version)
    }
    .ok_or_else(|| anyhow!("version not found in .yml config"))?;

    let package = path.file_name().map(|s| s.to_string_lossy()).unwrap();

    creating_info(&package, version);
    create_package(path, &package, version, folder, &create.profile)?;
    at_create_info(&package, version);

    Ok(())
}

pub fn parse_create(create: Create) -> Result<()> {
    let path: &Path = create.path.as_ref();

    if let Some(ref pat) = create.pattern {
        let regex = Regex::new(&pat)?;
        for path in fs::read_dir(&create.path)? {
            let buf = path?.path();
            let str = buf.to_string_lossy();
            if buf.is_dir() && regex.is_match(&str) {
                let create = Create {
                    path: str.to_string(),
                    pattern: None,
                    ..create.clone()
                };
                parse_create(create)?;
            }
        }
        Ok(())
    } else {
        if fs::try_exists(path.join("config.yml"))? {
            with_config(create)
        } else {
            bail!(
                "`config.yml` is not exists in {path:?}, \n \
            at the current time conan support only `config.yml` as config file"
            )
        }
    }
}
