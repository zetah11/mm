use crate::note::Note;
use crate::parse::Parser;
use crate::{check, implicit, melody, Allocator, Error, Names};

pub fn compile<N, Id, A>(
    alloc: &mut A,
    names: &mut Names,
    name: Id,
    source: &str,
) -> Result<melody::Program<N, Id, A>, Vec<Error<Id>>>
where
    N: Note,
    Id: Clone + Eq,
    A: Allocator<implicit::Melody<N, Id, A>>,
    A: Allocator<melody::Melody<N, Id, A>>,
{
    let parsed = match Parser::parse(alloc, names, name, source) {
        Ok(parsed) => parsed,
        Err(es) => return Err(es.into_iter().map(Into::into).collect()),
    };

    check::check(alloc, parsed).map_err(|err| err.into_iter().map(Into::into).collect())
}
