use typed_arena::Arena;

use crate::note::Note;
use crate::parse::Parser;
use crate::{check, implicit, melody, Error};

pub fn compile<'a, 'src, N: Note, Id: Clone + Eq>(
    implicits: &'a Arena<implicit::Melody<'a, 'src, N, Id>>,
    explicits: &'a Arena<melody::Melody<'a, 'src, N, Id>>,
    name: Id,
    source: &'src str,
) -> Result<melody::Program<'a, 'src, N, Id>, Vec<Error<'src, Id>>> {
    let parsed = match Parser::parse(implicits, name, source) {
        Ok(parsed) => parsed,
        Err(es) => return Err(es.into_iter().map(Into::into).collect()),
    };

    check::check(explicits, parsed).map_err(|err| err.into_iter().map(Into::into).collect())
}
