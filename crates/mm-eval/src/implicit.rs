use std::collections::HashMap;

use crate::span::Span;
use crate::{Factor, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program<'a, 'src, N> {
    pub defs: HashMap<Name<'src>, &'a Melody<'a, 'src, N>>,
    pub spans: HashMap<Name<'src>, Span<'src>>,
    pub public: Vec<Name<'src>>,
    pub source: Span<'src>,
}

impl<'src, N> Program<'_, 'src, N> {
    pub fn new(source: Span<'src>) -> Self {
        Self {
            defs: HashMap::new(),
            spans: HashMap::new(),
            public: Vec::new(),
            source,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Melody<'a, 'src, N> {
    Pause(Span<'src>),
    Note(Span<'src>, N),
    Name(Span<'src>, Name<'src>),
    Scale(Span<'src>, Factor, &'a Melody<'a, 'src, N>),
    Sharp(Span<'src>, usize, &'a Melody<'a, 'src, N>),
    Offset(Span<'src>, isize, &'a Melody<'a, 'src, N>),
    Sequence(&'a [Melody<'a, 'src, N>]),
    Stack(&'a [Melody<'a, 'src, N>]),
}

impl<'src, N> Melody<'_, 'src, N> {
    pub fn span(&self) -> Span<'src> {
        match self {
            Self::Pause(span) => *span,
            Self::Note(span, _) => *span,
            Self::Name(span, _) => *span,
            Self::Scale(factor_span, _, inner) => *factor_span + inner.span(),
            Self::Sharp(sharp_span, _, inner) => *sharp_span + inner.span(),
            Self::Offset(offset_span, _, inner) => *offset_span + inner.span(),

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
