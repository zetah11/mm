use std::hash::Hash;

use egui::epaint::RectShape;
use egui::{pos2, vec2, PointerButton, Rect, Rounding, Sense, Shape, Stroke, Ui, Widget};
use mm_eval::span::Span;
use mm_eval::{Length, Time};
use mm_media::midi::Pitch;
use num_traits::ToPrimitive;

use crate::audio::Beat;
use crate::grid::{Bounds, Grid};

pub struct NoteView<'notes, 'm, Id> {
    notes: &'notes [(Pitch, Span<Id>, Time, Length)],
    hover: &'m mut Option<&'notes Span<Id>>,
    id: egui::Id,
    time: Beat,
    divisions: i64,
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
        }
    }
}

impl<Id> Widget for NoteView<'_, '_, Id> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let (rect, response) = ui.allocate_at_least(ui.available_size(), Sense::drag());
        let painter = ui.painter().with_clip_rect(rect);

        let width = 10.0;
        let beat_height = 30.0;

        let mut bounds: Bounds = ui
            .ctx()
            .memory_mut(|mem| mem.data.get_temp(self.id).unwrap_or_default());

        if response.dragged_by(PointerButton::Primary) {
            let delta = response.drag_delta();
            bounds.translate(delta);
        }

        if response.double_clicked_by(PointerButton::Primary) {
            bounds.reset_on(-self.time.to_f32() * beat_height);
        }

        ui.ctx()
            .memory_mut(|mem| mem.data.insert_temp(self.id, bounds));

        let hover = if response.hovered() {
            response.hover_pos()
        } else {
            None
        };

        let mut shapes = Vec::new();
        Grid::new(ui.style(), bounds, self.divisions, beat_height)
            .with_x(width)
            .draw(&mut shapes, rect);

        let mut hover_span = None;
        let fill = ui.style().visuals.error_fg_color;
        let hover_stroke = ui.style().visuals.widgets.hovered.fg_stroke;
        let rounding = Rounding::same(width / 4.0).at_most(width / 2.0);

        for (pitch, at, start, length) in self.notes {
            let x = pitch.offset(&Pitch::A4);
            let x = rect.min.x + rect.width() / 2.0 + x as f32 * width + bounds.pitch_offset();

            let Some(y) = start.0.to_f32() else { break; };
            let y = rect.min.y + y * beat_height + bounds.time_offset();

            let Length::Bounded(height) = length else { unreachable!("note lengths are always finite"); };
            let Some(height) = height.to_f32() else { break; };
            let height = height * beat_height;

            let rect = Rect::from_min_size(pos2(x, y), vec2(width, height));

            let stroke = if hover.map_or(false, |pos| rect.contains(pos)) {
                hover_span = Some(at);
                hover_stroke
            } else {
                Stroke::NONE
            };

            shapes.push(Shape::Rect(RectShape {
                rect,
                rounding,
                fill,
                stroke,
            }));
        }

        let line = {
            let y = rect.min.y + self.time.to_f32() * beat_height + bounds.time_offset();
            let from = pos2(rect.min.x, y);
            let to = pos2(rect.max.x, y);
            Shape::line_segment(
                [from, to],
                ui.style().visuals.widgets.noninteractive.fg_stroke,
            )
        };

        shapes.push(line);
        painter.extend(shapes);

        *self.hover = hover_span;
        response
    }
}
