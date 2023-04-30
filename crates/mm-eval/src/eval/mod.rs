#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::melody::{Melody, Node};
use crate::note::Note;
use crate::span::Span;
use crate::{Factor, Length, Name, Time};

pub const DEFAULT_MAX_DEPTH: usize = 10;

pub struct Evaluator<'a, 'src, N> {
    program: HashMap<Name<'src>, &'a Melody<'a, 'src, N>>,
    entry: Name<'src>,
    max_depth: usize,
}

impl<'a, 'src, N: Note> Evaluator<'a, 'src, N> {
    pub fn new(program: HashMap<Name<'src>, &'a Melody<'a, 'src, N>>, entry: Name<'src>) -> Self {
        Self {
            program,
            entry,
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }

    pub fn with_max_depth(self, max_depth: usize) -> Self {
        Self { max_depth, ..self }
    }

    pub fn iter(&self) -> impl Iterator<Item = (N, Span<'src>, Time, Length)> + '_ {
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
                offset: 0,
                sharps: 0,
            }]),
        }
    }
}

struct NextMelody<'a, 'src, N> {
    melody: &'a Melody<'a, 'src, N>,
    depth: usize,
    start: Time,

    factor: Factor,
    offset: isize,
    sharps: usize,
}

impl<N> Eq for NextMelody<'_, '_, N> {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<N> PartialEq for NextMelody<'_, '_, N> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl<N> PartialOrd for NextMelody<'_, '_, N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<N> Ord for NextMelody<'_, '_, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        let Time(this) = &self.start;
        let Time(other) = &other.start;
        this.cmp(other).reverse()
    }
}

struct Iter<'a, 'src, N> {
    evaluator: &'a Evaluator<'a, 'src, N>,
    queue: BinaryHeap<NextMelody<'a, 'src, N>>,
}

impl<'a, 'src, N: Note> Iterator for Iter<'a, 'src, N> {
    type Item = (N, Span<'src>, Time, Length);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.queue.pop() {
            let start = next.start;
            let depth = next.depth;
            let factor = next.factor;
            let offset = next.offset;
            let sharps = next.sharps;
            let span = next.melody.span;

            if depth >= self.evaluator.max_depth {
                continue;
            }

            match &next.melody.node {
                Node::Pause => {}
                Node::Note(note) => {
                    let length = &next.melody.length * &factor;
                    let note = note.add_octave(offset).add_sharp(sharps);
                    return Some((note, span, start, length));
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
                        offset,
                        sharps,
                    });
                }

                Node::Scale(scale, melody) => {
                    let factor = Factor(factor.0 * scale.0.clone());
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
                    for melody in *melodies {
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
                    for melody in *melodies {
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
