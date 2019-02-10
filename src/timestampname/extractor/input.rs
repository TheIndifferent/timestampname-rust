use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;

use super::Endianness;
use super::Failure;
use super::inputbox::InputBox;

pub struct Input<'f> {
    file: &'f InputBox,
    offset: u64,
    limit: u64,
    cursor: u64,
}

impl<'f> Input<'f> {
    pub fn create(input_box: &InputBox) -> Input {
        return Input {
            file: input_box,
            offset: 0,
            limit: input_box.size(),
            cursor: 0,
        };
    }
    pub fn name(&self) -> &str {
        return self.file.name();
    }
    pub fn ext(&self) -> &str {
        return self.file.ext();
    }

    pub fn read_u16(&mut self, bo: &Endianness) -> io::Result<u16> {
        // TODO overflow check
        if self.cursor + 2 >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: reading 2 bytes from {}, input length: {}", self.cursor, self.limit)));
        }
        let mut buf: [u8; 2] = [0; 2];
        self.file.file().read_exact(&mut buf)?;
        self.cursor = self.cursor + 2;
        match bo {
            Endianness::Big => Ok(u16::from_be_bytes(buf)),
            Endianness::Little => Ok(u16::from_le_bytes(buf))
        }
    }
    pub fn read_u32(&mut self, bo: &Endianness) -> io::Result<u32> {
        // TODO overflow check
        if self.cursor + 4 >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: reading 4 bytes from {}, input length: {}", self.cursor, self.limit)));
        }
        let mut buf: [u8; 4] = [0; 4];
        self.file.file().read_exact(&mut buf)?;
        self.cursor = self.cursor + 4;
        match bo {
            Endianness::Big => Ok(u32::from_be_bytes(buf)),
            Endianness::Little => Ok(u32::from_le_bytes(buf))
        }
    }
    pub fn read_u64(&mut self, bo: &Endianness) -> io::Result<u64> {
        // TODO overflow check
        if self.cursor + 8 >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: reading 8 bytes from {}, input length: {}", self.cursor, self.limit)));
        }
        let mut buf: [u8; 8] = [0; 8];
        self.file.file().read_exact(&mut buf)?;
        self.cursor = self.cursor + 8;
        match bo {
            Endianness::Big => Ok(u64::from_be_bytes(buf)),
            Endianness::Little => Ok(u64::from_le_bytes(buf))
        }
    }
    pub fn read_string(&mut self, len: u64) -> io::Result<String> {
        // TODO overflow check
        if self.cursor + len >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: reading {} bytes from {}, input length: {}", len, self.cursor, self.limit)));
        }
        let mut take_input = self.file.file().take(len);
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
    pub fn seek(&mut self, pos: u64) -> io::Result<()> {
        if pos >= self.limit {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                format!("EOF: seeking position {}, input length: {}", pos, self.limit)));
        }
        // TODO overflow check
        self.file.file().seek(SeekFrom::Start(self.offset + pos))?;
        self.cursor = pos;
        return Ok(());
    }
    pub fn ff(&mut self, len: u64) -> io::Result<()> {
        // TODO maybe implement with SeekFrom::Current?
        return self.seek(self.cursor + len);
    }
    pub fn section(&mut self, len: u64) -> Input<'f> {
        return Input {
            file: self.file,
            offset: self.cursor,
            limit: len,
            cursor: 0,
        };
    }
}
