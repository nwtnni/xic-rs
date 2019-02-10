use crate::ast;
use crate::error;
use crate::parse;
use crate::span;
use crate::token;
use crate::util::Conv;

pub struct Driver<'main> {
    directory: &'main std::path::Path,
    diagnostic: bool,
}

type Spanned = Result<(span::Point, token::Token, span::Point), error::Error>;

impl<'main> Driver<'main> {
    pub fn new(directory: &'main std::path::Path, diagnostic: bool) -> Self {
        Driver { directory, diagnostic }
    }

    pub fn drive<I>(&self, iter: I) -> Result<char, error::Error> where I: IntoIterator<Item = Spanned>
    {
        parse::TestParser::new()
            .parse(iter)
            .map_err(Conv::conv::<parse::Error>)
            .map_err(Conv::conv::<error::Error>)
    }
}
