use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;

use super::failures::Failure;
use super::FileMetadata;
use super::extractor::input::Input;
use crate::timestampname::extractor::inputbox::InputBox;

mod inputbox;
mod input;
mod tiff;
mod quicktime;
mod mp4;

pub fn extract_metadata_creation_timestamp(path: &PathBuf) -> Result<Option<FileMetadata>, Failure> {
    let ext: String = path.extension()
        .and_then(|x| x.to_str())
        .map_or("".to_string(), |x| x.to_lowercase());

    match ext.as_str() {
        "nef" => tiff::tiff_extract_metadata_creation_timestamp(&mut Input::create(&InputBox::create(path, ext)?)),
        "dng" => tiff::tiff_extract_metadata_creation_timestamp(&mut Input::create(&InputBox::create(path, ext)?)),
        _ => Ok(None)
    }
}

enum Endianness {
    Big,
    Little,
}
