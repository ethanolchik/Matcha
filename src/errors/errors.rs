// Ethan Olchik
// src/errors/errors.rs
// This file contains the Error struct which is used throughout the compiler to represent errors.

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        emit, Config,
        termcolor::{ColorChoice, StandardStream}
    }
};

/// Error(labels, message, code) struct
/// This struct is used to represent errors throughout the compiler.
pub struct Error {
    pub files: SimpleFiles<String, String>,
    pub writer: StandardStream,
    pub config: Config,

    pub labels: Vec<Label<usize>>,
    pub message: String,
    pub code: String
}

//> Implementation

impl Error {
    pub fn new(message: String, code: String) -> Error {
        Error {
            files: SimpleFiles::new(),
            writer: StandardStream::stderr(ColorChoice::Always),
            config: Config::default(),

            labels: vec![],
            message,
            code
        }
    }

    fn add_file(&mut self, file_name: String, file_content: String) -> usize {
        self.files.add(file_name, file_content)
    }

    pub fn build(&mut self, file_name: String, file_content: String, labels: Vec<Label<usize>>)
    {
        let file_id = self.add_file(file_name, file_content);

        for mut label in labels {
            label.file_id = file_id;
            self.labels.push(label)
        }
    }

    pub fn emit(&mut self) {
        let diagnostic: Diagnostic<usize>= Diagnostic::error()
            .with_message(self.message.clone())
            .with_code(self.code.clone())
            .with_labels(self.labels.clone());

        emit(&mut self.writer.lock(), &self.config, &self.files, &diagnostic).unwrap()
    }
} 