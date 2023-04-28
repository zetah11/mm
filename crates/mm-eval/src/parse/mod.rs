mod lex;
mod rules;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use logos::{Logos, SpannedIter};
use typed_arena::Arena;

use crate::implicit::Melody;
use crate::note::Note;
use crate::span::Span;
use crate::Name;

use self::lex::Token;

pub struct Parser<'a, 'src, N> {
    source: &'src str,
    arena: &'a Arena<Melody<'a, 'src, N>>,
    lexer: SpannedIter<'src, Token<'src>>,
    next: Option<(Token<'src>, Span<'src>)>,
}

impl<'a, 'src, N: Note> Parser<'a, 'src, N> {
    pub fn parse(
        arena: &'a Arena<Melody<'a, 'src, N>>,
        source: &'src str,
    ) -> HashMap<Name, &'a Melody<'a, 'src, N>> {
        let mut parser = Self::new(arena, source);
        parser.advance();
        parser.parse_program()
    }

    fn new(arena: &'a Arena<Melody<'a, 'src, N>>, source: &'src str) -> Self {
        Self {
            source,
            arena,
            lexer: Token::lexer(source).spanned(),
            next: None,
        }
    }

    fn advance(&mut self) {
        self.next = None;

        for (next, span) in self.lexer.by_ref() {
            if let Ok(token) = next {
                self.next = Some((token, Span::new(self.source, span)));
                break;
            }
        }
    }

    fn peek(&self, m: impl Matcher) -> Option<Span<'src>> {
        if let Some((token, span)) = self.next.as_ref() {
            m.matches(token).then_some(*span)
        } else {
            None
        }
    }

    fn consume(&mut self, m: impl Matcher) -> Option<Span<'src>> {
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
