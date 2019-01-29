use std::str::FromStr;

use simple_symbol::store;

use crate::error;
use crate::lex;
use crate::span;
use crate::token;

/// Stateful Xi lexer.
/// Converts a stream of source characters into a stream of `Token`s.
pub struct Lexer<'source> {
    /// View into original source
    source: &'source str,

    /// Iterator over source characters
    stream: std::iter::Peekable<std::str::CharIndices<'source>>,

    /// Next index and character in stream
    next: Option<(usize, char)>,

    /// Current byte index
    idx: usize,

    /// Current row position
    row: usize,

    /// Current column position
    col: usize,
}

impl<'source> Lexer<'source> {
    /// Construct a new lexer
    pub fn new(source: &'source str) -> Self {
        let mut stream = source.char_indices().peekable();
        let next = stream.next();
        Lexer { source, stream, next, idx: 0, row: 0, col: 0 }
    }

    /// Look at the next character without consuming
    fn peek(&self) -> Option<char> {
        self.next.map(|(_, c)| c)
    }

    /// Look at the next next character without consuming
    fn peeeek(&self) -> Option<char> {
        self.stream.clone().peek().map(|(_, c)| *c)
    }

    /// Return the current byte index in the source file
    fn index(&self) -> usize {
        self.idx
    }

    /// Return the current position in the source file
    fn point(&self) -> span::Point {
        span::Point {
            row: self.row,
            col: self.col,
        }
    }

    /// Skip the next character in the stream
    fn skip(&mut self) {
        match self.next {
        | Some((i, '\n')) => {
            self.col += 1;
            self.row = 0;
            self.idx = i;
            self.next = self.stream.next();
        }
        | Some((i, _)) => {
            self.row += 1;
            self.idx = i;
            self.next = self.stream.next();
        },
        | None => (),
        }
    }

    /// Read the next character in the stream
    fn advance(&mut self) -> Option<char> {
        let next = self.next.map(|(_, c)| c);
        self.skip();
        next
    }

    /// Skip past whitespace and comments
    fn forward(&mut self) {
        loop {
            match self.peek() {
            | Some('\n') | Some('\t') | Some('\r') | Some(' ') => {
                self.skip();
            }
            | Some('/') if self.peeeek() == Some('/') => {
                while self.peek().is_some() && self.peek() != Some('\n') { self.skip(); }
                self.skip();
            }
            | None | Some(_) => {
                return
            }
            }
        }
    }

    fn take_while<F>(&mut self, f: F) where F: Fn(char) -> bool {
        while let Some((_, c)) = self.next {
            if !f(c) { return }
            self.skip();
        }
    }
}

fn is_digit(c: char) -> bool {
    match c {
    | '0'..='9' => true,
    | _         => false,
    }
}

fn is_ident(c: char) -> bool {
    match c {
    | 'a'..='z'
    | 'A'..='Z'
    | '0'..='9'
    | '_'
    | '\'' => true,
    | _    => false,
    }
}

impl<'source> Iterator for Lexer<'source> {

    type Item = Result<token::Token, error::Error>;

    fn next(&mut self) -> Option<Self::Item> {

        // Skip past whitespace and comments
        self.forward();

        if let None = self.peek() { return None }

        use token::Token::*;

        let c = self
            .advance()
            .expect("Always safe to unwrap here");

        let start_index = self.index();
        let start_point = self.point();

        let result = match c {
        | 'a'..='z' | 'A'..='Z' => {
            self.take_while(is_ident);
            let end_index = self.index();
            let end_point = self.point();
            let span = span::Span { lo: start_point, hi: end_point };
            match &self.source[start_index..=end_index] {
            | "use"    => Ok(USE),
            | "if"     => Ok(IF),
            | "while"  => Ok(WHILE),
            | "else"   => Ok(ELSE),
            | "return" => Ok(RETURN),
            | "length" => Ok(LENGTH),
            | "int"    => Ok(INT),
            | "bool"   => Ok(BOOL),
            | "true"   => Ok(TRUE),
            | "false"  => Ok(FALSE),
            | ident    => Ok(IDENT(store(ident))),
            }
        }
        | '\''                  => unimplemented!(),
        | '"'                   => unimplemented!(),
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
        | '0'..='9'
        | '-' if self.peek().map_or(false, is_digit) => {
            self.take_while(is_digit);
            let end_index = self.index();
            let end_point = self.point();
            let span = span::Span { lo: start_point, hi: end_point };
            i64::from_str(&self.source[start_index..=end_index])
                .map_err(|_| error::Error::lexical(span, lex::Error::InvalidInteger))
                .map(token::Token::INTEGER)
        }
        | '-' => Ok(SUB),
        | '!' if self.peek() == Some('=') => {
            self.skip();
            Ok(NEQ)
        }
        | '!' => Ok(NOT),
        | '<' if self.peek() == Some('=') => {
            self.skip();
            Ok(LE)
        }
        | '<' => Ok(LT),
        | '>' if self.peek() == Some('=') => {
            self.skip();
            Ok(GE)
        }
        | '>' => Ok(GT),
        | '=' if self.peek() == Some('=') => {
            self.skip();
            Ok(EQ)
        }
        | '=' => Ok(ASSIGN),
        | '*' if self.peek() == Some('>') && self.peeeek() == Some('>') => {
            self.skip();
            self.skip();
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
