use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::ToPrimitive;

use super::lex::Token;
use super::{Error, Parser};
use crate::implicit::{Melody, Program};
use crate::note::Note;
use crate::span::Span;
use crate::{Factor, Name};

impl<'a, 'src, N: Note> Parser<'a, 'src, N> {
    pub(super) fn parse_program(&mut self) -> Program<'a, 'src, N> {
        let mut program = Program::new(self.span);

        while self.next.is_some() {
            let Some((name, name_span, is_public, body)) = self.definition() else { continue; };

            if let Some(previous) = program.spans.get(&name).copied() {
                self.errors.push(Error::Redefinition {
                    previous,
                    new: name_span,
                });
            }

            program.defs.insert(name, body);
            program.spans.insert(name, name_span);

            if is_public {
                program.public.push(name);
            }
        }

        program
    }

    fn definition(&mut self) -> Option<(Name<'src>, Span<'src>, bool, &'a Melody<'a, 'src, N>)> {
        let (name, name_span) = match self.advance() {
            Some((Token::Name(name), span)) => {
                if N::parse(name).is_some() {
                    self.errors.push(Error::ExpectedName(span));
                }

                (Name(name), span)
            }

            _ => {
                self.errors.push(Error::ExpectedName(self.span));
                return None;
            }
        };

        let is_public = self.consume(Token::Exclaim).is_some();

        if self.consume(Token::Equal).is_none() {
            self.errors.push(Error::ExpectedEqual(self.span));
            return None;
        }

        let body = self.expression();
        let body = self.arena.alloc(body);
        Some((name, name_span, is_public, body))
    }

    fn expression(&mut self) -> Melody<'a, 'src, N> {
        self.stack()
    }

    fn stack(&mut self) -> Melody<'a, 'src, N> {
        let mut melodies = vec![self.sequence()];

        while self.consume(Token::Pipe).is_some() {
            melodies.push(self.sequence());
        }

        if melodies.len() == 1 {
            melodies.remove(0)
        } else {
            let melodies = self.arena.alloc_extend(melodies);
            Melody::Stack(melodies)
        }
    }

    fn sequence(&mut self) -> Melody<'a, 'src, N> {
        let mut melodies = vec![self.scale()];

        while self.consume(Token::Comma).is_some() {
            melodies.push(self.scale());
        }

        if melodies.len() == 1 {
            melodies.remove(0)
        } else {
            let melodies = self.arena.alloc_extend(melodies);
            Melody::Sequence(melodies)
        }
    }

    fn scale(&mut self) -> Melody<'a, 'src, N> {
        let mut melody = if self.peek(Token::Number("")).is_some() {
            let (by, factor_span) = self.factor();
            let melody = self.simple();
            let melody = self.arena.alloc(melody);
            Melody::Scale(factor_span, by, melody)
        } else {
            self.simple()
        };

        let mut sharps = 0;
        let mut sharp_span = None;
        while let Some((_, span)) = self.consume(Token::Sharp) {
            sharps += 1;
            if let Some(sharp_span) = sharp_span.as_mut() {
                *sharp_span = *sharp_span + span;
            } else {
                sharp_span = Some(span);
            }
        }

        if let Some(sharp_span) = sharp_span {
            let inner = self.arena.alloc(melody);
            melody = Melody::Sharp(sharp_span, sharps, inner);
        }

        if self.peek([Token::Minus, Token::Plus]).is_some() {
            let (offset, offset_span) = self.offset();
            let melody = self.arena.alloc(melody);
            Melody::Offset(offset_span, offset, melody)
        } else {
            melody
        }
    }

    fn simple(&mut self) -> Melody<'a, 'src, N> {
        let melody = match self.advance() {
            Some((Token::Name(n), span)) => match N::parse(n) {
                Some(note) => Melody::Note(span, note),
                None => Melody::Name(span, Name(n)),
            },

            Some((Token::Pause, span)) => Melody::Pause(span),

            Some((Token::LeftParen, opener)) => {
                let melody = self.expression();

                if self.consume(Token::RightParen).is_none() {
                    self.errors.push(Error::UnclosedParen {
                        opener,
                        at: self.span,
                    });
                }

                melody
            }

            _ => {
                self.errors.push(Error::ExpectedNote(self.span));
                Melody::Pause(self.span)
            }
        };

        melody
    }

    fn offset(&mut self) -> (isize, Span<'src>) {
        let (sign, span) = match self.advance() {
            Some((Token::Plus, span)) => (1, span),
            Some((Token::Minus, span)) => (-1, span),
            _ => unreachable!(),
        };

        let (num, span): (BigInt, _) =
            if let Some((Token::Number(s), num_span)) = self.consume(Token::Number("")) {
                (sign * Self::parse_int(s), num_span + span)
            } else {
                todo!()
            };

        if let Some(offset) = num.to_isize() {
            (offset, span)
        } else {
            todo!()
        }
    }

    fn factor(&mut self) -> (Factor, Span<'src>) {
        let (first, mut span) = match self.advance() {
            Some((Token::Number(s), span)) => (Self::parse_int(s), span),
            _ => unreachable!(),
        };

        let second = if self.consume(Token::Slash).is_some() {
            let (mut num, second_span) =
                if let Some((Token::Number(s), span)) = self.consume(Token::Number("")) {
                    (Self::parse_int(s), span)
                } else {
                    self.errors.push(Error::ExpectedNumber(self.span));
                    (BigInt::from(1), span)
                };

            if num == BigInt::from(0) {
                self.errors.push(Error::DivisionByZero(second_span));
                num = BigInt::from(1);
            }

            span = span + second_span;
            num
        } else {
            BigInt::from(1)
        };

        (Factor(BigRational::new(first, second)), span)
    }

    fn parse_int(s: &str) -> BigInt {
        let mut res = BigInt::from(0);
        for c in s.chars() {
            let v = match c {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                '8' => 8,
                '9' => 9,
                _ => continue,
            };

            res = res * 10 + v;
        }
        res
    }
}
