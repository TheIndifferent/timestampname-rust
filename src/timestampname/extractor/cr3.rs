use super::input::Input;
use super::FileMetadata;
use super::Failure;
use super::tiff::tiff_extract_metadata_creation_timestamp;

// following resources were used to implement this parser:
// https://github.com/lclevy/canon_cr3

fn canon_box_uuid() -> Result<(u64, u64), Failure> {
    let canon_uuid = "85c0b687820f11e08111f4ce462b6a48";
    let msb = u64::from_str_radix(&canon_uuid[0..16], 16)
        .map_err(|e| Failure::env_failure_caused(
            "parsing Canon Box UUID MSB to u64".to_string(),
            e))?;
    let lsb = u64::from_str_radix(&canon_uuid[16..32], 16)
        .map_err(|e| Failure::env_failure_caused(
            "parsing Canon Box UUID LSB to u64".to_string(),
            e))?;
    return Ok((msb, lsb));
}

fn extract_timestamp_from_tiff_box(box_name: &str, input: &mut Input) -> Result<Option<FileMetadata>, Failure> {
    let file_name = input.name().to_string();
    let mut target_box = input.quicktime_search_box(box_name)
        .map_err(|e| Failure::file_failure_caused(
            file_name.to_string(),
            format!("failed to find box: {}", box_name),
            e))?;
    return tiff_extract_metadata_creation_timestamp(&mut target_box);
}

pub fn cr3_extract_metadata_creation_timestamp(input: &mut Input) -> Result<Option<FileMetadata>, Failure> {
    let file_name = input.name().to_string();
    let mut moov_box = input.quicktime_search_box("moov")
        .map_err(|e| Failure::file_failure_caused(
            file_name.to_string(),
            "moov box not found".to_string(),
            e))?;
    let mut canon_box = moov_box.quicktime_search_uuid_box(canon_box_uuid()?)
        .map_err(|e| Failure::file_failure_caused(
            file_name.to_string(),
            "canon box not found".to_string(),
            e))?;

    let cmt1_timestamp = extract_timestamp_from_tiff_box("CMT1", &mut canon_box)?;

    canon_box.seek(0)
        .map_err(|e| Failure::file_failure_caused(
            file_name.to_string(),
            "failed to rewind till canon box start".to_string(),
            e))?;
    let cmt2_timestamp = extract_timestamp_from_tiff_box("CMT2", &mut canon_box)?;

    match cmt1_timestamp {
        None => {
            match cmt2_timestamp {
                None => return Err(Failure::file_failure(
                    file_name.to_string(),
                    "timestamps not found in CMT1 and CMT2 boxes".to_string())),
                Some(md2) => return Ok(Some(md2))
            }
        }
        Some(md1) => {
            match cmt2_timestamp {
                None => return Ok(Some(md1)),
                Some(md2) => {
                    if md1.creation_timestamp < md2.creation_timestamp {
                        return Ok(Some(md1));
                    } else {
                        return Ok(Some(md2));
                    }
                }
            }
        }
    }
}
