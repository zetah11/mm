mod lex;
mod rules;

#[cfg(test)]
mod tests;

use logos::{Logos, SpannedIter};
use typed_arena::Arena;

use crate::implicit::{Melody, Program};
use crate::note::Note;
use crate::span::Span;
use crate::Length;

use self::lex::Token;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Error<Id> {
    ExpectedEqual(Span<Id>),
    ExpectedName(Span<Id>),
    ExpectedNote(Span<Id>),
    ExpectedNumber(Span<Id>),

    Redefinition { previous: Span<Id>, new: Span<Id> },

    DivisionByZero(Span<Id>),
    UnclosedParen { opener: Span<Id>, at: Span<Id> },
}

pub struct Parser<'a, 'src, N, Id> {
    name: Id,
    arena: &'a Arena<Melody<'a, 'src, N, Id>>,
    lexer: SpannedIter<'src, Token<'src>>,
    next: Option<(Token<'src>, Span<Id>)>,
    span: Span<Id>,

    errors: Vec<Error<Id>>,
}

impl<'a, 'src, N: Note, Id: Clone + Eq> Parser<'a, 'src, N, Id> {
    pub fn parse(
        arena: &'a Arena<Melody<'a, 'src, N, Id>>,
        name: Id,
        source: &'src str,
    ) -> Result<Program<'a, 'src, N, Id>, Vec<Error<Id>>> {
        let mut parser = Self::new(arena, name, source);
        parser.advance();
        let parsed = parser.parse_program();
        if parser.errors.is_empty() {
            Ok(parsed)
        } else {
            Err(parser.errors)
        }
    }

    pub fn parse_length(name: Id, source: &'src str) -> Result<Length, Vec<Error<Id>>> {
        let arena = Arena::new();
        let mut parser: Parser<char, _> = Parser::new(&arena, name, source);
        parser.advance();
        let (parsed, _) = parser.parse_factor();
        if parser.errors.is_empty() {
            Ok(Length::Bounded(parsed.0))
        } else {
            Err(parser.errors)
        }
    }

    fn new(arena: &'a Arena<Melody<'a, 'src, N, Id>>, name: Id, source: &'src str) -> Self {
        Self {
            arena,
            lexer: Token::lexer(source).spanned(),
            next: None,
            span: Span::new(name.clone(), 0..0),
            errors: Vec::new(),
            name,
        }
    }

    fn advance(&mut self) -> Option<(Token<'src>, Span<Id>)> {
        let prev = self.next.take();

        for (next, span) in self.lexer.by_ref() {
            if let Ok(token) = next {
                let span = Span::new(self.name.clone(), span);
                self.next = Some((token, span.clone()));
                self.span = span;
                break;
            }
        }

        prev
    }

    fn peek(&self, m: impl Matcher) -> Option<(Token<'src>, Span<Id>)> {
        if let Some((token, span)) = self.next.as_ref() {
            m.matches(token).then_some((*token, span.clone()))
        } else {
            None
        }
    }

    fn consume(&mut self, m: impl Matcher) -> Option<(Token<'src>, Span<Id>)> {
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

impl<const N: usize> Matcher for [Token<'_>; N] {
    fn matches(&self, token: &Token) -> bool {
        self.iter().any(|t| t.matches(token))
    }
}
