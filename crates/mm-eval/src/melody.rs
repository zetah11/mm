use crate::{Factor, Length, Name};

#[derive(Clone, Debug)]
pub struct Melody<'a> {
    pub node: Node<'a>,
    pub length: Length,
}

#[derive(Clone, Debug)]
pub enum Node<'a> {
    Pause,
    Note(char),
    Name(Name),
    Scale(Factor, &'a Melody<'a>),
    Sequence(&'a [Melody<'a>]),
    Stack(&'a [Melody<'a>]),
}
