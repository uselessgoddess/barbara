use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use colored::Colorize;
use regex::Regex;
use async_recursion::async_recursion;

use serde_yaml::{Mapping, Value};

#[derive(clap::Parser, Clone)]
/// Wrapper on `conan create`
pub struct Create {
    /// Path to [package folder]/[pattern folder]
    path: String,

    /// Desired version TODO: bug with `--pattern`
    #[clap(long = "version", default_value = "latest")]
    version: String,

    /// Profile to `create`
    #[clap(long = "profile", default_value = "default")]
    profile: String,

    /// Pattern to `create` some package from `path` with `pattern`
    #[clap(long = "pattern")]
    pattern: Option<String>,
}

fn to_ver_folder(versions: &'a Value) -> Vec<(&'a str, &'a str)> {
    versions.as_mapping().unwrap().iter()
        .map(|(ver, val)| (ver.as_str().unwrap(), val["folder"].as_str().unwrap()))
        .collect()
}

async fn create_package(path: &Path, package: &str, version: &str, folder: &str, profile: &str) -> std::io::Result<Output> {
    let pattern = format!("{package}/{version}@",
        package=package, version=version
    );
    let to_folder = path.join(folder);
    Command::new("conan")
        .args(["create", to_folder.to_str().unwrap(), &pattern])
        .arg(format!("-pr={}", profile))
        .output()
}

fn to_pattern(package: &str, version: &str) -> String {
    format!("{package}/{version}@",
        package=package, version=version
    )
}

fn on_creating(package: &str, version: &str) {
    let pattern = to_pattern(package, version);
    print!("{} ", "creating".bright_cyan());
    println!("{}", pattern.as_str().bright_green());
}

fn on_created(package: &str, version: &str) {
    let pattern = to_pattern(package, version);
    print!("{}", pattern.as_str().bright_green());
    println!(" - {}", "created".bright_cyan());
}

async fn with_config(create: Create) -> Result<(), Box<dyn Error>> {
    let path: &Path = create.path.as_ref();

    let config = fs::read_to_string(path.join("config.yml")).unwrap();
    let yaml: Value = serde_yaml::from_str(&config)?;

    let notation = to_ver_folder(&yaml["versions"]);
    let (version, folder) = if create.version == "latest" {
        notation.iter().max().unwrap()
    } else {
        notation.iter().find(|(ver, _)| ver == &create.version).unwrap_or_else(|| {
            panic!("version not found");
        })
    };

    let package = path.file_name().map(|s|
        s.to_string_lossy().to_string()
    ).unwrap();

    on_creating(&package, version);
    create_package(path, &package, version, folder, &create.profile).await?;
    on_created(&package, version);

    Ok(())
}

#[async_recursion]
pub async fn parse_create(create: Create) -> Result<(), Box<dyn Error>> {
    let path: &Path = create.path.as_ref();

    if create.pattern.is_some() {
        let pattern = create.pattern.unwrap();
        let regex = Regex::new(&pattern)?;
        for path in std::fs::read_dir(&create.path)? {
            let buf = path?.path();
            let str = buf.to_str().ok_or("non-UTF-8 compatible string")?;
            if buf.is_dir() && regex.is_match(str) {
                let create = Create {
                    path: buf.to_string_lossy().to_string(),
                    version: create.version.clone(),
                    profile: create.profile.clone(),
                    pattern: None
                };
                parse_create(create).await?;
            }
        }
        Ok(())
    } else {
        let result = if fs::try_exists(path.join("config.yml"))? {
            with_config(create)
        } else {
            todo!()
        };
        Ok(result.await?)
    }
}