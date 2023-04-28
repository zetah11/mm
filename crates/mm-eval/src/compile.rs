use typed_arena::Arena;

use crate::note::Note;
use crate::parse::Parser;
use crate::{check, implicit, melody, Error};

pub fn compile<'a, 'src, N: Note>(
    implicits: &'a Arena<implicit::Melody<'a, 'src, N>>,
    explicits: &'a Arena<melody::Melody<'a, 'src, N>>,
    source: &'src str,
) -> Result<melody::Program<'a, 'src, N>, Vec<Error<'src>>> {
    let parsed = match Parser::parse(implicits, source) {
        Ok(parsed) => parsed,
        Err(es) => return Err(es.into_iter().map(Into::into).collect()),
    };

    check::check(explicits, parsed).map_err(|err| err.into_iter().map(Into::into).collect())
}
