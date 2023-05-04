use std::collections::HashSet;

use crate::note::Note;
use crate::{implicit, melody, Allocator, Length, Name};

use super::{Checker, Error};

impl<'a, N, Id, A> Checker<'a, N, Id, A>
where
    N: Note,
    Id: Clone + Eq,
    A: Allocator<implicit::Melody<N, Id, A>>,
    A: Allocator<melody::Melody<N, Id, A>>,
{
    pub fn lower(
        &mut self,
        component: &HashSet<&Name>,
        melody: &implicit::Melody<N, Id, A>,
    ) -> melody::Melody<N, Id, A> {
        let span = melody.span();

        let (node, length) = match melody {
            implicit::Melody::Pause(_) => (melody::Node::Pause, Length::one()),
            implicit::Melody::Note(_, note) => (melody::Node::Note(note.clone()), Length::one()),

            implicit::Melody::Name(_, name) => {
                let var = self
                    .context
                    .get(name)
                    .expect("all names are given variables");

                let length = self
                    .lengths
                    .get(var)
                    .expect("unknown names are reported before checking");

                if component.contains(name) && !length.is_unbounded() {
                    (melody::Node::Recur(*name), length.clone())
                } else {
                    (melody::Node::Name(*name), length.clone())
                }
            }

            implicit::Melody::Scale(_, by, melody) => {
                let melody = self.lower(component, A::as_ref(melody));
                let length = by * &melody.length;
                let melody = self.alloc.pack(melody);

                (melody::Node::Scale(by.clone(), melody), length)
            }

            implicit::Melody::Sharp(_, by, melody) => {
                let melody = self.lower(component, A::as_ref(melody));
                let length = melody.length.clone();
                let melody = self.alloc.pack(melody);

                (melody::Node::Sharp(*by, melody), length)
            }

            implicit::Melody::Offset(_, by, melody) => {
                let melody = self.lower(component, A::as_ref(melody));
                let length = melody.length.clone();
                let melody = self.alloc.pack(melody);

                (melody::Node::Offset(*by, melody), length)
            }

            implicit::Melody::Sequence(melodies) => {
                let melodies: Vec<_> = A::as_slice(melodies)
                    .iter()
                    .map(|melody| self.lower(component, melody))
                    .collect();

                for melody in melodies.iter().rev().skip(1) {
                    if melody.length.is_unbounded() {
                        self.errors
                            .push(Error::UnboundedNotLast(melody.span.clone()));
                    }
                }

                let length = melodies
                    .iter()
                    .map(|melody| &melody.length)
                    .fold(Length::zero(), |a, b| &a + b);

                let melodies = self.alloc.pack_many(melodies);

                (melody::Node::Sequence(melodies), length)
            }

            implicit::Melody::Stack(melodies) => {
                let melodies: Vec<_> = A::as_slice(melodies)
                    .iter()
                    .map(|melody| self.lower(component, melody))
                    .collect();

                let length = melodies
                    .iter()
                    .map(|melody| melody.length.clone())
                    .max()
                    .unwrap_or_else(Length::zero);

                let melodies = self.alloc.pack_many(melodies);

                (melody::Node::Stack(melodies), length)
            }
        };

        melody::Melody { node, span, length }
    }
}
