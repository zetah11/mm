use std::collections::HashMap;

use typed_arena::Arena;

use crate::note::Note;
use crate::parse::Parser;
use crate::{check, implicit, melody, Error, Name};

pub fn compile<'a, 'src, N: Note>(
    implicits: &'a Arena<implicit::Melody<'a, 'src, N>>,
    explicits: &'a Arena<melody::Melody<'a, 'src, N>>,
    source: &'src str,
) -> Result<HashMap<Name, &'a melody::Melody<'a, 'src, N>>, Vec<Error>> {
    let parsed = Parser::parse(implicits, source);
    check::check(explicits, &parsed).map_err(|err| err.into_iter().map(Into::into).collect())
}
