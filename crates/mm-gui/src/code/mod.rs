pub use self::highlight::highlight;
pub use self::program_buffer::{EditBuffer, ProgramBuffer};

mod highlight;
mod program_buffer;

use egui::{Color32, FontId, Style, TextFormat, TextStyle};

pub struct CodeTheme {
    comment: TextFormat,
    number: TextFormat,
    plain: TextFormat,
    punctuation: TextFormat,

    hover: Color32,
}

impl CodeTheme {
    pub fn new(style: &Style) -> Self {
        let font = TextStyle::Monospace.resolve(style);
        if style.visuals.dark_mode {
            Self::dark(font)
        } else {
            Self::light(font)
        }
    }

    pub fn dark(font: FontId) -> Self {
        Self {
            comment: TextFormat::simple(font.clone(), Color32::GRAY),
            number: TextFormat::simple(font.clone(), Color32::from_rgb(248, 107, 84)),
            plain: TextFormat::simple(font.clone(), Color32::WHITE),
            punctuation: TextFormat::simple(font, Color32::LIGHT_GRAY),

            hover: Color32::DARK_GRAY,
        }
    }

    pub fn light(font: FontId) -> Self {
        Self {
            comment: TextFormat::simple(font.clone(), Color32::GRAY),
            number: TextFormat::simple(
                font.clone(),
                Color32::from_rgb(255 - 248, 255 - 107, 255 - 84),
            ),
            plain: TextFormat::simple(font.clone(), Color32::BLACK),
            punctuation: TextFormat::simple(font, Color32::DARK_GRAY),

            hover: Color32::LIGHT_GRAY,
        }
    }
}
