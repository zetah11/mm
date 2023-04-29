use std::collections::HashMap;
use std::io;

use ariadne::{Cache, Label, Report, ReportKind, Source};
use mm_eval::{check, parse, Error};

#[derive(Debug, Default)]
pub struct Sources {
    sources: Vec<(String, String, Source)>,
}

impl Sources {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add(&mut self, name: impl Into<String>, source: impl Into<String>) {
        let source: String = source.into();
        let cached = Source::from(source.clone()); // eugh
        self.sources.push((name.into(), source, cached));
    }

    pub fn cache(&self) -> SourceCache {
        SourceCache::new(self)
    }
}

pub struct SourceCache<'src> {
    sources: &'src Sources,
    map: HashMap<&'src str, usize>,
}

impl<'src> SourceCache<'src> {
    pub fn iter(&self) -> impl Iterator<Item = (&'src str, &'src str)> {
        self.sources
            .sources
            .iter()
            .map(|(name, source, _)| (name.as_str(), source.as_str()))
    }

    pub fn report(&self, w: impl io::Write, e: Error<'src>) -> io::Result<()> {
        make_report(e).write(self, w)
    }

    fn new(sources: &'src Sources) -> Self {
        let map: HashMap<_, _> = sources
            .sources
            .iter()
            .enumerate()
            .map(|(index, (_, source, _))| (source.as_str(), index))
            .collect();

        Self { map, sources }
    }
}

impl<'src> Cache<&'src str> for &'_ SourceCache<'src> {
    fn fetch(&mut self, id: &&'src str) -> Result<&ariadne::Source, Box<dyn std::fmt::Debug + '_>> {
        match self.map.get(id) {
            Some(index) => Ok(&self.sources.sources[*index].2),

            None => Err(Box::new("Unknown source")),
        }
    }

    fn display<'a>(&self, id: &'a &'src str) -> Option<Box<dyn std::fmt::Display + 'a>> {
        if let Some(index) = self.map.get(id) {
            Some(Box::new(&self.sources.sources[*index].0))
        } else {
            None
        }
    }
}

impl<I, S> From<I> for Sources
where
    I: IntoIterator<Item = (S, S)>,
    S: Into<String>,
{
    fn from(iter: I) -> Self {
        let mut sources = Self::new();
        for (name, source) in iter {
            sources.add(name, source)
        }

        sources
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct Span<'src>(mm_eval::span::Span<'src>);

impl<'src> ariadne::Span for Span<'src> {
    type SourceId = &'src str;

    fn source(&self) -> &Self::SourceId {
        &self.0.source
    }

    fn start(&self) -> usize {
        self.0.start
    }

    fn end(&self) -> usize {
        self.0.end
    }
}

fn make_report(e: Error) -> Report<Span> {
    match e {
        Error::Parse(parse::Error::ExpectedEqual(at)) => {
            Report::build(ReportKind::Error, at.source, at.start)
                .with_message("Expected equals sign '='")
                .with_label(Label::new(Span(at)))
                .finish()
        }

        Error::Parse(parse::Error::ExpectedName(at)) => {
            Report::build(ReportKind::Error, at.source, at.start)
                .with_message("Expected a name")
                .with_label(Label::new(Span(at)))
                .finish()
        }

        Error::Parse(parse::Error::ExpectedNote(at)) => {
            Report::build(ReportKind::Error, at.source, at.start)
                .with_message("Expected a note")
                .with_label(Label::new(Span(at)))
                .finish()
        }

        Error::Parse(parse::Error::ExpectedNumber(at)) => {
            Report::build(ReportKind::Error, at.source, at.start)
                .with_message("Expected a number")
                .with_label(Label::new(Span(at)))
                .finish()
        }

        Error::Parse(parse::Error::DivisionByZero(at)) => {
            Report::build(ReportKind::Error, at.source, at.start)
                .with_message("Length factor cannot be divided by zero")
                .with_label(Label::new(Span(at)))
                .finish()
        }

        Error::Parse(parse::Error::Redefinition { previous, new }) => {
            Report::build(ReportKind::Error, new.source, new.start)
                .with_message("Name cannot be redefined")
                .with_label(Label::new(Span(new)))
                .with_label(Label::new(Span(previous)).with_message("previous definition here"))
                .finish()
        }

        Error::Parse(parse::Error::UnclosedParen { opener, at }) => {
            Report::build(ReportKind::Error, opener.source, opener.start)
                .with_message("Unclosed parenthesis")
                .with_label(Label::new(Span(opener)).with_message("this opening parenthesis"))
                .with_label(
                    Label::new(Span(at)).with_message("expected a closing parenthesis ')' here"),
                )
                .finish()
        }

        Error::Check(check::Error::UnknownName(at, name)) => {
            Report::build(ReportKind::Error, at.source, at.start)
                .with_message(format!("Unknown name '{name}'"))
                .with_label(Label::new(Span(at)))
                .finish()
        }

        Error::Check(check::Error::UnboundedNotLast(at)) => {
            Report::build(ReportKind::Error, at.source, at.start)
                .with_message("Unbounded melody must be last in a sequence")
                .with_label(Label::new(Span(at)))
                .finish()
        }
    }
}
