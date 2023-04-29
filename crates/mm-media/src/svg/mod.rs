use std::fs::{self};
use std::path::Path;

use mm_eval::span::Span;
use mm_eval::{Length, Time};

use crate::midi::Pitch;

mod draw;
mod render;

pub fn write<'src>(
    notes: impl Iterator<Item = (Pitch, Span<'src>, Time, Length)>,
    to: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let canvas = draw::draw(notes);
    let mut svg = String::new();
    render::render(canvas, &mut svg)?;

    fs::write(to, svg)?;
    Ok(())
}
