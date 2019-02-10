use std::path::PathBuf;

use super::failures::Failure;
use super::FileMetadata;
use super::extractor::input::Input;
use super::extractor::inputbox::InputBox;

mod inputbox;
mod input;
mod tiff;
mod quicktime;
mod mp4;

pub fn extract_metadata_creation_timestamp(path: &PathBuf, utc: bool) -> Result<Option<FileMetadata>, Failure> {
    let ext: String = path.extension()
        .and_then(|x| x.to_str())
        .map_or("".to_string(), |x| x.to_lowercase());

    match ext.as_str() {
        "nef" => tiff::tiff_extract_metadata_creation_timestamp(&mut Input::create(&InputBox::create(path, ext)?)),
        "dng" => tiff::tiff_extract_metadata_creation_timestamp(&mut Input::create(&InputBox::create(path, ext)?)),
        "mp4" => mp4::mp4_extract_metadata_creation_timestamp(&mut Input::create(&InputBox::create(path, ext)?), utc),
        _ => Ok(None)
    }
}

pub enum Endianness {
    Big,
    Little,
}
