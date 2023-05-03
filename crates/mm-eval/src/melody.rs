use std::collections::HashMap;

use crate::span::Span;
use crate::{Factor, Length, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program<'a, N, Id> {
    pub defs: HashMap<Name, &'a Melody<'a, N, Id>>,
    pub spans: HashMap<Name, Span<Id>>,
    pub public: Vec<Name>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Melody<'a, N, Id> {
    pub node: Node<'a, N, Id>,
    pub span: Span<Id>,
    pub length: Length,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node<'a, N, Id> {
    Pause,
    Note(N),
    Name(Name),
    Recur(Name),
    Scale(Factor, &'a Melody<'a, N, Id>),
    Sharp(usize, &'a Melody<'a, N, Id>),
    Offset(isize, &'a Melody<'a, N, Id>),
    Sequence(&'a [Melody<'a, N, Id>]),
    Stack(&'a [Melody<'a, N, Id>]),
}
