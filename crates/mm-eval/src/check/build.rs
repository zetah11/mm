use super::equation::Variable;
use super::Checker;
use crate::check::equation::{Sum, Term};
use crate::implicit::Melody;
use crate::note::Note;
use crate::{Factor, Length};

impl<N: Note, Id> Checker<'_, '_, N, Id> {
    pub fn build_equation(&self, melody: &Melody<N, Id>) -> Vec<Sum> {
        self.build(&Factor::one(), melody)
    }

    fn build(&self, factor: &Factor, melody: &Melody<N, Id>) -> Vec<Sum> {
        match melody {
            Melody::Pause(_) => Self::constant(factor * &Length::one()),
            Melody::Note(_, _) => Self::constant(factor * &Length::one()),

            Melody::Name(_, name) => Self::variable(
                factor.clone(),
                *self
                    .context
                    .get(name)
                    .expect("all names are bound to a var before use"),
            ),

            Melody::Scale(_, scale, melody) => {
                let factor = factor * scale;
                self.build(&factor, melody)
            }

            Melody::Sharp(_, _, melody) => self.build(factor, melody),
            Melody::Offset(_, _, melody) => self.build(factor, melody),

            Melody::Sequence(melodies) => {
                Self::sum(melodies.iter().map(|melody| self.build(factor, melody)))
            }

            Melody::Stack(melodies) => {
                Self::max(melodies.iter().map(|melody| self.build(factor, melody)))
            }
        }
    }

    fn constant(length: Length) -> Vec<Sum> {
        vec![Sum {
            terms: vec![Term::Constant(length)],
        }]
    }

    fn variable(factor: Factor, var: Variable) -> Vec<Sum> {
        vec![Sum {
            terms: vec![Term::Variable(factor, var)],
        }]
    }

    fn max(parts: impl IntoIterator<Item = Vec<Sum>>) -> Vec<Sum> {
        parts.into_iter().flatten().collect()
    }

    fn sum(parts: impl IntoIterator<Item = Vec<Sum>>) -> Vec<Sum> {
        parts
            .into_iter()
            .reduce(|a, b| {
                let mut result = Vec::with_capacity(a.len() * b.len());

                for sum1 in a {
                    for sum2 in b.iter() {
                        let terms = sum1
                            .terms
                            .iter()
                            .chain(sum2.terms.iter())
                            .cloned()
                            .collect();

                        result.push(Sum { terms });
                    }
                }

                result
            })
            .unwrap_or_default()
    }
}
