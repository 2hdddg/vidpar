use std::fs::File;
use std::io;
use std::io::Write;
use std::io::stdout;
use std::env;

extern crate parser;
use parser::bitreader::BitReader;

mod current;
mod command;

use command::invoke;
use current::Current;

fn main() {
    /* Retrieve path to h264 file */
    let path = env::args().nth(1).unwrap_or("sw.h264".to_string());
    let file = File::open(&path);
    if file.is_err() {
        println!("Unable to open h264 file: {}", path);
        return;
    }

    let mut bitreader = BitReader::new(file.unwrap());
    let mut current = Current::new();

    loop {
        let mut input = String::new();
        /* Read next command */
        print!(">");
        stdout().flush().unwrap();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Error reading from stdin");
            return;
        }
        /* Remove newline */
        input.pop();
        if !invoke(input, &mut current, &mut bitreader) {
            break;
        }
    }
}
