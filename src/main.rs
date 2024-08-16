mod ast;
mod errors;
mod frontend;
mod semantic;
mod utils;
mod codegen;

use std::{env, fmt::write};
use utils::compile::compile;

const PRINT_HELP: fn() -> () = || {
    println!("Usage: matcha [options] <filename>");
    println!("Options:");
    println!("\t-d, --debug\tEnable debug mode");
    println!("\t-nc, --no-colour\tDisable coloured output");
    println!("\t-h, --help\tDisplay this help message");
};

// TODO: Create a repl
fn main() {
    let mut filename: Option<String> = None;
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        PRINT_HELP();
        return;
    }

    for arg in args.iter() {
        match arg.as_str() {
            "-d" | "--debug" => set_flag_str!("debug"),
            "-nc" | "--no-colour" => set_flag_str!("no colour"),
            "-ast" => set_flag_str!("ast"),
            "-h" | "--help" => {
                PRINT_HELP();
                return;
            }
            _ => {
                filename = Some(arg.clone());
            }
        }
    }

    if filename.is_none() {
        panic!("No filename provided");
    }

    let statements = compile(String::from(filename.clone().unwrap()));

    // if is_flag_set_str!("ast") {
        let mut output = String::new();
        write(&mut output, format_args!("{:#?}", statements)).unwrap();

        // write to file
        let output_filename = filename.unwrap() + ".ast";
        match std::fs::write(&output_filename, output) {
            Ok(_) => println!("Successfully wrote to file {}", output_filename),
            Err(err) => panic!("Failed to write to file {}: {}", output_filename, err),
        }
    // }
}
