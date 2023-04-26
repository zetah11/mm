use rational::Rational;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Name(pub String);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Factor(pub Rational);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Length(pub Rational);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Time(pub Rational);

#[derive(Clone, Debug)]
pub struct Melody<'a> {
    pub node: Node<'a>,
    pub length: Length,
}

#[derive(Clone, Debug)]
pub enum Node<'a> {
    Pause,
    Note(char),
    Name(Name),
    Scale(Factor, &'a Melody<'a>),
    Sequence(&'a [Melody<'a>]),
    Stack(&'a [Melody<'a>]),
}
