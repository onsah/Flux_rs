#[derive(Copy, Clone, Debug)]
pub enum LexError {
    // Expected different char
    UnexpectedChar { line: usize },
    // source is unfinished
    TooShort { line: usize },
    // Invalid character
    InvalidChar { ch: char, line: usize },
    Eof,
}
