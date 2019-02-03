use std::path::PathBuf;
use std::io::Seek;
use std::io::Read;
use std::fs::File;
use std::io::Take;
use std::io::SeekFrom;
use std::io;
use std::fs;

use super::failures::Failure;
use super::FileMetadata;

mod tiff;
//mod quicktime;
//mod mp4;

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

pub trait ByteRead: Read + Seek {
    fn ff(&mut self, position: u64) -> io::Result<()> {
        return self.seek(SeekFrom::Start(position))
            .map(|_v| ());
    }
    fn read_u16(&mut self, bo: &Endianness) -> io::Result<u16> {
        let mut buf: [u8; 2] = [0; 2];
        self.read_exact(&mut buf)?;
        match bo {
            Endianness::Big => Ok(u16::from_be_bytes(buf)),
            Endianness::Little => Ok(u16::from_le_bytes(buf))
        }
    }
    fn read_u32(&mut self, bo: &Endianness) -> io::Result<u32> {
        let mut buf: [u8; 4] = [0; 4];
        self.read_exact(&mut buf)?;
        match bo {
            Endianness::Big => Ok(u32::from_be_bytes(buf)),
            Endianness::Little => Ok(u32::from_le_bytes(buf))
        }
    }
    fn read_u64(&mut self, bo: &Endianness) -> io::Result<u64> {
        let mut buf: [u8; 8] = [0; 8];
        self.read_exact(&mut buf)?;
        match bo {
            Endianness::Big => Ok(u64::from_be_bytes(buf)),
            Endianness::Little => Ok(u64::from_le_bytes(buf))
        }
    }
    fn read_string(&mut self, len: u64) -> io::Result<String> {
        let mut take_input = self.take(len);
        let mut buffer = String::new();
        let read = take_input.read_to_string(&mut buffer)?;
        if (read as u64) < len {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "string read result is shorter than expected"));
        }
        return Ok(buffer);
    }
}

impl ByteRead for fs::File {}
// goddammit...
// https://github.com/rust-lang/rust/issues/37214
//impl<T: ByteRead> ByteRead for io::Take<T> {}
