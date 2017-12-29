use std::io::prelude::*;
use std::vec::Vec;

use bitreader::BitReader;

#[derive(Debug)]
pub struct NalUnit {
    nal_ref_idc: u8,
    nal_unit_type: u8,

    svc_extension_flag: bool,
    avc_3d_extension_flag: bool,

    rbsp: Vec<u8>,
}

impl NalUnit {
    /// Starts parsing of NAL unit at the current position of the
    /// bitreader. Caller should make sure that position is after
    /// the start code (0x00000001/0x000001) and on an even byte
    /// boundary.
    pub fn parse<R: Read>(r: &mut BitReader<R>) ->
                          Result<NalUnit, &'static str> {

        if !r.is_byte_aligned() {
            return Err("Should be byte aligned at start of nal");
        }

        let forbidden_zero_bit = r.u(1)?;
        if forbidden_zero_bit != 0 {
            return Err("forbidden_zero_bit is not 0");
        }

        let nal_ref_idc = r.u(2)? as u8;
        let nal_unit_type = r.u(5)? as u8;

        let mut svc_extension_flag = false;
        let mut avc_3d_extension_flag = false;

        if nal_unit_type == 14 || nal_unit_type == 20 ||
           nal_unit_type == 21 {

            if nal_unit_type != 21 {
                svc_extension_flag = r.flag()?;
            }
            else {
                avc_3d_extension_flag = r.flag()?;
            }

            if svc_extension_flag {
                /* 3 bytes, svc extension */
                return Err("nal unit svc extension not implemented");
            }
            else if avc_3d_extension_flag {
                return Err("nal unit avc 3d extension not implemented");
            }
            else {
                return Err("nal unit mvc extension not implemented");
            }
        }

        if !r.is_byte_aligned() {
            return Err("Should be byte aligned at start of nal rbsp");
        }

        /* Read RBSP bytes until next nal or end of data */
        let mut rbsp = Vec::new();
        let mut num_zeroes = 0;
        loop {
            match r.b() {
                Ok(b) => {
                    match b {
                        0x00 => num_zeroes += 1,
                        0x01 => {
                            if num_zeroes == 2 ||
                               num_zeroes == 3 {
                                    let len = rbsp.len();
                                    rbsp.truncate(len - num_zeroes);
                                    break;
                            }
                            num_zeroes = 0;
                        }
                        _ => num_zeroes = 0,
                    }
                    rbsp.push(b);
                },
                Err(s) => {
                    if r.reached_end_of_data() {
                        break;
                    }
                    return Err(s);
                }
            }
        }

        Ok(NalUnit {
            nal_ref_idc,
            nal_unit_type,
            svc_extension_flag,
            avc_3d_extension_flag,
            rbsp,
        })
    }

    /// Positions bitreader right after startcode.
    /// Call upon start of parsing and whenever parsing fails to
    /// reposition on start of new nal.
    pub fn next<R: Read>(r: &mut BitReader<R>) ->
                         Result<bool, &'static str> {

        let mut num_zeroes = 0;

        r.byte_align();
        loop {
            match r.b() {
                Ok(b) => {
                    match b {
                        0x00 => num_zeroes += 1,
                        0x01 => {
                            if num_zeroes == 2 ||
                               num_zeroes == 3 {
                                return Ok(true);
                            }
                            num_zeroes = 0;
                        }
                        _ => num_zeroes = 0,
                    }
                },
                Err(s) => {
                    if r.reached_end_of_data() {
                        return Ok(false);
                    }
                    return Err(s);
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn parse() {
        let buf = vec![
          /*<NAL      > <RBSP          > */
            0b01010000, 0x42, 0xff, 0x01,
        ];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let nal = NalUnit::parse(&mut reader).unwrap();

        assert_eq!(nal.nal_ref_idc, 0b10);
        assert_eq!(nal.nal_unit_type, 0b10000);
        assert_eq!(nal.rbsp, [0x42, 0xff, 0x01]);
    }

    #[test]
    fn parse_forbidden_zero_bit_is_1() {
        let buf = vec![
          /*<NAL      > <RBSP          > */
          /* First bit should be zero when valid */
            0b11010000, 0x42, 0xff, 0x01,
        ];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let nal = NalUnit::parse(&mut reader);

        assert!(nal.is_err());
    }

    #[test]
    fn parse_not_byte_aligned() {
        let buf = vec![
          /*<NAL> <RBSP          > */
            0x67, 0x42, 0xff, 0x01,
        ];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);
        /* Read one bit to make reader not aligned on byte */
        reader.u(1).unwrap();

        let nal = NalUnit::parse(&mut reader);

        assert!(nal.is_err());
    }

    /* Verifies that position after parse is correct and that
     * parse can handle end of data correctly. */
    #[test]
    fn parse_sequence() {
        let buf = vec![
          /*<START CODE           > <NAL> <RBSP                 > */
            0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0xff, 0x01,
            0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x01, 0x02, 0x03,
            0x00, 0x00, 0x00, 0x01, 0x67, 0x42,
        ];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        /* Position at start of nal, bypass start code ! */
        NalUnit::next(&mut reader).unwrap();

        let nal1 = NalUnit::parse(&mut reader).unwrap();
        let nal2 = NalUnit::parse(&mut reader).unwrap();
        let nal3 = NalUnit::parse(&mut reader).unwrap();
        /* After third nal there should be read error */
        let end  = NalUnit::parse(&mut reader);

        assert_eq!(nal1.rbsp.len(), 3);
        assert_eq!(nal2.rbsp.len(), 4);
        assert_eq!(nal3.rbsp.len(), 1);
        assert!(end.is_err() && reader.reached_end_of_data());
    }
}