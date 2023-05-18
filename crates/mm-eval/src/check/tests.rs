use std::collections::HashMap;

use num_bigint::BigInt;
use num_rational::BigRational;

use super::Error;
use crate::names::names;
use crate::span::span;
use crate::{implicit, melody, Allocator, Factor, Heap, Length, Name};

fn r(n: i128, d: i128) -> BigRational {
    BigRational::new(BigInt::from(n), BigInt::from(d))
}

fn check_ok(
    expected: HashMap<Name, <Heap as Allocator<melody::Melody<char, &'static str, Heap>>>::Holder>,
    program: HashMap<Name, <Heap as Allocator<implicit::Melody<char, &'static str, Heap>>>::Holder>,
) {
    let mut alloc = Heap;
    let spans = program.keys().map(|name| (*name, span())).collect();

    let actual = super::check(
        &mut alloc,
        implicit::Program {
            defs: program,
            spans,
            public: vec![names()("it")],
            source: span(),
        },
    )
    .map(|program| program.defs);

    assert_eq!(Ok(expected), actual);
}

fn check_err(
    expected: Vec<Error<&str>>,
    program: HashMap<Name, <Heap as Allocator<implicit::Melody<char, &str, Heap>>>::Holder>,
) {
    let spans = program.keys().map(|name| (*name, span())).collect();

    let actual = super::check(
        &mut Heap,
        implicit::Program {
            defs: program,
            spans,
            public: vec![names()("it")],
            source: span(),
        },
    );

    assert_eq!(Err(expected), actual);
}

#[test]
fn lone_pause() {
    let mut name = names();

    let melody = implicit::Melody::Pause(span());
    let program = HashMap::from([(name("x"), Box::new(melody))]);

    let melody = melody::Melody {
        node: melody::Node::Pause,
        span: span(),
        length: Length::one(),
    };

    let expected = HashMap::from([(name("x"), Box::new(melody))]);
    check_ok(expected, program);
}

#[test]
fn lone_note() {
    let mut name = names();

    let melody = implicit::Melody::Note(span(), 'a');
    let program = HashMap::from([(name("x"), Box::new(melody))]);

    let melody = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let expected = HashMap::from([(name("x"), Box::new(melody))]);
    check_ok(expected, program);
}

#[test]
fn scaled_note() {
    let mut name = names();

    let melody = implicit::Melody::Note(span(), 'a');
    let melody = implicit::Melody::Scale(span(), Factor(r(1, 2)), Box::new(melody));
    let program = HashMap::from([(name("x"), Box::new(melody))]);

    let melody = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let melody = melody::Melody {
        node: melody::Node::Scale(Factor(r(1, 2)), Box::new(melody)),
        span: span(),
        length: Length::Bounded(r(1, 2)),
    };

    let expected = HashMap::from([(name("x"), Box::new(melody))]);
    check_ok(expected, program);
}

