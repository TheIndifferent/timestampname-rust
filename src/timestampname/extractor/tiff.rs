use std::path::PathBuf;

use super::FileMetadata;
use super::Failure;
use std::fs::File;
use crate::timestampname::extractor::Endianness;
use std::error::Error;
use super::Input;

struct TiffError {
    description: String,
    cause: String,
}

fn tiff_err(description: String) -> TiffError {
    return TiffError {
        description,
        cause: "".to_string(),
    };
}

fn tiff_err_cause<E: Error>(description: String, cause: E) -> TiffError {
    return TiffError {
        description,
        cause: format!("{}", cause),
    };
}

pub fn tiff_extract_metadata_creation_timestamp_file(path: &PathBuf, ext: &String) -> Result<Option<FileMetadata>, Failure> {
    let file_name: String = path.file_name()
        .and_then(|f| f.to_str())
        .map(|f| f.to_string())
        .expect("TIFF got path without a file name");
    let mut open_file = File::open(path)
        .map_err(|e| Failure::file_failure_caused(
            file_name.to_string(),
            "failed to open file".to_string(),
            e))?;
    return tiff_extract_metadata_creation_timestamp(&mut open_file)
        .map(|t| {
            Some(FileMetadata {
                file_name: file_name.to_string(),
                creation_timestamp: t,
                extension: format!(".{}", ext),
            })
        })
        .map_err(|e| Failure::file_failure_strcause(
            file_name,
            e.description,
            e.cause));
}

