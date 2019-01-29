use std::str::FromStr;

use simple_symbol::store;

use crate::error;
use crate::lex;
use crate::span;
use crate::token;

pub type Spanned = (span::Point, token::Token, span::Point);

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
            idx: self.idx,
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

    fn take_while<F>(&mut self, f: F) -> span::Point
        where F: Fn(char) -> bool
    {
        while let Some((_, c)) = self.next {
            if !f(c) { return self.point() }
            self.skip();
        }
        let end = self.point();
        span::Point {
            idx: end.idx + 1,
            row: end.row,
            col: end.col + 1
        }
    }

    fn lex_ident(&mut self, start: span::Point) -> Result<Spanned, error::Error> {
        self.take_while(is_ident);
        use token::Token::*;
        let end = self.point();
        let token = match &self.source[start.idx..end.idx] {
        | "use"    => USE,
        | "if"     => IF,
        | "while"  => WHILE,
        | "else"   => ELSE,
        | "return" => RETURN,
        | "length" => LENGTH,
        | "int"    => INT,
        | "bool"   => BOOL,
        | "true"   => TRUE,
        | "false"  => FALSE,
        | ident    => IDENT(store(ident)),
        };
        Ok((start, token, end))
    }

    fn lex_integer(&mut self, start: span::Point) -> Result<Spanned, error::Error> {
        self.take_while(is_digit);
        let end = self.point();
        let span = span::Span::new(start, end);
        i64::from_str(&self.source[start.idx..end.idx])
            .map_err(|_| error::Error::lexical(span, lex::Error::InvalidInteger))
            .map(token::Token::INTEGER)
            .map(|token| (start, token, end))
    }
}

impl<'source> Iterator for Lexer<'source> {

    type Item = Result<(span::Point, token::Token, span::Point), error::Error>;

    fn next(&mut self) -> Option<Self::Item> {

        // Skip past whitespace and comments
        self.forward();

        if let None = self.peek() { return None }

        use token::Token::*;

        let start = self.point();
        let ch = self.advance().unwrap();
        let mut end = self.point();

        macro_rules! eat {
            ($token:expr) => { { self.skip(); end = self.point(); $token } }
        }

        let token = match ch {
        | 'a'..='z' | 'A'..='Z' => {
            return Some(self.lex_ident(start))
        }
        | '0'..='9' | '-' if self.peek().map_or(false, is_digit) => {
            return Some(self.lex_integer(start))
        }
        | '\'' => unimplemented!(),
        | '"'  => unimplemented!(),
        | '_' => UNDERSCORE,
        | ',' => COMMA,
        | ';' => SEMICOLON,
        | ':' => COLON,
        | '[' => LBRACE,
        | ']' => RBRACE,
        | '{' => LBRACK,
        | '}' => RBRACK,
        | '(' => LPAREN,
        | ')' => RPAREN,
        | '&' => LAND,
        | '|' => LOR,
        | '+' => ADD,
        | '-' => SUB,
        | '%' => MOD,
        | '/' => DIV,
        | '!' if self.peek() == Some('=') => eat!(NEQ),
        | '!' => NOT,
        | '<' if self.peek() == Some('=') => eat!(LE),
        | '<' => LT,
        | '>' if self.peek() == Some('=') => eat!(GE),
        | '>' => GT,
        | '=' if self.peek() == Some('=') => eat!(EQ),
        | '=' => ASSIGN,
        | '*' if self.peek() == Some('>') && self.peeeek() == Some('>') => {
            self.skip();
            self.skip();
            end = self.point();
            HMUL
        }
        | '*' => MUL,
        | _ => {
            let error = lex::Error::UnknownCharacter;
            let span = span::Span::new(start, end);
            return Some(Err(error::Error::lexical(span, error)))
        }
        };

        Some(Ok((start, token, end)))
    }
}
