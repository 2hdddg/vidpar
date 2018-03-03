extern crate parser;

use std::fs::File;
use std::io::prelude::*;
use std::env;

use parser::bitreader::BitReader;
use parser::nalunit::NalUnit;


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

fn main() {
    /* Retrieve path to h264 file */
    let path = env::args().nth(1).unwrap_or("sw.h264".to_string());
    let file = File::open(&path);
    if file.is_err() {
        println!("Unable to open h264 file: {}", path);
        return;
    }

    let mut bitreader = BitReader::new(file.unwrap());
    if !reposition(&mut bitreader) {
        println!("Unable to find valid nal in h264 file: {}", path);
        return;
    }

    let mut count = 0;
    loop {
        count += 1;

        match NalUnit::parse(&mut bitreader) {
            Ok((mut nal, rbsp)) => {
                /* Got a NAL, handle whatever it contains */
                match nal.parse_payload(rbsp) {
                    Ok(payload) => println!("Parsed payload: {:?}", payload),
                    Err(s) => println!("Failed to parse payload: {:?}", s),
                }
            },
            Err(s) => {
                println!("Parser error: {:?}", s);
                /* Find next nal */
                if !reposition(&mut bitreader) {
                    break;
                }
            },
        }

        if count > 50 {
            println!("Breaking...");
            break;
        }
    }
}
