pub mod check;
pub mod eval;
pub mod implicit;
pub mod melody;
pub mod note;
pub mod parse;
pub mod span;

pub use crate::compile::compile;
pub use crate::time::{Factor, Length, Time};

mod compile;
mod time;

mod dependency;
mod topology;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Name(pub String);

#[derive(Debug)]
pub enum Error<'src> {
    Parse(parse::Error<'src>),
    Check(check::Error<'src>),
}

impl<'src> From<check::Error<'src>> for Error<'src> {
    fn from(value: check::Error<'src>) -> Self {
        Self::Check(value)
    }
}

impl<'src> From<parse::Error<'src>> for Error<'src> {
    fn from(value: parse::Error<'src>) -> Self {
        Self::Parse(value)
    }
}
