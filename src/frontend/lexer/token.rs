 // Ethan Olchik
// src/frontend/lexer/token.rs
// This file contains the Token struct TokenType enum and Type enum which are used throughout
// the compiler to represent tokens and their types.

//> Struct Definitions

use std::vec;

/// Token(kind, lexeme, line, pos) struct <br>
/// impl Debug, Clone, PartialEq
#[derive(Clone)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub pos: usize,
}

/// TokenType enum <br>
/// impl Debug Copy Clone PartialEq
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Single-character tokens
    Plus, Minus, Star, Slash, LeftParen, RightParen,
    LeftBrace, RightBrace, LeftBracket, RightBracket,
    Comma, Dot, Equals, Semicolon, Colon, Bang,
    Less, Greater, Ampersand, Pipe, Percent, Caret,

    // Double-character tokens
    PlusEquals, MinusEquals, StarEquals, SlashEquals,
    EqualsEquals, BangEquals, LessEquals, GreaterEquals,
    AmpersandAmpersand, PipePipe, PlusPlus, MinusMinus,
    AmpersandEquals, PipeEquals, CaretEquals, PercentEquals,
    StarStar, Arrow,

    // Literals
    Identifier, Integer, Float, String,

    // Keywords
    If, Else, While, For, Return, Break, Continue,
    Import, Module,
    Var, Func, Struct, Enum,
    True, False, Null,
    Export, Pub, As, Extern, Static, Const, // TODO: Depreciate `static` for func (T) n() -> T {}

    // Other
    Eof
}

//> Trait Implementations

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} '{}' (at {}:{})", self.kind, self.lexeme, self.line, self.pos)
    }
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String, line: usize, pos: usize) -> Self {
        Token {
            kind,
            lexeme,
            line,
            pos
        }
    }
}

impl TokenType {
    pub fn assignment_operators() -> Vec<Self> {
        vec![
            TokenType::Equals,
            TokenType::PlusEquals,
            TokenType::MinusEquals,
            TokenType::StarEquals,
            TokenType::SlashEquals,
            TokenType::AmpersandEquals,
            TokenType::PipeEquals,
            TokenType::CaretEquals,
            TokenType::PercentEquals
        ]
    }
}