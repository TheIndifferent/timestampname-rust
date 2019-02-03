use std::path::PathBuf;
use std::fs::File;

use super::FileMetadata;
use super::Failure;
use super::FileInput;

pub fn mp4_extract_metadata_creation_timestamp_file(path: &PathBuf, ext: &String) -> Result<Option<FileMetadata>, Failure> {
    let file_name: String = path.file_name()
        .and_then(|f| f.to_str())
        .map(|f| f.to_string())
        .expect("mp4 got path without a file name");
    let open_file = File::open(path)
        .map_err(|e| Failure::file_failure_caused(
            file_name.to_string(),
            "failed to open file".to_string(),
            e))?;
    let mut input = FileInput {
        file_name: file_name.to_string(),
        open_file,
    };
}
