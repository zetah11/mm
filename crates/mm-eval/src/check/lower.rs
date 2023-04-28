use crate::note::Note;
use crate::{implicit, melody, Length};

use super::{Checker, Error};

impl<'a, 'src, N: Note> Checker<'a, 'src, N> {
    pub fn lower(&mut self, melody: &implicit::Melody<'_, 'src, N>) -> melody::Melody<'a, 'src, N> {
        let span = melody.span();

        let (node, length) = match melody {
            implicit::Melody::Pause(_) => (melody::Node::Pause, Length::one()),
            implicit::Melody::Note(_, note) => (melody::Node::Note(note.clone()), Length::one()),

            implicit::Melody::Name(_, name) => {
                let var = self
                    .context
                    .get(name)
                    .expect("all names are given variables");

                match self.lengths.get(var) {
                    Some(length) => (melody::Node::Name(name.clone()), *length),
                    None => {
                        self.errors.push(Error::UnknownName(span, name.0.clone()));
                        (melody::Node::Pause, Length::one())
                    }
                }
            }

            implicit::Melody::Scale(_, by, melody) => {
                let melody = self.lower(melody);
                let melody = self.arena.alloc(melody);
                let length = *by * melody.length;

                (melody::Node::Scale(*by, melody), length)
            }

            implicit::Melody::Sequence(melodies) => {
                let melodies: Vec<_> = melodies.iter().map(|melody| self.lower(melody)).collect();

                for melody in melodies.iter().rev().skip(1) {
                    if melody.length.is_unbounded() {
                        self.errors.push(Error::UnboundedNotLast(melody.span));
                    }
                }

                let melodies = self.arena.alloc_extend(melodies);

                let length = melodies.iter().map(|melody| melody.length).sum();

                (melody::Node::Sequence(melodies), length)
            }

            implicit::Melody::Stack(melodies) => {
                let melodies: Vec<_> = melodies.iter().map(|melody| self.lower(melody)).collect();

                let melodies = self.arena.alloc_extend(melodies);

                let length = melodies
                    .iter()
                    .map(|melody| melody.length)
                    .max()
                    .unwrap_or_else(Length::zero);

                (melody::Node::Stack(melodies), length)
            }
        };

        melody::Melody { node, span, length }
    }
}
