use std::io;

use super::Endianness;
use super::input::Input;

// following documents were used to implement this parser:
// http://l.web.umkc.edu/lizhu/teaching/2016sp.video-communication/ref/mp4.pdf
// https://mpeg.chiariglione.org/standards/mpeg-4/iso-base-media-file-format

impl<'f> Input<'f> {
    fn quicktime_scan_for_box(&mut self, name: &str,
                              uuid: Option<(u64, u64)>) -> io::Result<Input<'f>> {
        // TODO this infinite loop will throw EOF if the box will not be found
        loop {
            let mut box_length: u64 = self.read_u32(&Endianness::Big)? as u64;
            let box_type: String = self.read_string(4)?;
            // checking for large box:
            if box_length == 1 {
                box_length = self.read_u64(&Endianness::Big)?;
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
                    None => return Ok(self.section(box_length)),
                    Some(u) => {
                        let msb = self.read_u64(&Endianness::Big)?;
                        let lsb = self.read_u64(&Endianness::Big)?;
                        if u.0 == msb && u.1 == lsb {
                            box_length = box_length - 16;
                            return Ok(self.section(box_length));
                        }
                    }
                }
            }
            self.ff(box_length)?;
        }
    }

    fn quicktime_search_box(&mut self, box_name: &str) -> io::Result<Input> {
        return self.quicktime_scan_for_box(box_name, None);
    }

    fn quicktime_search_uuid_box(&mut self, box_uuid: (u64, u64)) -> io::Result<Input> {
        return self.quicktime_scan_for_box(&"uuid", Some(box_uuid));
    }
}
