use super::Input;
use super::Endianness;
use std::io;

// following documents were used to implement this parser:
// http://l.web.umkc.edu/lizhu/teaching/2016sp.video-communication/ref/mp4.pdf
// https://mpeg.chiariglione.org/standards/mpeg-4/iso-base-media-file-format

fn quicktime_scan_for_box<'a>(name: &str,
                              uuid: Option<(u64, u64)>,
                              input: &'a mut Input) -> io::Result<Input<'a>> {
    // TODO this infinite loop will throw EOF if the box will not be found
    loop {
        let mut box_length: u64 = input.read_u32(&Endianness::Big)? as u64;
        let box_type: String = input.read_string(4)?;
        // checking for large box:
        if box_length == 1 {
            box_length = input.read_u64(&Endianness::Big)?;
            // box length includes header, have to make adjustments:
            // 4 bytes for box length
            // 4 bytes for box type
            // 8 bytes for box large length
            box_length = box_length - 16;
        } else {
            // box length includes header, have to make adjustments:
            // 4 bytes for box length
            // 4 bytes for box type
            box_length = box_length - 8;
        }
        if &box_type == name {
            match uuid {
                None => return Ok(input.section(box_length)),
                Some(u) => {
                    let msb = input.read_u64(&Endianness::Big)?;
                    let lsb = input.read_u64(&Endianness::Big)?;
                    if u.0 == msb && u.1 == lsb {
                        box_length = box_length - 16;
                        return Ok(input.section(box_length));
                    }
                }
            }
        }
        input.ff(box_length)?;
    }
}

pub fn quicktime_search_box<'a>(box_name: &str, input: &'a mut Input) -> io::Result<Input<'a>> {
    return quicktime_scan_for_box(box_name, None, input);
}

pub fn quicktime_search_uuid_box<'a>(box_uuid: &str, input: &'a mut Input) -> io::Result<Input<'a>> {
    let msb = u64::from_str_radix(&box_uuid[0..16], 16)?;
    let lsb = u64::from_str_radix(&box_uuid[16..32], 16)?;
    return quicktime_scan_for_box(box_uuid, (msb, lsb), input);
}
