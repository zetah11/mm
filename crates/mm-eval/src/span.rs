use std::fmt;
use std::ops::{Add, Range};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Span<Id> {
    pub source: Id,
    pub start: usize,
    pub end: usize,
}

impl<Id> Span<Id> {
    pub fn new(source: Id, range: Range<usize>) -> Self {
        Self {
            source,
            start: range.start,
            end: range.end,
        }
    }
}

impl<Id: Clone + Eq> Add for Span<Id> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        debug_assert!(self.source == rhs.source);

        Self {
            source: self.source.clone(),
            start: self.start.min(rhs.start),
            end: self.end.max(rhs.end),
        }
    }
}

impl<Id: fmt::Debug> fmt::Debug for Span<Id> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = self.start;
        let end = self.end;
        write!(f, "[{start}..{end}]@{:?}", self.source)
    }
}

/// Create an empty span.
#[cfg(test)]
pub(crate) fn span() -> Span<&'static str> {
    Span {
        source: "",
        start: 0,
        end: 0,
    }
}

/// Create a closure which creates a span from the given indicies in the given
/// source.
#[cfg(test)]
pub(crate) fn span_in<'src>(source: &'src str) -> impl Fn(usize, usize) -> Span<&'src str> {
    |start, end| Span { source, start, end }
}
