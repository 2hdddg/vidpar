use std::io::prelude::*;

pub struct BitReader<R> {
    bits: u8,
    valid_bits: u8,
    num_zeroes: u8,
    reader: R,
    end_of_data: bool,
}

impl<R: Read> BitReader<R> {
    pub fn new(r: R) -> BitReader<R> {
        BitReader { bits: 0,
                    valid_bits: 0,
                    num_zeroes: 0,
                    reader: r,
                    end_of_data: false }
    }

    /* Ensures that bits exists */
    fn ensure(&mut self) -> Result<(), &'static str> {
        if self.valid_bits > 0 {
            return Ok(())
        }

        let mut buf: [u8; 1] = [0];
        match self.reader.read(&mut buf) {
            Ok(1) => {
                self.bits = buf[0];
                self.valid_bits = 8;

                if self.num_zeroes == 2 &&
                    self.bits == 0x03 {

                    self.num_zeroes = 0;
                    self.bits = 0;
                    return self.ensure()
                }
                else if self.bits == 0x00 {
                    self.num_zeroes += 1;
                }
                else {
                    self.num_zeroes = 0;
                }

                return Ok(());
            },
            Ok(_) => {
                self.end_of_data = true;
                return Err("Tried to read too many bits")
            },
            Err(_) => Err("IO error"),
        }
    }

    /* Reads n number of bits into unsigned */
    pub fn u(&mut self, n: u8) -> Result<u32, &'static str> {
        let mut requested_bits = n;
        let mut u: u32 = 0;

        while requested_bits > 0 {
            self.ensure()?;
            if requested_bits >= self.valid_bits {
                let muted_bits = 8 - self.valid_bits;
                u <<= self.valid_bits;
                u |= (self.bits >> muted_bits) as u32;
                requested_bits -= self.valid_bits;
                self.valid_bits = 0;
            }
            else {
                let muted_bits = 8 - requested_bits;
                u <<= requested_bits;
                u |= (self.bits >> muted_bits) as u32;
                self.bits <<= requested_bits;
                self.valid_bits -= requested_bits;
                requested_bits = 0;
            }
        }

        Ok(u)
    }

    pub fn b(&mut self) -> Result<u8, &'static str> {
        Ok(self.u(8)? as u8)
    }

    pub fn flag(&mut self) -> Result<bool, &'static str> {
        Ok(self.u(1)? == 1)
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

        let n1 = reader.u(8).unwrap();
        let n2 = reader.u(8).unwrap();
        let n3 = reader.u(8).unwrap();
        let n4 = reader.u(8).unwrap();

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

        let n1 = reader.u(3).unwrap();
        let n2 = reader.u(5).unwrap();
        let n3 = reader.u(7).unwrap();
        let n4 = reader.u(1).unwrap();

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

        let n1 = reader.u(15).unwrap();
        let n2 = reader.u(1).unwrap();

        assert_eq!(n1, 0b100000000000001);
        assert_eq!(n2, 0b1);
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

        let _ = reader.u(1).unwrap();

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

        let _n = reader.u(1).unwrap();
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
}
