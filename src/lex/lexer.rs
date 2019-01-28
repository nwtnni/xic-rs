use crate::error;
use crate::lex;
use crate::token;

/// Stateful Xi lexer.
/// Converts a stream of source characters into a stream of `Token`s.
pub struct Lexer<'source> {
    /// Iterator over source characters
    stream: std::str::Chars<'source>,

    /// Next character in stream
    next: Option<char>,

    /// Current row position
    row: usize,

    /// Current column position
    col: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        let mut stream = source.chars();
        let next = stream.next();
        Lexer { stream, next, row: 0, col: 0 }
    }
}

impl<'source> Iterator for Lexer<'source> {

    type Item = Result<token::Token, error::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
