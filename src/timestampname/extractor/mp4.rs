use chrono::{Local, TimeZone, Utc};

use super::Endianness;
use super::Failure;
use super::FileMetadata;
use super::Input;

// pre-calculated duration between 1904-01-01 and 1970-01-01:
const MP4_EPOCH_OFFSET: u64 = 2082844800;

fn format_mp4_timestamp(timestamp: u64, input: &mut Input, utc: bool) -> Result<Option<FileMetadata>, Failure> {
    let ts_in_unix_epoch = timestamp - MP4_EPOCH_OFFSET;
    if ts_in_unix_epoch > i64::max_value() as u64 {
        return Err(Failure::file_failure(
            input.name().to_string(),
            format!("mp4 timestamp overflows i64: {}", ts_in_unix_epoch)));
    }
    let ts_casted = ts_in_unix_epoch as i64;
    let formatted = match utc {
        true => Utc.timestamp(ts_casted, 0).format("%Y%m%d-%H%M%S").to_string(),
        false => Local.timestamp(ts_casted, 0).format("%Y%m%d-%H%M%S").to_string(),
    };
    return Ok(Some(FileMetadata {
        file_name: input.name().to_string(),
        creation_timestamp: formatted,
        extension: input.ext().to_string(),
    }));
}

pub fn mp4_extract_metadata_creation_timestamp(input: &mut Input, utc: bool) -> Result<Option<FileMetadata>, Failure> {
    let file_name = input.name().to_string();
    let mut moov_box = input.quicktime_search_box("moov")
        .map_err(|e| Failure::file_failure_caused(
            file_name.to_string(),
            "moov box not found".to_string(),
            e))?;
    let mut mvhd_box = moov_box.quicktime_search_box("mvhd")
        .map_err(|e| Failure::file_failure_caused(
            file_name.to_string(),
            "mvhd box not found".to_string(),
            e))?;
    let mvhd_version_and_flags = mvhd_box.read_u32(&Endianness::Big)
        .map_err(|e| Failure::file_failure_caused(
            file_name.to_string(),
            "failed to read mvhd version".to_string(),
            e))?;
    let mvhd_version = mvhd_version_and_flags >> 24;
    match mvhd_version {
        0 => {
            let creation_time = input.read_u32(&Endianness::Big)
                .map_err(|e| Failure::file_failure_caused(
                    file_name.to_string(),
                    "failed to read creation time".to_string(),
                    e))?;
            let modification_time = input.read_u32(&Endianness::Big)
                .map_err(|e| Failure::file_failure_caused(
                    file_name.to_string(),
                    "failed to read modification time".to_string(),
                    e))?;
            return format_mp4_timestamp(creation_time as u64, input, utc);
        }
        1 => {
            let creation_time = input.read_u64(&Endianness::Big)
                .map_err(|e| Failure::file_failure_caused(
                    file_name.to_string(),
                    "failed to read creation time".to_string(),
                    e))?;
            let modification_time = input.read_u64(&Endianness::Big)
                .map_err(|e| Failure::file_failure_caused(
                    file_name.to_string(),
                    "failed to read modification time".to_string(),
                    e))?;
            return format_mp4_timestamp(creation_time, input, utc);
        }
        _ => return Err(Failure::file_failure(
            file_name.to_string(),
            format!("unsupported mvhd version: {}", mvhd_version)))
    }
}
