use std::env;
use std::process;

mod timestampname;

fn main() {
    let mut dry_run: bool = false;
    let mut no_prefix: bool = false;
    let mut debug_output: bool = false;
    let mut utc: bool = false;
    for arg in env::args().skip(1) {
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
            "-h" => {
                print_help_and_exit();
            },
            _ => {
                eprintln!("Unrecognized argument: {}", arg);
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

fn print_help_and_exit() {
    println!("
Usage: TimestampNameRust [ options ]

Options:
    -h          Display help and exit.
    -dry        Only show the operations but do not perform a rename.
    -debug      Enable debug output.
    -noprefix   Do not add numerical prefix to the renamed files
                (works if not more than one file is shot per second).
    -utc        Do not reinterpret MP4 timestamps into local time zone.
                Even though specification suggests to use UTC for CreationDate
                and ModificationDate, some cameras (DJI?) are saving it
                in a local time zone, so the time zone offset will double
                if we will apply conversion to local time zone on top of it.
                This option will produce incorrectly named files if a folder
                contains video files from DJI and Samsung for example.");
    process::exit(0);
}
