// Ethan Olchik
// src/utils/compile.rs
// This file contains the compile function which is used to compile the source code.

//> Imports

use crate::{
    ast::ast::Module, frontend::{
        lexer::lexer::Lexer,
        parser::parser::Parser
    }
};

use std::fs::read_to_string;

//> Definitions

pub fn compile(filename: String) -> Option<Module> {
    let source = match read_to_string(&filename) {
        Ok(content) => content,
        Err(err) => panic!("Failed to read file {}: {}", filename.clone(), err),
    };

    let mut lexer = Lexer::new(filename.clone(), source.clone());
    let tokens = lexer.scan_tokens();

    if lexer.had_error {
        return None;
    }

    let mut parser = Parser::new(filename.clone(), source.clone(), tokens);

    let statements = parser.parse();

    if parser.had_error {
        return None;
    }

    Some(statements)
}

pub fn parse(filename: String) -> Option<Module> {
    let source = match read_to_string(&filename) {
        Ok(content) => content,
        Err(err) => panic!("Failed to read file {}: {}", filename.clone(), err),
    };

    let mut lexer = Lexer::new(String::from(filename.clone()), source.clone());
    let tokens = lexer.scan_tokens();

    if lexer.had_error {
        return None;
    }

    let mut parser = Parser::new(String::from(filename.clone()), source.clone(), tokens);

    let statements = parser.parse();

    if parser.had_error {
        return None;
    }

    Some(statements)
}