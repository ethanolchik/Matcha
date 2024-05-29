mod frontend;
mod errors;
mod ast;
mod semantic;
mod utils;

use std::{
    fmt::write,
    env
};
use utils::compile::compile;

fn main() {
    let filename = env::args().nth(1).expect("No filename provided");
    let statements = compile(String::from(filename.clone()));
    let mut output = String::new();
    write(&mut output, format_args!("{:#?}", statements)).unwrap();

    // write to file
    let output_filename = filename + ".ast";
    match std::fs::write(&output_filename, output) {
        Ok(_) => println!("Successfully wrote to file {}", output_filename),
        Err(err) => panic!("Failed to write to file {}: {}", output_filename, err),
    }
}