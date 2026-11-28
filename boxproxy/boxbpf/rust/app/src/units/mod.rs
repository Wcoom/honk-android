//! Written by `rust-anything app lift`: every command is answered in Rust.

use std::process::ExitCode;

use clap::ArgMatches;

mod config;
mod loader;

pub fn dispatch(command: &str, matches: &ArgMatches) -> Option<ExitCode> {
    match command {
        "config-check" => Some(config::run(command, matches)),
        "boxbpf" => Some(loader::run(command, matches)),
        _ => None,
    }
}
