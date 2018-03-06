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
            println!("Failed to parse NAL: {:?}",
                     curr.parser_error.as_ref().unwrap());
        },
        Some(ref nal) => {
            println!("Parsed NAL of type {}.", nal.nal_unit_type);
            match curr.payload {
                None => println!("Failed to parse payload: {:?}",
                                 curr.parser_error.as_ref().unwrap()),
                Some(ref payload) => println!("Parsed {}", "x"),
            }
        },
    }
}

fn print_payload_bytes(curr: &Current) {
    if curr.rbsp.is_none() {
        println!("No rbsp");
        return;
    }

    let mut i = 0;
    let num = 10;
    for x in curr.rbsp.as_ref().unwrap() {
        if i % num == 0 {
            if i > 0 {
                println!("");
            }
            print!("{:08} ", i);
        }
        print!("{:02x} ", x);
        i += 1;
    }
    println!("");
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
        "bytes" => {
            print_payload_bytes(current);
        },
        _ => {
            println!("Unknown command: {}", command);
        },
    };

    return true;
}
