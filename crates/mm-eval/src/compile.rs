use typed_arena::Arena;

use crate::note::Note;
use crate::parse::Parser;
use crate::{check, implicit, melody, Error, Names};

pub fn compile<'a, N: Note, Id: Clone + Eq>(
    names: &mut Names,
    implicits: &'a Arena<implicit::Melody<'a, N, Id>>,
    explicits: &'a Arena<melody::Melody<'a, N, Id>>,
    name: Id,
    source: &str,
) -> Result<melody::Program<'a, N, Id>, Vec<Error<Id>>> {
    let parsed = match Parser::parse(names, implicits, name, source) {
        Ok(parsed) => parsed,
        Err(es) => return Err(es.into_iter().map(Into::into).collect()),
    };

    check::check(explicits, parsed).map_err(|err| err.into_iter().map(Into::into).collect())
}