#[test]
fn pause_note_sequence() {
    let mut name = names();

    let first = implicit::Melody::Pause(span());
    let second = implicit::Melody::Note(span(), 'a');
    let melody = implicit::Melody::Sequence(vec![first, second]);
    let program = HashMap::from([(name("x"), Box::new(melody))]);

    let first = melody::Melody {
        node: melody::Node::Pause,
        span: span(),
        length: Length::one(),
    };

    let second = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let melody = melody::Melody {
        node: melody::Node::Sequence(vec![first, second]),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let expected = HashMap::from([(name("x"), Box::new(melody))]);
    check_ok(expected, program);
}

#[test]
fn pause_note_stack() {
    let mut name = names();

    let first = implicit::Melody::Pause(span());
    let second = implicit::Melody::Note(span(), 'a');
    let melody = implicit::Melody::Stack(vec![first, second]);
    let program = HashMap::from([(name("x"), Box::new(melody))]);

    let first = melody::Melody {
        node: melody::Node::Pause,
        span: span(),
        length: Length::one(),
    };

    let second = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let melody = melody::Melody {
        node: melody::Node::Stack(vec![first, second]),
        span: span(),
        length: Length::one(),
    };

    let expected = HashMap::from([(name("x"), Box::new(melody))]);
    check_ok(expected, program);
}

#[test]
fn a_name() {
    let mut name = names();

    let a = implicit::Melody::Note(span(), 'a');
    let pause = implicit::Melody::Pause(span());
    let to_a = implicit::Melody::Name(span(), name("a"));

    let b = implicit::Melody::Sequence(vec![pause, to_a]);

    let program = HashMap::from([(name("a"), Box::new(a)), (name("b"), Box::new(b))]);

    let a = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let pause = melody::Melody {
        node: melody::Node::Pause,
        span: span(),
        length: Length::one(),
    };

    let to_a = melody::Melody {
        node: melody::Node::Name(name("a")),
        span: span(),
        length: Length::one(),
    };

    let b = melody::Melody {
        node: melody::Node::Sequence(vec![pause, to_a]),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let expected = HashMap::from([(name("a"), Box::new(a)), (name("b"), Box::new(b))]);
    check_ok(expected, program);
}

#[test]
fn fractal() {
    let mut name = names();

    let note = implicit::Melody::Note(span(), 'a');
    let to_a = implicit::Melody::Name(span(), name("a"));
    let scale = implicit::Melody::Scale(span(), Factor(r(1, 2)), Box::new(to_a));

    let melody = implicit::Melody::Sequence(vec![note, scale]);

    let program = HashMap::from([(name("a"), Box::new(melody))]);

    let note = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let to_a = melody::Melody {
        node: melody::Node::Recur(name("a")),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let scale = melody::Melody {
        node: melody::Node::Scale(Factor(r(1, 2)), Box::new(to_a)),
        span: span(),
        length: Length::one(),
    };

    let melody = melody::Melody {
        node: melody::Node::Sequence(vec![note, scale]),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let expected = HashMap::from([(name("a"), Box::new(melody))]);
    check_ok(expected, program);
}

#[test]
fn infinite() {
    let mut name = names();

    let a = implicit::Melody::Note(span(), 'a');
    let b = implicit::Melody::Note(span(), 'b');
    let to_x = implicit::Melody::Name(span(), name("x"));

    let melody = implicit::Melody::Sequence(vec![a, b, to_x]);

    let program = HashMap::from([(name("x"), Box::new(melody))]);

    let a = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let b = melody::Melody {
        node: melody::Node::Note('b'),
        span: span(),
        length: Length::one(),
    };

    let to_x = melody::Melody {
        node: melody::Node::Name(name("x")),
        span: span(),
        length: Length::Unbounded,
    };

    let melody = melody::Melody {
        node: melody::Node::Sequence(vec![a, b, to_x]),
        span: span(),
        length: Length::Unbounded,
    };

    let expected = HashMap::from([(name("x"), Box::new(melody))]);

    check_ok(expected, program);
}

#[test]
fn fractal_names() {
    // it = 1/2 (at, it, bt)
    // at = a, b
    // bt = b, c
    let mut name = names();

    let a = implicit::Melody::Note(span(), 'a');
    let b1 = implicit::Melody::Note(span(), 'b');
    let b2 = implicit::Melody::Note(span(), 'b');
    let c = implicit::Melody::Note(span(), 'c');

    let at = implicit::Melody::Sequence(vec![a, b1]);

    let bt = implicit::Melody::Sequence(vec![b2, c]);

    let to_at = implicit::Melody::Name(span(), name("at"));
    let to_bt = implicit::Melody::Name(span(), name("bt"));
    let to_it = implicit::Melody::Name(span(), name("it"));

    let inner = implicit::Melody::Sequence(vec![to_at, to_it, to_bt]);

    let it = implicit::Melody::Scale(span(), Factor(r(1, 2)), Box::new(inner));

    let program = HashMap::from([
        (name("it"), Box::new(it)),
        (name("at"), Box::new(at)),
        (name("bt"), Box::new(bt)),
    ]);

    let a = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let b1 = melody::Melody {
        node: melody::Node::Note('b'),
        span: span(),
        length: Length::one(),
    };

    let b2 = melody::Melody {
        node: melody::Node::Note('b'),
        span: span(),
        length: Length::one(),
    };

    let c = melody::Melody {
        node: melody::Node::Note('c'),
        span: span(),
        length: Length::one(),
    };

    let at = melody::Melody {
        node: melody::Node::Sequence(vec![a, b1]),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let bt = melody::Melody {
        node: melody::Node::Sequence(vec![b2, c]),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let to_at = melody::Melody {
        node: melody::Node::Name(name("at")),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let to_it = melody::Melody {
        node: melody::Node::Recur(name("it")),
        span: span(),
        length: Length::Bounded(r(4, 1)),
    };

    let to_bt = melody::Melody {
        node: melody::Node::Name(name("bt")),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let inner = melody::Melody {
        node: melody::Node::Sequence(vec![to_at, to_it, to_bt]),
        span: span(),
        length: Length::Bounded(r(8, 1)),
    };

    let it = melody::Melody {
        node: melody::Node::Scale(Factor(r(1, 2)), Box::new(inner)),
        span: span(),
        length: Length::Bounded(r(4, 1)),
    };

    let expected = HashMap::from([
        (name("it"), Box::new(it)),
        (name("at"), Box::new(at)),
        (name("bt"), Box::new(bt)),
    ]);
    check_ok(expected, program);
}

#[test]
fn wrong_unbounded() {
    let mut name = names();

    let a = implicit::Melody::Note(span(), 'a');
    let to_x = implicit::Melody::Name(span(), name("x"));
    let b = implicit::Melody::Note(span(), 'b');

    let melody = implicit::Melody::Sequence(vec![a, to_x, b]);
    let program = HashMap::from([(name("x"), Box::new(melody))]);
    let expected = vec![Error::UnboundedNotLast(span())];
    check_err(expected, program);
}

#[test]
fn empty_recursive() {
    let mut name = names();
    let x = implicit::Melody::Name(span(), name("x"));
    let program = HashMap::from([(name("x"), Box::new(x))]);

    let expected = vec![Error::UnfoundedRecursion(span())];
    check_err(expected, program);
}

#[test]
fn empty_mutually_recursive() {
    let mut name = names();

    let x = implicit::Melody::Name(span(), name("y"));
    let y = implicit::Melody::Name(span(), name("x"));
    let program = HashMap::from([(name("x"), Box::new(x)), (name("y"), Box::new(y))]);

    let expected = vec![Error::UnfoundedRecursion(span())];
    check_err(expected, program);
}

#[test]
fn recursive_stack() {
    let mut name = names();

    let a = implicit::Melody::Note(span(), 'a');
    let to_x = implicit::Melody::Name(span(), name("x"));
    let x = implicit::Melody::Stack(vec![a, to_x]);
    let program = HashMap::from([(name("x"), Box::new(x))]);

    let expected = vec![Error::UnfoundedRecursion(span())];
    check_err(expected, program);
}
