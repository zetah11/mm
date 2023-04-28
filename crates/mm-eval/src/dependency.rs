use std::collections::{HashMap, HashSet};

use crate::implicit::Melody;
use crate::Name;

/// Compute the dependency graph of the given program. Each returned entry
/// contains "outgoing" edges: `a` is in the set of names referred to by `b` if
/// the definition of `b` refers to `a` at any place.
pub fn dependencies<N>(program: &HashMap<Name, &Melody<N>>) -> HashMap<Name, HashSet<Name>> {
    program
        .iter()
        .map(|(name, melody)| {
            let mut refers = HashSet::new();
            compute(&mut refers, melody);
            (name.clone(), refers)
        })
        .collect()
}

/// Add the names referred to by `melody` to `within`.
fn compute<N>(within: &mut HashSet<Name>, melody: &Melody<N>) {
    match melody {
        Melody::Pause | Melody::Note(_) => {}

        Melody::Name(name) => {
            within.insert(name.clone());
        }

        Melody::Scale(_, melody) => compute(within, melody),

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
    use crate::Name;

    use super::dependencies;

    #[test]
    fn disjoint() {
        let a = Melody::Pause;
        let b = Melody::Note('b');
        let c = Melody::Note('c');

        let seq = [b, c];
        let seq = Melody::Sequence(&seq);

        let program = HashMap::from([(Name("a".into()), &a), (Name("b".into()), &seq)]);

        let expected = HashMap::from([
            (Name("a".into()), HashSet::new()),
            (Name("b".into()), HashSet::new()),
        ]);

        let actual = dependencies(&program);

        assert_eq!(expected, actual);
    }

    #[test]
    fn chain() {
        let a: Melody<char> = Melody::Pause;
        let b = Melody::Name(Name("a".into()));
        let c = Melody::Name(Name("b".into()));

        let program = HashMap::from([
            (Name("a".into()), &a),
            (Name("b".into()), &b),
            (Name("c".into()), &c),
        ]);

        let expected = HashMap::from([
            (Name("a".into()), HashSet::new()),
            (Name("b".into()), HashSet::from([Name("a".into())])),
            (Name("c".into()), HashSet::from([Name("b".into())])),
        ]);

        let actual = dependencies(&program);

        assert_eq!(expected, actual);
    }

    #[test]
    fn fork_join() {
        let a: Melody<char> = Melody::Pause;
        let to_a = Melody::Name(Name("a".into()));
        let to_b = Melody::Name(Name("b".into()));
        let to_c = Melody::Name(Name("c".into()));

        let seq = [to_b, to_c];
        let d = Melody::Sequence(&seq);

        let program = HashMap::from([
            (Name("a".into()), &a),
            (Name("b".into()), &to_a),
            (Name("c".into()), &to_a),
            (Name("d".into()), &d),
        ]);

        let expected = HashMap::from([
            (Name("a".into()), HashSet::new()),
            (Name("b".into()), HashSet::from([Name("a".into())])),
            (Name("c".into()), HashSet::from([Name("a".into())])),
            (
                Name("d".into()),
                HashSet::from([Name("b".into()), Name("c".into())]),
            ),
        ]);

        let actual = dependencies(&program);

        assert_eq!(expected, actual);
    }

    #[test]
    fn cycles() {
        let a: Melody<char> = Melody::Name(Name("c".into()));
        let b = Melody::Name(Name("a".into()));
        let c = Melody::Name(Name("b".into()));

        let program = HashMap::from([
            (Name("a".into()), &a),
            (Name("b".into()), &b),
            (Name("c".into()), &c),
        ]);

        let expected = HashMap::from([
            (Name("a".into()), HashSet::from([Name("c".into())])),
            (Name("b".into()), HashSet::from([Name("a".into())])),
            (Name("c".into()), HashSet::from([Name("b".into())])),
        ]);

        let actual = dependencies(&program);

        assert_eq!(expected, actual);
    }
}
