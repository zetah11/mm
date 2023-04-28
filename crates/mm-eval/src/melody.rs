use crate::span::Span;
use crate::{Factor, Length, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Melody<'a, 'src, N> {
    pub node: Node<'a, 'src, N>,
    pub span: Span<'src>,
    pub length: Length,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node<'a, 'src, N> {
    Pause,
    Note(N),
    Name(Name),
    Scale(Factor, &'a Melody<'a, 'src, N>),
    Sequence(&'a [Melody<'a, 'src, N>]),
    Stack(&'a [Melody<'a, 'src, N>]),
}
