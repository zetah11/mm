mod lex;
mod rules;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use logos::{Logos, SpannedIter};
use typed_arena::Arena;

use crate::implicit::Melody;
use crate::note::Note;
use crate::Name;

use self::lex::Token;

pub struct Parser<'a, 'src, N> {
    arena: &'a Arena<Melody<'a, N>>,
    lexer: SpannedIter<'src, Token<'src>>,
    next: Option<Token<'src>>,
}

impl<'a, 'src, N: Note> Parser<'a, 'src, N> {
    pub fn parse(
        arena: &'a Arena<Melody<'a, N>>,
        source: &'src str,
    ) -> HashMap<Name, &'a Melody<'a, N>> {
        let mut parser = Self::new(arena, source);
        parser.advance();
        parser.parse_program()
    }

    fn new(arena: &'a Arena<Melody<'a, N>>, source: &'src str) -> Self {
        Self {
            arena,
            lexer: Token::lexer(source).spanned(),
            next: None,
        }
    }

    fn advance(&mut self) {
        self.next = None;

        for (next, _) in self.lexer.by_ref() {
            if let Ok(token) = next {
                self.next = Some(token);
                break;
            }
        }
    }

    fn peek(&self, m: impl Matcher) -> Option<()> {
        if let Some(next) = self.next.as_ref() {
            m.matches(next).then_some(())
        } else {
            None
        }
    }

    fn consume(&mut self, m: impl Matcher) -> Option<()> {
        if let Some(v) = self.peek(m) {
            self.advance();
            Some(v)
        } else {
            None
        }
    }
}

trait Matcher {
    fn matches(&self, token: &Token) -> bool;
}

impl Matcher for Token<'_> {
    fn matches(&self, token: &Token) -> bool {
        match (self, token) {
            (Token::Name(_), Token::Name(_)) => true,
            (Token::Number(_), Token::Number(_)) => true,

            _ => self == token,
        }
    }
}
