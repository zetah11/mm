use std::collections::{HashMap, HashSet};

use crate::implicit::Melody;
use crate::{check, Name};

/// Compute the dependency graph of the given program. Each returned entry
/// contains "outgoing" edges: `a` is in the set of names referred to by `b` if
/// the definition of `b` refers to `a` at any place.
pub fn dependencies<'src, N, Id: Clone>(
    program: &HashMap<Name<'src>, &Melody<'_, 'src, N, Id>>,
) -> Result<HashMap<Name<'src>, HashSet<Name<'src>>>, Vec<check::Error<'src, Id>>> {
    let mut errs = Vec::new();
    let program = program
        .iter()
        .map(|(name, melody)| {
            let mut refers = HashSet::new();
            if let Err(es) = compute(program, &mut refers, melody) {
                errs.extend(es);
            }
            (*name, refers)
        })
        .collect();

    errs.is_empty().then_some(program).ok_or(errs)
}

/// Add the names referred to by `melody` to `within`.
fn compute<'src, N, Id: Clone>(
    program: &HashMap<Name<'src>, &Melody<'_, 'src, N, Id>>,
    within: &mut HashSet<Name<'src>>,
    melody: &Melody<'_, 'src, N, Id>,
) -> Result<(), Vec<check::Error<'src, Id>>> {
    match melody {
        Melody::Pause(_) | Melody::Note(..) => Ok(()),

        Melody::Name(span, name) => {
            within.insert(*name);

            if !program.contains_key(name) {
                Err(vec![check::Error::UnknownName(span.clone(), name.0)])
            } else {
                Ok(())
            }
        }

        Melody::Scale(_, _, melody) => compute(program, within, melody),
        Melody::Sharp(_, _, melody) => compute(program, within, melody),
        Melody::Offset(_, _, melody) => compute(program, within, melody),

        Melody::Sequence(melodies) | Melody::Stack(melodies) => {
            let mut errs = vec![];
            for melody in *melodies {
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
    use crate::span::span;
    use crate::Name;

    use super::dependencies;

    #[test]
    fn disjoint() {
        let a = Melody::Pause(span());
        let b = Melody::Note(span(), 'b');
        let c = Melody::Note(span(), 'c');

        let seq = [b, c];
        let seq = Melody::Sequence(&seq);

        let program = HashMap::from([(Name("a"), &a), (Name("b"), &seq)]);
        let expected = HashMap::from([(Name("a"), HashSet::new()), (Name("b"), HashSet::new())]);

        let actual = dependencies(&program);

        assert_eq!(Ok(expected), actual);
    }

    #[test]
    fn chain() {
        let a: Melody<char, _> = Melody::Pause(span());
        let b = Melody::Name(span(), Name("a"));
        let c = Melody::Name(span(), Name("b"));

        let program = HashMap::from([(Name("a"), &a), (Name("b"), &b), (Name("c"), &c)]);

        let expected = HashMap::from([
            (Name("a"), HashSet::new()),
            (Name("b"), HashSet::from([Name("a")])),
            (Name("c"), HashSet::from([Name("b")])),
        ]);

        let actual = dependencies(&program);

        assert_eq!(Ok(expected), actual);
    }

    #[test]
    fn fork_join() {
        let a: Melody<char, _> = Melody::Pause(span());
        let to_a = Melody::Name(span(), Name("a"));
        let to_b = Melody::Name(span(), Name("b"));
        let to_c = Melody::Name(span(), Name("c"));

        let seq = [to_b, to_c];
        let d = Melody::Sequence(&seq);

        let program = HashMap::from([
            (Name("a"), &a),
            (Name("b"), &to_a),
            (Name("c"), &to_a),
            (Name("d"), &d),
        ]);

        let expected = HashMap::from([
            (Name("a"), HashSet::new()),
            (Name("b"), HashSet::from([Name("a")])),
            (Name("c"), HashSet::from([Name("a")])),
            (Name("d"), HashSet::from([Name("b"), Name("c")])),
        ]);

        let actual = dependencies(&program);

        assert_eq!(Ok(expected), actual);
    }

    #[test]
    fn cycles() {
        let a: Melody<char, _> = Melody::Name(span(), Name("c"));
        let b = Melody::Name(span(), Name("a"));
        let c = Melody::Name(span(), Name("b"));

        let program = HashMap::from([(Name("a"), &a), (Name("b"), &b), (Name("c"), &c)]);

        let expected = HashMap::from([
            (Name("a"), HashSet::from([Name("c")])),
            (Name("b"), HashSet::from([Name("a")])),
            (Name("c"), HashSet::from([Name("b")])),
        ]);

        let actual = dependencies(&program);

        assert_eq!(Ok(expected), actual);
    }
}
