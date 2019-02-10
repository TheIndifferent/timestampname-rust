pub mod failures;
mod renamer;
mod verifier;
mod executor;
mod extractor;

use std::fs::ReadDir;
use std::io;
use std::path::PathBuf;

use super::timestampname::failures::Failure;

pub struct CommandLineArguments {
    pub dry_run: bool,
    pub no_prefix: bool,
    pub debug_output: bool,
    pub utc: bool,
}

pub struct FileMetadata {
    file_name: String,
    creation_timestamp: String,
    extension: String
}

pub struct RenameOperation {
    from: String,
    to: String,
}

struct CollectedMetadata {
    items: Vec<FileMetadata>,
    longest_source_name: usize,
}

pub fn execute(cwd: PathBuf, cmd_args: CommandLineArguments) -> Result<(), Failure> {
    print!("Scanning for files...");
    let files: Vec<PathBuf> = list_files(cwd)?;
    println!(" {} files found.", files.len());

    let collected_metadata: CollectedMetadata = process_files(files, cmd_args.utc)?;

    if collected_metadata.items.is_empty() {
        println!("No supported files found.");
        return Ok(());
    }

    print!("Preparing rename operations...");
    let operations: Vec<RenameOperation>
        = renamer::prepare_rename_operations(collected_metadata.items, cmd_args.no_prefix)?;
    println!(" done.");

    println!("Verifying:");
    verifier::verify_operations(&operations, collected_metadata.longest_source_name)?;
    println!("done.");

    executor::execute_operations(&operations, cmd_args.dry_run)?;

    println!("\nFinished.");
    return Ok(());
}

fn list_files(cwd: PathBuf) -> Result<Vec<PathBuf>, Failure> {
    let read_dir_res: Result<ReadDir, io::Error> = cwd.read_dir();
    let files: Result<Vec<PathBuf>, io::Error> = read_dir_res.and_then(|iterator| {
        let mut vec: Vec<PathBuf> = Vec::new();
        for entry in iterator {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                vec.push(entry.path());
            }
        }
        return Ok(vec);
    });
    let res: Result<Vec<PathBuf>, Failure> = files
        .map_err(|e| Failure::env_failure_caused("Failed to list directory contents".to_string(), e));
    return res;
}

fn process_files(files: Vec<PathBuf>, utc: bool) -> Result<CollectedMetadata, Failure> {
    let mut res: Vec<FileMetadata> = Vec::new();
    let mut longest_source_name: usize = 0;
    for (index, element) in files.iter().enumerate() {
        print!("\rProcessing files: {}/{}...", index + 1, files.len());
        let md: Option<FileMetadata> = extractor::extract_metadata_creation_timestamp(element, utc)?;
        match md {
            Some(x) => {
                if x.file_name.len() > longest_source_name {
                    longest_source_name = x.file_name.len();
                }
                res.push(x);
            }
            _ => {}
        }
    }
    println!(" {} supported files found.", res.len());
    return Ok(CollectedMetadata {
        items: res,
        longest_source_name,
    });
}
