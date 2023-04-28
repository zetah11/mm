use std::collections::HashMap;

use rational::extras::r;
use typed_arena::Arena;

use crate::span::span;
use crate::{implicit, melody, Factor, Length, Name};

use super::Error;

fn check_ok(
    expected: HashMap<Name, &melody::Melody<char>>,
    program: HashMap<Name, &implicit::Melody<char>>,
) {
    let arena = Arena::new();
    let actual = super::check(&arena, &program);
    assert_eq!(Ok(expected), actual);
}

fn check_err(expected: Vec<Error>, program: HashMap<Name, &implicit::Melody<char>>) {
    let arena = Arena::new();
    let actual = super::check(&arena, &program);

    assert_eq!(Err(expected), actual);
}

#[test]
fn lone_pause() {
    let melody = implicit::Melody::Pause(span());
    let program = HashMap::from([(Name("x".into()), &melody)]);

    let melody = melody::Melody {
        node: melody::Node::Pause,
        span: span(),
        length: Length::one(),
    };

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn lone_note() {
    let melody = implicit::Melody::Note(span(), 'a');
    let program = HashMap::from([(Name("x".into()), &melody)]);

    let melody = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn scaled_note() {
    let melody = implicit::Melody::Note(span(), 'a');
    let melody = implicit::Melody::Scale(span(), Factor(r(1, 2)), &melody);
    let program = HashMap::from([(Name("x".into()), &melody)]);

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

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn pause_note_sequence() {
    let first = implicit::Melody::Pause(span());
    let second = implicit::Melody::Note(span(), 'a');
    let seq = [first, second];
    let melody = implicit::Melody::Sequence(&seq);
    let program = HashMap::from([(Name("x".into()), &melody)]);

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

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn pause_note_stack() {
    let first = implicit::Melody::Pause(span());
    let second = implicit::Melody::Note(span(), 'a');
    let seq = [first, second];
    let melody = implicit::Melody::Stack(&seq);
    let program = HashMap::from([(Name("x".into()), &melody)]);

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

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn a_name() {
    let a = implicit::Melody::Note(span(), 'a');
    let pause = implicit::Melody::Pause(span());
    let to_a = implicit::Melody::Name(span(), Name("a".into()));

    let b = [pause, to_a];
    let b = implicit::Melody::Sequence(&b);

    let program = HashMap::from([(Name("a".into()), &a), (Name("b".into()), &b)]);

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
        node: melody::Node::Name(Name("a".into())),
        span: span(),
        length: Length::one(),
    };

    let b = [pause, to_a];
    let b = melody::Melody {
        node: melody::Node::Sequence(&b),
        span: span(),
        length: Length::Bounded(r(2, 1)),
    };

    let expected = HashMap::from([(Name("a".into()), &a), (Name("b".into()), &b)]);

    check_ok(expected, program);
}

#[test]
fn fractal() {
    let note = implicit::Melody::Note(span(), 'a');
    let to_a = implicit::Melody::Name(span(), Name("a".into()));
    let scale = implicit::Melody::Scale(span(), Factor(r(1, 2)), &to_a);

    let melody = [note, scale];
    let melody = implicit::Melody::Sequence(&melody);

    let program = HashMap::from([(Name("a".into()), &melody)]);

    let note = melody::Melody {
        node: melody::Node::Note('a'),
        span: span(),
        length: Length::one(),
    };

    let to_a = melody::Melody {
        node: melody::Node::Name(Name("a".into())),
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

    let expected = HashMap::from([(Name("a".into()), &melody)]);

    check_ok(expected, program);
}

#[test]
fn infinite() {
    let a = implicit::Melody::Note(span(), 'a');
    let b = implicit::Melody::Note(span(), 'b');
    let to_x = implicit::Melody::Name(span(), Name("x".into()));

    let melody = [a, b, to_x];
    let melody = implicit::Melody::Sequence(&melody);

    let program = HashMap::from([(Name("x".into()), &melody)]);

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
        node: melody::Node::Name(Name("x".into())),
        span: span(),
        length: Length::Unbounded,
    };

    let melody = [a, b, to_x];
    let melody = melody::Melody {
        node: melody::Node::Sequence(&melody),
        span: span(),
        length: Length::Unbounded,
    };

    let expected = HashMap::from([(Name("x".into()), &melody)]);

    check_ok(expected, program);
}

#[test]
fn wrong_unbounded() {
    let a = implicit::Melody::Note(span(), 'a');
    let to_x = implicit::Melody::Name(span(), Name("x".into()));
    let b = implicit::Melody::Note(span(), 'b');

    let melody = [a, to_x, b];
    let melody = implicit::Melody::Sequence(&melody);

    let program = HashMap::from([(Name("x".into()), &melody)]);

    let expected = vec![Error::UnboundedNotLast];
    check_err(expected, program);
}
