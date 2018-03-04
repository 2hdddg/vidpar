use std::io::prelude::*;

use parser::bitreader::BitReader;
use parser::nalunit::NalUnit;
use parser::nalunit::NalPayload;

pub struct Current {
    pub nal: Option<NalUnit>,
    pub payload: Option<NalPayload>,
    pub rbsp: Option<Vec<u8>>,
}

fn reposition<R: Read>(r: &mut BitReader<R>) -> bool {
    match NalUnit::next(r) {
        /* Non recoverable */
        Err(s) => {
            println!("Error: {:?}", s);
            return false;
        },
        Ok(false) => {
            println!("No start of nal found!");
            return false;
        },
        Ok(true) => true,
    }
}

impl Current {
    pub fn new() -> Current {
        Current {
            nal: None,
            payload: None,
            rbsp: None,
        }
    }

    /* Return false when no more data */
    pub fn next<R: Read>(&mut self, bitreader: &mut BitReader<R>) -> bool {
        self.nal = None;
        self.payload = None;
        self.rbsp = None;

        if self.nal.is_none() {
            if !reposition(bitreader) {
                return false;
            }
        }

        let parsed_nal = NalUnit::parse(bitreader);
        if parsed_nal.is_err() {
            return true;
        }

        let (mut nal, rbsp) = parsed_nal.unwrap();
        let parsed_payload = nal.parse_payload(&rbsp);
        self.nal = Some(nal);
        self.rbsp = Some(rbsp);
        if parsed_payload.is_err() {
            return true;
        }

        self.payload = Some(parsed_payload.unwrap());

        return true;
    }
}
