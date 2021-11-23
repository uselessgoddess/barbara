#![feature(path_try_exists)]
#![feature(in_band_lifetimes)]

mod create;

use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use clap::Parser;
use colored::Colorize;
use regex::Regex;
use serde_yaml::Value;

/// Tool for conan
#[derive(Parser)]
#[clap(version = "1.0")]
struct Options {
    #[clap(subcommand)]
    sub_cmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Create(create::Create),
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Options = Options::parse();

    match opts.sub_cmd {
        SubCommand::Create(create) => {
            create::parse_create(create).await
        }
        _ => {
            todo!()
        }
    }
}
