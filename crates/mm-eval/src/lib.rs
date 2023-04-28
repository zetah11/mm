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
pub enum Error {
    Check(check::Error),
}

impl From<check::Error> for Error {
    fn from(value: check::Error) -> Self {
        Self::Check(value)
    }
}
