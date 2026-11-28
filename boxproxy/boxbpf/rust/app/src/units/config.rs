//! Glue for `config-rs` (`boxproxy/boxbpf/config.c`).

use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use clap::ArgMatches;
use config_rs::port::read_config_json;

pub fn run(_command: &str, matches: &ArgMatches) -> ExitCode {
    let path = matches
        .get_many::<OsString>("args")
        .and_then(|mut it| it.next())
        .map(Path::new);
    match path.and_then(read_config_json) {
        Some(_) => ExitCode::SUCCESS,
        None => ExitCode::from(2),
    }
}
