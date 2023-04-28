use std::collections::HashMap;

use rational::extras::r;
use typed_arena::Arena;

use crate::implicit::Melody;
use crate::{Factor, Name};

use super::Parser;

fn check_ok(expected: HashMap<Name, &Melody<char>>, source: &str) {
    let arena = Arena::new();
    let actual = Parser::parse(&arena, source);
    assert_eq!(expected, actual);
}

#[test]
fn simple_sequence() {
    let source = r#"
        it = A, B, C
    "#;

    let a = Melody::Note('A');
    let b = Melody::Note('B');
    let c = Melody::Note('C');

    let sequence = [a, b, c];
    let sequence = Melody::Sequence(&sequence);

    let expected = HashMap::from([(Name("it".into()), &sequence)]);

    check_ok(expected, source);
}

#[test]
fn some_scales() {
    let source = r#"
        it = 1/2 A, 2/3 (B | C)
    "#;

    let a = Melody::Note('A');
    let b = Melody::Note('B');
    let c = Melody::Note('C');

    let first = Melody::Scale(Factor(r(1, 2)), &a);

    let stack = [b, c];
    let stack = Melody::Stack(&stack);

    let second = Melody::Scale(Factor(r(2, 3)), &stack);

    let sequence = [first, second];
    let sequence = Melody::Sequence(&sequence);

    let expected = HashMap::from([(Name("it".into()), &sequence)]);

    check_ok(expected, source);
}

#[test]
fn mutual() {
    let source = r#"
        it = A, 1/2 it, 1/3 at
        at = B, 1/3 at, 1/2 it
    "#;

    let a = Melody::Note('A');
    let b = Melody::Note('B');

    let to_it = Melody::Name(Name("it".into()));
    let to_at = Melody::Name(Name("at".into()));

    let half_it = Melody::Scale(Factor(r(1, 2)), &to_it);
    let third_at = Melody::Scale(Factor(r(1, 3)), &to_at);

    let it = [a, half_it.clone(), third_at.clone()];
    let it = Melody::Sequence(&it);

    let at = [b, third_at, half_it];
    let at = Melody::Sequence(&at);

    let expected = HashMap::from([(Name("it".into()), &it), (Name("at".into()), &at)]);

    check_ok(expected, source);
}
