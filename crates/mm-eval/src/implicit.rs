use crate::{Factor, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Melody<'a, N> {
    Pause,
    Note(N),
    Name(Name),
    Scale(Factor, &'a Melody<'a, N>),
    Sequence(&'a [Melody<'a, N>]),
    Stack(&'a [Melody<'a, N>]),
}
