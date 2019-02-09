use super::Input;
use super::Endianness;

// following documents were used to implement this parser:
// http://l.web.umkc.edu/lizhu/teaching/2016sp.video-communication/ref/mp4.pdf
// https://mpeg.chiariglione.org/standards/mpeg-4/iso-base-media-file-format

fn quicktime_scan_for_box(name: String,
                          uuid: Option<String>,
                          input: &mut Input) -> io::Result<ByteRead> {
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
        if box_type == name {
            match uuid {
                None => return Ok(input.)
            }
        }
    }
}

pub fn quicktime_search_box(box_name: &str, input: &mut impl Input) {}

pub fn quicktime_search_uuid_box(box_uuid: &str, input: &mut impl Input) {}
