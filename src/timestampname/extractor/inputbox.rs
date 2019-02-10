use std::fs::File;
use std::path::Path;

use super::Failure;

pub struct InputBox {
    file_name: String,
    file_ext: String,
    file_size: u64,
    file: File,
}

impl InputBox {
    pub fn create(path: &Path, ext: String) -> Result<InputBox, Failure> {
        let file_name: String = path.file_name()
            .and_then(|f| f.to_str())
            .map(|f| f.to_string())
            .expect("failed to create Input from path without filename");
        let file = File::open(path)
            .map_err(|e| Failure::file_failure_caused(
                file_name.to_string(),
                "failed to open file".to_string(),
                e))?;
        let file_size = file.metadata().map(|m| m.len())
            .map_err(|e| Failure::file_failure_caused(
                file_name.to_string(),
                "failed to get file metadata".to_string(),
                e))?;
        return Ok(InputBox {
            file_name,
            file_ext: ext,
            file_size,
            file,
        });
    }
    pub fn name(&self) -> &str {
        return &self.file_name;
    }
    pub fn ext(&self) -> &str {
        return &self.file_ext;
    }
    pub fn size(&self) -> u64 {
        return self.file_size;
    }
    pub fn file(&self) -> &File {
        return &self.file;
    }
}
