use crate::symbol;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    IDENT(symbol::Symbol),
    CHARACTER(char),
    INTEGER(String),
    STRING(String),
    USE, 
    IF,
    WHILE,
    ELSE,
    RETURN,
    LENGTH,
    INT,
    BOOL,
    TRUE,
    FALSE,
    ASSIGN,
    NOT,
    MUL,
    HMUL,
    DIV,
    MOD,
    ADD,
    SUB,
    LE,
    LT,
    GE,
    GT,
    EQ,
    NEQ,
    LAND,
    LOR,
    LPAREN,
    RPAREN,
    LBRACK,
    RBRACK,
    LBRACE,
    RBRACE,
    COLON,
    SEMICOLON,
    COMMA,
    UNDERSCORE,
}

fn unescape(c: char, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    match c {
    | '\n'   => write!(fmt, "\\n"),
    | '\r'   => write!(fmt, "\\r"),
    | '\t'   => write!(fmt, "\\t"),
    | '\x08' => write!(fmt, "\\b"),
    | '\x0C' => write!(fmt, "\\f"),
    | _      => write!(fmt, "{}", c),
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        | Token::CHARACTER(c) => {
            write!(fmt, "character ")?;
            unescape(*c, fmt)
        }
        | Token::STRING(s) => {
            write!(fmt, "string ")?;
            for c in s.chars() { unescape(c, fmt)?; }
            Ok(())
        },
        | Token::IDENT(i)   => write!(fmt, "id {}", symbol::resolve(*i)),
        | Token::INTEGER(i) => write!(fmt, "integer {}", i),
        | Token::USE        => write!(fmt, "use"),
        | Token::IF         => write!(fmt, "if"),
        | Token::WHILE      => write!(fmt, "while"),
        | Token::ELSE       => write!(fmt, "else"),
        | Token::RETURN     => write!(fmt, "return"),
        | Token::LENGTH     => write!(fmt, "length"),
        | Token::INT        => write!(fmt, "int"),
        | Token::BOOL       => write!(fmt, "bool"),
        | Token::TRUE       => write!(fmt, "true"),
        | Token::FALSE      => write!(fmt, "false"),
        | Token::ASSIGN     => write!(fmt, "="),
        | Token::NOT        => write!(fmt, "!"),
        | Token::MUL        => write!(fmt, "*"),
        | Token::HMUL       => write!(fmt, "*>>"),
        | Token::DIV        => write!(fmt, "/"),
        | Token::MOD        => write!(fmt, "%"),
        | Token::ADD        => write!(fmt, "+"),
        | Token::SUB        => write!(fmt, "-"),
        | Token::LE         => write!(fmt, "<="),
        | Token::LT         => write!(fmt, "<"),
        | Token::GE         => write!(fmt, ">="),
        | Token::GT         => write!(fmt, ">"),
        | Token::EQ         => write!(fmt, "=="),
        | Token::NEQ        => write!(fmt, "!="),
        | Token::LAND       => write!(fmt, "&"),
        | Token::LOR        => write!(fmt, "|"),
        | Token::LPAREN     => write!(fmt, "("),
        | Token::RPAREN     => write!(fmt, ")"),
        | Token::LBRACK     => write!(fmt, "{{"),
        | Token::RBRACK     => write!(fmt, "}}"),
        | Token::LBRACE     => write!(fmt, "["),
        | Token::RBRACE     => write!(fmt, "]"),
        | Token::COLON      => write!(fmt, ":"),
        | Token::SEMICOLON  => write!(fmt, ";"),
        | Token::COMMA      => write!(fmt, ","),
        | Token::UNDERSCORE => write!(fmt, "_"),
        }
    }
}
