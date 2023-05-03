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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Name<'src>(pub &'src str);

#[derive(Debug)]
pub enum Error<'src, Id> {
    Parse(parse::Error<Id>),
    Check(check::Error<'src, Id>),
}

impl<'src, Id> From<check::Error<'src, Id>> for Error<'src, Id> {
    fn from(value: check::Error<'src, Id>) -> Self {
        Self::Check(value)
    }
}

impl<Id> From<parse::Error<Id>> for Error<'_, Id> {
    fn from(value: parse::Error<Id>) -> Self {
        Self::Parse(value)
    }
}
