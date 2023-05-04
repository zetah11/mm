pub mod check;
pub mod eval;
pub mod implicit;
pub mod melody;
pub mod names;
pub mod note;
pub mod parse;
pub mod span;

pub use crate::alloc::{Allocator, Arena, Heap};
pub use crate::compile::compile;
pub use crate::names::{Name, Names};
pub use crate::time::{Factor, Length, Time};

mod alloc;
mod compile;
mod time;

mod dependency;
mod topology;

#[derive(Debug)]
pub enum Error<Id> {
    Parse(parse::Error<Id>),
    Check(check::Error<Id>),
}

impl<Id> From<check::Error<Id>> for Error<Id> {
    fn from(value: check::Error<Id>) -> Self {
        Self::Check(value)
    }
}

impl<Id> From<parse::Error<Id>> for Error<Id> {
    fn from(value: parse::Error<Id>) -> Self {
        Self::Parse(value)
    }
}
