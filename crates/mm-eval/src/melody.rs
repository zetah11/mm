use std::collections::HashMap;
use std::fmt;

use crate::span::Span;
use crate::{Allocator, Factor, Length, Name};

pub struct Program<N, Id, A: Allocator<Melody<N, Id, A>>> {
    pub defs: HashMap<Name, A::Holder>,
    pub spans: HashMap<Name, Span<Id>>,
    pub public: Vec<Name>,
}

pub struct Melody<N, Id, A: Allocator<Self>> {
    pub node: Node<N, Id, A>,
    pub span: Span<Id>,
    pub length: Length,
}

pub enum Node<N, Id, A: Allocator<Melody<N, Id, A>>> {
    Pause,
    Note(N),
    Name(Name),
    Recur(Name),
    Scale(Factor, A::Holder),
    Sharp(usize, A::Holder),
    Offset(isize, A::Holder),
    Sequence(A::Several),
    Stack(A::Several),
}

impl<N: PartialEq, Id: PartialEq, A: Allocator<Melody<N, Id, A>>> PartialEq for Program<N, Id, A> {
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

        self.public == other.public && self.spans == other.spans
    }
}

impl<N: PartialEq, Id: PartialEq, A: Allocator<Self>> PartialEq for Melody<N, Id, A> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node && self.span == other.span && self.length == other.length
    }
}

impl<N: PartialEq, Id: PartialEq, A: Allocator<Melody<N, Id, A>>> PartialEq for Node<N, Id, A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Pause, Self::Pause) => true,
            (Self::Note(n), Self::Note(m)) => n == m,
            (Self::Name(n), Self::Name(m)) => n == m,
            (Self::Recur(n), Self::Recur(m)) => n == m,
            (Self::Scale(n, i), Self::Scale(m, j)) => n == m && A::as_ref(i) == A::as_ref(j),
            (Self::Sharp(n, i), Self::Sharp(m, j)) => n == m && A::as_ref(i) == A::as_ref(j),
            (Self::Offset(n, i), Self::Offset(m, j)) => n == m && A::as_ref(i) == A::as_ref(j),
            (Self::Sequence(ns), Self::Sequence(ms)) => A::as_slice(ns) == A::as_slice(ms),
            (Self::Stack(ns), Self::Stack(ms)) => A::as_slice(ns) == A::as_slice(ms),

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

        write!(f, ", public: {:?}, spans: {:?} }}", self.public, self.spans)
    }
}

impl<N: fmt::Debug, Id: fmt::Debug, A: Allocator<Self>> fmt::Debug for Melody<N, Id, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Melody")
            .field("node", &self.node)
            .field("length", &self.length)
            .field("span", &self.span)
            .finish()
    }
}

impl<N: fmt::Debug, Id: fmt::Debug, A: Allocator<Melody<N, Id, A>>> fmt::Debug for Node<N, Id, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pause => write!(f, "Pause"),
            Self::Note(n) => write!(f, "Note({n:?})"),
            Self::Name(n) => write!(f, "Name({n:?})"),
            Self::Recur(n) => write!(f, "Recur({n:?})"),
            Self::Scale(factor, n) => write!(f, "Scale({factor:?}, {:?})", A::as_ref(n)),
            Self::Sharp(by, n) => write!(f, "Sharp({by:?}, {:?})", A::as_ref(n)),
            Self::Offset(by, n) => write!(f, "Offset({by:?}, {:?})", A::as_ref(n)),
            Self::Sequence(ns) => {
                write!(f, "Sequence")?;
                f.debug_list().entries(A::as_slice(ns)).finish()
            }
            Self::Stack(ns) => {
                write!(f, "Stack")?;
                f.debug_list().entries(A::as_slice(ns)).finish()
            }
        }
    }
}
