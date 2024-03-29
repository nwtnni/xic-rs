use std::path::Path;

use crate::data::span;
use crate::data::symbol;
use crate::data::symbol::Symbol;
use crate::data::token;
use crate::error;
use crate::lex::Error;
use crate::lex::ErrorKind;
use crate::util;
use crate::util::TakeUntil as _;
use crate::util::Tap as _;

pub fn lex(path: &Path) -> Result<token::Tokens, crate::Error> {
    log::info!(
        "[{}] Lexing {}...",
        std::any::type_name::<Path>(),
        path.display()
    );
    util::time!(
        "[{}] Done lexing {}",
        std::any::type_name::<Path>(),
        path.display()
    );

    let source = std::fs::read_to_string(path)?;
    let path = symbol::intern(path.to_str().unwrap());
    let tokens = Lexer::new(&path, &source).lex();
    Ok(tokens)
}

/// Stateful Xi lexer.
/// Converts a stream of source characters into a stream of `Token`s.
struct Lexer<'source> {
    /// Path to source file for diagnostics
    path: Symbol,

    /// View into original source
    source: &'source str,

    /// Iterator over source characters
    stream: std::iter::Peekable<std::str::CharIndices<'source>>,

    /// Next index and character in stream
    next: Option<(usize, char)>,

    /// Current byte index
    index: usize,

    /// Current row position
    row: usize,

    /// Current column position
    column: usize,
}

fn is_digit(c: char) -> bool {
    matches!(c, '0'..='9')
}

fn is_hex_digit(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F')
}

fn is_ident(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '\'')
}

fn is_whitespace(c: char) -> bool {
    matches!(c, '\n' | '\t' | '\r' | ' ')
}

impl<'source> Lexer<'source> {
    /// Construct a new lexer
    pub fn new(path: &Symbol, source: &'source str) -> Self {
        let mut stream = source.char_indices().peekable();
        let next = stream.next();
        Lexer {
            path: *path,
            source,
            stream,
            next,
            index: 0,
            row: 1,
            column: 1,
        }
    }

    pub fn lex(&mut self) -> token::Tokens {
        self.take_until(Result::is_err)
            .collect::<Vec<_>>()
            .tap(token::Tokens::new)
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
        span::Point::new(self.path, self.index, self.row, self.column)
    }

