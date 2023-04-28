#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::melody::{Melody, Node};
use crate::{Factor, Length, Name, Time};

pub const DEFAULT_MAX_DEPTH: usize = 10;

pub struct Evaluator<'a> {
    program: HashMap<Name, &'a Melody<'a>>,
    entry: Name,
    max_depth: usize,
}

impl<'a> Evaluator<'a> {
    pub fn new(program: HashMap<Name, &'a Melody<'a>>, entry: Name) -> Self {
        Self {
            program,
            entry,
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    pub fn with_max_depth(self, max_depth: usize) -> Self {
        Self { max_depth, ..self }
    }

    pub fn iter(&self) -> impl Iterator<Item = (char, Time, Length)> + '_ {
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

struct NextMelody<'a> {
    melody: &'a Melody<'a>,
    depth: usize,
    start: Time,
    factor: Factor,
}

impl Eq for NextMelody<'_> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for NextMelody<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.start.eq(&other.start)
    }
}

impl PartialOrd for NextMelody<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NextMelody<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let Time(this) = self.start;
        let Time(other) = other.start;
        this.cmp(&other).reverse()
    }
}

struct Iter<'a> {
    evaluator: &'a Evaluator<'a>,
    queue: BinaryHeap<NextMelody<'a>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (char, Time, Length);

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
                    return Some((*note, start, length));
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
