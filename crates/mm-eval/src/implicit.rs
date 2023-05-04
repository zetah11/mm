use std::collections::HashMap;
use std::fmt;

use crate::span::Span;
use crate::{Allocator, Factor, Name};

#[derive(Eq)]
pub struct Program<N, Id, A: Allocator<Melody<N, Id, A>>> {
    pub defs: HashMap<Name, A::Holder>,
    pub spans: HashMap<Name, Span<Id>>,
    pub public: Vec<Name>,
    pub source: Span<Id>,
}

impl<N, Id, A: Allocator<Melody<N, Id, A>>> Program<N, Id, A> {
    pub fn new(source: Span<Id>) -> Self {
        Self {
            defs: HashMap::new(),
            spans: HashMap::new(),
            public: Vec::new(),
            source,
        }
    }
}

#[derive(Eq)]
pub enum Melody<N, Id, A: Allocator<Self>> {
    Pause(Span<Id>),
    Note(Span<Id>, N),
    Name(Span<Id>, Name),
    Scale(Span<Id>, Factor, A::Holder),
    Sharp(Span<Id>, usize, A::Holder),
    Offset(Span<Id>, isize, A::Holder),
    Sequence(A::Several),
    Stack(A::Several),
}

impl<N, Id: Clone + Eq, A: Allocator<Self>> Melody<N, Id, A> {
    pub fn span(&self) -> Span<Id> {
        match self {
            Self::Pause(span) => span.clone(),
            Self::Note(span, _) => span.clone(),
            Self::Name(span, _) => span.clone(),
            Self::Scale(factor_span, _, inner) => factor_span.clone() + A::as_ref(inner).span(),
            Self::Sharp(sharp_span, _, inner) => sharp_span.clone() + A::as_ref(inner).span(),
            Self::Offset(offset_span, _, inner) => offset_span.clone() + A::as_ref(inner).span(),

            Self::Sequence(melodies) => A::as_slice(melodies)
                .iter()
                .map(|melody| melody.span())
                .reduce(|a, b| a + b)
                .unwrap(),

            Self::Stack(melodies) => A::as_slice(melodies)
                .iter()
                .map(|melody| melody.span())
                .reduce(|a, b| a + b)
                .unwrap(),
        }
    }
}

impl<N: Eq, Id: Eq, A: Allocator<Melody<N, Id, A>>> PartialEq for Program<N, Id, A> {
    fn eq(&self, other: &Self) -> bool {
        for (k1, v1) in self.defs.iter() {
            if let Some(v2) = other.defs.get(k1) {
                if A::as_ref(v1) != A::as_ref(v2) {
                    return false;
                }
            } else {
                return false;
            }
        }

        for (k1, v1) in other.defs.iter() {
            if let Some(v2) = self.defs.get(k1) {
                if A::as_ref(v1) != A::as_ref(v2) {
                    return false;
                }
            } else {
                return false;
            }
        }

        self.public == other.public && self.source == other.source && self.spans == other.spans
    }
}

impl<N: Eq, Id: Eq, A: Allocator<Self>> PartialEq for Melody<N, Id, A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Melody::Pause(a), Melody::Pause(b)) => a == b,
            (Melody::Note(a, n), Melody::Note(b, m)) => a == b && n == m,
            (Melody::Name(a, n), Melody::Name(b, m)) => a == b && n == m,

            (Melody::Scale(a, f, i), Melody::Scale(b, g, j)) => {
                a == b && f == g && A::as_ref(i) == A::as_ref(j)
            }

            (Melody::Sharp(a, f, i), Melody::Sharp(b, g, j)) => {
                a == b && f == g && A::as_ref(i) == A::as_ref(j)
            }

            (Melody::Offset(a, f, i), Melody::Offset(b, g, j)) => {
                a == b && f == g && A::as_ref(i) == A::as_ref(j)
            }

            (Melody::Sequence(ns), Melody::Sequence(ms)) => A::as_slice(ns) == A::as_slice(ms),
            (Melody::Stack(ns), Melody::Stack(ms)) => A::as_slice(ns) == A::as_slice(ms),

            _ => false,
        }
    }
}

impl<N: fmt::Debug, Id: fmt::Debug, A: Allocator<Melody<N, Id, A>>> fmt::Debug
    for Program<N, Id, A>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Program {{ defs: ")?;

        f.debug_map()
            .entries(self.defs.iter().map(|(k, v)| (k, A::as_ref(v))))
            .finish()?;

        write!(
            f,
            ", public: {:?}, source: {:?}, spans: {:?} }}",
            self.public, self.source, self.spans
        )
    }
}

impl<N: fmt::Debug, Id: fmt::Debug, A: Allocator<Self>> fmt::Debug for Melody<N, Id, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Melody::Pause(span) => write!(f, "Pause({span:?})"),
            Melody::Note(span, note) => write!(f, "Note({span:?}, {note:?})"),
            Melody::Name(span, name) => write!(f, "Name({span:?}, {name:?})"),

            Melody::Scale(span, factor, inner) => {
                write!(f, "Scale({span:?}, {factor:?}, {:?})", A::as_ref(inner))
            }

            Melody::Sharp(span, by, inner) => {
                write!(f, "Sharp({span:?}, {by:?}, {:?})", A::as_ref(inner))
            }

            Melody::Offset(span, by, inner) => {
                write!(f, "Offset({span:?}, {by:?}, {:?})", A::as_ref(inner))
            }

            Melody::Sequence(melodies) => {
                write!(f, "Sequence")?;
                f.debug_list().entries(A::as_slice(melodies)).finish()
            }

            Melody::Stack(melodies) => {
                write!(f, "Stack")?;
                f.debug_list().entries(A::as_slice(melodies)).finish()
            }
        }
    }
}
