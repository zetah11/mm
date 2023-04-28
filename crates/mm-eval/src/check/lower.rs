use itertools::Itertools;

use crate::{implicit, melody, Factor, Length};

use super::{Checker, Error};

impl<'a> Checker<'a> {
    pub fn lower_melody(&self, melody: &implicit::Melody) -> Result<melody::Melody<'a>, Error> {
        self.lower(Factor::one(), melody)
    }

    fn lower(
        &self,
        factor: Factor,
        melody: &implicit::Melody,
    ) -> Result<melody::Melody<'a>, Error> {
        let (node, length) = match melody {
            implicit::Melody::Pause => (melody::Node::Pause, Length::one()),
            implicit::Melody::Note(note) => (melody::Node::Note(*note), Length::one()),

            implicit::Melody::Name(name) => {
                let var = self
                    .context
                    .get(name)
                    .expect("all names are solved before lowering");
                let length = self.lengths.get(var).expect("no variable is unsolved");
                (melody::Node::Name(name.clone()), *length)
            }

            implicit::Melody::Scale(by, melody) => {
                let melody = self.lower(*by * factor, melody)?;
                let melody = self.arena.alloc(melody);
                let length = *by * melody.length;
                (melody::Node::Scale(*by, melody), length)
            }

            implicit::Melody::Sequence(melodies) => {
                let melodies: Vec<_> = melodies
                    .iter()
                    .map(|melody| self.lower(factor, melody))
                    .try_collect()?;

                for melody in melodies.iter().rev().skip(1) {
                    if melody.length.is_unbounded() {
                        return Err(Error::UnboundedNotLast);
                    }
                }

                let melodies = self.arena.alloc_extend(melodies);
                let length = melodies.iter().map(|melody| melody.length).sum();

                (melody::Node::Sequence(melodies), length)
            }

            implicit::Melody::Stack(melodies) => {
                let melodies: Vec<_> = melodies
                    .iter()
                    .map(|melody| self.lower(factor, melody))
                    .try_collect()?;

                let melodies = self.arena.alloc_extend(melodies);
                let length = melodies
                    .iter()
                    .map(|melody| melody.length)
                    .max()
                    .unwrap_or_else(Length::zero);

                (melody::Node::Stack(melodies), length)
            }
        };

        Ok(melody::Melody { node, length })
    }
}
