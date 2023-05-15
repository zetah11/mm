use egui::plot::{log_grid_spacer, GridInput};
use egui::{lerp, pos2, remap_clamp, Color32, Rect, Shape, Stroke, Style, Vec2};

#[derive(Clone, Copy, Debug, Default)]
pub struct Bounds {
    offset_x: f32,
    offset_y: f32,
}

impl Bounds {
    pub fn translate(&mut self, delta: Vec2) {
        self.offset_x += delta.x;
        self.offset_y += delta.y;
    }

    pub fn reset_on(&mut self, time: f32) {
        self.offset_x = 0.0;
        self.offset_y = time;
    }

    pub fn pitch_offset(&self) -> f32 {
        self.offset_x
    }

    pub fn time_offset(&self) -> f32 {
        self.offset_y
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Grid {
    bounds: Bounds,
    x_spacing: Option<f32>,
    y_base: i64,
    y_unit_height: f32,

    bg: Color32,
    fg: Color32,
}

impl Grid {
    pub fn new(style: &Style, bounds: Bounds, y_base: i64, y_unit_height: f32) -> Self {
        Self {
            bounds,
            x_spacing: None,
            y_base,
            y_unit_height,

            bg: style.visuals.widgets.open.fg_stroke.color,
            fg: style.visuals.extreme_bg_color,
        }
    }

    pub fn with_x(self, x_spacing: f32) -> Self {
        Self {
            x_spacing: Some(x_spacing),
            ..self
        }
    }

    pub fn draw(&self, shapes: &mut Vec<Shape>, rect: Rect) {
        if let Some(x_spacing) = self.x_spacing {
            self.draw_verticals(shapes, rect, x_spacing);
        }

        self.draw_horizontals(shapes, rect);
    }

    fn draw_verticals(&self, shapes: &mut Vec<Shape>, rect: Rect, x_spacing: f32) {
        let steps = (rect.width() / x_spacing) as usize;
        let color = self.color(0.5);
        let stroke = Stroke::new(0.2, color);

        let offset = (self.bounds.offset_x + rect.width() / 2.0).rem_euclid(x_spacing);

        for i in 0..steps {
            let x = rect.min.x + offset + i as f32 * x_spacing;

            let from = pos2(x, rect.min.y);
            let to = pos2(x, rect.max.y);

            shapes.push(Shape::line_segment([from, to], stroke));
        }
    }

    fn draw_horizontals(&self, shapes: &mut Vec<Shape>, rect: Rect) {
        let min = -self.bounds.offset_y / self.y_unit_height;
        let max = min + rect.height() / self.y_unit_height;

        let input = GridInput {
            bounds: (min as f64, max as f64),
            base_step_size: (self.y_unit_height / rect.height()) as f64,
        };

        let max = self.y_base as f64 * self.y_base as f64;
        for mark in log_grid_spacer(self.y_base)(input) {
            let weight = remap_clamp(mark.step_size, 0.0..=max, 0.0..=1.0).sqrt() as f32;
            let color = self.color(weight);
            let stroke = Stroke::new(weight, color);

            let y = rect.min.y + self.bounds.offset_y + mark.value as f32 * self.y_unit_height;

            let from = pos2(rect.min.x, y);
            let to = pos2(rect.max.x, y);

            shapes.push(Shape::line_segment([from, to], stroke));
        }
    }

    fn color(&self, contrast: f32) -> Color32 {
        let mix = 0.5 * contrast.sqrt();
        Color32::from_rgb(
            lerp((self.bg.r() as f32)..=(self.fg.r() as f32), mix) as u8,
            lerp((self.bg.g() as f32)..=(self.fg.g() as f32), mix) as u8,
            lerp((self.bg.b() as f32)..=(self.fg.b() as f32), mix) as u8,
        )
    }
}
