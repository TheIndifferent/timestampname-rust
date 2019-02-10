use super::input::Input;
use super::FileMetadata;
use super::Failure;
use super::Endianness;
use crate::timestampname::extractor::tiff::tiff_extract_metadata_creation_timestamp;
use std::io;

// following resources were used to implement this parser:
// https://www.media.mit.edu/pia/Research/deepview/exif.html
// https://www.fileformat.info/format/jpeg/egff.htm
// http://vip.sugovica.hu/Sardi/kepnezo/JPEG%20File%20Layout%20and%20Format.htm

const JPEG_SOI: u16 = 0xFFD8;
const JPEG_APP1: u16 = 0xFFE1;
const EXIF_HEADER_SUFFIX: u16 = 0x0000;

#[derive(Debug)]
enum JpegError {
    Input {
        description: String
    },
    Io {
        operation: String,
        cause: io::Error,
    },
}

fn err_input(description: String) -> JpegError {
    return JpegError::Input {
        description
    };
}

fn err_io(operation: String, cause: io::Error) -> JpegError {
    return JpegError::Io {
        operation,
        cause,
    };
}

fn jpeg_scan_for_app1<'a>(input: &'a mut Input) -> Result<Input<'a>, JpegError> {
    // checking JPEG SOI:
    let jpeg_soi = input.read_u16(&Endianness::Big)
        .map_err(|e| err_io("reading jpeg header".to_string(), e))?;
    if jpeg_soi != JPEG_SOI {
        return Err(err_input(format!("unexpected JPEG SOI: {}", jpeg_soi)));
    }
    // scrolling through fields until we find APP1:
    loop {
        let field_marker = input.read_u16(&Endianness::Big)
            .map_err(|e| err_io("reading jpeg field marker".to_string(), e))?;
        let field_length = input.read_u16(&Endianness::Big)
            .map_err(|e| err_io("reading jpeg field length".to_string(), e))?;
        if field_marker == JPEG_APP1 {
            // APP1 marker found, checking Exif header:
            let exif_header = input.read_string(4)
                .map_err(|e| err_io("reading jpeg exif header".to_string(), e))?;
            let exif_header_suffix = input.read_u16(&Endianness::Big)
                .map_err(|e| err_io("reading jpeg exif header suffix".to_string(), e))?;
            if &exif_header != "Exif" || exif_header_suffix != EXIF_HEADER_SUFFIX {
                return Err(err_input("JPEG APP1 field does not have valid Exif header".to_string()));
            }
            // body is a valid TIFF,
            // size decrements:
            //   -2 field length
            //   -4 exif header
            //   -2 exif header suffix
            let exif_input = input.section(field_length as u64 - 8);
            return Ok(exif_input);
        }
        // length includes the length itself:
        input.ff(field_length as u64 - 2)
            .map_err(|e| err_io("fast-forward jpeg field".to_string(), e))?;
    }
}

pub fn jpeg_extract_metadata_creation_timestamp(input: &mut Input) -> Result<Option<FileMetadata>, Failure> {
    let file_name = input.name().to_string();
    let mut exif_input = jpeg_scan_for_app1(input)
        .map_err(|e| match e {
            JpegError::Input { description } =>
                Failure::file_failure(file_name.to_string(), description),
            JpegError::Io { operation, cause } =>
                Failure::file_failure_caused(file_name.to_string(), operation, cause),
        })?;
    return tiff_extract_metadata_creation_timestamp(&mut exif_input);
}
