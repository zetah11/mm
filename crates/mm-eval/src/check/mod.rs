mod build;
mod equation;
mod lower;
mod matrix;
mod solve;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use typed_arena::Arena;

use crate::dependency::dependencies;
use crate::note::Note;
use crate::span::Span;
use crate::{implicit, melody, topology, Length, Name};

use self::equation::{Equation, Variable};

#[derive(Debug, Eq, PartialEq)]
pub enum Error<Id> {
    NoPublicNames(Span<Id>),
    UnknownName(Span<Id>, Name),
    UnboundedNotLast(Span<Id>),
}

pub fn check<'a, N: Note, Id: Clone + Eq>(
    arena: &'a Arena<melody::Melody<'a, N, Id>>,
    program: implicit::Program<'_, N, Id>,
) -> Result<melody::Program<'a, N, Id>, Vec<Error<Id>>> {
    let graph = dependencies(&program.defs)?;
    let mut checker = Checker::new(arena);

    for names in topology::order(&graph) {
        checker.check_component(&program.defs, names);
    }

    if program.public.is_empty() {
        checker.errors.push(Error::NoPublicNames(program.source));
    }

    if checker.errors.is_empty() {
        Ok(melody::Program {
            defs: checker.defs,
            spans: program.spans,
            public: program.public,
        })
    } else {
        Err(checker.errors)
    }
}

struct Checker<'a, N, Id> {
    arena: &'a Arena<melody::Melody<'a, N, Id>>,
    defs: HashMap<Name, &'a melody::Melody<'a, N, Id>>,
    context: HashMap<Name, Variable>,
    lengths: HashMap<Variable, Length>,
    counter: usize,

    errors: Vec<Error<Id>>,
}

impl<'a, N: Note, Id: Clone + Eq> Checker<'a, N, Id> {
    pub fn new(arena: &'a Arena<melody::Melody<'a, N, Id>>) -> Self {
        Self {
            arena,
            defs: HashMap::new(),
            context: HashMap::new(),
            lengths: HashMap::new(),
            counter: 0,

            errors: Vec::new(),
        }
    }

    pub fn check_component(
        &mut self,
        program: &HashMap<Name, &implicit::Melody<'_, N, Id>>,
        names: HashSet<&Name>,
    ) {
        for name in names.iter() {
            let var = self.fresh();
            self.context.insert(Name::clone(name), var);
        }

        let mut equations = Vec::with_capacity(names.len());
        for name in names.iter() {
            let var = *self
                .context
                .get(name)
                .expect("all names are bound a variable");

            let Some(melody) = program.get(name) else {
                continue;
            };

            let sums = self.build_equation(melody);

            equations.push(Equation { var, sums });
        }

        self.solve(equations);

        for name in names.iter() {
            let Some(melody) = program.get(name) else { continue; };

            let melody = self.lower(&names, melody);
            let melody = self.arena.alloc(melody);
            let prev = self.defs.insert(**name, melody);
            debug_assert!(prev.is_none());
        }
    }
}
