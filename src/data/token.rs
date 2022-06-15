use std::fmt;
use std::vec;

use crate::data::span;
use crate::data::symbol;
use crate::data::symbol::Symbol;

pub type Spanned = Result<(span::Point, Token, span::Point), crate::Error>;

#[derive(Debug)]
pub struct Tokens(Vec<Spanned>);

impl Tokens {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Tokens(tokens)
    }
}

impl IntoIterator for Tokens {
    type Item = Spanned;
    type IntoIter = vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for Tokens {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for spanned in &self.0 {
            match spanned {
                Ok((left, token, _)) => writeln!(fmt, "{} {}", left, token)?,
                Err(error) => writeln!(fmt, "{}", error)?,
            }
        }
        Ok(())
    }
}

/// Represents a possible lexical token in the Xi language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    /// Identifier
    Identifier(Symbol),

    /// Character literal
    Character(char),

    /// Integer literal
    Integer(String),

    /// String literal
    String(String),

    /// `use` keyword
    Use,

    /// `template` keyword
    Template,

    /// `class` keyword
    Class,

    /// `this` keyword
    This,

    /// `super` keyword
    Super,

    /// `new` keyword
    New,

    /// `extends` keyword
    Extends,

    /// `final` keyword
    Final,

    /// `null` keyword
    Null,

    /// `break` keyword
    Break,

    /// `if` keyword
    If,

    /// `do` keyword
    Do,

    /// `while` keyword
    While,

    /// `else` keyword
    Else,

    /// `return` keyword
    Return,

    /// `length` keyword
    Length,

    /// `int` keyword
    Int,

    /// `bool` keyword
    Bool,

    /// `true` keyword
    True,

    /// `false` keyword
    False,

    /// `=` symbol
    Assign,

    /// `!` symbol
    Not,

    /// `*` symbol
    Mul,

    /// `*>>` symbol
    Hul,

    /// `/` symbol
    Div,

    /// `%` symbol
    Mod,

    /// `+` symbol
    Add,

    /// `-` symbol
    Sub,

    /// `<` symbol
    Lt,

    /// `<=` symbol
    Le,

    /// `>=` symbol
    Ge,

    /// `>` symbol
    Gt,

    /// `==` symbol
    Eq,

    /// `!=` symbol
    Ne,

    /// `&` symbol
    And,

    /// `|` symbol
    Or,

    /// `(` symbol
    LParen,

    /// `)` symbol
    RParen,

    /// `[` symbol
    LBrack,

    /// `]` symbol
    RBrack,

    /// `{` symbol
    LBrace,

    /// `}` symbol
    RBrace,

    /// `:` symbol
    Colon,

    /// `;` symbol
    Semicolon,

    /// `,` symbol
    Comma,

    /// `_` symbol
    Underscore,

    /// `.` symbol
    Period,
}

impl std::fmt::Display for Token {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Character(c) => match unescape_char(*c) {
                Some(s) => write!(fmt, "character {}", s),
                None => write!(fmt, "character {}", c),
            },
            Token::String(s) => write!(fmt, "string {}", unescape_str(s)),
            Token::Identifier(i) => write!(fmt, "id {}", symbol::resolve(*i)),
            Token::Integer(i) => write!(fmt, "integer {}", i),
            Token::Use => write!(fmt, "use"),
            Token::Template => write!(fmt, "template"),
            Token::Class => write!(fmt, "class"),
            Token::This => write!(fmt, "this"),
            Token::Super => write!(fmt, "super"),
            Token::New => write!(fmt, "new"),
            Token::Extends => write!(fmt, "extends"),
            Token::Final => write!(fmt, "final"),
            Token::Null => write!(fmt, "null"),
            Token::Break => write!(fmt, "break"),
            Token::If => write!(fmt, "if"),
            Token::Do => write!(fmt, "do"),
            Token::While => write!(fmt, "while"),
            Token::Else => write!(fmt, "else"),
            Token::Return => write!(fmt, "return"),
            Token::Length => write!(fmt, "length"),
            Token::Int => write!(fmt, "int"),
            Token::Bool => write!(fmt, "bool"),
            Token::True => write!(fmt, "true"),
            Token::False => write!(fmt, "false"),
            Token::Assign => write!(fmt, "="),
            Token::Not => write!(fmt, "!"),
            Token::Mul => write!(fmt, "*"),
            Token::Hul => write!(fmt, "*>>"),
            Token::Div => write!(fmt, "/"),
            Token::Mod => write!(fmt, "%"),
            Token::Add => write!(fmt, "+"),
            Token::Sub => write!(fmt, "-"),
            Token::Le => write!(fmt, "<="),
            Token::Lt => write!(fmt, "<"),
            Token::Ge => write!(fmt, ">="),
            Token::Gt => write!(fmt, ">"),
            Token::Eq => write!(fmt, "=="),
            Token::Ne => write!(fmt, "!="),
            Token::And => write!(fmt, "&"),
            Token::Or => write!(fmt, "|"),
            Token::LParen => write!(fmt, "("),
            Token::RParen => write!(fmt, ")"),
            Token::LBrack => write!(fmt, "["),
            Token::RBrack => write!(fmt, "]"),
            Token::LBrace => write!(fmt, "{{"),
            Token::RBrace => write!(fmt, "}}"),
            Token::Colon => write!(fmt, ":"),
            Token::Semicolon => write!(fmt, ";"),
            Token::Comma => write!(fmt, ","),
            Token::Underscore => write!(fmt, "_"),
            Token::Period => write!(fmt, "."),
        }
    }
}

pub fn unescape_char(char: char) -> Option<&'static str> {
    match char {
        '\n' => Some("\\n"),
        '\r' => Some("\\r"),
        '\t' => Some("\\t"),
        '\x08' => Some("\\b"),
        '\x0C' => Some("\\f"),
        _ => None,
    }
}

pub fn unescape_str(string: &str) -> String {
    let mut buffer = String::new();
    for char in string.chars() {
        match unescape_char(char) {
            Some(unescaped) => buffer.push_str(unescaped),
            None => buffer.push(char),
        }
    }
    buffer
}
