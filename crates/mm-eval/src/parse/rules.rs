use std::collections::HashMap;

use rational::Rational;

use super::lex::Token;
use super::Parser;
use crate::implicit::Melody;
use crate::note::Note;
use crate::{Factor, Name};

impl<'a, N: Note> Parser<'a, '_, N> {
    pub(super) fn parse_program(&mut self) -> HashMap<Name, &'a Melody<'a, N>> {
        let mut program = HashMap::new();

        while self.next.is_some() {
            let (name, body) = self.definition();
            program.insert(name, body);
        }

        program
    }

    fn definition(&mut self) -> (Name, &'a Melody<'a, N>) {
        let name = match self.next {
            Some(Token::Name(name)) => Name(name.into()),
            Some(Token::It) => Self::it(),

            _ => todo!(),
        };

        self.advance();

        if self.consume(Token::Equal).is_none() {
            todo!()
        }

        let body = self.expression();
        let body = self.arena.alloc(body);
        (name, body)
    }

    fn expression(&mut self) -> Melody<'a, N> {
        self.stack()
    }

    fn stack(&mut self) -> Melody<'a, N> {
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

    fn sequence(&mut self) -> Melody<'a, N> {
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

    fn scale(&mut self) -> Melody<'a, N> {
        if self.peek(Token::Number("")).is_some() {
            let by = self.factor();
            let melody = self.simple();
            let melody = self.arena.alloc(melody);
            Melody::Scale(by, melody)
        } else {
            self.simple()
        }
    }

    fn simple(&mut self) -> Melody<'a, N> {
        let melody = match self.next {
            Some(Token::Name(n)) => match N::parse(n) {
                Some(note) => Melody::Note(note),
                None => Melody::Name(Name(n.into())),
            },

            Some(Token::Pause) => Melody::Pause,
            Some(Token::It) => Melody::Name(Self::it()),

            Some(Token::LeftParen) => {
                self.advance();
                let melody = self.expression();

                if self.consume(Token::RightParen).is_none() {
                    todo!()
                }

                return melody;
            }

            _ => todo!("{:?}", self.next),
        };

        self.advance();
        melody
    }

    fn factor(&mut self) -> Factor {
        let first = match self.next {
            Some(Token::Number(s)) => Self::parse_int(s),

            _ => unreachable!(),
        };

        self.advance();

        let second = if self.consume(Token::Slash).is_some() {
            let num = match self.next {
                Some(Token::Number(s)) => Self::parse_int(s),
                _ => todo!(),
            };

            self.advance();
            num
        } else {
            1
        };

        Factor(Rational::new(first, second))
    }

    fn parse_int(s: &str) -> i128 {
        let mut res = 0;
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

    fn it() -> Name {
        Name("it".into())
    }
}
