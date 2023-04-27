use crate::{Factor, Length, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Melody<'a> {
    pub node: Node<'a>,
    pub length: Length,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node<'a> {
    Pause,
    Note(char),
    Name(Name),
    Scale(Factor, &'a Melody<'a>),
    Sequence(&'a [Melody<'a>]),
    Stack(&'a [Melody<'a>]),
}
