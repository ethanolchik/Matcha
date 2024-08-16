// Ethan Olchik
// src/utils/compile.rs
// This file contains the compile function which is used to compile the source code.

//> Imports

use crate::{
    ast::ast::Module,
    debug,
    frontend::{lexer::lexer::Lexer, parser::parser::Parser},
    semantic::{
        first_pass::FirstPassResolver, recursion::RecursionChecker, resolver::Resolver, SymbolTable,
    },
    codegen::codegen::Codegen
};

use std::{fs::read_to_string, os::unix::process::ExitStatusExt};
use std::process::Command;

//> Definitions

pub fn compile(filename: String) -> Option<Module> {
    let m = parse(filename.clone()).unwrap();
    let symtable = resolve(filename.clone(), &m).unwrap();

    debug!(format!("{:#?}", symtable.decl_queue));

    let mut codegen = Codegen::new(filename.clone(), symtable.clone(), symtable.import_handler);
    codegen.generate(&m);
    
    let output = Command::new("gcc")
        .arg(filename.clone() + ".c")
        .arg("-o")
        .arg("out")
        .output();

    match output {
        Ok(_) => (),
        Err(err) => panic!("Failed to compile file {}: {}", filename.clone(), err),
    }

    // Command::new("rm")
    //     .arg(filename.clone() + ".c")
    //     .output()
    //     .unwrap();

    let exit_code = Command::new("./out").status().unwrap();

    // Command::new("rm")
    //     .arg("out")
    //     .output()
    //     .unwrap();

    match exit_code.code() {
        Some(0) => (),
        Some(code) => panic!("Program exited with code {}", code),
        None => panic!("Program exited with signal {}", exit_code.signal().unwrap()),
    }

    Some(m)
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

pub fn resolve(filename: String, statements: &Module) -> Option<SymbolTable> {
    let mut first_pass_resolver = FirstPassResolver::new(filename.clone());
    first_pass_resolver.resolve(statements);

    first_pass_resolver.symtable.import_handler = first_pass_resolver.import_handler.clone();

    let mut resolver = Resolver::new(
        first_pass_resolver.symtable,
        filename.clone(),
        String::from(""),
        first_pass_resolver.import_handler,
    );
    resolver.resolve(statements);

    if resolver.had_error {
        return None;
    }

    let mut recursion_checker = RecursionChecker::new(filename.clone(), resolver.symtable.clone(), resolver.import_handler);
    recursion_checker.check(statements);

    if recursion_checker.had_error {
        return None;
    }

    Some(resolver.symtable)
}
