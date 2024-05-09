
/// Ethan Olchik
/// src/utils/mod.rs


pub mod imports;
pub mod compile;
pub mod module;

//> Definitions


/// Position struct
/// This struct represents a position in the source code.
/// [Debug, Clone]

#[derive(Debug, Clone)]
pub struct Position {
    pub start_line: usize,
    pub end_line: usize,

    pub start_pos: usize,
    pub end_pos: usize
}

//> Implementations

impl Position {
    pub fn empty() -> Self {
        Self {
            start_line: 0,
            end_line: 0,
            start_pos: 0,
            end_pos: 0
        }
    }
}