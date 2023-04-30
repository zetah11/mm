use std::collections::{HashMap, HashSet};

use crate::implicit::Melody;
use crate::Name;

/// Compute the dependency graph of the given program. Each returned entry
/// contains "outgoing" edges: `a` is in the set of names referred to by `b` if
/// the definition of `b` refers to `a` at any place.
pub fn dependencies<'src, N>(
    program: &HashMap<Name<'src>, &Melody<'_, 'src, N>>,
) -> HashMap<Name<'src>, HashSet<Name<'src>>> {
    program
        .iter()
        .map(|(name, melody)| {
            let mut refers = HashSet::new();
            compute(&mut refers, melody);
            (*name, refers)
        })
        .collect()
}

/// Add the names referred to by `melody` to `within`.
fn compute<'src, N>(within: &mut HashSet<Name<'src>>, melody: &Melody<'_, 'src, N>) {
    match melody {
        Melody::Pause(_) | Melody::Note(..) => {}

        Melody::Name(_, name) => {
            within.insert(*name);
        }

        Melody::Scale(_, _, melody) => compute(within, melody),
        Melody::Sharp(_, _, melody) => compute(within, melody),
        Melody::Offset(_, _, melody) => compute(within, melody),

        Melody::Sequence(melodies) | Melody::Stack(melodies) => {
            for melody in *melodies {
                compute(within, melody);
            }
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

        assert_eq!(expected, actual);
    }

    #[test]
    fn chain() {
        let a: Melody<char> = Melody::Pause(span());
        let b = Melody::Name(span(), Name("a"));
        let c = Melody::Name(span(), Name("b"));

        let program = HashMap::from([(Name("a"), &a), (Name("b"), &b), (Name("c"), &c)]);

        let expected = HashMap::from([
            (Name("a"), HashSet::new()),
            (Name("b"), HashSet::from([Name("a")])),
            (Name("c"), HashSet::from([Name("b")])),
        ]);

        let actual = dependencies(&program);

        assert_eq!(expected, actual);
    }

    #[test]
    fn fork_join() {
        let a: Melody<char> = Melody::Pause(span());
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

        assert_eq!(expected, actual);
    }

    #[test]
    fn cycles() {
        let a: Melody<char> = Melody::Name(span(), Name("c"));
        let b = Melody::Name(span(), Name("a"));
        let c = Melody::Name(span(), Name("b"));

        let program = HashMap::from([(Name("a"), &a), (Name("b"), &b), (Name("c"), &c)]);

        let expected = HashMap::from([
            (Name("a"), HashSet::from([Name("c")])),
            (Name("b"), HashSet::from([Name("a")])),
            (Name("c"), HashSet::from([Name("b")])),
        ]);

        let actual = dependencies(&program);

        assert_eq!(expected, actual);
    }
}
