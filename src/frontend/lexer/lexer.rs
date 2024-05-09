// Ethan Olchik
// src/frontend/lexer/lexer.rs
// This file contains the Lexer struct and its implementation. The Lexer struct is used to
// convert a string of source code into a vector of tokens.


//> Imports

use codespan_reporting::diagnostic::Label;

use crate::{
    errors::errors::Error,
    frontend::lexer::token::{Token, TokenType}
};

//> Struct Definitions


/// Lexer(source) struct
/// [Debug, Clone]

#[derive(Debug, Clone)]
pub struct Lexer {
    pub source: String,
    pub filename: String,
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub pos: usize,

    pub had_error: bool
}

//> Implementation
impl Lexer {
    pub fn new(filename: String, source: String) -> Self {
        Self {
            source,
            filename,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            pos: 1,

            had_error: false
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token {
            kind: TokenType::Eof,
            lexeme: String::from(""),
            line: self.line,
            pos: self.pos,
        });
        self.tokens.clone()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c: char = self.advance();

        match c {
            //> Whitespace
            ' ' | '\r' | '\t' | '\n' => {},
            //> Single Character Tokens
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            ';' => self.add_token(TokenType::Semicolon),
            ':' => self.add_token(TokenType::Colon),

            //> One or Two Character Tokens
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::BangEquals);
                } else {
                    self.add_token(TokenType::Bang);
                }
            }
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::EqualsEquals);
                } else {
                    self.add_token(TokenType::Equals);
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.add_token(TokenType::LessEquals);
                    self.advance();
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::GreaterEquals);
                } else {
                    self.add_token(TokenType::Greater);
                }
            }
            '+' => {
                if self.peek() == '+' {
                    self.advance();
                    self.add_token(TokenType::PlusPlus);
                } else if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::PlusEquals);
                } else {
                    self.add_token(TokenType::Plus);
                }
            }
            '-' => {
                if self.peek() == '-' {
                    self.advance();
                    self.add_token(TokenType::MinusMinus);
                } else if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::MinusEquals);
                } else if self.peek() == '>' {
                    self.advance();
                    self.add_token(TokenType::Arrow);
                } else {
                    self.add_token(TokenType::Minus);
                }
            }
            '*' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::StarEquals);
                } else if self.peek() == '*' {
                    self.advance();
                    self.add_token(TokenType::StarStar);
                } else {
                    self.add_token(TokenType::Star);
                }
            }
            '/' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::SlashEquals);
                } else if self.peek() == '/' {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.peek() == '*' {
                    self.multi_line_comment();
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            '&' => {
                if self.peek() == '&' {
                    self.advance();
                    self.add_token(TokenType::AmpersandAmpersand);
                } else if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::AmpersandEquals);
                } else {
                    self.add_token(TokenType::Ampersand);
                }
            }
            '|' => {
                if self.peek() == '|' {
                    self.advance();
                    self.add_token(TokenType::PipePipe);
                } else if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::PipeEquals);
                } else {
                    self.add_token(TokenType::Pipe);
                }
            }
            '^' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::CaretEquals);
                } else {
                    self.add_token(TokenType::Caret);
                }
            }
            '%' => {
                if self.peek() == '=' {
                    self.advance();
                    self.add_token(TokenType::PercentEquals);
                } else {
                    self.add_token(TokenType::Percent);
                }
            }

            //> Literals
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),

            //> Other
            '"' | '\'' => self.string(c),
            '\0' => {
                // This means that we have reached the end of the file whilst scanning a token.
                // We don't need to do anything here because we will add an EOF token after the loop.
            }
            _ => {
                let mut error = Error::new(
                    format!("Unexpected character: {}", c),
                    String::from("E001")
                );

                error.build(
                    self.filename.clone(),
                    self.source.clone(),
                    vec![
                        Label::primary(0, self.start..self.current).with_message("Unexpected character.")
                    ]
                );

                error.emit();

                self.had_error = true;
            }
        }
    }

    fn number(&mut self) {
        let mut is_float = false;
        while self.peek().is_digit(10) {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            is_float = true;
            self.advance();
            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        self.add_token(if is_float { TokenType::Float } else { TokenType::Integer });
    }

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let kind = match text {
            "var" => TokenType::Var,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "return" => TokenType::Return,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "func" => TokenType::Func,
            "struct" => TokenType::Struct,
            "enum" => TokenType::Enum,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "export" => TokenType::Export,
            "pub" => TokenType::Pub,
            "import" => TokenType::Import,
            "module" => TokenType::Module,
            "as" => TokenType::As,
            "extern" => TokenType::Extern,
            "static" => TokenType::Static,
            "const" => TokenType::Const,
            _ => TokenType::Identifier
        };

        self.add_token(kind);
    }

    fn string(&mut self, quote: char) {
        while self.peek() != quote && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.pos = 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            // Error: Unterminated String
            let mut error = Error::new(
                String::from("Unterminated string"),
                String::from("E002")
            );

            error.build(
                self.filename.clone(),
                self.source.clone(),
                vec![
                    Label::primary(0, self.start..self.current).with_message(format!("Expected {:?}, got EOF.", quote))
                ]
            );

            error.emit();
            self.had_error = true;
            return;
        }

        self.advance();

        let value = (&self.source[self.start + 1..self.current - 1]).to_owned();
        self.add_token_with_lexeme(TokenType::String, value);
    }

    fn advance(&mut self) -> char {
        let current_char = self.source.chars().nth(self.current).unwrap();
        self.current += 1;

        if current_char == '\n' {
            self.line += 1;
            self.pos = 1;
        } else {
            self.pos += 1;
        }

        current_char
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn multi_line_comment(&mut self) {
        let mut nesting: i32 = 1;

        while nesting > 0 {
            if self.is_at_end() {
                return;
            }

            if self.peek() == '/' && self.source.chars().nth(self.current + 1).unwrap() == '*' {
                self.advance();
                self.advance();
                nesting += 1;
            } else if self.peek() == '*' && self.source.chars().nth(self.current + 1).unwrap() == '/' {
                self.advance();
                self.advance();
                nesting -= 1;
            } else {
                self.advance();
            }
        }
    }

    fn add_token(&mut self, kind: TokenType) {
        self.tokens.push(
            Token::new(
                kind,
                String::from(self.source[self.start..self.current].to_owned()),
                self.line,
                self.pos
            )
        );

        self.start = self.current;
    }
    fn add_token_with_lexeme(&mut self, kind: TokenType, lexeme: String) {
        self.tokens.push(
            Token::new(
                kind,
                lexeme,
                self.line,
                self.pos
            )
        );

        self.start = self.current;
    }
}


//> Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let filename = "C:/Users/OLCHIK/Matcha/src/test.mt";
        let source = match std::fs::read_to_string(&filename) {
            Ok(content) => content,
            Err(err) => panic!("Failed to read file {}: {}", &filename, err), // Error: Failed to read file
        };
        let mut lexer = Lexer::new(String::from(filename), source.clone());
        let tokens = lexer.scan_tokens();

        assert_eq!(tokens[0], Token::new(TokenType::Import, String::from("import"), 1, 7));

        // TODO: Finish writing these tests
    }
}