use std::fs::ReadDir;
use std::io;
use std::path::PathBuf;

use crate::timestampname::failures::Failure;

pub mod failures;

pub struct CommandLineArguments {
    pub dry_run: bool,
    pub no_prefix: bool,
    pub debug_output: bool,
    pub utc: bool,
}

struct FileMetadata {
    file_name: String,
    creation_timestamp: String
}

struct CollectedMetadata {
    items: Vec<FileMetadata>,
    longest_source_name: usize
}

pub fn execute(cwd: PathBuf, cmd_args: CommandLineArguments) -> Result<(), Failure> {
    print!("Scanning for files... ");
    let files: Vec<PathBuf> = list_files(cwd)?;
    println!("{} files found.", files.len());

    let collected_metadata: CollectedMetadata = process_files(files, cmd_args.utc)?;

    return Ok(());
}

fn list_files(cwd: PathBuf) -> Result<Vec<PathBuf>, Failure> {
    let read_dir_res: Result<ReadDir, io::Error> = cwd.read_dir();
    let files: Result<Vec<PathBuf>, io::Error> = read_dir_res.and_then(|iterator|{
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
        print!("\rProcessing files: {}/{}...", index+1, files.len());
        let md: Option<FileMetadata> = extract_creation_timestamp(element)?;
        match md {
            Some(x) => {
                if x.file_name.len() > longest_source_name {
                    longest_source_name = x.file_name.len();
                }
                res.push(x);
            },
            _ => {}
        }
    }
    println!("{} supported files found.", res.len());
    return Ok(CollectedMetadata {
        items: res,
        longest_source_name
    });
}

fn extract_creation_timestamp(path: &PathBuf) -> Result<Option<FileMetadata>, Failure> {
    unimplemented!("extract creation timestamp from file")
}
