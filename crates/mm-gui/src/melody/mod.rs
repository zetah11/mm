use egui::{pos2, vec2, Color32, Rect, Rounding, Sense, Stroke, Widget};
use mm_eval::span::Span;
use mm_eval::{Length, Time};
use mm_media::midi::Pitch;
use num_traits::ToPrimitive;

pub struct NoteView<'hover, Id> {
    notes: Vec<(Pitch, Span<Id>, Time, Length)>,
    hover: &'hover mut Option<Span<Id>>,
}

impl<'hover, Id> NoteView<'hover, Id> {
    pub fn new(
        hover: &'hover mut Option<Span<Id>>,
        notes: impl IntoIterator<Item = (Pitch, Span<Id>, Time, Length)>,
    ) -> Self {
        Self {
            notes: notes.into_iter().collect(),
            hover,
        }
    }
}

impl<Id> Widget for NoteView<'_, Id> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_at_least(ui.available_size(), Sense::hover());
        let painter = ui.painter();

        let hover = if response.hovered() {
            response.hover_pos()
        } else {
            None
        };

        let width = 10.0;
        let beat_height = 20.0;

        let mut hover_span = None;
        let color = Color32::RED;
        let stroke_color = Color32::GOLD;
        let rounding = Rounding::same(width / 4.0).at_most(width / 2.0);

        for (pitch, at, start, length) in self.notes {
            let x = pitch.offset(&Pitch::A4);
            let x = rect.min.x + rect.width() / 2.0 + (x - 1) as f32 * width;

            let Some(y) = start.0.to_f32() else { break; };
            let y = rect.min.y + y * beat_height;

            let Length::Bounded(height) = length else { unreachable!("note lengths are always finite"); };
            let Some(height) = height.to_f32() else { break; };
            let height = height * beat_height;

            let rect = Rect::from_min_size(pos2(x, y), vec2(width, height));

            let stroke = if let Some(pos) = hover {
                if rect.contains(pos) {
                    hover_span = Some(at);
                    Stroke::new(2.0, stroke_color)
                } else {
                    Stroke::NONE
                }
            } else {
                Stroke::NONE
            };

            painter.rect(rect, rounding, color, stroke);
        }

        *self.hover = hover_span;
        response
    }
}
