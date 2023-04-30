use std::collections::HashMap;

use num_bigint::BigInt;
use num_rational::BigRational;
use typed_arena::Arena;

use crate::span::span;
use crate::{implicit, melody, Factor, Length, Name};

use super::Error;

fn r(n: i128, d: i128) -> BigRational {
    BigRational::new(BigInt::from(n), BigInt::from(d))
}

fn check_ok(
    expected: HashMap<Name, &melody::Melody<char>>,
    program: HashMap<Name, &implicit::Melody<char>>,
) {
    let arena = Arena::new();
    let actual = super::check(
        &arena,
        implicit::Program {
            defs: program,
            spans: HashMap::new(),
        },
    )
    .map(|program| program.defs);

    assert_eq!(Ok(expected), actual);
}

fn check_err(expected: Vec<Error>, program: HashMap<Name, &implicit::Melody<char>>) {
    let arena = Arena::new();
    let actual = super::check(
        &arena,
        implicit::Program {
            defs: program,
            spans: HashMap::new(),
        },
    );

    assert_eq!(Err(expected), actual);
}

#[test]
fn lone_pause() {
    let melody = implicit::Melody::Pause(span());
    let program = HashMap::from([(Name("x"), &melody)]);

    let melody = melody::Melody {
        node: melody::Node::Pause,
        span: span(),
        length: Length::one(),
    };

    let expected = HashMap::from([(Name("x"), &melody)]);
    check_ok(expected, program);
}

#[test]
fn lone_note() {
    let melody = implicit::Melody::Note(span(), 'a');
    let program = HashMap::from([(Name("x"), &melody)]);

    let melody = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let expected = HashMap::from([(Name("x"), &melody)]);
    check_ok(expected, program);
}

#[test]
fn scaled_note() {
    let melody = implicit::Melody::Note(span(), 'a');
    let melody = implicit::Melody::Scale(span(), Factor(r(1, 2)), &melody);
    let program = HashMap::from([(Name("x"), &melody)]);

    let melody = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let melody = melody::Melody {
        node: melody::Node::Scale(Factor(r(1, 2)), &melody),
        span: span(),
        length: Length::Bounded(r(1, 2)),
    };

    let expected = HashMap::from([(Name("x"), &melody)]);
    check_ok(expected, program);
}

