use std::path::PathBuf;
use std::path::Path;
use crate::timestampname::failures::Failure;

pub mod failures;

pub struct CommandLineArguments {
    pub dry_run: bool,
    pub no_prefix: bool,
    pub debug_output: bool,
    pub utc: bool,
}

pub fn execute(cwd: PathBuf, cmd_args: CommandLineArguments) -> Result<(), failures::Failure> {
    println!("path: {}, args: {}", cwd.to_str().unwrap(), cmd_args.debug_output);
    return Ok(());
}

fn list_files(cwd: PathBuf) -> Result<Vec<PathBuf>, Failure> {
    let res: Result<Vec<PathBuf>, Failure> = cwd.read_dir()
        .and_then(|iterator| {
            let mut vec: Vec<PathBuf> = Vec::new();
            for entry in iterator {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    continue;
                }
                vec.push(entry.path());
            }
            return Ok(vec);
        })
        .map_err(|e|
            Failure::env_failure_caused("Failed to list directory contents".to_string(), e));
    return res;
}
