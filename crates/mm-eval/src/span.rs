use std::fmt;
use std::ops::{Add, Range};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Span<'src> {
    pub source: &'src str,
    pub start: usize,
    pub end: usize,
}

impl<'src> Span<'src> {
    pub fn new(source: &'src str, range: Range<usize>) -> Self {
        Self {
            source,
            start: range.start,
            end: range.end,
        }
    }
}

impl Add for Span<'_> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        debug_assert!(std::ptr::eq(self.source, rhs.source));

        Self {
            source: self.source,
            start: self.start.min(rhs.start),
            end: self.end.max(rhs.end),
        }
    }
}

impl fmt::Debug for Span<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.source as *const str as *const () as usize;
        let start = self.start;
        let end = self.end;
        write!(f, "{s:x}[{start}..{end}]")
    }
}

/// Create an empty span.
#[cfg(test)]
pub(crate) fn span() -> Span<'static> {
    Span {
        source: "",
        start: 0,
        end: 0,
    }
}

/// Create a closure which creates a span from the given indicies in the given
/// source.
#[cfg(test)]
pub(crate) fn span_in<'src>(source: &'src str) -> impl Fn(usize, usize) -> Span<'src> {
    |start, end| Span { source, start, end }
}
