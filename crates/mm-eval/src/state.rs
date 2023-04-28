use std::collections::HashMap;

use typed_arena::Arena;

use crate::note::Note;
use crate::parse::Parser;
use crate::{check, implicit, melody, Name};

pub struct CompilerState<'a, N> {
    implicits: &'a Arena<implicit::Melody<'a, N>>,
    explicits: &'a Arena<melody::Melody<'a, N>>,
}

impl<'a, N: Note> CompilerState<'a, N> {
    pub fn new(
        implicits: &'a Arena<implicit::Melody<'a, N>>,
        explicits: &'a Arena<melody::Melody<'a, N>>,
    ) -> Self {
        Self {
            implicits,
            explicits,
        }
    }

    pub fn compile(
        &self,
        source: &str,
    ) -> Result<HashMap<Name, &'a melody::Melody<'a, N>>, Vec<check::Error>> {
        let parsed = Parser::parse(self.implicits, source);
        check::check(self.explicits, &parsed)
    }
}
