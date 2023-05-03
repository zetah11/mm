mod code;
mod melody;

use egui::{Frame, TextEdit};
use melody::NoteView;
use mm_eval::eval::Evaluator;
use mm_eval::span::Span;
use mm_eval::{compile, Length, Name, Time};
use mm_media::midi::Pitch;
use typed_arena::Arena;

use crate::code::CodeTheme;

const SOURCE: &str = r#"-- mm is for makin' music!
it! = (G, A | D, C), 4 F"#;

#[derive(Debug, Default)]
pub struct Gui {
    content: String,
    pitches: Vec<(Pitch, Span<()>, Time, Length)>,
    hover: Option<Span<()>>,
}

impl Gui {
    pub fn new() -> Self {
        let implicits = Arena::new();
        let explicits = Arena::new();

        let program = compile(&implicits, &explicits, (), SOURCE).unwrap();
        let eval = Evaluator::new(program.defs, Name("it")).with_max_depth(5);
        let pitches = eval.iter().take(50).collect();

        Self {
            content: String::from(SOURCE),
            pitches,
            hover: None,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            Frame::canvas(columns[0].style()).show(&mut columns[0], |ui| {
                ui.add(NoteView::new(&mut self.hover, self.pitches.iter().cloned()));
            });

            egui::ScrollArea::both()
                .id_source("editor")
                .show(&mut columns[1], |ui| {
                    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                        let theme = CodeTheme::new(ui.style());
                        let mut layout_job = code::highlight(&theme, self.hover, text);
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::multiline(&mut self.content)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    );
                });
        });
    }
}
