use std::collections::HashMap;

use rational::Rational;
use typed_arena::Arena;

use crate::{implicit, melody, Factor, Length, Name};

fn check_ok(expected: HashMap<Name, &melody::Melody>, program: HashMap<Name, &implicit::Melody>) {
    let arena = Arena::new();
    let actual = super::check(&arena, &program);
    assert_eq!(Ok(expected), actual);
}

#[test]
fn lone_pause() {
    let melody = implicit::Melody::Pause;
    let program = HashMap::from([(Name("x".into()), &melody)]);

    let melody = melody::Melody {
        node: melody::Node::Pause,
        length: Length::one(),
    };

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn lone_note() {
    let melody = implicit::Melody::Note('a');
    let program = HashMap::from([(Name("x".into()), &melody)]);

    let melody = melody::Melody {
        node: melody::Node::Note('a'),
        length: Length::one(),
    };

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn scaled_note() {
    let melody = implicit::Melody::Note('a');
    let melody = implicit::Melody::Scale(Factor(Rational::new(1, 2)), &melody);
    let program = HashMap::from([(Name("x".into()), &melody)]);

    let melody = melody::Melody {
        node: melody::Node::Note('a'),
        length: Length::one(),
    };

    let melody = melody::Melody {
        node: melody::Node::Scale(Factor(Rational::new(1, 2)), &melody),
        length: Length(Rational::new(1, 2)),
    };

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn pause_note_sequence() {
    let first = implicit::Melody::Pause;
    let second = implicit::Melody::Note('a');
    let seq = [first, second];
    let melody = implicit::Melody::Sequence(&seq);
    let program = HashMap::from([(Name("x".into()), &melody)]);

    let first = melody::Melody {
        node: melody::Node::Pause,
        length: Length::one(),
    };

    let second = melody::Melody {
        node: melody::Node::Note('a'),
        length: Length::one(),
    };

    let seq = [first, second];

    let melody = melody::Melody {
        node: melody::Node::Sequence(&seq),
        length: Length(Rational::integer(2)),
    };

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn pause_note_stack() {
    let first = implicit::Melody::Pause;
    let second = implicit::Melody::Note('a');
    let seq = [first, second];
    let melody = implicit::Melody::Stack(&seq);
    let program = HashMap::from([(Name("x".into()), &melody)]);

    let first = melody::Melody {
        node: melody::Node::Pause,
        length: Length::one(),
    };

    let second = melody::Melody {
        node: melody::Node::Note('a'),
        length: Length::one(),
    };

    let seq = [first, second];

    let melody = melody::Melody {
        node: melody::Node::Stack(&seq),
        length: Length::one(),
    };

    let expected = HashMap::from([(Name("x".into()), &melody)]);
    check_ok(expected, program);
}

#[test]
fn a_name() {
    let a = implicit::Melody::Note('a');
    let pause = implicit::Melody::Pause;
    let to_a = implicit::Melody::Name(Name("a".into()));

    let b = [pause, to_a];
    let b = implicit::Melody::Sequence(&b);

    let program = HashMap::from([(Name("a".into()), &a), (Name("b".into()), &b)]);

    let a = melody::Melody {
        node: melody::Node::Note('a'),
        length: Length::one(),
    };

    let pause = melody::Melody {
        node: melody::Node::Pause,
        length: Length::one(),
    };

    let to_a = melody::Melody {
        node: melody::Node::Name(Name("a".into())),
        length: Length::one(),
    };

    let b = [pause, to_a];
    let b = melody::Melody {
        node: melody::Node::Sequence(&b),
        length: Length(Rational::integer(2)),
    };

    let expected = HashMap::from([(Name("a".into()), &a), (Name("b".into()), &b)]);

    check_ok(expected, program);
}

#[test]
fn fractal() {
    let note = implicit::Melody::Note('a');
    let to_a = implicit::Melody::Name(Name("a".into()));
    let scale = implicit::Melody::Scale(Factor(Rational::new(1, 2)), &to_a);

    let melody = [note, scale];
    let melody = implicit::Melody::Sequence(&melody);

    let program = HashMap::from([(Name("a".into()), &melody)]);

    let note = melody::Melody {
        node: melody::Node::Note('a'),
        length: Length::one(),
    };

    let to_a = melody::Melody {
        node: melody::Node::Name(Name("a".into())),
        length: Length(Rational::integer(2)),
    };

    let scale = melody::Melody {
        node: melody::Node::Scale(Factor(Rational::new(1, 2)), &to_a),
        length: Length::one(),
    };

    let melody = [note, scale];
    let melody = melody::Melody {
        node: melody::Node::Sequence(&melody),
        length: Length(Rational::integer(2)),
    };

    let expected = HashMap::from([(Name("a".into()), &melody)]);

    check_ok(expected, program);
}