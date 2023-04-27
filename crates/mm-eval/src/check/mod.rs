mod build;
mod equation;
mod lower;
mod matrix;
mod solve;

use std::collections::{HashMap, HashSet};

use typed_arena::Arena;

use crate::dependency::dependencies;
use crate::{implicit, melody, topology, Length, Name};

use self::equation::{Equation, Variable};

#[derive(Debug)]
pub enum Error {
    UnknownName(String),
    // ...
}

pub fn check<'a>(
    arena: &'a Arena<melody::Melody<'a>>,
    program: &HashMap<Name, &implicit::Melody>,
) -> Result<HashMap<Name, &'a melody::Melody<'a>>, Vec<Error>> {
    let graph = dependencies(program);
    let mut checker = Checker::new(arena);

    let mut errors = Vec::new();
    for names in topology::order(&graph) {
        if let Err(e) = checker.check_component(program, names) {
            errors.push(e);
        }
    }

    if errors.is_empty() {
        Ok(checker.defs)
    } else {
        Err(errors)
    }
}

struct Checker<'a> {
    arena: &'a Arena<melody::Melody<'a>>,
    defs: HashMap<Name, &'a melody::Melody<'a>>,
    context: HashMap<Name, Variable>,
    lengths: HashMap<Variable, Length>,
    counter: usize,
}

impl<'a> Checker<'a> {
    pub fn new(arena: &'a Arena<melody::Melody<'a>>) -> Self {
        Self {
            arena,
            defs: HashMap::new(),
            context: HashMap::new(),
            lengths: HashMap::new(),
            counter: 0,
        }
    }

    pub fn check_component(
        &mut self,
        program: &HashMap<Name, &implicit::Melody>,
        names: HashSet<&Name>,
    ) -> Result<(), Error> {
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

            let melody = program
                .get(name)
                .ok_or_else(|| Error::UnknownName(name.0.clone()))?;

            let sums = self.build_equation(melody);

            equations.push(Equation { var, sums });
        }

        self.solve(equations);

        for name in names {
            let melody = program
                .get(name)
                .expect("unknown names are reported earlier");

            let melody = self.lower_melody(melody);
            let melody = self.arena.alloc(melody);
            debug_assert!(self.defs.insert(name.clone(), melody).is_none());
        }

        Ok(())
    }
}
