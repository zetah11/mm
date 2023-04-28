use std::collections::HashMap;

use crate::span::Span;
use crate::{Factor, Name};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Program<'a, 'src, N> {
    pub defs: HashMap<Name, &'a Melody<'a, 'src, N>>,
    pub spans: HashMap<Name, Span<'src>>,
}

impl<N> Program<'_, '_, N> {
    pub fn new() -> Self {
        Self {
            defs: HashMap::new(),
            spans: HashMap::new(),
        }
    }
}

impl<N> Default for Program<'_, '_, N> {
    fn default() -> Self {
        Self {
            defs: HashMap::default(),
            spans: HashMap::default(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Melody<'a, 'src, N> {
    Pause(Span<'src>),
    Note(Span<'src>, N),
    Name(Span<'src>, Name),
    Scale(Span<'src>, Factor, &'a Melody<'a, 'src, N>),
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
