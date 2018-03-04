use std::fs::File;
use std::io;
use std::io::Write;
use std::io::stdout;
use std::env;

extern crate parser;
use parser::bitreader::BitReader;

mod current;
use current::Current;


fn print_help() {
    println!("n | next - Decodes next unit.");
    println!("q | quit - Quits program.");
    println!("? | help - Shows this text.");
}

fn print_curr_slim(curr: &Current) {
    match curr.nal {
        None => {
            println!("Failed to parse NAL.");
        },
        Some(ref nal) => {
            println!("Parsed NAL of type {}.", nal.nal_unit_type);
            match curr.payload {
                None => println!("Failed to parse payload."),
                Some(ref payload) => println!("Parsed {}", "x"),
            }
        },
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
    let mut command = String::new();
    let mut current = Current::new();

    loop {
        /* Read next command */
        print!(">");
        stdout().flush().unwrap();
        if io::stdin().read_line(&mut command).is_err() {
            println!("Error reading from stdin");
            return;
        }
        /* Remove newline */
        command.pop();

        match command.as_ref() {
            "" => {},
            "q" | "quit" => return,
            "?" | "help" => print_help(),
            "n" | "next" => {
                match current.next(&mut bitreader) {
                    false => println!("Reached end of data"),
                    true => print_curr_slim(&current)
                }
            },
            "nal" => {
                match current.nal {
                    None => println!("No valid NAL."),
                    Some(ref nal) => println!("{:#?}", nal),
                }
            },
            "payload" => {
                match current.payload {
                    None => println!("No valid payload."),
                    Some(ref payload) => println!("{:#?}", payload),
                }
            },
            _ => {
                println!("Unknown command: {}", command);
            },
        }
        command.clear();
    }
}
