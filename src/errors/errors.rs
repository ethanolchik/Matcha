// Ethan Olchik
// src/errors/errors.rs
// This file contains the Error struct which is used throughout the compiler to represent errors.

/// Error(labels, message, code) struct
/// This struct is used to represent errors throughout the compiler.
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub filename: String
}

/// ErrorKind enum
/// Represents the kind of error that occurred.
pub enum DiagnosticKind {
    Error,
    Warning,
    Note
}

impl Diagnostic {
    pub fn new(kind: DiagnosticKind, message: String, line: usize, col: usize, filename: String) -> Self {
        Self {
            kind,
            message,
            line,
            col,
            filename
        }
    }

    pub fn emit(&self) {
        eprintln!("{}:{}:{}: {}:\x1b[0m {}", self.filename, self.line, self.col, self.kind.as_string(), self.message);
        eprintln!("\t{} | {}\n", self.line, self.get_line_from_file(self.line));
    }

    fn get_line_from_file(&self, line: usize) -> String {
        let file = std::fs::read_to_string(&self.filename).unwrap();
        let lines = file.lines().collect::<Vec<&str>>();
        let line = lines.get(line - 1).unwrap();

        return String::from(*line);
    }
}

impl DiagnosticKind {
    pub fn colour(&self) -> String {
        match self {
            DiagnosticKind::Error => String::from("\x1b[91m"),
            DiagnosticKind::Warning => String::from("\x1b[93m"),
            DiagnosticKind::Note => String::from("\x1b[96m")
        }
    }

    pub fn as_string(&self) -> String {
        let x = match self {
            DiagnosticKind::Error => String::from("error"),
            DiagnosticKind::Warning => String::from("warning"),
            DiagnosticKind::Note => String::from("note")
        };

        return self.colour() + &x;
    }
}