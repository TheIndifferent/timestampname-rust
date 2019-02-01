use std::path::PathBuf;
use std::error::Error;

pub mod failures;

pub struct CommandLineArguments {
    pub dry_run: bool,
    pub no_prefix: bool,
    pub debug_output: bool,
    pub utc: bool
}

pub fn execute(cwd: PathBuf, cmd_args: CommandLineArguments) -> Result<(), Error> {
    return Ok(())
}
