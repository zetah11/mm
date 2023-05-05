use egui::plot::{log_grid_spacer, GridInput, GridMark};
use egui::{
    lerp, pos2, remap_clamp, vec2, Color32, PointerButton, Rect, Rounding, Sense, Shape, Stroke,
    Ui, Vec2, Widget,
};
use mm_eval::span::Span;
use mm_eval::{Length, Time};
use mm_media::midi::Pitch;
use num_traits::ToPrimitive;

#[derive(Clone, Copy, Debug, Default)]
pub struct Bounds {
    offset_pitch: f32,
    offset_time: f32,
}

impl Bounds {
    pub fn translate(&mut self, delta: Vec2) {
        self.offset_pitch += delta.x;
        self.offset_time += delta.y;
    }
}

pub struct NoteView<'notes, 'm, Id> {
    notes: &'notes [(Pitch, Span<Id>, Time, Length)],
    hover: &'m mut Option<&'notes Span<Id>>,
    bounds: &'m mut Bounds,
}

impl<'notes, 'm, Id> NoteView<'notes, 'm, Id> {
    pub fn new(
        notes: &'notes [(Pitch, Span<Id>, Time, Length)],
        hover: &'m mut Option<&'notes Span<Id>>,
        bounds: &'m mut Bounds,
    ) -> Self {
        Self {
            notes,
            hover,
            bounds,
        }
    }
}

impl<Id> Widget for NoteView<'_, '_, Id> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let (rect, response) = ui.allocate_at_least(ui.available_size(), Sense::drag());
        let painter = ui.painter().with_clip_rect(rect);

        if response.dragged_by(PointerButton::Primary) {
            let delta = response.drag_delta();
            self.bounds.translate(delta);
        }

        let hover = if response.hovered() {
            response.hover_pos()
        } else {
            None
        };

        let width = 10.0;
        let beat_height = 30.0;

        let mut shapes = Vec::new();
        self.draw_grid(ui, &mut shapes, rect, width, beat_height);
        painter.extend(shapes);

        let mut hover_span = None;
        let color = ui.style().visuals.error_fg_color;
        let hover_stroke = ui.style().visuals.widgets.hovered.fg_stroke;
        let rounding = Rounding::same(width / 4.0).at_most(width / 2.0);

        for (pitch, at, start, length) in self.notes {
            let x = pitch.offset(&Pitch::A4);
            let x = rect.min.x + rect.width() / 2.0 + x as f32 * width + self.bounds.offset_pitch;

            let Some(y) = start.0.to_f32() else { break; };
            let y = rect.min.y + y * beat_height + self.bounds.offset_time;

            let Length::Bounded(height) = length else { unreachable!("note lengths are always finite"); };
            let Some(height) = height.to_f32() else { break; };
            let height = height * beat_height;

            let rect = Rect::from_min_size(pos2(x, y), vec2(width, height));

            let stroke = if let Some(pos) = hover {
                if rect.contains(pos) {
                    hover_span = Some(at);
                    hover_stroke
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

impl<Id> NoteView<'_, '_, Id> {
    fn draw_grid(
        &self,
        ui: &Ui,
        shapes: &mut Vec<Shape>,
        rect: Rect,
        pitch_width: f32,
        beat_height: f32,
    ) {
        let pitch_marks = {
            let mut marks = Vec::new();

            let steps = (rect.width() / pitch_width) as isize;
            let leftmost = self.bounds.offset_pitch.rem_euclid(pitch_width);

            for i in 0..steps {
                let value = rect.min.x + leftmost + i as f32 * pitch_width;
                marks.push(GridMark {
                    value: value as f64,
                    step_size: pitch_width as f64,
                });
            }

            marks
        };

        let time_marks = {
            let spacer = log_grid_spacer(4);
            let min = -self.bounds.offset_time / beat_height;
            let max = min + rect.height() / beat_height;

            let input = GridInput {
                bounds: (min as f64, max as f64),
                base_step_size: (beat_height / rect.height()) as f64,
            };

            spacer(input)
        };

        for mark in pitch_marks {
            let color = color_from_contrast(ui, 0.5);
            let stroke = Stroke::new(0.2, color);

            let x = mark.value as f32;

            let from = pos2(x, rect.min.y);
            let to = pos2(x, rect.max.y);

            shapes.push(Shape::line_segment([from, to], stroke));
        }

        for mark in time_marks {
            let weight = remap_clamp(mark.step_size, 0.0..=16.0, 0.0..=1.0).sqrt() as f32;
            let color = color_from_contrast(ui, weight);
            let stroke = Stroke::new(weight, color);

            let y = mark.value as f32 * beat_height + rect.min.y + self.bounds.offset_time;

            let from = pos2(rect.min.x, y);
            let to = pos2(rect.max.x, y);

            shapes.push(Shape::line_segment([from, to], stroke));
        }

        fn color_from_contrast(ui: &Ui, contrast: f32) -> Color32 {
            let bg = ui.visuals().extreme_bg_color;
            let fg = ui.visuals().widgets.open.fg_stroke.color;
            let mix = 0.5 * contrast.sqrt();
            Color32::from_rgb(
                lerp((bg.r() as f32)..=(fg.r() as f32), mix) as u8,
                lerp((bg.g() as f32)..=(fg.g() as f32), mix) as u8,
                lerp((bg.b() as f32)..=(fg.b() as f32), mix) as u8,
            )
        }
    }
}
