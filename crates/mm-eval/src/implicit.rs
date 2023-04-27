use crate::{Factor, Name};

#[derive(Debug)]
pub enum Melody<'a> {
    Pause,
    Note(char),
    Name(Name),
    Scale(Factor, &'a Melody<'a>),
    Sequence(&'a [Melody<'a>]),
    Stack(&'a [Melody<'a>]),
}
