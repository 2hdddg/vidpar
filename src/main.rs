extern crate parser;

use std::fs::File;
use std::io::prelude::*;

use parser::bitreader::BitReader;
use parser::nalunit::NalUnit;


fn reposition<R: Read>(r: &mut BitReader<R>) -> bool {
    match NalUnit::next(r) {
        /* Non recoverable */
        Err(s) => {
            println!("Error: {}", s);
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
    let f = File::open("sw.h264").unwrap();
    let mut bitreader = BitReader::new(f);
    let mut count = 0;

    if !reposition(&mut bitreader) {
        return;
    }

    loop {
        count += 1;

        match NalUnit::parse(&mut bitreader) {
            Ok(nal) => println!("On nal: {:?}", nal),
            Err(s) => {
                println!("Parser error: {}", s);
                /* Find next nal */
                if !reposition(&mut bitreader) {
                    break;
                }
            }
        }

        if count > 50 {
            println!("Breaking...");
            break;
        }
    }
}
