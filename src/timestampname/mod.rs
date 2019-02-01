use std::path::PathBuf;

pub mod failures;

pub struct CommandLineArguments {
    pub dry_run: bool,
    pub no_prefix: bool,
    pub debug_output: bool,
    pub utc: bool
}

pub fn execute(cwd: PathBuf, cmd_args: CommandLineArguments) -> Result<(), failures::Failure> {
    println!("path: {}, args: {}", cwd.to_str().unwrap(), cmd_args.debug_output);
    return Ok(())
}
