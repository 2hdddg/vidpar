use std::io::prelude::*;
use std::io::SeekFrom;
use std::error::Error;
use std;

use super::*;

pub struct BitReader<R> {
    bits: u8,
    valid_bits: u8,
    num_zeroes: u8,
    reader: R,
    pub pos: usize,
    end_of_data: bool,
}

fn err(text: &str) -> ParserError {
    let description = String::from(text);
    let error = BitReaderError { description };

    ParserError::BitReaderError(error)
}

/* Ensures that bits exists */
fn ensure<R: Read>(reader: &mut BitReader<R>) -> Result<()> {
    if reader.valid_bits > 0 {
        return Ok(())
    }

    let mut buf: [u8; 1] = [0];
    match reader.reader.read(&mut buf) {
        Ok(1) => {
            reader.bits = buf[0];
            reader.valid_bits = 8;
            reader.pos += 1;

            if reader.num_zeroes == 2 &&
                reader.bits == 0x03 {

                reader.num_zeroes = 0;
                reader.bits = 0;
                return ensure(reader)
            }
            else if reader.bits == 0x00 {
                reader.num_zeroes += 1;
            }
            else {
                reader.num_zeroes = 0;
            }

            return Ok(());
        },
        /* Zero bytes read, no more data */
        Ok(_) => {
            reader.end_of_data = true;
            return Err(ParserError::BitReaderEndOfStream());
        },
        Err(e) => Err(err(e.description())),
    }
}

/* Reads n number of bits into unsigned */
fn read<R: Read>(reader: &mut BitReader<R>, n: u8) -> Result<u64> {
    let mut requested_bits = n;
    let mut u: u64 = 0;

    if n > 64 {
        return Err(err("too many bits, > 64"));
    }

    while requested_bits > 0 {
        ensure(reader)?;
        if requested_bits >= reader.valid_bits {
            let muted_bits = 8 - reader.valid_bits;
            u <<= reader.valid_bits;
            u |= (reader.bits >> muted_bits) as u64;
            requested_bits -= reader.valid_bits;
            reader.valid_bits = 0;
        }
        else {
            let muted_bits = 8 - requested_bits;
            u <<= requested_bits;
            u |= (reader.bits >> muted_bits) as u64;
            reader.bits <<= requested_bits;
            reader.valid_bits -= requested_bits;
            requested_bits = 0;
        }
    }

    Ok(u)
}

impl<R: Read> BitReader<R> {
    pub fn new(r: R) -> BitReader<R> {
        BitReader {
            bits: 0,
            valid_bits: 0,
            num_zeroes: 0,
            reader: r,
            pos: 0,
            end_of_data: false
        }
    }

    /* Reads n number of bits into unsigned */
    pub fn u64(&mut self, n: u8) -> Result<u64> {
        read(self, n)
    }

    pub fn u32(&mut self, n: u8) -> Result<u32> {
        if n > 32 {
            return Err(err("too many bits, > 32"));
        }

        let u = self.u64(n)?;
        if u > std::u32::MAX as u64 {
            return Err(err("u32 overflow"));
        }

        Ok(u as u32)
    }

    pub fn u8(&mut self, n: u8) -> Result<u8> {
        if n > 8 {
            return Err(err("too many bits, > 8"));
        }

        let u = self.u64(n)?;
        if u > std::u8::MAX as u64 {
            return Err(err("u8 overflow"));
        }

        Ok(u as u8)
    }

    pub fn b(&mut self) -> Result<u8> {
        Ok(self.u8(8)?)
    }

    pub fn ue64(&mut self) -> Result<u64> {
        let mut leading_zeroes: i32 = -1;
        let mut bit = 0;

        while bit == 0 {
            bit = self.u64(1)?;
            leading_zeroes += 1;
        }

        let bits = self.u64(leading_zeroes as u8)?;

        Ok(2u64.pow(leading_zeroes as u32) - 1 + bits)
    }

    pub fn ue32(&mut self) -> Result<u32> {
        let ue = self.ue64()?;

        if ue > std::u32::MAX as u64 {
            return Err(err("u32 overflow"));
        }

        Ok(ue as u32)
    }

    pub fn ue8(&mut self) -> Result<u8> {
        let ue = self.ue64()?;

        if ue > std::u8::MAX as u64 {
            return Err(err("u8 overflow"));
        }

        Ok(ue as u8)
    }

    pub fn se64(&mut self) -> Result<i64> {
        let code_num = self.ue64()?;
        let half = (code_num as f64 / 2.0).ceil() as i64;
        match (code_num & 1) == 1 {
            /* Odd */
            true => Ok(half),
            /* Even */
            false => Ok(-half),
        }
    }

    pub fn se8(&mut self) -> Result<i8> {
        let se = self.se64()?;

        if se > std::i8::MAX as i64 {
            return Err(err("u8 overflow"));
        }
        if se < std::i8::MIN as i64 {
            return Err(err("u8 underflow"));
        }

        Ok(se as i8)
    }

