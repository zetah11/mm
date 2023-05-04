mod code;
mod melody;

use code::{EditBuffer, ProgramBuffer};
use egui::{Frame, TextEdit};
use melody::NoteView;
use mm_eval::eval::Evaluator;
use mm_eval::span::Span;
use mm_eval::{compile, Heap, Length, Names, Time};
use mm_media::midi::Pitch;

use crate::code::CodeTheme;

const SOURCE: &str = r#"-- mm is for makin' music!
it! = (G, A | D, C), 4 F"#;

pub struct Gui {
    names: Names,
    edits: EditBuffer<()>,
    program: ProgramBuffer<()>,

    pitches: Vec<(Pitch, Span<()>, Time, Length)>,
    hover: Option<Span<()>>,
}

impl Default for Gui {
    fn default() -> Self {
        Self::new()
    }
}

impl Gui {
    pub fn new() -> Self {
        let mut names = Names::new();

        let program = compile(&mut Heap, &mut names, (), SOURCE).unwrap();
        let eval: Evaluator<Pitch, (), Heap> =
            Evaluator::new(program.defs, names.make("it")).with_max_depth(5);

        let pitches = eval.iter().take(100).collect();

        let (program, edits) = ProgramBuffer::new((), SOURCE);

        Self {
            names,
            program,
            edits,

            pitches,
            hover: None,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.program.update(&mut self.edits, |_name, code| {
            match mm_eval::compile::<Pitch, _, _>(&mut Heap, &mut self.names, (), code) {
                Ok(program) => {
                    let eval: Evaluator<_, _, Heap> =
                        Evaluator::new(program.defs, self.names.make("it")).with_max_depth(5);
                    self.pitches = eval.iter().take(100).collect();

                    Ok(())
                }

                Err(es) => Err(es.len()),
            }
        });

        ui.columns(2, |columns| {
            Frame::canvas(columns[0].style()).show(&mut columns[0], |ui| {
                ui.add(NoteView::new(&mut self.hover, self.pitches.iter().cloned()));
            });

            egui::ScrollArea::both()
                .id_source("editor")
                .show(&mut columns[1], |ui| {
                    let mut layouter = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                        let theme = CodeTheme::new(ui.style());
                        let mut layout_job = code::highlight(&theme, &self.edits, self.hover, text);
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::multiline(&mut self.program)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter),
                    );
                });
        });
    }
}