#[test]
fn pause_note_sequence() {
    let first = implicit::Melody::Pause(span());
    let second = implicit::Melody::Note(span(), 'a');
    let seq = [first, second];
    let melody = implicit::Melody::Sequence(&seq);
    let program = HashMap::from([(Name("x"), &melody)]);

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

    let seq = [first, second];

    let melody = melody::Melody {
        node: melody::Node::Sequence(&seq),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let expected = HashMap::from([(Name("x"), &melody)]);
    check_ok(expected, program);
}

#[test]
fn pause_note_stack() {
    let first = implicit::Melody::Pause(span());
    let second = implicit::Melody::Note(span(), 'a');
    let seq = [first, second];
    let melody = implicit::Melody::Stack(&seq);
    let program = HashMap::from([(Name("x"), &melody)]);

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

    let seq = [first, second];

    let melody = melody::Melody {
        node: melody::Node::Stack(&seq),
        span: span(),
        length: Length::one(),
    };

    let expected = HashMap::from([(Name("x"), &melody)]);
    check_ok(expected, program);
}

#[test]
fn a_name() {
    let a = implicit::Melody::Note(span(), 'a');
    let pause = implicit::Melody::Pause(span());
    let to_a = implicit::Melody::Name(span(), Name("a"));

    let b = [pause, to_a];
    let b = implicit::Melody::Sequence(&b);

    let program = HashMap::from([(Name("a"), &a), (Name("b"), &b)]);

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
        node: melody::Node::Name(Name("a")),
        span: span(),
        length: Length::one(),
    };

    let b = [pause, to_a];
    let b = melody::Melody {
        node: melody::Node::Sequence(&b),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let expected = HashMap::from([(Name("a"), &a), (Name("b"), &b)]);

    check_ok(expected, program);
}

#[test]
fn fractal() {
    let note = implicit::Melody::Note(span(), 'a');
    let to_a = implicit::Melody::Name(span(), Name("a"));
    let scale = implicit::Melody::Scale(span(), Factor(r(1, 2)), &to_a);

    let melody = [note, scale];
    let melody = implicit::Melody::Sequence(&melody);

    let program = HashMap::from([(Name("a"), &melody)]);

    let note = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let to_a = melody::Melody {
        node: melody::Node::Recur(Name("a")),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let scale = melody::Melody {
        node: melody::Node::Scale(Factor(r(1, 2)), &to_a),
        span: span(),
        length: Length::one(),
    };

    let melody = [note, scale];
    let melody = melody::Melody {
        node: melody::Node::Sequence(&melody),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let expected = HashMap::from([(Name("a"), &melody)]);

    check_ok(expected, program);
}

#[test]
fn infinite() {
    let a = implicit::Melody::Note(span(), 'a');
    let b = implicit::Melody::Note(span(), 'b');
    let to_x = implicit::Melody::Name(span(), Name("x"));

    let melody = [a, b, to_x];
    let melody = implicit::Melody::Sequence(&melody);

    let program = HashMap::from([(Name("x"), &melody)]);

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
        node: melody::Node::Name(Name("x")),
        span: span(),
        length: Length::Unbounded,
    };

    let melody = [a, b, to_x];
    let melody = melody::Melody {
        node: melody::Node::Sequence(&melody),
        span: span(),
        length: Length::Unbounded,
    };

    let expected = HashMap::from([(Name("x"), &melody)]);

    check_ok(expected, program);
}

#[test]
fn fractal_names() {
    // it = 1/2 (at, it, bt)
    // at = a, b
    // bt = b, c
    let a = implicit::Melody::Note(span(), 'a');
    let b = implicit::Melody::Note(span(), 'b');
    let c = implicit::Melody::Note(span(), 'c');

    let at = [a, b.clone()];
    let at = implicit::Melody::Sequence(&at);

    let bt = [b, c];
    let bt = implicit::Melody::Sequence(&bt);

    let to_at = implicit::Melody::Name(span(), Name("at"));
    let to_bt = implicit::Melody::Name(span(), Name("bt"));
    let to_it = implicit::Melody::Name(span(), Name("it"));

    let inner = [to_at, to_it, to_bt];
    let inner = implicit::Melody::Sequence(&inner);

    let it = implicit::Melody::Scale(span(), Factor(r(1, 2)), &inner);

    let program = HashMap::from([(Name("it"), &it), (Name("at"), &at), (Name("bt"), &bt)]);

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

    let c = melody::Melody {
        node: melody::Node::Note('c'),
        span: span(),
        length: Length::one(),
    };

    let at = [a, b.clone()];
    let at = melody::Melody {
        node: melody::Node::Sequence(&at),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let bt = [b, c];
    let bt = melody::Melody {
        node: melody::Node::Sequence(&bt),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let to_at = melody::Melody {
        node: melody::Node::Name(Name("at")),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let to_it = melody::Melody {
        node: melody::Node::Recur(Name("it")),
        span: span(),
        length: Length::Bounded(r(4, 1)),
    };

    let to_bt = melody::Melody {
        node: melody::Node::Name(Name("bt")),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let inner = [to_at, to_it, to_bt];
    let inner = melody::Melody {
        node: melody::Node::Sequence(&inner),
        span: span(),
        length: Length::Bounded(r(8, 1)),
    };

    let it = melody::Melody {
        node: melody::Node::Scale(Factor(r(1, 2)), &inner),
        span: span(),
        length: Length::Bounded(r(4, 1)),
    };

    let expected = HashMap::from([(Name("it"), &it), (Name("at"), &at), (Name("bt"), &bt)]);

    check_ok(expected, program);
}

#[test]
fn wrong_unbounded() {
    let a = implicit::Melody::Note(span(), 'a');
    let to_x = implicit::Melody::Name(span(), Name("x"));
    let b = implicit::Melody::Note(span(), 'b');

    let melody = [a, to_x, b];
    let melody = implicit::Melody::Sequence(&melody);

    let program = HashMap::from([(Name("x"), &melody)]);

    let expected = vec![Error::UnboundedNotLast(span())];
    check_err(expected, program);
}
