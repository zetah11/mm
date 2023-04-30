use std::collections::HashSet;

use crate::note::Note;
use crate::{implicit, melody, Length, Name};

use super::{Checker, Error};

impl<'a, 'src, N: Note> Checker<'a, 'src, N> {
    pub fn lower(
        &mut self,
        component: &HashSet<&Name<'src>>,
        melody: &implicit::Melody<'_, 'src, N>,
    ) -> melody::Melody<'a, 'src, N> {
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
                    Some(length) if component.contains(name) && !length.is_unbounded() => {
                        (melody::Node::Recur(*name), length.clone())
                    }

                    Some(length) => (melody::Node::Name(*name), length.clone()),
                    None => {
                        self.errors.push(Error::UnknownName(span, name.0));
                        (melody::Node::Pause, Length::one())
                    }
                }
            }

            implicit::Melody::Scale(_, by, melody) => {
                let melody = self.lower(component, melody);
                let melody = self.arena.alloc(melody);
                let length = by * &melody.length;

                (melody::Node::Scale(by.clone(), melody), length)
            }

            implicit::Melody::Sharp(_, by, melody) => {
                let melody = self.lower(component, melody);
                let melody = self.arena.alloc(melody);
                let length = melody.length.clone();

                (melody::Node::Sharp(*by, melody), length)
            }

            implicit::Melody::Offset(_, by, melody) => {
                let melody = self.lower(component, melody);
                let melody = self.arena.alloc(melody);
                let length = melody.length.clone();

                (melody::Node::Offset(*by, melody), length)
            }

            implicit::Melody::Sequence(melodies) => {
                let melodies: Vec<_> = melodies
                    .iter()
                    .map(|melody| self.lower(component, melody))
                    .collect();

                for melody in melodies.iter().rev().skip(1) {
                    if melody.length.is_unbounded() {
                        self.errors.push(Error::UnboundedNotLast(melody.span));
                    }
                }

                let melodies = self.arena.alloc_extend(melodies);

                let length = melodies
                    .iter()
                    .map(|melody| &melody.length)
                    .fold(Length::zero(), |a, b| &a + b);

                (melody::Node::Sequence(melodies), length)
            }

            implicit::Melody::Stack(melodies) => {
                let melodies: Vec<_> = melodies
                    .iter()
                    .map(|melody| self.lower(component, melody))
                    .collect();

                let melodies = self.arena.alloc_extend(melodies);

                let length = melodies
                    .iter()
                    .map(|melody| melody.length.clone())
                    .max()
                    .unwrap_or_else(Length::zero);

                (melody::Node::Stack(melodies), length)
            }
        };

        melody::Melody { node, span, length }
    }
}
