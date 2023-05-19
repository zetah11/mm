pub use tracks::{Track, TrackId, Tracks};

mod tracks;

use egui::{pos2, vec2, Rect, Response, Rounding, Sense, Shape, Ui, Widget};

use self::tracks::ClipId;
use crate::audio::Beat;

pub struct Timeline<'a> {
    tracks: &'a mut Tracks,
    time: Beat,
}

impl<'a> Timeline<'a> {
    pub fn new(tracks: &'a mut Tracks, time: Beat) -> Self {
        Self { tracks, time }
    }
}

impl Widget for Timeline<'_> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_at_least(ui.available_size(), Sense::drag());
        let painter = ui.painter().with_clip_rect(rect);

        let beat_width = 40.0;
        let track_height = 20.0;
        let mut shapes = Vec::new();

        let (response, input) = self.handle_input(rect, response, beat_width, track_height);
        self.draw(ui, &mut shapes, rect, input, beat_width, track_height);

        painter.extend(shapes);
        response
    }
}

impl Timeline<'_> {
    fn draw(
        &self,
        ui: &Ui,
        shapes: &mut Vec<Shape>,
        rect: Rect,
        input: InputResult,
        beat_width: f32,
        track_height: f32,
    ) {
        let fill_color = ui.visuals().error_fg_color;
        let hover_stroke = ui.visuals().widgets.hovered.fg_stroke;
        let rounding = Rounding::same(track_height / 4.0);

        for (i, (id, track)) in self.tracks.tracks().enumerate() {
            let y = track_height * i as f32;

            for (id, clip) in track.clips(id) {
                let x = beat_width * clip.start.to_f32();
                let width = beat_width * clip.length.to_f32();

                let min = pos2(x, y) + rect.min.to_vec2();
                let size = vec2(width, track_height);
                let rect = Rect::from_min_size(min, size);

                shapes.push(Shape::rect_filled(rect, rounding, fill_color));

                if input.hover == Some(id) {
                    shapes.push(Shape::rect_stroke(rect, rounding, hover_stroke));
                }
            }
        }

        shapes.push({
            let x = rect.min.x + beat_width * self.time.to_f32();
            let stroke = ui.style().visuals.widgets.noninteractive.fg_stroke;

            let y1 = rect.min.y;
            let y2 = rect.max.y;

            Shape::line_segment([pos2(x, y1), pos2(x, y2)], stroke)
        });
    }

    fn handle_input(
        &mut self,
        rect: Rect,
        response: Response,
        beat_width: f32,
        track_height: f32,
    ) -> (Response, InputResult) {
        let cursor = response.hover_pos();

        let mut hovered_clip = None;
        for (i, (id, track)) in self.tracks.tracks().enumerate() {
            let y = track_height * i as f32;
            for (id, clip) in track.clips(id) {
                let x = beat_width * clip.start.to_f32();
                let width = beat_width * clip.length.to_f32();

                let min = pos2(x, y) + rect.min.to_vec2();
                let size = vec2(width, track_height);
                let rect = Rect::from_min_size(min, size);

                if let Some(cursor) = cursor {
                    if rect.contains(cursor) {
                        hovered_clip = Some(id);
                    }
                }
            }
        }

        let input = InputResult {
            hover: hovered_clip,
        };

        (response, input)
    }
}

struct InputResult {
    hover: Option<ClipId>,
}
