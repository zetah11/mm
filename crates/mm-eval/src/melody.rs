use crate::{Factor, Length, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Melody<'a, N> {
    pub node: Node<'a, N>,
    pub length: Length,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node<'a, N> {
    Pause,
    Note(N),
    Name(Name),
    Scale(Factor, &'a Melody<'a, N>),
    Sequence(&'a [Melody<'a, N>]),
    Stack(&'a [Melody<'a, N>]),
}
