use std::collections::{HashMap, HashSet};

use crate::implicit::Melody;
use crate::{check, Allocator, Name};

/// Compute the dependency graph of the given program. Each returned entry
/// contains "outgoing" edges: `a` is in the set of names referred to by `b` if
/// the definition of `b` refers to `a` at any place.
pub fn dependencies<N, Id: Clone, A: Allocator<Melody<N, Id, A>>>(
    program: &HashMap<Name, A::Holder>,
) -> Result<HashMap<Name, HashSet<Name>>, Vec<check::Error<Id>>> {
    let mut errs = Vec::new();
    let program = program
        .iter()
        .map(|(name, melody)| {
            let mut refers = HashSet::new();
            if let Err(es) = compute(program, &mut refers, A::as_ref(melody)) {
                errs.extend(es);
            }
            (*name, refers)
        })
        .collect();

    errs.is_empty().then_some(program).ok_or(errs)
}

/// Add the names referred to by `melody` to `within`.
fn compute<N, Id: Clone, A: Allocator<Melody<N, Id, A>>>(
    program: &HashMap<Name, A::Holder>,
    within: &mut HashSet<Name>,
    melody: &Melody<N, Id, A>,
) -> Result<(), Vec<check::Error<Id>>> {
    match melody {
        Melody::Pause(_) | Melody::Note(..) => Ok(()),

        Melody::Name(span, name) => {
            within.insert(*name);

            if !program.contains_key(name) {
                Err(vec![check::Error::UnknownName(span.clone(), *name)])
            } else {
                Ok(())
            }
        }

        Melody::Scale(_, _, melody) => compute(program, within, A::as_ref(melody)),
        Melody::Sharp(_, _, melody) => compute(program, within, A::as_ref(melody)),
        Melody::Offset(_, _, melody) => compute(program, within, A::as_ref(melody)),

        Melody::Sequence(melodies) | Melody::Stack(melodies) => {
            let mut errs = vec![];
            for melody in A::as_slice(melodies) {
                if let Err(es) = compute(program, within, melody) {
                    errs.extend(es);
                }
            }

            errs.is_empty().then_some(()).ok_or(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::implicit::Melody;
    use crate::names::names;
    use crate::span::span;
    use crate::Heap;

    use super::dependencies;

    #[test]
    fn disjoint() {
        let mut name = names();

        let a = Melody::Pause(span());
        let b = Melody::Note(span(), 'b');
        let c = Melody::Note(span(), 'c');

        let seq = Melody::Sequence(vec![b, c]);

        let program = HashMap::from([(name("a"), Box::new(a)), (name("b"), Box::new(seq))]);
        let expected = HashMap::from([(name("a"), HashSet::new()), (name("b"), HashSet::new())]);

        let actual = dependencies::<char, &str, Heap>(&program);

        assert_eq!(Ok(expected), actual);
    }

    #[test]
    fn chain() {
        let mut name = names();

        let a = Melody::Pause(span());
        let b = Melody::Name(span(), name("a"));
        let c = Melody::Name(span(), name("b"));

        let program = HashMap::from([
            (name("a"), Box::new(a)),
            (name("b"), Box::new(b)),
            (name("c"), Box::new(c)),
        ]);

        let expected = HashMap::from([
            (name("a"), HashSet::new()),
            (name("b"), HashSet::from([name("a")])),
            (name("c"), HashSet::from([name("b")])),
        ]);

        let actual = dependencies::<char, &str, Heap>(&program);

        assert_eq!(Ok(expected), actual);
    }

    #[test]
    fn fork_join() {
        let mut name = names();

        let a = Melody::Pause(span());
        let to_a1 = Melody::Name(span(), name("a"));
        let to_a2 = Melody::Name(span(), name("a"));
        let to_b = Melody::Name(span(), name("b"));
        let to_c = Melody::Name(span(), name("c"));

        let d = Melody::Sequence(vec![to_b, to_c]);

        let program = HashMap::from([
            (name("a"), Box::new(a)),
            (name("b"), Box::new(to_a1)),
            (name("c"), Box::new(to_a2)),
            (name("d"), Box::new(d)),
        ]);

        let expected = HashMap::from([
            (name("a"), HashSet::new()),
            (name("b"), HashSet::from([name("a")])),
            (name("c"), HashSet::from([name("a")])),
            (name("d"), HashSet::from([name("b"), name("c")])),
        ]);

        let actual = dependencies::<char, &str, Heap>(&program);

        assert_eq!(Ok(expected), actual);
    }

    #[test]
    fn cycles() {
        let mut name = names();

        let a = Melody::Name(span(), name("c"));
        let b = Melody::Name(span(), name("a"));
        let c = Melody::Name(span(), name("b"));

        let program = HashMap::from([
            (name("a"), Box::new(a)),
            (name("b"), Box::new(b)),
            (name("c"), Box::new(c)),
        ]);

        let expected = HashMap::from([
            (name("a"), HashSet::from([name("c")])),
            (name("b"), HashSet::from([name("a")])),
            (name("c"), HashSet::from([name("b")])),
        ]);

        let actual = dependencies::<char, &str, Heap>(&program);

        assert_eq!(Ok(expected), actual);
    }
}
