use std::collections::HashMap;

use num_bigint::BigInt;
use num_rational::BigRational;
use typed_arena::Arena;

use crate::implicit::Melody;
use crate::span::span_in;
use crate::{Factor, Name};

use super::{Error, Parser};

fn r(n: i128, d: i128) -> BigRational {
    BigRational::new(BigInt::from(n), BigInt::from(d))
}

fn check_ok(expected: HashMap<Name, &Melody<char>>, source: &str) {
    let arena = Arena::new();
    let actual = Parser::parse(&arena, source).map(|program| program.defs);
    assert_eq!(Ok(expected), actual);
}

fn check_err(expected: Vec<Error>, source: &str) {
    let arena: Arena<Melody<char>> = Arena::new();
    let actual = Parser::parse(&arena, source);
    assert_eq!(Err(expected), actual);
}

#[test]
fn simple_sequence() {
    let source = r#"it = A, B, C"#;
    let s = span_in(source);

    let a = Melody::Note(s(5, 6), 'A');
    let b = Melody::Note(s(8, 9), 'B');
    let c = Melody::Note(s(11, 12), 'C');

    let sequence = [a, b, c];
    let sequence = Melody::Sequence(&sequence);

    let expected = HashMap::from([(Name("it"), &sequence)]);

    check_ok(expected, source);
}

#[test]
fn some_scales() {
    let source = r#"it = 1/2 A, 2/3 (B | C)"#;
    let s = span_in(source);

    let a = Melody::Note(s(9, 10), 'A');
    let b = Melody::Note(s(17, 18), 'B');
    let c = Melody::Note(s(21, 22), 'C');

    let first = Melody::Scale(s(5, 8), Factor(r(1, 2)), &a);

    let stack = [b, c];
    let stack = Melody::Stack(&stack);

    let second = Melody::Scale(s(12, 15), Factor(r(2, 3)), &stack);

    let sequence = [first, second];
    let sequence = Melody::Sequence(&sequence);

    let expected = HashMap::from([(Name("it"), &sequence)]);

    check_ok(expected, source);
}

#[test]
fn mutual() {
    let source = r#"
        it = A, 1/2 it, 1/3 at
        at = B, 1/3 at, 1/2 it
    "#;

    let s = span_in(source);

    let a = Melody::Note(s(14, 15), 'A');
    let b = Melody::Note(s(45, 46), 'B');

    let to_it1 = Melody::Name(s(21, 23), Name("it"));
    let to_it2 = Melody::Name(s(60, 62), Name("it"));
    let to_at1 = Melody::Name(s(29, 31), Name("at"));
    let to_at2 = Melody::Name(s(52, 54), Name("at"));

    let half_it1 = Melody::Scale(s(17, 20), Factor(r(1, 2)), &to_it1);
    let half_it2 = Melody::Scale(s(56, 59), Factor(r(1, 2)), &to_it2);
    let third_at1 = Melody::Scale(s(25, 28), Factor(r(1, 3)), &to_at1);
    let third_at2 = Melody::Scale(s(48, 51), Factor(r(1, 3)), &to_at2);

    let it = [a, half_it1, third_at1];
    let it = Melody::Sequence(&it);

    let at = [b, third_at2, half_it2];
    let at = Melody::Sequence(&at);

    let expected = HashMap::from([(Name("it"), &it), (Name("at"), &at)]);

    check_ok(expected, source);
}

#[test]
fn some_comments() {
    let source = "-- beep--=\nit = <>--";
    let s = span_in(source);

    let a = Melody::Pause(s(16, 18));
    let expected = HashMap::from([(Name("it"), &a)]);

    check_ok(expected, source);
}

#[test]
fn sharps() {
    let source = r#"it = (a#)##"#;
    let s = span_in(source);

    let a = Melody::Note(s(6, 7), 'a');
    let inner = Melody::Sharp(s(7, 8), 1, &a);
    let outer = Melody::Sharp(s(9, 11), 2, &inner);
    let expected = HashMap::from([(Name("it"), &outer)]);

    check_ok(expected, source);
}

#[test]
fn offsets() {
    let source = r#"it = (a+1)-1"#;
    let s = span_in(source);

    let a = Melody::Note(s(6, 7), 'a');
    let inner = Melody::Offset(s(7, 9), 1, &a);
    let outer = Melody::Offset(s(10, 12), -1, &inner);
    let expected = HashMap::from([(Name("it"), &outer)]);

    check_ok(expected, source);
}

#[test]
fn expected_equal() {
    let source = r#"aa bb = A"#;
    let s = span_in(source);

    let expected = vec![Error::ExpectedEqual(s(3, 5))];
    check_err(expected, source);
}

#[test]
fn glasses() {
    let source = r#"a = a"#;
    let s = span_in(source);

    let expected = vec![Error::ExpectedName(s(0, 1))];
    check_err(expected, source);
}

#[test]
fn not_a_number() {
    let source = r#"it = 1/at"#;
    let s = span_in(source);

    let expected = vec![Error::ExpectedNumber(s(7, 9))];
    check_err(expected, source);
}

#[test]
fn division_by_zero() {
    let source = r#"it = 1/0 a"#;
    let s = span_in(source);

    let expected = vec![Error::DivisionByZero(s(7, 8))];
    check_err(expected, source);
}

#[test]
fn expected_note() {
    let source = r#"it = 1,"#;
    let s = span_in(source);

    let expected = vec![Error::ExpectedNote(s(6, 7))];
    check_err(expected, source);
}

#[test]
fn odd_parens() {
    let source = r#"it = (((a))"#;
    let s = span_in(source);

    let expected = vec![Error::UnclosedParen {
        opener: s(5, 6),
        at: s(10, 11),
    }];

    check_err(expected, source);
}

#[test]
fn redefinition() {
    let source = r#"it = a it = b"#;
    let s = span_in(source);

    let expected = vec![Error::Redefinition {
        previous: s(0, 2),
        new: s(7, 9),
    }];

    check_err(expected, source);
}
