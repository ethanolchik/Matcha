mod frontend;
mod errors;
mod ast;
mod semantic;
mod utils;

use std::fmt::write;
use utils::compile::compile;

fn main() {
    let statements = compile(String::from("C:/Users/OLCHIK/Matcha/test/src/test.mt"));
    let mut output = String::new();
    write(&mut output, format_args!("{:#?}", statements)).unwrap();

    // write to file
    let output_filename = "C:/Users/OLCHIK/Matcha/test/src/test.ast";
    match std::fs::write(&output_filename, output) {
        Ok(_) => println!("Successfully wrote to file {}", output_filename),
        Err(err) => panic!("Failed to write to file {}: {}", output_filename, err),
    }
}