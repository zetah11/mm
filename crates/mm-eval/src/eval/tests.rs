use std::collections::HashMap;

use num_bigint::BigInt;
use num_rational::BigRational;

use crate::melody::{Melody, Node};
use crate::names::names;
use crate::span::{span, Span};
use crate::{Allocator, Factor, Heap, Length, Name, Time};

use super::Evaluator;

fn r(n: i128, d: i128) -> BigRational {
    BigRational::new(BigInt::from(n), BigInt::from(d))
}

fn check(
    expected: Vec<(char, Span<&str>, Time, Length)>,
    program: HashMap<Name, <Heap as Allocator<Melody<char, &str, Heap>>>::Holder>,
    entry: Name,
) {
    let eval: Evaluator<_, _, Heap> = Evaluator::new(&program, entry);
    let actual: Vec<_> = eval.iter().collect();
    assert_eq!(expected, actual);
}

#[test]
fn simple_sequence() {
    let mut name = names();
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

    let melody = Melody {
        node: Node::Sequence(vec![a, b, c]),
        length: Length::Bounded(r(3, 1)),
        span,
    };

    let program = HashMap::from([(name("it"), Box::new(melody))]);

    let expected = vec![
        ('a', span, Time(r(0, 1)), Length::one()),
        ('c', span, Time(r(2, 1)), Length::one()),
    ];

    check(expected, program, name("it"));
}

#[test]
fn simple_stack() {
    let mut name = names();
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

    let melody = Melody {
        node: Node::Stack(vec![a, b, c]),
        length: Length::one(),
        span,
    };

    let program = HashMap::from([(name("it"), Box::new(melody))]);

    let expected = vec![
        ('a', span, Time::zero(), Length::one()),
        ('c', span, Time::zero(), Length::one()),
    ];

    check(expected, program, name("it"));
}

#[test]
fn unending_stack() {
    let mut name = names();
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

    let to_bot1 = Melody {
        node: Node::Name(name("bot")),
        length: Length::Unbounded,
        span,
    };

    let to_bot2 = Melody {
        node: Node::Name(name("bot")),
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

    let to_top1 = Melody {
        node: Node::Name(name("top")),
        length: Length::Unbounded,
        span,
    };

    let to_top2 = Melody {
        node: Node::Name(name("top")),
        length: Length::Unbounded,
        span,
    };

    let bot = Melody {
        node: Node::Sequence(vec![a, b, to_bot1]),
        length: Length::Unbounded,
        span,
    };

    let top = Melody {
        node: Node::Sequence(vec![c, d, to_top1]),
        length: Length::Unbounded,
        span,
    };

    let stack = Melody {
        node: Node::Stack(vec![to_bot2, to_top2]),
        length: Length::Unbounded,
        span,
    };

    let program = HashMap::from([
        (name("bot"), Box::new(bot)),
        (name("top"), Box::new(top)),
        (name("stack"), Box::new(stack)),
    ]);

    let evaluator: Evaluator<_, _, Heap> =
        Evaluator::new(&program, name("stack")).with_max_depth(5);

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
    let mut name = names();
    let span = span();

    let a = Melody {
        node: Node::Note('a'),
        length: Length::one(),
        span,
    };

    let to_fractal = Melody {
        node: Node::Recur(name("fractal")),
        length: Length::Bounded(r(2, 1)),
        span,
    };

    let scale = Melody {
        node: Node::Scale(Factor(r(1, 2)), Box::new(to_fractal)),
        length: Length::one(),
        span,
    };

    let melody = Melody {
        node: Node::Sequence(vec![a, scale]),
        length: Length::Bounded(r(2, 1)),
        span,
    };

    let program = HashMap::from([(name("fractal"), Box::new(melody))]);

    let evaluator: Evaluator<_, _, Heap> =
        Evaluator::new(&program, name("fractal")).with_max_depth(5);

    let expected = vec![
        ('a', span, Time(r(0, 1)), Length::Bounded(r(1, 1))),
        ('a', span, Time(r(1, 1)), Length::Bounded(r(1, 2))),
        ('a', span, Time(r(3, 2)), Length::Bounded(r(1, 4))),
        ('a', span, Time(r(7, 4)), Length::Bounded(r(1, 8))),
        ('a', span, Time(r(15, 8)), Length::Bounded(r(1, 16))),
    ];

    let actual: Vec<_> = evaluator.iter().take(100).collect();

    assert_eq!(expected, actual);
}
