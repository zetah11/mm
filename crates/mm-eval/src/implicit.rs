use crate::{Factor, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Melody<'a> {
    Pause,
    Note(char),
    Name(Name),
    Scale(Factor, &'a Melody<'a>),
    Sequence(&'a [Melody<'a>]),
    Stack(&'a [Melody<'a>]),
}
