#![feature(fs_try_exists)]

mod create;

use clap::Parser;
use colored::Colorize;
use regex::Regex;
use serde_yaml::Value;

/// Tool for conan
#[derive(clap::Parser)]
#[clap(version = "1.0")]
struct Options {
    #[clap(subcommand)]
    sub_cmd: SubCommand,
}

#[derive(clap::Subcommand)]
enum SubCommand {
    Create(create::Create),
}

fn main() {
    let opts: Options = Options::parse();

    let result = match opts.sub_cmd {
        SubCommand::Create(create) => create::parse_create(create),
        #[allow(unreachable_patterns)]
        _non_exhaustive => {
            todo!()
        }
    };

    if let Err(err) = result {
        println!("{}: {}", "ERROR".red(), err.to_string().bold());
    }
}