    /// Skip the next character in the stream
    fn skip(&mut self) {
        if let Some((_, '\n')) = self.next {
            self.row += 1;
            self.column = 0;
        }
        self.next = self.stream.next();
        if let Some((i, _)) = self.next {
            self.column += 1;
            self.index = i;
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
                Some('\n') | Some('\t') | Some('\r') | Some(' ') => {
                    self.skip();
                }
                Some('/') if self.peeeek() == Some('/') => {
                    self.take_while(|c| c != '\n');
                    self.skip();
                }
                None | Some(_) => return,
            }
        }
    }

    /// Advance iterator while predicate holds and return end point
    fn take_while<F>(&mut self, mut f: F) -> span::Point
    where
        F: FnMut(char) -> bool,
    {
        while let Some((_, c)) = self.next {
            if !f(c) {
                return self.point();
            }
            self.skip();
        }
        self.point().bump()
    }

    /// Lex a single identifier
    fn lex_ident(&mut self, start: span::Point) -> token::Spanned {
        use token::Token::*;
        let end = self.take_while(is_ident);
        let token = match &self.source[start.index()..end.index()] {
            "use" => Use,
            "template" => Template,
            "class" => Class,
            "this" => This,
            "super" => Super,
            "new" => New,
            "extends" => Extends,
            "final" => Final,
            "null" => Null,
            "break" => Break,
            "if" => If,
            "do" => Do,
            "while" => While,
            "else" => Else,
            "return" => Return,
            "length" => Length,
            r#type @ ("int" | "int64" | "uint" | "uint64") => Int {
                signed: r#type.starts_with('i'),
                size: token::Size::_64,
            },
            r#type @ ("int32" | "uint32") => Int {
                signed: r#type.starts_with('i'),
                size: token::Size::_32,
            },
            r#type @ ("int16" | "uint16") => Int {
                signed: r#type.starts_with('i'),
                size: token::Size::_16,
            },
            r#type @ ("int8" | "uint8") => Int {
                signed: r#type.starts_with('i'),
                size: token::Size::_8,
            },
            "bool" => Bool,
            "true" => True,
            "false" => False,
            ident => Identifier(symbol::intern(ident)),
        };
        Ok((start, token, end))
    }

    /// Lex a single integer literal
    fn lex_integer(&mut self, start: span::Point) -> token::Spanned {
        let end = self.take_while(is_digit);
        let int = self.source[start.index()..end.index()]
            .to_string()
            .tap(token::Token::Integer);
        Ok((start, int, end))
    }

    /// Lex and unescape a single char
    fn lex_char(&mut self, start: span::Point, string: bool) -> Result<char, error::Error> {
        match self.advance() {
            Some('\n') | Some('\r') | Some('\x08') | Some('\x0C') => {
                let span = start.into();
                let kind = if string {
                    ErrorKind::InvalidString
                } else {
                    ErrorKind::InvalidCharacter
                };
                Err(Error::new(span, kind).into())
            }
            Some('\'') if !string => {
                let span = start.into();
                let kind = ErrorKind::InvalidCharacter;
                Err(Error::new(span, kind).into())
            }
            Some('\\') => match self.advance() {
                Some('n') => Ok('\n'),
                Some('r') => Ok('\r'),
                Some('t') => Ok('\t'),
                Some('b') => Ok('\x08'),
                Some('f') => Ok('\x0C'),
                Some('\\') => Ok('\\'),
                Some('\'') => Ok('\''),
                Some('\"') => Ok('\"'),
                Some('u') | Some('x') => {
                    let mut count = 0;
                    let start = self.point();
                    let end = self.take_while(|c| {
                        count += 1;
                        is_hex_digit(c) && count <= 4
                    });
                    let span = span::Span::new(start, end);
                    u32::from_str_radix(&self.source[start.index()..end.index()], 16)
                        .ok()
                        .and_then(std::char::from_u32)
                        .ok_or_else(|| Error::new(span, ErrorKind::InvalidCharacter).into())
                }
                _ => {
                    let span = start.into();
                    let kind = ErrorKind::InvalidEscape;
                    Err(Error::new(span, kind).into())
                }
            },
            Some(ch) => Ok(ch),
            None => {
                let span = start.into();
                let kind = ErrorKind::UnclosedCharacter;
                Err(Error::new(span, kind).into())
            }
        }
    }

    /// Lex a single character literal
    fn lex_character(&mut self, start: span::Point) -> token::Spanned {
        let ch = self.lex_char(start, false).map(token::Token::Character)?;
        if let Some('\'') = self.advance() {
            Ok((start, ch, self.point()))
        } else {
            let span = start.into();
            let kind = ErrorKind::UnclosedCharacter;
            Err(Error::new(span, kind).into())
        }
    }

    /// Lex a single string literal
    fn lex_string(&mut self, start: span::Point) -> token::Spanned {
        let mut buffer = String::new();
        let mut skip = false;
        while let Some(ch) = self.peek() {
            match (skip, ch) {
                (true, ch) if is_whitespace(ch) => self.skip(),
                (true, _) => skip = false,

                (false, '\\') if self.peeeek().map_or(false, is_whitespace) => {
                    self.skip();
                    skip = true;
                }
                (false, '\"') => {
                    self.skip();
                    return Ok((start, token::Token::String(buffer), self.point()));
                }
                (false, _) => {
                    buffer.push(self.lex_char(start, true)?);
                }
            }
        }
        let span = span::Span::new(start, self.point());
        let kind = ErrorKind::UnclosedString;
        Err(Error::new(span, kind).into())
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = token::Spanned;

    fn next(&mut self) -> Option<Self::Item> {
        use token::Token::*;

        // Skip past whitespace and comments
        self.forward();

        self.peek()?;

        let start = self.point();
        let ch = self.advance().unwrap();
        let mut end = self.point();

        macro_rules! eat {
            ($token:expr) => {{
                self.skip();
                end = self.point();
                $token
            }};
        }

        let token = match ch {
            'a'..='z' | 'A'..='Z' => return Some(self.lex_ident(start)),
            '\'' => return Some(self.lex_character(start)),
            '"' => return Some(self.lex_string(start)),
            '0'..='9' => return Some(self.lex_integer(start)),
            '_' => Underscore,
            '.' => Period,
            ',' => Comma,
            ';' => Semicolon,
            ':' => Colon,
            '{' => LBrace,
            '}' => RBrace,
            '[' => LBrack,
            ']' => RBrack,
            '(' => LParen,
            ')' => RParen,
            '&' => And,
            '|' => Or,
            '+' => Add,
            '-' => Sub,
            '%' => Mod,
            '/' => Div,
            '!' if self.peek() == Some('=') => eat!(Ne),
            '!' => Not,
            '<' if self.peek() == Some('=') => eat!(Le),
            '<' => Lt,
            '>' if self.peek() == Some('=') => eat!(Ge),
            '>' => Gt,
            '=' if self.peek() == Some('=') => eat!(Eq),
            '=' => Assign,
            '*' if self.peek() == Some('>') && self.peeeek() == Some('>') => {
                self.skip();
                self.skip();
                end = self.point();
                Hul
            }
            '*' => Mul,
            _ => {
                let span = span::Span::new(start, end);
                let kind = ErrorKind::UnknownCharacter;
                return Some(Err(Error::new(span, kind).into()));
            }
        };

        Some(Ok((start, token, end)))
    }
}
