use std::io::prelude::*;

use parser::bitreader::BitReader;
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

pub fn invoke<R: Read>(command: String,
                       current: &mut Current,
                       bitreader: &mut BitReader<R>) -> bool {
    match command.as_ref() {
        "" => {},
        "q" | "quit" => return false,
        "?" | "help" => print_help(),
        "n" | "next" => {
            match current.next(bitreader) {
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
    };

    return true;
}