// https://www.adobe.io/content/dam/udp/en/open/standards/tiff/TIFF6.pdf
fn tiff_extract_metadata_creation_timestamp(input: &mut impl Input) -> Result<String, TiffError> {
    // Bytes 0-1: The byte order used within the file. Legal values are:
    // “II” (4949.H)
    // “MM” (4D4D.H)
    let tiff_endianness_header: String = input.read_string(2)
        .map_err(|e| tiff_err_cause(
            "TIFF failed to read endianness header".to_string(), e))?;
    // In the “II” format, byte order is always from the least significant byte to the most
    // significant byte, for both 16-bit and 32-bit integers.
    // This is called little-endian byte order.
    //  In the “MM” format, byte order is always from most significant to least
    // significant, for both 16-bit and 32-bit integers.
    // This is called big-endian byte order
    let endianness: Endianness = match tiff_endianness_header.as_str() {
        "II" => Ok(Endianness::Little),
        "MM" => Ok(Endianness::Big),
        _ => Err(tiff_err(format!("invalid TIFF file header: {}", tiff_endianness_header)))
    }?;

    // Bytes 2-3 An arbitrary but carefully chosen number (42)
    // that further identifies the file as a TIFF file.
    let tiff_magic: u16 = input.read_u16(&endianness)
        .map_err(|e| tiff_err_cause(
            "TIFF failed to read magic number header".to_string(), e))?;
    if tiff_magic != 42 {
        return Err(tiff_err(format!("invalid TIFF magic number: {}", tiff_magic)));
    }

    let mut ifd_offsets: Vec<u32> = Vec::new();
    let mut date_tag_offsets: Vec<u32> = Vec::new();

    // Bytes 4-7 The offset (in bytes) of the first IFD.
    ifd_offsets.push(input.read_u32(&endianness)
        .map_err(|e| tiff_err_cause(
            "TIFF failed to read first IFD offset".to_string(), e))?);

    let mut earliest_creation_date: String = String::new();

    loop {
        if ifd_offsets.is_empty() && date_tag_offsets.is_empty() {
            // TIFF no more offsets to scavenge
            break;
        }

        // TODO should sorting happen here?
        // sorting to traverse file forward-only:
        ifd_offsets.sort_unstable();
        date_tag_offsets.sort_unstable();

        if !ifd_offsets.is_empty() || !date_tag_offsets.is_empty() {
            // TODO find a better way to solve this, maybe match?
            let next_date_offset: u32 = match date_tag_offsets.len() {
                x if x > 0 => date_tag_offsets[0],
                _ => u32::max_value()
            };
            let next_ifd_offset: u32 = match ifd_offsets.len() {
                x if x > 0 => ifd_offsets[0],
                _ => u32::max_value()
            };

            if next_date_offset < next_ifd_offset {
                // TIFF collecting date at offset
                date_tag_offsets.remove(0);
                input.seek(next_date_offset as u64)
                    .map_err(|e| tiff_err_cause(
                        format!("TIFF failed to fast-forward to next date tag offset: {}", next_date_offset), e))?;
                // reading 19 characters of string:
                // yyyy-dd-mm HH:MM:SS
                let date_tag: String = input.read_string(19)
                    .map_err(|e| tiff_err_cause(
                        format!("TIFF failed to read date tag at offset: {}", next_date_offset), e))?;
                if earliest_creation_date.is_empty() {
                    earliest_creation_date = date_tag;
                } else {
                    if date_tag < earliest_creation_date {
                        earliest_creation_date = date_tag;
                    }
                }
            } else {
                // TIFF scavenging IFD at offset
                ifd_offsets.remove(0);
                input.seek(next_ifd_offset as u64)
                    .map_err(|e| tiff_err_cause(
                        format!("TIFF failed to fast-forward to next IFD offset: {}", next_ifd_offset), e))?;

                // 2-byte count of the number of directory entries (i.e., the number of fields)
                let fields = input.read_u16(&endianness)
                    .map_err(|e| tiff_err_cause(
                        format!("TIFF failed to read IFD field count at offset: {}", next_ifd_offset), e))?;
                let mut i: u16 = 0;
                while i < fields {

                    // Bytes 0-1 The Tag that identifies the field
                    let field_tag = input.read_u16(&endianness)
                        .map_err(|e| tiff_err_cause(
                            format!("TIFF failed to read field tag for field: {}", i), e))?;

                    // Bytes 2-3 The field Type
                    let field_type = input.read_u16(&endianness)
                        .map_err(|e| tiff_err_cause(
                            format!("TIFF failed to read field type for field: {}", i), e))?;

                    // Bytes 4-7 The number of values, Count of the indicated Type
                    let field_count = input.read_u32(&endianness)
                        .map_err(|e| tiff_err_cause(
                            format!("TIFF failed to read field count for field: {}", i), e))?;

                    // Bytes 8-11 The Value Offset, the file offset (in bytes) of the Value for the field
                    let field_value_offset = input.read_u32(&endianness)
                        .map_err(|e| tiff_err_cause(
                            format!("TIFF failed to read field value offset for field: {}", i), e))?;

                    // 0x0132: DateTime
                    // 0x9003: DateTimeOriginal
                    // 0x9004: DateTimeDigitized
                    if field_tag == 0x0132 || field_tag == 0x9003 || field_tag == 0x9004 {
                        if field_type != 2 {
                            return Err(tiff_err(format!("expected tag has unexpected type: {} == {}", field_tag, field_type)));
                        }
                        if field_count != 20 {
                            return Err(tiff_err(format!("expected tag has unexpected count: {} == {}", field_tag, field_count)));
                        }
                        date_tag_offsets.push(field_value_offset);
                    }
                    // 0x8769: ExifIFDPointer
                    if field_tag == 0x8769 {
                        if field_type != 4 {
                            return Err(tiff_err(format!("EXIF pointer tag has unexpected type: {} == {}", field_tag, field_type)));
                        }
                        if field_count != 1 {
                            return Err(tiff_err(format!("EXIF pointer tag has unexpected size: {} == {}", field_tag, field_count)));
                        }
                        ifd_offsets.push(field_value_offset);
                    }
                    i = i + 1;
                }

                // followed by a 4-byte offset of the next IFD (or 0 if none).
                // (Do not forget to write the 4 bytes of 0 after the last IFD.)
                let next_ifd_offset = input.read_u32(&endianness)
                    .map_err(|e| tiff_err_cause(
                        format!("TIFF failed to read next IFD offset"), e))?;
                if next_ifd_offset != 0 {
                    ifd_offsets.push(next_ifd_offset);
                }
            }
        }
    }

    if earliest_creation_date.is_empty() {
        return Err(tiff_err("TIFF no date tags were found".to_string()));
    }
    return match_and_format_exif_date(&mut earliest_creation_date);
}

fn match_and_format_exif_date(exif_date: &mut String) -> Result<String, TiffError> {
    let chars: Vec<char> = exif_date.chars().collect();
    if chars[0].is_digit(10)
        && chars[1].is_digit(10)
        && chars[2].is_digit(10)
        && chars[3].is_digit(10)
        // due to Samsung bug, have to check for both : and -
        && (chars[4] == ':' || chars[4] == '-')
        && chars[5].is_digit(10)
        && chars[6].is_digit(10)
        // due to Samsung bug, have to check for both : and -
        && (chars[7] == ':' || chars[7] == '-')
        && chars[8].is_digit(10)
        && chars[9].is_digit(10)
        && chars[10] == ' '
        && chars[11].is_digit(10)
        && chars[12].is_digit(10)
        && chars[13] == ':'
        && chars[14].is_digit(10)
        && chars[15].is_digit(10)
        && chars[16] == ':'
        && chars[17].is_digit(10)
        && chars[18].is_digit(10) {
        exif_date.remove(16);
        exif_date.remove(13);
        exif_date.remove(10);
        exif_date.remove(7);
        exif_date.remove(4);
        exif_date.insert(8, '-');
        return Ok(exif_date.to_string());
    }
    return Err(tiff_err(format!("invalid exif date format: {}", exif_date)));
}
