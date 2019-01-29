use crate::error;
use crate::lex;
use crate::span;
use crate::token;

/// Stateful Xi lexer.
/// Converts a stream of source characters into a stream of `Token`s.
pub struct Lexer<'source> {
    /// Iterator over source characters
    stream: std::iter::Peekable<std::str::Chars<'source>>,

    /// Next character in stream
    next: Option<char>,

    /// Current row position
    row: usize,

    /// Current column position
    col: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        let mut stream = source.chars().peekable();
        let next = stream.next();
        Lexer { stream, next, row: 0, col: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.next
    }

    fn peeeek(&self) -> Option<char> {
        self.stream.clone().peek().cloned()
    }

    fn point(&self) -> span::Point {
        span::Point {
            row: self.row,
            col: self.col,
        }
    }

    fn advance(&mut self) -> Option<char> {
        match self.next {
        | Some('\n') => { self.col += 1; self.row = 0; },
        | Some(_)    => { self.row += 1; },
        | None       => (),
        };

        let next = self.next;
        self.next = self.stream.next();
        next
    }
}

impl<'source> Iterator for Lexer<'source> {

    type Item = Result<token::Token, error::Error>;

    fn next(&mut self) -> Option<Self::Item> {

        if let None = self.peek() { return None }

        use token::Token::*;

        let result = match self.advance().expect("Always safe to unwrap here") {
        | 'a'..='z' | 'A'..='Z' => unimplemented!(),
        | '\''                  => unimplemented!(),
        | '"'                   => unimplemented!(),
        | '0'..='9'             => unimplemented!(),
        | '_' => Ok(UNDERSCORE),
        | ',' => Ok(COMMA),
        | ';' => Ok(SEMICOLON),
        | ':' => Ok(COLON),
        | '[' => Ok(LBRACE),
        | ']' => Ok(RBRACE),
        | '{' => Ok(LBRACK),
        | '}' => Ok(RBRACK),
        | '(' => Ok(LPAREN),
        | ')' => Ok(RPAREN),
        | '&' => Ok(LAND),
        | '|' => Ok(LOR),
        | '+' => Ok(ADD),
        | '%' => Ok(MOD),
        | '/' => Ok(DIV),
        | '-' if self.peek().map_or(false, |c| c.is_ascii_digit()) => {
            unimplemented!()
        }
        | '-' => Ok(SUB),
        | '!' if self.peek() == Some('=') => {
            self.advance();
            Ok(NEQ)
        }
        | '!' => Ok(NOT),
        | '<' if self.peek() == Some('=') => {
            self.advance();
            Ok(LE)
        }
        | '<' => Ok(LT),
        | '>' if self.peek() == Some('=') => {
            self.advance();
            Ok(GE)
        }
        | '>' => Ok(GT),
        | '=' if self.peek() == Some('=') => {
            self.advance();
            Ok(EQ)
        }
        | '=' => Ok(ASSIGN),
        | '*' if self.peek() == Some('>') && self.peeeek() == Some('>') => {
            self.advance();
            self.advance();
            Ok(HMUL)
        }
        | '*' => Ok(MUL),
        | _ => {
            let error = lex::Error::UnknownCharacter;
            let span = self.point().into();
            Err(error::Error::lexical(span, error))
        }
        };

        Some(result)
    }
}
