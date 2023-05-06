#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use num_bigint::BigInt;
use num_rational::BigRational;

use crate::melody::{Melody, Node};
use crate::note::Note;
use crate::span::Span;
use crate::{Allocator, Factor, Length, Name, Time};

pub const DEFAULT_MAX_DEPTH: usize = 10;

pub struct Evaluator<'a, N, Id, A: Allocator<Melody<N, Id, A>>> {
    program: &'a HashMap<Name, A::Holder>,
    entry: Name,
    max_depth: usize,
    min_length: Length,
}

impl<'a, N, Id, A> Evaluator<'a, N, Id, A>
where
    N: Note,
    Id: Clone,
    A: Allocator<Melody<N, Id, A>>,
{
    pub fn new(program: &'a HashMap<Name, A::Holder>, entry: Name) -> Self {
        Self {
            program,
            entry,
            max_depth: DEFAULT_MAX_DEPTH,
            min_length: Length::Bounded(BigRational::new(BigInt::from(1), BigInt::from(512))),
        }
    }

    pub fn with_max_depth(self, max_depth: usize) -> Self {
        Self { max_depth, ..self }
    }

    pub fn with_min_length(self, min_length: Length) -> Self {
        Self { min_length, ..self }
    }

    pub fn iter(&self) -> impl Iterator<Item = (N, Span<Id>, Time, Length)> + '_ {
        let melody = self.program.get(&self.entry).expect("entry exists");
        let melody = A::as_ref(melody);
        let start = Time::zero();
        let factor = Factor::one();

        Iter {
            evaluator: self,
            queue: BinaryHeap::from([NextMelody {
                melody,
                depth: 0,
                start,

                factor,
                offset: 0,
                sharps: 0,
            }]),
        }
    }
}

struct NextMelody<'a, N, Id, A: Allocator<Melody<N, Id, A>>> {
    melody: &'a Melody<N, Id, A>,
    depth: usize,
    start: Time,

    factor: Factor,
    offset: isize,
    sharps: usize,
}

impl<N, Id, A: Allocator<Melody<N, Id, A>>> Eq for NextMelody<'_, N, Id, A> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<N, Id, A: Allocator<Melody<N, Id, A>>> PartialEq for NextMelody<'_, N, Id, A> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<N, Id, A: Allocator<Melody<N, Id, A>>> PartialOrd for NextMelody<'_, N, Id, A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<N, Id, A: Allocator<Melody<N, Id, A>>> Ord for NextMelody<'_, N, Id, A> {
    fn cmp(&self, other: &Self) -> Ordering {
        let Time(this) = &self.start;
        let Time(other) = &other.start;
        this.cmp(other).reverse()
    }
}

struct Iter<'a, N, Id, A: Allocator<Melody<N, Id, A>>> {
    evaluator: &'a Evaluator<'a, N, Id, A>,
    queue: BinaryHeap<NextMelody<'a, N, Id, A>>,
}

impl<'a, N, Id, A> Iterator for Iter<'a, N, Id, A>
where
    N: Note,
    Id: Clone,
    A: Allocator<Melody<N, Id, A>>,
{
    type Item = (N, Span<Id>, Time, Length);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.queue.pop() {
            let start = next.start;
            let depth = next.depth;
            let factor = next.factor;
            let offset = next.offset;
            let sharps = next.sharps;
            let melody = next.melody;
            let length = &melody.length * &factor;

            if depth >= self.evaluator.max_depth || length < self.evaluator.min_length {
                continue;
            }

            match &melody.node {
                Node::Pause => {}
                Node::Note(note) => {
                    let note = note.add_octave(offset).add_sharp(sharps);
                    return Some((note, melody.span.clone(), start, length));
                }

                Node::Recur(name) => {
                    let melody = self
                        .evaluator
                        .program
                        .get(name)
                        .expect("all names are defined");

                    let melody = A::as_ref(melody);
                    self.queue.push(NextMelody {
                        melody,
                        depth: depth + 1,
                        start,
                        factor,
                        offset,
                        sharps,
                    });
                }

                Node::Name(name) => {
                    let melody = self
                        .evaluator
                        .program
                        .get(name)
                        .expect("all names are defined");

                    let melody = A::as_ref(melody);
                    self.queue.push(NextMelody {
                        melody,
                        depth,
                        start,
                        factor,
                        offset,
                        sharps,
                    });
                }

                Node::Scale(scale, melody) => {
                    let factor = Factor(factor.0 * scale.0.clone());
                    let melody = A::as_ref(melody);
                    self.queue.push(NextMelody {
                        melody,
                        depth,
                        start,
                        factor,
                        offset,
                        sharps,
                    });
                }

                Node::Sharp(by, melody) => {
                    let sharps = sharps + *by;
                    let melody = A::as_ref(melody);
                    self.queue.push(NextMelody {
                        melody,
                        depth,
                        start,
                        factor,
                        offset,
                        sharps,
                    });
                }

                Node::Offset(by, melody) => {
                    let offset = offset + *by;
                    let melody = A::as_ref(melody);
                    self.queue.push(NextMelody {
                        melody,
                        depth,
                        start,
                        factor,
                        offset,
                        sharps,
                    });
                }

                Node::Sequence(melodies) => {
                    let mut start = start;
                    for melody in A::as_slice(melodies) {
                        let length = &melody.length;
                        self.queue.push(NextMelody {
                            melody,
                            depth,
                            start: start.clone(),
                            factor: factor.clone(),
                            offset,
                            sharps,
                        });

                        if matches!(length, Length::Unbounded) {
                            break;
                        }

                        start = &start + &(&factor * length);
                    }
                }

                Node::Stack(melodies) => {
                    for melody in A::as_slice(melodies) {
                        self.queue.push(NextMelody {
                            melody,
                            depth,
                            start: start.clone(),
                            factor: factor.clone(),
                            offset,
                            sharps,
                        });
                    }
                }
            }
        }

        None
    }
}
