use crate::{melody, Allocator, Factor, Length};

use super::Checker;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Variable(usize);

/// Equates a [`Variable`] with the maximum value of one or more [`Sum`s](Sum).
pub struct Equation {
    pub var: Variable,
    pub sums: Vec<Sum>,
}

/// A linear sum of [`Term`s](Term), where each term is a constant or a
/// variable scaled by some constant factor.
pub struct Sum {
    pub terms: Vec<Term>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Term {
    Constant(Length),
    Variable(Factor, Variable),
}

impl<N, Id, A: Allocator<melody::Melody<N, Id, A>>> Checker<'_, N, Id, A> {
    /// Create a fresh and unique length variable.
    pub fn fresh(&mut self) -> Variable {
        let var = Variable(self.counter);
        self.counter += 1;
        var
    }
}
