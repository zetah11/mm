use std::collections::HashMap;

use crate::span::Span;
use crate::{Factor, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program<'a, 'src, N, Id> {
    pub defs: HashMap<Name<'src>, &'a Melody<'a, 'src, N, Id>>,
    pub spans: HashMap<Name<'src>, Span<Id>>,
    pub public: Vec<Name<'src>>,
    pub source: Span<Id>,
}

impl<'src, N, Id> Program<'_, 'src, N, Id> {
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
pub enum Melody<'a, 'src, N, Id> {
    Pause(Span<Id>),
    Note(Span<Id>, N),
    Name(Span<Id>, Name<'src>),
    Scale(Span<Id>, Factor, &'a Melody<'a, 'src, N, Id>),
    Sharp(Span<Id>, usize, &'a Melody<'a, 'src, N, Id>),
    Offset(Span<Id>, isize, &'a Melody<'a, 'src, N, Id>),
    Sequence(&'a [Melody<'a, 'src, N, Id>]),
    Stack(&'a [Melody<'a, 'src, N, Id>]),
}

impl<'src, N, Id: Clone + Eq> Melody<'_, 'src, N, Id> {
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
