use std::hash::Hash;

use egui::{pos2, vec2, Shape, Style, Ui, Widget};
use mm_eval::span::Span;
use mm_eval::{Length, Time};
use mm_media::midi::Pitch;
use num_traits::ToPrimitive;

use crate::audio::Beat;
use crate::ui::plate::{Course, Meal, Plate};

pub struct NoteView<'notes, 'm, Id> {
    notes: &'notes [(Pitch, Span<Id>, Time, Length)],
    hover: &'m mut Option<&'notes Span<Id>>,
    id: egui::Id,
    time: Beat,
    divisions: i64,

    width: f32,
    beat_height: f32,
}

impl<'notes, 'm, Id> NoteView<'notes, 'm, Id> {
    pub fn new(
        notes: &'notes [(Pitch, Span<Id>, Time, Length)],
        hover: &'m mut Option<&'notes Span<Id>>,
        id: impl Hash,
        time: Beat,
        grid_divisions: usize,
    ) -> Self {
        Self {
            notes,
            hover,
            id: egui::Id::new(id),
            time,
            divisions: grid_divisions as i64,

            width: 10.0,
            beat_height: 30.0,
        }
    }
}

impl<Id> Course for NoteView<'_, '_, Id> {
    type MealId = usize;

    fn hover(&mut self, meal: Option<&Self::MealId>) {
        *self.hover = if let Some(meal) = meal {
            let (_, span, _, _) = &self.notes[*meal];
            Some(span)
        } else {
            None
        };
    }

    fn meals(&self) -> Vec<(Self::MealId, Meal)> {
        self.notes.iter()
            .enumerate()
            .filter_map(|(i, (pitch, _, start, length))| {
                let x = pitch.offset(&Pitch::A4);
                let x = x as f32 * self.width;

                let Some(y) = start.0.to_f32() else { return None; };
                let y = y * self.beat_height;

                let Length::Bounded(height) = length else { unreachable!("note lengths are always finite"); };
                let Some(height) = height.to_f32() else { return None; };
                let height = height * self.beat_height;

                let meal = Meal {
                    top_left: pos2(x, y),
                    size: vec2(self.width, height),
                };

                Some((i, meal))
            })
            .collect()
    }

    fn background(&self, style: &Style, shapes: &mut Vec<Shape>, bounds: crate::ui::plate::Bounds) {
        // Draw verticals
        let steps = (bounds.visible.width() / self.width) as usize;
        let offset = bounds.top_left.x.div_euclid(self.width);

        for i in 0..steps {
            let x = i as f32 - offset;
            let x = bounds.apply(pos2(x * self.width, 0.0)).x;

            let from = pos2(x, bounds.visible.min.y);
            let to = pos2(x, bounds.visible.max.y);

            shapes.push(Shape::line_segment(
                [from, to],
                style.visuals.widgets.noninteractive.bg_stroke,
            ));
        }

        // Draw horizontals
        let d = self.divisions as isize;
        let d2 = d * d;

        let steps = (bounds.visible.height() / self.beat_height).ceil() as isize;
        let min = (-bounds.top_left.y / self.beat_height).floor() as isize * d;
        let max = min + steps * d;

        for i in min..=max {
            let y = self.beat_height * i as f32 / d as f32;
            let y = bounds.apply(pos2(0.0, y)).y;
            let from = pos2(bounds.visible.left(), y);
            let to = pos2(bounds.visible.right(), y);

            let mut stroke = style.visuals.widgets.noninteractive.bg_stroke;
            stroke.width = if i % d2 == 0 {
                1.5
            } else if i % d == 0 {
                0.75
            } else {
                0.25
            };

            shapes.push(Shape::line_segment([from, to], stroke));
        }
    }

    fn decoration(&self, style: &Style, shapes: &mut Vec<Shape>, bounds: crate::ui::plate::Bounds) {
        shapes.push({
            let y = bounds
                .apply(pos2(0.0, self.time.to_f32() * self.beat_height))
                .y;

            let from = pos2(bounds.visible.min.x, y);
            let to = pos2(bounds.visible.max.x, y);

            Shape::line_segment([from, to], style.visuals.widgets.noninteractive.fg_stroke)
        })
    }
}

impl<Id> Widget for NoteView<'_, '_, Id> {
    fn ui(mut self, ui: &mut Ui) -> egui::Response {
        Plate::new(self.id).show(ui, &mut self)
    }
}
