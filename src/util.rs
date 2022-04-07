pub mod sexp;
pub mod span;
pub mod symbol;
mod traits;

pub use traits::Conv;
pub use traits::TakeUntil;
pub use traits::Tap;

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
