use std::collections::HashMap;

use crate::span::Span;
use crate::{Factor, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program<'a, N, Id> {
    pub defs: HashMap<Name, &'a Melody<'a, N, Id>>,
    pub spans: HashMap<Name, Span<Id>>,
    pub public: Vec<Name>,
    pub source: Span<Id>,
}

impl<N, Id> Program<'_, N, Id> {
    pub fn new(source: Span<Id>) -> Self {
        Self {
            defs: HashMap::new(),
            spans: HashMap::new(),
            public: Vec::new(),
            source,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Melody<'a, N, Id> {
    Pause(Span<Id>),
    Note(Span<Id>, N),
    Name(Span<Id>, Name),
    Scale(Span<Id>, Factor, &'a Melody<'a, N, Id>),
    Sharp(Span<Id>, usize, &'a Melody<'a, N, Id>),
    Offset(Span<Id>, isize, &'a Melody<'a, N, Id>),
    Sequence(&'a [Melody<'a, N, Id>]),
    Stack(&'a [Melody<'a, N, Id>]),
}

impl<N, Id: Clone + Eq> Melody<'_, N, Id> {
    pub fn span(&self) -> Span<Id> {
        match self {
            Self::Pause(span) => span.clone(),
            Self::Note(span, _) => span.clone(),
            Self::Name(span, _) => span.clone(),
            Self::Scale(factor_span, _, inner) => factor_span.clone() + inner.span(),
            Self::Sharp(sharp_span, _, inner) => sharp_span.clone() + inner.span(),
            Self::Offset(offset_span, _, inner) => offset_span.clone() + inner.span(),

            Self::Sequence(melodies) => melodies
                .iter()
                .map(|melody| melody.span())
                .reduce(|a, b| a + b)
                .unwrap(),

            Self::Stack(melodies) => melodies
                .iter()
                .map(|melody| melody.span())
                .reduce(|a, b| a + b)
                .unwrap(),
        }
    }
}
