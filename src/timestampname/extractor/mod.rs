use std::path::PathBuf;
use std::io::Seek;
use std::io::Read;
use std::fs::File;
use std::io::SeekFrom;
use std::io;

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

impl ByteRead for File {}

// goddammit...
// https://github.com/rust-lang/rust/issues/37214
//impl<T: ByteRead> ByteRead for io::Take<T> {}

struct FileSection {
    open_file: File,
    offset: u64,
    limit: u64,
    cursor: u64,
}

impl Read for FileSection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.open_file.read(buf);
        match &read {
            Err(e) => return read,
            Ok(b) => {
                self.cursor = self.cursor + *b as u64;
                return read;
            }
        }
    }
}

impl Seek for FileSection {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(p) => {
                // TODO check for overflow:
                if p + self.offset >= self.limit {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "seek beyond section end"));
                }
                self.cursor = p + self.offset;
                let new_pos = SeekFrom::Start(self.cursor);
                return self.open_file.seek(new_pos);
            }
            SeekFrom::End(p) => {
                // according to documentation, it is end of the file + position,
                // seeking forward means EOF for us:
                if p > 0 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "seek beyond section end"));
                }
                // if we are still here then p is either negative or 0:
                // u64 + (-i64), how to do that properly?
                // try to transform both into u64 and then do - operation:
                let minus_p = (p * -1) as u64;
                // the p value might get us before the offset:
                if minus_p > self.limit {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "seek before section start"));
                }
                self.cursor = self.limit - minus_p;
                // TODO check for overflow:
                let new_pos = SeekFrom::Start(self.offset + self.cursor);
                return self.open_file.seek(new_pos);
            }
            SeekFrom::Current(p) => {
                // TODO check for overflow:
                // positive p can go over the limit:
                if p > 0 && self.cursor + (p as u64) >= self.limit {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "seek beyond section end"));
                }
                // negative p larger than cursor means we go below 0:
                if p < 0 && (p * -1) as u64 > self.cursor {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "seek before section start"));
                }
                // should be a better way to work with u64 + i64:
                if p > 0 {
                    // TODO check for overflow:
                    self.cursor = self.cursor + p as u64;
                } else {
                    self.cursor = self.cursor - (p * -1) as u64;
                }
                let new_pos = SeekFrom::Current(p);
                return self.open_file.seek(new_pos);
            }
        }
    }
}

impl ByteRead for FileSection {
    fn read_u16(&mut self, bo: &Endianness) -> io::Result<u16> {
        if self.cursor + 2 >= self.limit {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_u16 beyond section end"));
        }
        self.cursor = self.cursor + 2;
        return self.open_file.read_u16(bo);
    }
    fn read_u32(&mut self, bo: &Endianness) -> io::Result<u32> {
        if self.cursor + 4 >= self.limit {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_u32 beyond section end"));
        }
        self.cursor = self.cursor + 4;
        return self.open_file.read_u32(bo);
    }
    fn read_u64(&mut self, bo: &Endianness) -> io::Result<u64> {
        if self.cursor + 8 >= self.limit {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_u64 beyond section end"));
        }
        self.cursor = self.cursor + 8;
        return self.open_file.read_u64(bo);
    }
    fn read_string(&mut self, len: u64) -> io::Result<String> {
        if self.cursor + len >= self.limit {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_string beyond section end"));
        }
        self.cursor = self.cursor + len;
        return self.open_file.read_string(len);
    }
}
