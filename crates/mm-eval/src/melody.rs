use std::collections::HashMap;

use crate::span::Span;
use crate::{Factor, Length, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program<'a, 'src, N> {
    pub defs: HashMap<Name<'src>, &'a Melody<'a, 'src, N>>,
    pub spans: HashMap<Name<'src>, Span<'src>>,
}

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
    Name(Name<'src>),
    Scale(Factor, &'a Melody<'a, 'src, N>),
    Sharp(usize, &'a Melody<'a, 'src, N>),
    Offset(isize, &'a Melody<'a, 'src, N>),
    Sequence(&'a [Melody<'a, 'src, N>]),
    Stack(&'a [Melody<'a, 'src, N>]),
}
