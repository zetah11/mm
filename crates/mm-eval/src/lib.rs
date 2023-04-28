pub mod check;
pub mod eval;
pub mod implicit;
pub mod melody;
pub mod note;
pub mod parse;

pub use crate::state::CompilerState;
pub use crate::time::{Factor, Length, Time};

mod state;
mod time;

mod dependency;
mod topology;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Name(pub String);