    pub fn flag(&mut self) -> Result<bool> {
        Ok(self.u64(1)? == 1)
    }

    pub fn is_byte_aligned(&self) -> bool {
        self.valid_bits == 0 || self.valid_bits == 8
    }

    pub fn byte_align(&mut self) {
        self.valid_bits = 0;
    }

    pub fn reached_end_of_data(&self) -> bool {
        self.end_of_data
    }

    /* Reads rbsp trailing bits */
    pub fn rbsp_trailing_bits(&mut self) -> Result<()> {
        /* Is this check correct? */
        /*
        if self.is_byte_aligned() {
            return Ok(());
        }
        */

        let rbsp_stop_one_bit = self.u8(1)?;

        if rbsp_stop_one_bit != 1 {
            return Err(err("rbsp_stop_one_bit is not 1"));
        }

        while !self.is_byte_aligned() {
            let rbsp_alignment_zero_bit = self.u8(1)?;
            if rbsp_alignment_zero_bit != 0 {
                return Err(err("rbsp_alignment_zero_bit is not 0"));
            }
        }

        Ok(())
    }
}


impl<R: Read+Seek> BitReader<R> {
    pub fn more_rbsp_data(&mut self) -> Result<bool> {
        /* Keep track of initial position in stream */
        let initial_pos = self.reader.seek(SeekFrom::Current(0)).unwrap();
        /* Keep state of self */
        let bits = self.bits;
        let valid_bits = self.valid_bits;
        let num_zeroes = self.num_zeroes;
        let end_of_data = self.end_of_data;
        let pos = self.pos;

        /* If next bit is 1 and the rest of the bits are zero than
         * we have stumbled upon the rbsp_stop_bit and therefore
         * we have no more rbsp data. */

        let mut next = read(self, 1);
        if next.is_err() {
            /* No more data, no need to restore state */
            return Ok(false);
        }

        let mut more_data = false;

        if next.unwrap() == 0 {
            more_data = true;
        }
        else {
            /* Next bit is 1, this might have been the rbsp_stop_bit. */
            loop {
                next = read(self, 1);
                if next.is_err() {
                    break;
                }
                /* A later 1 bit found, we didn't find the rbsp_stop_bit. */
                if next.unwrap() == 1 {
                    more_data = true;
                    break;
                }
            }
        }

        /* Restore */
        self.bits = bits;
        self.valid_bits = valid_bits;
        self.num_zeroes = num_zeroes;
        self.end_of_data = end_of_data;
        self.pos = pos;
        if self.reader.seek(SeekFrom::Start(initial_pos)).is_err() {
            return Err(err("Failed to restore reader"));
        }

        Ok(more_data)
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn u_8_8_8_8() {
        let buf: [u8; 4] = [0, 1, 2, 3,];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let n1 = reader.u64(8).unwrap();
        let n2 = reader.u64(8).unwrap();
        let n3 = reader.u64(8).unwrap();
        let n4 = reader.u64(8).unwrap();

        assert_eq!(n1, 0);
        assert_eq!(n2, 1);
        assert_eq!(n3, 2);
        assert_eq!(n4, 3);
    }

    #[test]
    fn u_3_5_7_1() {
        let buf: [u8; 2] = [0b10010000, 0b10000001];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let n1 = reader.u64(3).unwrap();
        let n2 = reader.u64(5).unwrap();
        let n3 = reader.u64(7).unwrap();
        let n4 = reader.u64(1).unwrap();

        assert_eq!(n1, 0b100);
        assert_eq!(n2, 0b10000);
        assert_eq!(n3, 0b1000000);
        assert_eq!(n4, 0b1);
    }

    #[test]
    fn u_15_1() {
        let buf: [u8; 2] = [0b10000000, 0b00000011];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let n1 = reader.u64(15).unwrap();
        let n2 = reader.u64(1).unwrap();

        assert_eq!(n1, 0b100000000000001);
        assert_eq!(n2, 0b1);
    }

    #[test]
    fn ue_0() {
        let buf: [u8; 1] = [0b10000000];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let ue = reader.ue64().unwrap();

        assert_eq!(ue, 0);
    }

    #[test]
    fn ue_1() {
        let buf: [u8; 1] = [0b01000000];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let ue = reader.ue64().unwrap();

        assert_eq!(ue, 1);
    }

    #[test]
    fn ue_8() {
        let buf: [u8; 1] = [0b00010010];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let ue = reader.ue64().unwrap();

        assert_eq!(ue, 8);
    }

    #[test]
    fn se_0() {
        let buf: [u8; 1] = [0b10000000];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let se = reader.se64().unwrap();

        assert_eq!(se, 0);
    }

    #[test]
    fn se_1() {
        let buf: [u8; 1] = [0b01000000];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let se = reader.se64().unwrap();

        assert_eq!(se, 1);
    }

    #[test]
    fn se_neg_1() {
        let buf: [u8; 1] = [0b01100000];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let se = reader.se64().unwrap();

        assert_eq!(se, -1);
    }

    #[test]
    fn se_4() {
        let buf: [u8; 1] = [0b00010000];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let se = reader.se64().unwrap();

        assert_eq!(se, 4);
    }

    #[test]
    fn se_neg_4() {
        let buf: [u8; 1] = [0b00010010];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let se = reader.se64().unwrap();

        assert_eq!(se, -4);
    }

    #[test]
    fn is_byte_aligned_initially() {
        let buf: [u8; 2] = [0, 0];
        let cursor = Cursor::new(buf);
        let reader = BitReader::new(cursor);

        assert!(reader.is_byte_aligned());
    }

    #[test]
    fn is_byte_aligned_1() {
        let buf: [u8; 2] = [0, 0];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let _ = reader.u64(1).unwrap();

        assert!(!reader.is_byte_aligned());
    }

    #[test]
    fn is_byte_aligned_8() {
        let buf: [u8; 2] = [0, 0];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let _ = reader.b().unwrap();

        assert!(reader.is_byte_aligned());
    }

    #[test]
    fn byte_align_initially() {
        let buf: [u8; 2] = [1, 2];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        reader.byte_align();
        /* Read to make sure we don't lose first byte */
        let n = reader.b().unwrap();

        assert!(reader.is_byte_aligned());
        assert_eq!(n, 1);
    }

    #[test]
    fn byte_align_after_1() {
        let buf: [u8; 2] = [1, 2];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let _n = reader.u64(1).unwrap();
        reader.byte_align();
        let was_byte_aligned = reader.is_byte_aligned();
        /* Read to make sure we get second byte */
        let n = reader.b().unwrap();

        assert!(was_byte_aligned);
        assert_eq!(n, 2);
    }

    #[test]
    fn emulation_prevention_0x000000() {
        let buf: [u8; 4] = [0x00, 0x00, 0x03, 0x00];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let n1 = reader.b().unwrap();
        let n2 = reader.b().unwrap();
        let n3 = reader.b().unwrap();

        assert_eq!(n1, 0x00);
        assert_eq!(n2, 0x00);
        assert_eq!(n3, 0x00);
    }

    #[test]
    fn pos_is_increased_at_read() {
        let buf: [u8; 4] = [0x00, 0x00, 0x03, 0x00];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        assert_eq!(reader.pos, 0);
        reader.flag().unwrap();
        assert_eq!(reader.pos, 1);
        reader.flag().unwrap();
        assert_eq!(reader.pos, 1);
    }

    #[test]
    fn rbsp_trailing_bits() {
        /* Check that we read pased the trailing bits,
         * S - stop bit, T - trailing, N - next bits
         *                    STTTTTT     NNNNNNN */
        let buf: [u8; 2] = [0b10000000, 0b11111111];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        /* Read past stop bit and trailing bits */
        reader.rbsp_trailing_bits().unwrap();

        /* Should be all 1s now */
        let ones = reader.b().unwrap();

        assert_eq!(ones, 0xff);
    }

    #[test]
    fn rbsp_trailing_bits_no_stop_bit() {
        let buf: [u8; 1] = [0b01000000];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let res = reader.rbsp_trailing_bits();

        assert!(res.is_err());
    }

    #[test]
    fn rbsp_trailing_bits_a_non_zero() {
        let buf: [u8; 1] = [0b10001000];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let res = reader.rbsp_trailing_bits();

        assert!(res.is_err());
    }

    #[test]
    fn more_rbsp_data_at_end() {
        let buf: [u8; 1] = [1];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        /* Read the one and only byte */
        reader.b().unwrap();

        let more_rbsp_data = reader.more_rbsp_data().unwrap();

        assert!(!more_rbsp_data);
    }

    #[test]
    fn more_rbsp_data_at_a_zero() {
        let buf: [u8; 1] = [0];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        /* Read all but one bit, last one should be zero */
        reader.u64(7).unwrap();

        let more_rbsp_data = reader.more_rbsp_data().unwrap();

        /* The last bit should still be here ! */
        let last = reader.u64(1).unwrap();

        /* Since this bit is zero there could be more rbsp data */
        assert!(more_rbsp_data);
        assert!(last == 0);
    }

    #[test]
    fn more_rbsp_data_with_rbsp_stop_bit() {
        let buf: [u8; 1] = [0b10000000];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let more_rbsp_data = reader.more_rbsp_data().unwrap();

        /* The first bit should still be here ! */
        let first = reader.u64(1).unwrap();

        /* We were at the rbsp stop bit */
        assert!(!more_rbsp_data);
        assert!(first == 1);
    }

    #[test]
    fn more_rbsp_data_with_rbsp_stop_bit_but_more_data() {
        let buf: [u8; 1] = [0b10000001];
        let cursor = Cursor::new(buf);
        let mut reader = BitReader::new(cursor);

        let more_rbsp_data = reader.more_rbsp_data().unwrap();

        /* The first bit should still be here ! */
        let first = reader.u64(1).unwrap();

        assert!(more_rbsp_data);
        assert!(first == 1);
    }
}
