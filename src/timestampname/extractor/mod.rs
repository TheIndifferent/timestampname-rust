use std::path::PathBuf;
use std::io::Seek;
use std::io::Read;
use std::fs::File;
use std::io::SeekFrom;
use std::io;

use super::failures::Failure;
use super::FileMetadata;
use std::io::ErrorKind;

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

pub trait Input {
    fn read_u16(&mut self, bo: &Endianness) -> io::Result<u16>;
    fn read_u32(&mut self, bo: &Endianness) -> io::Result<u32>;
    fn read_u64(&mut self, bo: &Endianness) -> io::Result<u64>;
    fn read_string(&mut self, len: u64) -> io::Result<String>;
    fn seek(&mut self, pos: u64) -> io::Result<()>;
    fn section<T>(&mut self, len: u64) -> T where T: Input;
}

struct FileInput<'f> {
    open_file: &'f File,
    offset: u64,
    limit: u64,
    cursor: u64
}

struct SectionInput<'s, P> {
    parent_input: &'s P,
    offset: u64,
    limit: u64,
    cursor: u64
}

impl<'s, P> Input for SectionInput<'s, P> {
    fn read_u16(&mut self, bo: &Endianness) -> io::Result<u16> {
        unimplemented!()
    }
    fn read_u32(&mut self, bo: &Endianness) -> io::Result<u32> {
        unimplemented!()
    }
    fn read_u64(&mut self, bo: &Endianness) -> io::Result<u64> {
        unimplemented!()
    }
    fn read_string(&mut self, len: u64) -> io::Result<String> {
        unimplemented!()
    }
    fn seek(&mut self, pos: u64) -> io::Result<()> {
        unimplemented!()
    }
    fn section<T>(&mut self, len: u64) -> T where T: Input {
        unimplemented!()
    }
}

impl<'f> Input for FileInput<'f> {
    fn read_u16(&mut self, bo: &Endianness) -> io::Result<u16> {
        // TODO overflow check
        if self.cursor + 2 >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: reading 2 bytes from {}, input length: {}", self.cursor, self.limit)));
        }
        let mut buf: [u8; 2] = [0; 2];
        self.open_file.read_exact(&mut buf)?;
        self.cursor = self.cursor + 2;
        match bo {
            Endianness::Big => Ok(u16::from_be_bytes(buf)),
            Endianness::Little => Ok(u16::from_le_bytes(buf))
        }
    }
    fn read_u32(&mut self, bo: &Endianness) -> io::Result<u32> {
        // TODO overflow check
        if self.cursor + 4 >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: reading 4 bytes from {}, input length: {}", self.cursor, self.limit)));
        }
        let mut buf: [u8; 4] = [0; 4];
        self.open_file.read_exact(&mut buf)?;
        self.cursor = self.cursor + 4;
        match bo {
            Endianness::Big => Ok(u32::from_be_bytes(buf)),
            Endianness::Little => Ok(u32::from_le_bytes(buf))
        }
    }
    fn read_u64(&mut self, bo: &Endianness) -> io::Result<u64> {
        // TODO overflow check
        if self.cursor + 8 >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: reading 8 bytes from {}, input length: {}", self.cursor, self.limit)));
        }
        let mut buf: [u8; 8] = [0; 8];
        self.open_file.read_exact(&mut buf)?;
        self.cursor = self.cursor + 8;
        match bo {
            Endianness::Big => Ok(u64::from_be_bytes(buf)),
            Endianness::Little => Ok(u64::from_le_bytes(buf))
        }
    }
    fn read_string(&mut self, len: u64) -> io::Result<String> {
        // TODO overflow check
        if self.cursor + len >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: reading {} bytes from {}, input length: {}", len, self.cursor, self.limit)));
        }
        let mut take_input = self.open_file.take(len);
        let mut buffer = String::new();
        let read = take_input.read_to_string(&mut buffer)?;
        if (read as u64) < len {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "string read result is shorter than expected"));
        }
        self.cursor = self.cursor + len;
        return Ok(buffer);
    }
    fn seek(&mut self, pos: u64) -> io::Result<()> {
        if pos >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: seeking position {}, input length: {}", pos, self.limit)));
        }
        // TODO overflow check
        self.open_file.seek(SeekFrom::Start(self.offset + pos))?;
        self.cursor = pos;
        return Ok(());
    }
    fn section<T: Input>(&mut self, len: u64) -> T {
        return SectionInput {
            parent_input: self,
            offset: self.cursor,
            limit: len,
            cursor: 0
        };
    }
}
