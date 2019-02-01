use std::env;
use std::process;

mod timestampname;

fn main() {
    let mut dry_run: bool = false;
    let mut no_prefix: bool = false;
    let mut debug_output: bool = false;
    let mut utc: bool = false;
    for arg in env::args() {
        match arg.as_ref() {
            "-dry" => {
                dry_run = true;
            },
            "-debug" => {
                debug_output = true;
            },
            "-noprefix" => {
                no_prefix = true;
            },
            "-utc" => {
                utc = true;
            },
            _ => {
                eprintln!("unrecognized argument: {}", arg);
                process::exit(1);
            }
        }
    }

    let cmd_args = timestampname::CommandLineArguments {
        dry_run,
        no_prefix,
        debug_output,
        utc
    };

    match env::current_dir()
        .map_err(|e|
            timestampname::failures::Failure::env_failure_caused(
                "Get current working directory".to_string(), e))
        .and_then(|path| timestampname::execute(path, cmd_args)) {
        Err(e) => {
            eprintln!("Failure:\n{}\n", e);
            process::exit(1);
        },
        Ok(_) => {}
    }
}
