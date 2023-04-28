use std::collections::HashMap;

use rational::extras::r;

use crate::melody::{Melody, Node};
use crate::span::{span, Span};
use crate::{Factor, Length, Name, Time};

use super::Evaluator;

fn check(expected: Vec<(char, Span, Time, Length)>, program: HashMap<Name, &Melody<char>>) {
    let eval = Evaluator::new(program, Name("it".into()));
    let actual: Vec<_> = eval.iter().collect();
    assert_eq!(expected, actual);
}

#[test]
fn simple_sequence() {
    let span = span();

    let a = Melody {
        node: Node::Note('a'),
        length: Length::one(),
        span,
    };

    let b = Melody {
        node: Node::Pause,
        length: Length::one(),
        span,
    };

    let c = Melody {
        node: Node::Note('c'),
        length: Length::one(),
        span,
    };

    let melody = [a, b, c];
    let melody = Melody {
        node: Node::Sequence(&melody),
        length: Length::Bounded(r(3, 1)),
        span,
    };

    let program = HashMap::from([(Name("it".into()), &melody)]);

    let expected = vec![
        ('a', span, Time(r(0, 1)), Length::one()),
        ('c', span, Time(r(2, 1)), Length::one()),
    ];

    check(expected, program);
}

#[test]
fn simple_stack() {
    let span = span();

    let a = Melody {
        node: Node::Note('a'),
        length: Length::one(),
        span,
    };

    let b = Melody {
        node: Node::Pause,
        length: Length::one(),
        span,
    };

    let c = Melody {
        node: Node::Note('c'),
        length: Length::one(),
        span,
    };

    let melody = [a, b, c];
    let melody = Melody {
        node: Node::Stack(&melody),
        length: Length::one(),
        span,
    };

    let program = HashMap::from([(Name("it".into()), &melody)]);

    let expected = vec![
        ('a', span, Time::zero(), Length::one()),
        ('c', span, Time::zero(), Length::one()),
    ];

    check(expected, program);
}

#[test]
fn unending_stack() {
    let span = span();

    let a = Melody {
        node: Node::Note('a'),
        length: Length::one(),
        span,
    };

    let b = Melody {
        node: Node::Note('b'),
        length: Length::one(),
        span,
    };

    let to_bot = Melody {
        node: Node::Name(Name("bot".into())),
        length: Length::Unbounded,
        span,
    };

    let c = Melody {
        node: Node::Note('c'),
        length: Length::one(),
        span,
    };

    let d = Melody {
        node: Node::Note('d'),
        length: Length::one(),
        span,
    };

    let to_top = Melody {
        node: Node::Name(Name("top".into())),
        length: Length::Unbounded,
        span,
    };

    let bot = [a, b, to_bot.clone()];
    let bot = Melody {
        node: Node::Sequence(&bot),
        length: Length::Unbounded,
        span,
    };

    let top = [c, d, to_top.clone()];
    let top = Melody {
        node: Node::Sequence(&top),
        length: Length::Unbounded,
        span,
    };

    let stack = [to_bot, to_top];
    let stack = Melody {
        node: Node::Stack(&stack),
        length: Length::Unbounded,
        span,
    };

    let program = HashMap::from([
        (Name("bot".into()), &bot),
        (Name("top".into()), &top),
        (Name("stack".into()), &stack),
    ]);

    let evaluator = Evaluator::new(program, Name("stack".into())).with_max_depth(5);

    let expected = vec![
        ('a', span, Time(r(0, 1)), Length::one()),
        ('c', span, Time(r(0, 1)), Length::one()),
        ('b', span, Time(r(1, 1)), Length::one()),
        ('d', span, Time(r(1, 1)), Length::one()),
        ('a', span, Time(r(2, 1)), Length::one()),
        ('c', span, Time(r(2, 1)), Length::one()),
        ('b', span, Time(r(3, 1)), Length::one()),
        ('d', span, Time(r(3, 1)), Length::one()),
        ('a', span, Time(r(4, 1)), Length::one()),
        ('c', span, Time(r(4, 1)), Length::one()),
        ('b', span, Time(r(5, 1)), Length::one()),
        ('d', span, Time(r(5, 1)), Length::one()),
    ];

    let actual: Vec<_> = evaluator.iter().take(12).collect();

    assert_eq!(expected, actual);
}

#[test]
fn fractal() {
    let span = span();

    let a = Melody {
        node: Node::Note('a'),
        length: Length::one(),
        span,
    };

    let to_fractal = Melody {
        node: Node::Name(Name("fractal".into())),
        length: Length::Bounded(r(2, 1)),
        span,
    };

    let scale = Melody {
        node: Node::Scale(Factor(r(1, 2)), &to_fractal),
        length: Length::one(),
        span,
    };

    let melody = [a, scale];
    let melody = Melody {
        node: Node::Sequence(&melody),
        length: Length::Bounded(r(2, 1)),
        span,
    };

    let program = HashMap::from([(Name("fractal".into()), &melody)]);

    let evaluator = Evaluator::new(program, Name("fractal".into())).with_max_depth(5);

    let expected = vec![
        ('a', span, Time(r(0, 1)), Length::Bounded(r(1, 1))),
        ('a', span, Time(r(1, 1)), Length::Bounded(r(1, 2))),
        ('a', span, Time(r(3, 2)), Length::Bounded(r(1, 4))),
        ('a', span, Time(r(7, 4)), Length::Bounded(r(1, 8))),
        ('a', span, Time(r(15, 8)), Length::Bounded(r(1, 16))),
    ];

    let actual: Vec<_> = evaluator.iter().take(100).collect();

    assert_eq!(actual, expected);
}
