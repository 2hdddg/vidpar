use std::io::prelude::*;
use std::fs::File;
use std::io;
use std::io::stdout;

use parser::bitreader::BitReader;
use current::Current;


fn print_help() {
    println!("n | next - Decodes next unit.");
    println!("q | quit - Quits program.");
    println!("? | help - Shows this text.");
    println!("nal - prints current nal.");
    println!("payload - prints current payload.");
    println!("bytes - prints payload raw bytes.");
}

fn print_curr_slim(curr: &Current) {
    match curr.nal {
        None => {
            println!("Failed to parse NAL: {:?}",
                     curr.parser_error.as_ref().unwrap());
        },
        Some(ref nal) => {
            print!("Parsed NAL of type {}. ", nal.nal_unit_type);
            match curr.payload {
                None => println!("Failed to parse payload: {:?}",
                                 curr.parser_error.as_ref().unwrap()),
                Some(ref _payload) => println!("Parsed {}",
                                     curr.payload.as_ref().unwrap()),
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

pub fn eval<R: Read>(command: String,
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

pub fn eval_loop(file: File) {
    let mut bitreader = BitReader::new(file);
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
        if !eval(input, &mut current, &mut bitreader) {
            break;
        }
    }
}
