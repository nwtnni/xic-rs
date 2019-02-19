pub mod sexp;
pub mod span;
pub mod symbol;
mod traits;

pub use traits::{Conv, Tap, TakeUntil};

pub fn unescape_char(c: char) -> Option<&'static str> {
    match c {
    | '\n'   => Some("\\n"),
    | '\r'   => Some("\\r"),
    | '\t'   => Some("\\t"),
    | '\x08' => Some("\\b"),
    | '\x0C' => Some("\\f"),
    | _      => None,
    }
}

pub fn unescape_str(s: &str) -> String {
    let mut buffer = String::new();
    for c in s.chars() {
        match unescape_char(c) {
        | Some(s) => buffer.push_str(s),
        | None => buffer.push(c),
        }
    }
    buffer
}
