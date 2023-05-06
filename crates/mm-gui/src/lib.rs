mod code;
mod melody;

use code::{EditBuffer, ProgramBuffer};
use egui::{
    CentralPanel, CollapsingHeader, ComboBox, Frame, RichText, ScrollArea, TextEdit,
    TopBottomPanel, Ui,
};
use melody::NoteView;
use mm_eval::eval::Evaluator;
use mm_eval::span::Span;
use mm_eval::{check, compile, parse, Heap, Length, Name, Names, Time};
use mm_media::midi::Pitch;

use crate::code::CodeTheme;

const SOURCE: &str = r#"-- mm is for makin' music!
it! = (G, A | D, C), 4 F"#;

pub struct Gui {
    names: Names,
    edits: EditBuffer<()>,
    program: ProgramBuffer<()>,
    time: f64,

    entry: Name,
    prev_entry: Name,

    entries: Vec<Name>,
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

        let stored = compile(&mut Heap, &mut names, (), SOURCE).unwrap();
        let entries = stored.public;
        let entry = *entries
            .first()
            .expect("valid programs have at least one public name");

        let eval: Evaluator<Pitch, (), Heap> = Evaluator::new(stored.defs, entry).with_max_depth(5);

        let pitches = eval.iter().take(100).collect();

        let (program, edits) = ProgramBuffer::new((), SOURCE);

        Self {
            names,
            program,
            edits,
            time: 0.0,

            entry,
            prev_entry: entry,

            entries,
            pitches,
            hover: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        self.handle_recompile();

        ui.columns(2, |columns| {
            Frame::canvas(columns[0].style()).show(&mut columns[0], |ui| {
                self.melody(ui);
            });

            TopBottomPanel::top("editor_options").show_inside(&mut columns[1], |ui| {
                self.editor_options(ui);
            });

            TopBottomPanel::bottom("error_panel").show_inside(&mut columns[1], |ui| {
                self.errors(ui);
            });

            CentralPanel::default().show_inside(&mut columns[1], |ui| {
                self.editor(ui);
            });
        });
    }

    fn editor(&mut self, ui: &mut Ui) {
        ScrollArea::both().id_source("editor").show(ui, |ui| {
            let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
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
    }

    fn editor_options(&mut self, ui: &mut Ui) {
        self.prev_entry = self.entry;
        ComboBox::new("entry_selector", "Entry")
            .selected_text(self.names.get(&self.entry))
            .show_ui(ui, |ui| {
                for name in &self.entries {
                    let label = self.names.get(name);
                    ui.selectable_value(&mut self.entry, *name, label);
                }
            });
    }

    fn errors(&mut self, ui: &mut Ui) {
        let errors = self.program.errors();
        let title = if errors.is_empty() {
            "No issues".into()
        } else if errors.len() == 1 {
            "1 issue".into()
        } else {
            format!("{} issues", errors.len())
        };

        CollapsingHeader::new(title)
            .id_source("issues_collapsible")
            .default_open(true)
            .show(ui, |ui| {
                let color = ui.style().visuals.error_fg_color;
                for (error, _) in errors {
                    ui.label(RichText::new(error).color(color));
                }
            });
    }

    fn melody(&mut self, ui: &mut Ui) {
        let mut hover = None;
        ui.add(NoteView::new(
            &self.pitches,
            &mut hover,
            "melody_view",
            self.time as f32,
        ));
        self.hover = hover.copied();
    }

    fn handle_recompile(&mut self) {
        self.program.update(&mut self.edits, |_name, code| {
            match mm_eval::compile::<Pitch, _, _>(&mut Heap, &mut self.names, (), code) {
                Ok(program) => {
                    self.entries = program.public.clone();
                    self.entry = self
                        .entries
                        .first()
                        .copied()
                        .expect("no public names reported by checking pass");
                    self.prev_entry = self.entry;

                    let eval: Evaluator<_, _, Heap> =
                        Evaluator::new(program.defs, self.entry).with_max_depth(5);

                    self.pitches = eval.iter().take(100).collect();

                    Ok(())
                }

                Err(es) => Err(es
                    .into_iter()
                    .map(|error| Self::report(&self.names, error))
                    .collect()),
            }
        });

        if self.prev_entry != self.entry {
            todo!("reevaluate");
        }
    }

    fn report(names: &Names, error: mm_eval::Error<()>) -> (String, Span<()>) {
        match error {
            mm_eval::Error::Parse(parse::Error::DivisionByZero(s)) => {
                ("Cannot divide by zero".into(), s)
            }

            mm_eval::Error::Parse(parse::Error::ExpectedEqual(s)) => ("Expected '='".into(), s),

            mm_eval::Error::Parse(parse::Error::ExpectedName(s)) => ("Expected a name".into(), s),

            mm_eval::Error::Parse(parse::Error::ExpectedNote(s)) => {
                ("Expected a note or other melody".into(), s)
            }

            mm_eval::Error::Parse(parse::Error::ExpectedNumber(s)) => {
                ("Expected a number".into(), s)
            }

            mm_eval::Error::Parse(parse::Error::Redefinition { new, .. }) => {
                ("Name already in use".into(), new)
            }

            mm_eval::Error::Parse(parse::Error::UnclosedParen { opener, .. }) => {
                ("Unclosed parenthesis".into(), opener)
            }

            mm_eval::Error::Check(check::Error::NoPublicNames(at)) => {
                ("No exported names".into(), at)
            }

            mm_eval::Error::Check(check::Error::UnboundedNotLast(at)) => {
                ("Unbounded melodies must be last in a sequence".into(), at)
            }

            mm_eval::Error::Check(check::Error::UnknownName(at, name)) => {
                (format!("Undefined name '{}'", names.get(&name)), at)
            }
        }
    }
}
