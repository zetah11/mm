use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Produce a topological ordering of a graph given in terms of sets of
/// dependencies.
pub fn order<T>(graph: &HashMap<T, HashSet<T>>) -> Vec<HashSet<&T>>
where
    T: Clone + Hash + Eq,
{
    let mut finder = ComponentFinder::new();

    for key in graph.keys() {
        finder.connect(graph, key);
    }

    finder.components
}

struct ComponentFinder<'a, T> {
    index: usize,

    indicies: HashMap<&'a T, usize>,
    lowlinks: HashMap<&'a T, usize>,

    stack: Vec<&'a T>,
    on_stack: HashSet<&'a T>,

    visited: HashSet<&'a T>,
    components: Vec<HashSet<&'a T>>,
}

impl<'a, T> ComponentFinder<'a, T>
where
    T: Clone + Eq + Hash,
{
    fn new() -> Self {
        Self {
            index: 0,
            indicies: HashMap::new(),
            lowlinks: HashMap::new(),
            stack: Vec::new(),
            on_stack: HashSet::new(),

            visited: HashSet::new(),
            components: Vec::new(),
        }
    }

    fn connect(&mut self, graph: &'a HashMap<T, HashSet<T>>, vertex: &'a T) {
        if !self.visited.insert(vertex) {
            return;
        }

        self.indicies.insert(vertex, self.index);
        self.lowlinks.insert(vertex, self.index);
        self.index += 1;

        self.stack.push(vertex);
        self.on_stack.insert(vertex);

        for child in graph.get(vertex).into_iter().flatten() {
            if !self.indicies.contains_key(child) {
                self.connect(graph, child);
                let lowlink = *self
                    .lowlinks
                    .get(&vertex)
                    .unwrap()
                    .min(self.lowlinks.get(child).unwrap());
                self.lowlinks.insert(vertex, lowlink);
            } else if self.on_stack.contains(child) {
                let lowlink = *self
                    .lowlinks
                    .get(&vertex)
                    .unwrap()
                    .min(self.indicies.get(child).unwrap());
                self.lowlinks.insert(vertex, lowlink);
            }
        }

        if self.lowlinks.get(&vertex) == self.indicies.get(&vertex) {
            let mut component = HashSet::new();
            while let Some(child) = self.stack.pop() {
                self.on_stack.remove(&child);
                component.insert(child);

                if child == vertex {
                    break;
                }
            }

            self.components.push(component);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::order;

    #[test]
    fn chain() {
        let graph = HashMap::from([
            (0, HashSet::new()),
            (1, HashSet::from([0])),
            (2, HashSet::from([1])),
        ]);

        let expected = vec![
            HashSet::from([&0]),
            HashSet::from([&1]),
            HashSet::from([&2]),
        ];

        let actual = order(&graph);

        assert_eq!(expected, actual);
    }

    #[test]
    fn cycle() {
        let graph = HashMap::from([
            (0, HashSet::from([2])),
            (1, HashSet::from([0])),
            (2, HashSet::from([1])),
        ]);

        let expected = vec![HashSet::from([&0, &1, &2])];

        let actual = order(&graph);

        assert_eq!(expected, actual);
    }

    #[test]
    fn depend_on_cycle() {
        let graph = HashMap::from([
            (0, HashSet::new()),
            (1, HashSet::from([2, 0])),
            (2, HashSet::from([1])),
            (3, HashSet::from([1])),
        ]);

        let expected = vec![
            HashSet::from([&0]),
            HashSet::from([&1, &2]),
            HashSet::from([&3]),
        ];

        let actual = order(&graph);

        assert_eq!(expected, actual);
    }

    #[test]
    fn disjoint() {
        let graph = HashMap::from([
            (0, HashSet::new()),
            (1, HashSet::new()),
            (2, HashSet::new()),
        ]);

        let actual = order(&graph);

        assert_eq!(3, actual.len());
        assert!(actual.contains(&HashSet::from([&0])));
        assert!(actual.contains(&HashSet::from([&1])));
        assert!(actual.contains(&HashSet::from([&2])));
    }
}
