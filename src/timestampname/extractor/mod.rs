use std::path::PathBuf;
use std::io::Seek;
use std::io::Read;
use std::fs::File;
use std::io::Take;
use std::io::SeekFrom;

use super::failures::Failure;
use super::FileMetadata;

mod tiff;

pub fn extract_metadata_creation_timestamp(path: &PathBuf) -> Result<Option<FileMetadata>, Failure> {
    let ext: String = path.extension()
        .and_then(|x| x.to_str())
        .map_or("".to_string(), |x| x.to_lowercase());

    match ext.as_str() {
        "nef" => tiff::tiff_extract_metadata_creation_timestamp_file(path, &ext),
        "dng" => tiff::tiff_extract_metadata_creation_timestamp_file(path, &ext),
        _ => Ok(None)
    }
}

enum Endianness {
    Big,
    Little,
}

pub trait Input {
    fn file_name(&self) -> String;
    fn seek(&mut self, position: u64) -> Result<(), Failure>;
    fn read_u16(&mut self, bo: &Endianness) -> Result<u16, Failure>;
    fn read_u32(&mut self, bo: &Endianness) -> Result<u32, Failure>;
    fn read_u64(&mut self, bo: &Endianness) -> Result<u64, Failure>;
    fn read_string(&mut self, len: u64) -> Result<String, Failure>;
}

struct FileInput {
    file_name: String,
    open_file: File,
}

impl Input for FileInput {
    fn file_name(&self) -> String {
        return self.file_name.to_string();
    }

    fn seek(&mut self, position: u64) -> Result<(), Failure> {
        return self.open_file.seek(SeekFrom::Start(position))
            .map_err(|e| Failure::file_failure_caused(
                self.file_name.to_string(),
                "seeking".to_string(),
                e))
            .map(|_v| ());
    }

    fn read_u16(&mut self, bo: &Endianness) -> Result<u16, Failure> {
        let mut buf: [u8; 2] = [0; 2];
        self.open_file.read_exact(&mut buf)
            .map_err(|e| Failure::file_failure_caused(
                self.file_name.to_string(),
                "reading u16".to_string(),
                e))?;
        match bo {
            Endianness::Big => Ok(u16::from_be_bytes(buf)),
            Endianness::Little => Ok(u16::from_le_bytes(buf))
        }
    }

    fn read_u32(&mut self, bo: &Endianness) -> Result<u32, Failure> {
        let mut buf: [u8; 4] = [0; 4];
        self.open_file.read_exact(&mut buf)
            .map_err(|e| Failure::file_failure_caused(
                self.file_name.to_string(),
                "reading u32".to_string(),
                e))?;
        match bo {
            Endianness::Big => Ok(u32::from_be_bytes(buf)),
            Endianness::Little => Ok(u32::from_le_bytes(buf))
        }
    }

    fn read_u64(&mut self, bo: &Endianness) -> Result<u64, Failure> {
        let mut buf: [u8; 8] = [0; 8];
        self.open_file.read_exact(&mut buf)
            .map_err(|e| Failure::file_failure_caused(
                self.file_name.to_string(),
                "reading u64".to_string(),
                e))?;
        match bo {
            Endianness::Big => Ok(u64::from_be_bytes(buf)),
            Endianness::Little => Ok(u64::from_le_bytes(buf))
        }
    }

    fn read_string(&mut self, len: u64) -> Result<String, Failure> {
        let open_file_ref: &mut File = self.open_file.by_ref();
        let mut take_input: Take<&mut File> = open_file_ref.take(len);
        let mut buffer = String::new();
        let read = take_input.read_to_string(&mut buffer)
            .map_err(|e| Failure::file_failure_caused(
                self.file_name.to_string(),
                "reading string".to_string(),
                e))?;
        if (read as u64) < len {
            return Err(Failure::file_failure(
                self.file_name.to_string(),
                "failed to read string in one block".to_string()));
        }
        return Ok(buffer);
    }
}
