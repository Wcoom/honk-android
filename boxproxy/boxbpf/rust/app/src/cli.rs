use std::ffi::OsString;

use clap::{Arg, Command, value_parser};

/// The command `units::dispatch` gets when argv names no subcommand.
pub const ROOT: &str = "boxbpf";

fn rest() -> Arg {
    Arg::new("args")
        .num_args(0..)
        .trailing_var_arg(true)
        .allow_hyphen_values(true)
        .value_parser(value_parser!(OsString))
}

/// Routes all argv flags (`--probe`, `--init`, `--apply`, `--update`, `--clear`) to `ROOT`.
pub fn command() -> Command {
    Command::new(ROOT)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .subcommand(Command::new("config-check").hide(true).arg(rest()))
        .arg(rest())
}
