use std::str::FromStr;

use crate::lex;
use crate::span;
use crate::symbol;
use crate::token;

pub type Spanned = Result<(span::Point, token::Token, span::Point), lex::Error>;

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
    | _ => false,
    }
}

fn is_hex_digit(c: char) -> bool {
    match c {
    | '0'..='9'
    | 'a'..='f'
    | 'A'..='F' => true,
    | _ => false,
    }
}

fn is_ident(c: char) -> bool {
    match c {
    | 'a'..='z'
    | 'A'..='Z'
    | '0'..='9'
    | '_'
    | '\'' => true,
    | _ => false,
    }
}

impl<'source> Lexer<'source> {
    /// Construct a new lexer
    pub fn new(source: &'source str) -> Self {
        let mut stream = source.char_indices().peekable();
        let next = stream.next();
        Lexer { source, stream, next, idx: 0, row: 1, col: 1 }
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
            idx: self.idx,
            row: self.row,
            col: self.col,
        }
    }

    /// Skip the next character in the stream
    fn skip(&mut self) {
        if let Some((_, '\n')) = self.next {
            self.row += 1;
            self.col = 0;
        }
        self.next = self.stream.next();
        if let Some((i, _)) = self.next {
            self.col += 1;
            self.idx = i;
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
                self.take_while(|c| c != '\n');
                self.skip();
            }
            | None | Some(_) => {
                return
            }
            }
        }
    }

    /// Advance iterator while predicate holds and return end point
    fn take_while<F>(&mut self, mut f: F) -> span::Point
        where F: FnMut(char) -> bool
    {
        while let Some((_, c)) = self.next {
            if !f(c) { return self.point() }
            self.skip();
        }
        self.point().bump()
    }

    /// Lex a single identifier
    fn lex_ident(&mut self, start: span::Point) -> Spanned {
        let end = self.take_while(is_ident);
        use token::Token::*;
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
        | ident    => IDENT(symbol::intern(ident)),
        };
        Ok((start, token, end))
    }

    /// Lex a single integer literal
    fn lex_integer(&mut self, start: span::Point) -> Spanned {
        let end = self.take_while(is_digit);
        let span = span::Span::new(start, end);
        let int = i64::from_str(&self.source[start.idx..end.idx])
            .map_err(|_| lex::Error::new(span, lex::ErrorKind::InvalidInteger))
            .map(token::Token::INTEGER)
            .map(|token| (start, token, end))?;
        Ok(int)
    }

    /// Lex and unescape a single char
    fn lex_char(&mut self) -> Result<char, lex::Error> {
        let start = self.point();
        match self.advance() {
        | Some('\\') => {
            match self.advance() {
            | Some('n')  => Ok('\n'),
            | Some('r')  => Ok('\r'),
            | Some('t')  => Ok('\t'),
            | Some('\\') => Ok('\\'),
            | Some('\'') => Ok('\''),
            | Some('\"') => Ok('\"'),
            | Some('x')  => {
                let mut count = 0;
                let start = self.point();
                let end = self.take_while(|c| { count += 1; is_hex_digit(c) && count <= 4 });
                let span = span::Span::new(start, end);
                u32::from_str_radix(&self.source[start.idx..end.idx], 16)
                    .ok()
                    .and_then(std::char::from_u32)
                    .ok_or_else(|| lex::Error::new(span, lex::ErrorKind::InvalidCharacter))
            }
            | _ => {
                let span = start.into();
                let kind = lex::ErrorKind::InvalidEscape;
                Err(lex::Error::new(span, kind))
            }
            }
        }
        | Some(ch) => Ok(ch),
        | None => {
            let span = start.into();
            let kind = lex::ErrorKind::UnclosedCharacter;
            Err(lex::Error::new(span, kind))
        }
        }
    }

    /// Lex a single character literal
    fn lex_character(&mut self, start: span::Point) -> Spanned {
        let ch = self.lex_char().map(token::Token::CHARACTER)?;
        if let Some('\'') = self.advance() {
            Ok((start, ch, self.point()))
        } else {
            let span = start.into();
            let kind = lex::ErrorKind::UnclosedCharacter;
            Err(lex::Error::new(span, kind))
        }
    }

    /// Lex a single string literal
    fn lex_string(&mut self, start: span::Point) -> Spanned {
        let mut buffer = String::new();
        while self.next.is_some() {
            if let Some('\"') = self.peek() {
                self.skip();
                return Ok((start, token::Token::STRING(buffer), self.point()))
            } else {
                buffer.push(self.lex_char()?)
            }
        }
        let span = span::Span::new(start, self.point());
        let kind = lex::ErrorKind::UnclosedString;
        Err(lex::Error::new(span, kind))
    }
}

impl<'source> Iterator for Lexer<'source> {

    type Item = Spanned;

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
        | '\'' => return Some(self.lex_character(start)),
        | '"'  => return Some(self.lex_string(start)),
        | '0'..='9' => return Some(self.lex_integer(start)),
        | 'a'..='z' | 'A'..='Z' => return Some(self.lex_ident(start)),
        | '-' if self.peek().map_or(false, is_digit) => return Some(self.lex_integer(start)),
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
            let span = span::Span::new(start, end);
            let kind = lex::ErrorKind::UnknownCharacter;
            return Some(Err(lex::Error::new(span, kind)))
        }
        };

        Some(Ok((start, token, end)))
    }
}
