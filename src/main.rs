use std::fs::File;
use std::env;

extern crate parser;

mod current;
mod shell;

fn main() {
    /* Retrieve path to h264 file */
    let path = env::args().nth(1).unwrap_or("sw.h264".to_string());
    let file = File::open(&path);
    if file.is_err() {
        println!("Unable to open h264 file: {}", path);
        return;
    }

    shell::eval_loop(file.unwrap());
}
