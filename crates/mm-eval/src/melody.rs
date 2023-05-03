use std::collections::HashMap;

use crate::span::Span;
use crate::{Factor, Length, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program<'a, 'src, N, Id> {
    pub defs: HashMap<Name<'src>, &'a Melody<'a, 'src, N, Id>>,
    pub spans: HashMap<Name<'src>, Span<Id>>,
    pub public: Vec<Name<'src>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Melody<'a, 'src, N, Id> {
    pub node: Node<'a, 'src, N, Id>,
    pub span: Span<Id>,
    pub length: Length,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node<'a, 'src, N, Id> {
    Pause,
    Note(N),
    Name(Name<'src>),
    Recur(Name<'src>),
    Scale(Factor, &'a Melody<'a, 'src, N, Id>),
    Sharp(usize, &'a Melody<'a, 'src, N, Id>),
    Offset(isize, &'a Melody<'a, 'src, N, Id>),
    Sequence(&'a [Melody<'a, 'src, N, Id>]),
    Stack(&'a [Melody<'a, 'src, N, Id>]),
}
