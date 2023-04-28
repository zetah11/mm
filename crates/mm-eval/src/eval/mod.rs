#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::melody::{Melody, Node};
use crate::note::Note;
use crate::{Factor, Length, Name, Time};

pub const DEFAULT_MAX_DEPTH: usize = 10;

pub struct Evaluator<'a, N> {
    program: HashMap<Name, &'a Melody<'a, N>>,
    entry: Name,
    max_depth: usize,
}

impl<'a, N: Note> Evaluator<'a, N> {
    pub fn new(program: HashMap<Name, &'a Melody<'a, N>>, entry: Name) -> Self {
        Self {
            program,
            entry,
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    pub fn with_max_depth(self, max_depth: usize) -> Self {
        Self { max_depth, ..self }
    }

    pub fn iter(&self) -> impl Iterator<Item = (N, Time, Length)> + '_ {
        let melody = *self.program.get(&self.entry).expect("entry exists");
        let start = Time::zero();
        let factor = Factor::one();

        Iter {
            evaluator: self,
            queue: BinaryHeap::from([NextMelody {
                melody,
                depth: 0,
                start,
                factor,
            }]),
        }
    }
}

struct NextMelody<'a, N> {
    melody: &'a Melody<'a, N>,
    depth: usize,
    start: Time,
    factor: Factor,
}

impl<N> Eq for NextMelody<'_, N> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<N> PartialEq for NextMelody<'_, N> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<N> PartialOrd for NextMelody<'_, N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<N> Ord for NextMelody<'_, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        let Time(this) = self.start;
        let Time(other) = other.start;
        this.cmp(&other).reverse()
    }
}

struct Iter<'a, N> {
    evaluator: &'a Evaluator<'a, N>,
    queue: BinaryHeap<NextMelody<'a, N>>,
}

impl<'a, N: Note> Iterator for Iter<'a, N> {
    type Item = (N, Time, Length);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.queue.pop() {
            let start = next.start;
            let depth = next.depth;
            let factor = next.factor;

            if depth >= self.evaluator.max_depth {
                continue;
            }

            match &next.melody.node {
                Node::Pause => {}
                Node::Note(note) => {
                    let length = next.melody.length * factor;
                    return Some((note.clone(), start, length));
                }

                Node::Name(name) => {
                    let melody = *self
                        .evaluator
                        .program
                        .get(name)
                        .expect("all names are defined");

                    let depth = if !melody.length.is_unbounded() {
                        depth + 1
                    } else {
                        depth
                    };

                    self.queue.push(NextMelody {
                        melody,
                        depth,
                        start,
                        factor,
                    });
                }

                Node::Scale(scale, melody) => {
                    let factor = Factor(factor.0 * scale.0);
                    self.queue.push(NextMelody {
                        melody,
                        depth,
                        start,
                        factor,
                    });
                }

                Node::Sequence(melodies) => {
                    let mut start = start;
                    for melody in *melodies {
                        let length = melody.length;
                        self.queue.push(NextMelody {
                            melody,
                            depth,
                            start,
                            factor,
                        });

                        if matches!(length, Length::Unbounded) {
                            break;
                        }

                        start = start + factor * length;
                    }
                }

                Node::Stack(melodies) => {
                    for melody in *melodies {
                        self.queue.push(NextMelody {
                            melody,
                            depth,
                            start,
                            factor,
                        });
                    }
                }
            }
        }

        None
    }
}
