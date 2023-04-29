use std::collections::HashSet;

use mm_eval::span::Span;
use mm_eval::{Length, Time};
use num_traits::ToPrimitive;

use crate::midi::{Interval, Pitch};

#[derive(Debug)]
pub struct Canvas {
    pub rectangles: Vec<Rectangle>,
    pub pitches: Vec<Label>,

    pub a4: f64,

    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,

    pub unit_width: f64,
    pub pitch_height: f64,
}

impl Canvas {
    fn new() -> Self {
        Self {
            rectangles: Vec::new(),
            pitches: Vec::new(),

            a4: 0.0,

            max_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,

            unit_width: 80.0,
            pitch_height: 10.0,
        }
    }

    fn update_bounds(&mut self) {
        self.max_x = self
            .rectangles
            .iter()
            .map(|rect| rect.x + rect.width)
            .reduce(|a, b| a.max(b))
            .unwrap_or_default();

        self.min_y = self
            .rectangles
            .iter()
            .map(|rect| rect.y)
            .reduce(|a, b| a.min(b))
            .unwrap_or_default();

        self.max_y = self
            .rectangles
            .iter()
            .map(|rect| rect.y + rect.height)
            .reduce(|a, b| a.max(b))
            .unwrap_or_default();

        for rect in self.rectangles.iter_mut() {
            rect.y -= self.min_y;
        }

        for label in self.pitches.iter_mut() {
            label.y -= self.min_y;
        }

        self.a4 -= self.min_y;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Label {
    pub pitch: Pitch,
    pub y: f64,
}

pub fn draw<'src>(notes: impl Iterator<Item = (Pitch, Span<'src>, Time, Length)>) -> Canvas {
    let mut canvas = Canvas::new();

    let mut pitches = HashSet::new();

    for (pitch, _, time, length) in notes {
        pitches.insert(pitch);

        let x = time
            .0
            .to_f64()
            .expect("time values are not unreasonably big")
            * canvas.unit_width;

        let y = Pitch::A4.offset(&pitch);
        let y = canvas.a4 + y as f64 * canvas.pitch_height;

        let width = match length {
            Length::Bounded(length) => {
                length
                    .to_f64()
                    .expect("length values are not unreasonably big")
                    * canvas.unit_width
            }

            Length::Unbounded => unreachable!("individual notes cannot be unbounded"),
        };

        canvas.rectangles.push(Rectangle {
            x,
            y,
            width,
            height: canvas.pitch_height,
        });
    }

    let deepest = *pitches.iter().min().unwrap_or(&Pitch::A4);
    let highest = *pitches.iter().max().unwrap_or(&Pitch::A4);

    for pitch in cmaj_between(deepest, highest) {
        let y = Pitch::A4.offset(&pitch);
        let y = canvas.a4 + y as f64 * canvas.pitch_height;

        canvas.pitches.push(Label { pitch, y });
    }

    canvas.update_bounds();
    canvas
}

/// Produce a series of pitches consisting of all those in the C major scale
/// between `from` and `to`, inclusive.
fn cmaj_between(from: Pitch, to: Pitch) -> Vec<Pitch> {
    let mut start = from.a_below();
    let mut result = Vec::new();

    let mut add = |p| {
        if p >= from && p <= to {
            result.push(p);
        }
    };

    while start <= to {
        let a = start;
        let b = a + Interval::WHOLETONE;
        let c = b + Interval::SEMITONE;
        let d = c + Interval::WHOLETONE;
        let e = d + Interval::WHOLETONE;
        let f = e + Interval::SEMITONE;
        let g = f + Interval::WHOLETONE;

        add(a);
        add(b);
        add(c);
        add(d);
        add(e);
        add(f);
        add(g);

        start = g + Interval::WHOLETONE;
    }

    result
}
