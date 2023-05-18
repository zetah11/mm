mod build;
mod equation;
mod lower;
mod matrix;
mod solve;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use crate::dependency::dependencies;
use crate::note::Note;
use crate::span::Span;
use crate::{implicit, melody, topology, Allocator, Length, Name};

use self::equation::{Equation, Variable};

#[derive(Debug, Eq, PartialEq)]
pub enum Error<Id> {
    NoPublicNames(Span<Id>),
    UnknownName(Span<Id>, Name),
    UnboundedNotLast(Span<Id>),
    UnfoundedRecursion(Span<Id>),
}

pub fn check<N, Id, A>(
    alloc: &mut A,
    program: implicit::Program<N, Id, A>,
) -> Result<melody::Program<N, Id, A>, Vec<Error<Id>>>
where
    N: Note,
    Id: Clone + Eq,
    A: Allocator<implicit::Melody<N, Id, A>>,
    A: Allocator<melody::Melody<N, Id, A>>,
{
    let graph = dependencies::<N, Id, A>(&program.defs)?;
    let mut checker = Checker::new(alloc);

    for names in topology::order(&graph) {
        let mut span = None;
        for name in names.iter() {
            let name_span = program.spans.get(name).expect("all names have a span");
            if let Some(span) = &mut span {
                *span += name_span.clone();
            } else {
                span = Some(name_span.clone());
            }
        }

        let span = span.expect("components have at least one name");
        checker.check_component(&program.defs, names, span);
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

struct Checker<'a, N, Id, A: Allocator<melody::Melody<N, Id, A>>> {
    alloc: &'a mut A,
    defs: HashMap<Name, A::Holder>,
    context: HashMap<Name, Variable>,
    lengths: HashMap<Variable, Length>,
    counter: usize,

    errors: Vec<Error<Id>>,
}

impl<'a, N, Id, A> Checker<'a, N, Id, A>
where
    N: Note,
    Id: Clone + Eq,
    A: Allocator<implicit::Melody<N, Id, A>>,
    A: Allocator<melody::Melody<N, Id, A>>,
{
    pub fn new(alloc: &'a mut A) -> Self {
        Self {
            alloc,
            defs: HashMap::new(),
            context: HashMap::new(),
            lengths: HashMap::new(),
            counter: 0,

            errors: Vec::new(),
        }
    }

    pub fn check_component(
        &mut self,
        program: &HashMap<Name, <A as Allocator<implicit::Melody<N, Id, A>>>::Holder>,
        names: HashSet<&Name>,
        span: Span<Id>,
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

            let sums = self.build_equation(A::as_ref(melody));

            equations.push(Equation { var, sums });
        }

        self.solve(equations, span);

        for name in names.iter() {
            let Some(melody) = program.get(name) else { continue; };

            let melody = self.lower(&names, A::as_ref(melody));
            let melody = self.alloc.pack(melody);
            let prev = self.defs.insert(**name, melody);
            debug_assert!(prev.is_none());
        }
    }
}
