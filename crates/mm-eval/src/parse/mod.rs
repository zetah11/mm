mod lex;
mod rules;

#[cfg(test)]
mod tests;

use logos::{Logos, SpannedIter};
use typed_arena::Arena;

use crate::implicit::{Melody, Program};
use crate::note::Note;
use crate::span::Span;

use self::lex::Token;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Error<'src> {
    ExpectedEqual(Span<'src>),
    ExpectedName(Span<'src>),
    ExpectedNote(Span<'src>),
    ExpectedNumber(Span<'src>),

    Redefinition {
        previous: Span<'src>,
        new: Span<'src>,
    },

    DivisionByZero(Span<'src>),
    UnclosedParen {
        opener: Span<'src>,
        at: Span<'src>,
    },
}

pub struct Parser<'a, 'src, N> {
    source: &'src str,
    arena: &'a Arena<Melody<'a, 'src, N>>,
    lexer: SpannedIter<'src, Token<'src>>,
    next: Option<(Token<'src>, Span<'src>)>,
    span: Span<'src>,

    errors: Vec<Error<'src>>,
}

impl<'a, 'src, N: Note> Parser<'a, 'src, N> {
    pub fn parse(
        arena: &'a Arena<Melody<'a, 'src, N>>,
        source: &'src str,
    ) -> Result<Program<'a, 'src, N>, Vec<Error<'src>>> {
        let mut parser = Self::new(arena, source);
        parser.advance();
        let parsed = parser.parse_program();
        if parser.errors.is_empty() {
            Ok(parsed)
        } else {
            Err(parser.errors)
        }
    }

    fn new(arena: &'a Arena<Melody<'a, 'src, N>>, source: &'src str) -> Self {
        Self {
            source,
            arena,
            lexer: Token::lexer(source).spanned(),
            next: None,
            span: Span::new(source, 0..0),
            errors: Vec::new(),
        }
    }

    fn advance(&mut self) -> Option<(Token<'src>, Span<'src>)> {
        let prev = self.next.take();

        for (next, span) in self.lexer.by_ref() {
            if let Ok(token) = next {
                let span = Span::new(self.source, span);
                self.next = Some((token, span));
                self.span = span;
                break;
            }
        }

        prev
    }

    fn peek(&self, m: impl Matcher) -> Option<(Token<'src>, Span<'src>)> {
        if let Some((token, span)) = self.next.as_ref() {
            m.matches(token).then_some((*token, *span))
        } else {
            None
        }
    }

    fn consume(&mut self, m: impl Matcher) -> Option<(Token<'src>, Span<'src>)> {
        self.peek(m).map(|v| {
            self.advance();
            v
        })
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
