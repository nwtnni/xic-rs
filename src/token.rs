use std::io::Write;

use crate::symbol;
use crate::util;

/// Represents a possible lexical token in the Xi language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    /// Identifier
    IDENT(symbol::Symbol),

    /// Character literal
    CHARACTER(char),

    /// Integer literal
    INTEGER(String),

    /// String literal
    STRING(String),

    /// `use` keyword
    USE, 

    /// `if` keyword
    IF,

    /// `while` keyword
    WHILE,

    /// `else` keyword
    ELSE,

    /// `return` keyword
    RETURN,

    /// `int` keyword
    INT,

    /// `bool` keyword
    BOOL,

    /// `true` keyword
    TRUE,

    /// `false` keyword
    FALSE,

    /// `=` symbol
    ASSIGN,

    /// `!` symbol
    NOT,

    /// `*` symbol
    MUL,

    /// `*>>` symbol
    HUL,

    /// `/` symbol
    DIV,

    /// `%` symbol
    MOD,

    /// `+` symbol
    ADD,

    /// `-` symbol
    SUB,

    /// `<` symbol
    LT,

    /// `<=` symbol
    LE,

    /// `>=` symbol
    GE,

    /// `>` symbol
    GT,

    /// `==` symbol
    EQ,

    /// `!=` symbol
    NE,

    /// `&` symbol
    AND,

    /// `|` symbol
    OR,

    /// `(` symbol
    LPAREN,

    /// `)` symbol
    RPAREN,

    /// `[` symbol
    LBRACK,

    /// `]` symbol
    RBRACK,

    /// `{` symbol
    LBRACE,

    /// `}` symbol
    RBRACE,

    /// `:` symbol
    COLON,

    /// `;` symbol
    SEMICOLON,

    /// `,` symbol
    COMMA,

    /// `_` symbol
    UNDERSCORE,
}

impl std::fmt::Display for Token {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        | Token::CHARACTER(c) => {
            match util::unescape_char(*c) {
            | Some(s) => write!(fmt, "character {}", s),
            | None => write!(fmt, "character {}", c),
            }
        }
        | Token::STRING(s)  => write!(fmt, "string {}", util::unescape_str(s)),
        | Token::IDENT(i)   => write!(fmt, "id {}", symbol::resolve(*i)),
        | Token::INTEGER(i) => write!(fmt, "integer {}", i),
        | Token::USE        => write!(fmt, "use"),
        | Token::IF         => write!(fmt, "if"),
        | Token::WHILE      => write!(fmt, "while"),
        | Token::ELSE       => write!(fmt, "else"),
        | Token::RETURN     => write!(fmt, "return"),
        | Token::INT        => write!(fmt, "int"),
        | Token::BOOL       => write!(fmt, "bool"),
        | Token::TRUE       => write!(fmt, "true"),
        | Token::FALSE      => write!(fmt, "false"),
        | Token::ASSIGN     => write!(fmt, "="),
        | Token::NOT        => write!(fmt, "!"),
        | Token::MUL        => write!(fmt, "*"),
        | Token::HUL        => write!(fmt, "*>>"),
        | Token::DIV        => write!(fmt, "/"),
        | Token::MOD        => write!(fmt, "%"),
        | Token::ADD        => write!(fmt, "+"),
        | Token::SUB        => write!(fmt, "-"),
        | Token::LE         => write!(fmt, "<="),
        | Token::LT         => write!(fmt, "<"),
        | Token::GE         => write!(fmt, ">="),
        | Token::GT         => write!(fmt, ">"),
        | Token::EQ         => write!(fmt, "=="),
        | Token::NE         => write!(fmt, "!="),
        | Token::AND        => write!(fmt, "&"),
        | Token::OR         => write!(fmt, "|"),
        | Token::LPAREN     => write!(fmt, "("),
        | Token::RPAREN     => write!(fmt, ")"),
        | Token::LBRACK     => write!(fmt, "["),
        | Token::RBRACK     => write!(fmt, "]"),
        | Token::LBRACE     => write!(fmt, "{{"),
        | Token::RBRACE     => write!(fmt, "}}"),
        | Token::COLON      => write!(fmt, ":"),
        | Token::SEMICOLON  => write!(fmt, ";"),
        | Token::COMMA      => write!(fmt, ","),
        | Token::UNDERSCORE => write!(fmt, "_"),
        }
    }
}
