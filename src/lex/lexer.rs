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

    /// Next character in stream
    next: Option<(usize, char)>,

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
        Lexer { source, stream, next, row: 0, col: 0 }
    }

    /// Look at the next character without consuming
    fn peek(&self) -> Option<char> {
        self.next.map(|(_, c)| c)
    }

    /// Look at the next next character without consuming
    fn peeeek(&self) -> Option<char> {
        self.stream.clone().peek().map(|(_, c)| *c)
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
        | Some((_, '\n')) => { self.col += 1; self.row = 0; },
        | Some(_)         => { self.row += 1; },
        | None            => (),
        };
        self.next = self.stream.next();
    }

    /// Read the next character in the stream
    fn advance(&mut self) -> Option<(usize, char)> {
        match self.next {
        | Some((_, '\n')) => { self.col += 1; self.row = 0; },
        | Some(_)         => { self.row += 1; },
        | None            => (),
        };
        let next = self.next;
        self.next = self.stream.next();
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

    fn take_while<F>(&mut self, start: usize, f: F) -> usize
        where F: Fn(char) -> bool
    {
        let mut end = start;
        while let Some((i, c)) = self.next {
            if !f(c) { break }
            self.skip();
            end = i;
        }
        end
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

        let (start, c) = self
            .advance()
            .expect("Always safe to unwrap here");

        let result = match c {
        | 'a'..='z' | 'A'..='Z' => {
            let end = self.take_while(start, is_ident);
            match &self.source[start..=end] {
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
            let end = self.take_while(start, is_digit);
            let span = self.point().into();
            i64::from_str(&self.source[start..=end])
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
