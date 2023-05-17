mod notes;

use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::hash::Hash;

use egui::{
    CentralPanel, CollapsingHeader, ComboBox, Frame, RichText, ScrollArea, TextEdit,
    TopBottomPanel, Ui,
};
use mm_eval::eval::Evaluator;
use mm_eval::span::Span;
use mm_eval::{check, compile, parse, Heap, Length, Name, Names, Time};
use mm_media::midi::Pitch;
use num_traits::ToPrimitive;

use self::notes::NoteView;
use crate::audio::event::{Event, EventKind, EventList};
use crate::audio::AudioState;
use crate::code::{self, CodeTheme, EditBuffer, ProgramBuffer};
use crate::structures::Latest;

const SOURCE: &str = r#"-- mm is for makin' music!
it! = (G, A | D, C), 4 F"#;

pub struct Editor<Id> {
    id: Id,
    audio_state: AudioState,
    events: Latest<EventList>,

    names: Names,
    edits: EditBuffer<Id>,
    buffer: ProgramBuffer<Id>,

    program: mm_eval::melody::Program<Pitch, Id, Heap>,
    entry: Name,
    prev_entry: Name,

    entries: Vec<Name>,
    pitches: Vec<(Pitch, Span<Id>, Time, Length)>,
    hover: Option<Span<Id>>,
}

impl<Id: Clone + Debug + Eq + Hash> Editor<Id> {
    pub fn new(id: Id, audio_state: AudioState, events: Latest<EventList>) -> Self {
        let mut names = Names::new();

        let program = compile(&mut Heap, &mut names, id.clone(), SOURCE).unwrap();
        let entries = program.public.clone();
        let entry = *entries
            .first()
            .expect("valid programs have at least one public name");

        let eval: Evaluator<_, _, Heap> = Evaluator::new(&program.defs, entry).with_max_depth(5);

        let pitches = eval.iter().take(1000).collect();

        let (buffer, edits) = ProgramBuffer::new(id.clone(), SOURCE);

        let this = Self {
            id,
            audio_state,
            events,

            names,
            buffer,
            edits,

            program,
            entry,
            prev_entry: entry,

            entries,
            pitches,
            hover: None,
        };

        this.send_events();
        this
    }

    pub fn draw(&mut self, ui: &mut Ui, grid_divisions: usize) {
        self.handle_recompile();

        ui.columns(2, |columns| {
            TopBottomPanel::bottom(self.id("editor_options"))
                .resizable(false)
                .show_separator_line(false)
                .show_inside(&mut columns[0], |ui| {
                    self.preview_options(ui);
                });

            CentralPanel::default().show_inside(&mut columns[0], |ui| {
                Frame::canvas(ui.style()).show(ui, |ui| {
                    self.melody(ui, grid_divisions);
                });
            });

            TopBottomPanel::bottom(self.id("error_panel"))
                .resizable(false)
                .show_separator_line(false)
                .show_inside(&mut columns[1], |ui| {
                    self.errors(ui);
                });

            CentralPanel::default().show_inside(&mut columns[1], |ui| {
                self.editor(ui);
            });
        });
    }

    fn id(&self, h: impl Hash) -> egui::Id {
        egui::Id::new((h, &self.id))
    }

    fn editor(&mut self, ui: &mut Ui) {
        ScrollArea::both()
            .id_source(self.id("editor"))
            .show(ui, |ui| {
                let mut layouter = |ui: &Ui, text: &str, wrap_width: f32| {
                    let theme = CodeTheme::new(ui.style());
                    let mut layout_job =
                        code::highlight(&theme, &self.edits, self.hover.clone(), text);
                    layout_job.wrap.max_width = wrap_width;
                    ui.fonts(|f| f.layout_job(layout_job))
                };

                ui.add_sized(
                    ui.available_size(),
                    TextEdit::multiline(&mut self.buffer)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
    }

    fn errors(&mut self, ui: &mut Ui) {
        let errors = self.buffer.errors();
        let title = if errors.is_empty() {
            "No issues".into()
        } else if errors.len() == 1 {
            "1 issue".into()
        } else {
            format!("{} issues", errors.len())
        };

        CollapsingHeader::new(title)
            .id_source(self.id("issues_collapsible"))
            .default_open(false)
            .show(ui, |ui| {
                let color = ui.style().visuals.error_fg_color;
                for (error, _) in errors {
                    ui.label(RichText::new(error).color(color));
                }
            });
    }

    fn preview_options(&mut self, ui: &mut Ui) {
        self.prev_entry = self.entry;
        ComboBox::new(self.id("entry_selector"), "Entry")
            .selected_text(self.names.get(&self.entry))
            .show_ui(ui, |ui| {
                for name in &self.entries {
                    let label = self.names.get(name);
                    ui.selectable_value(&mut self.entry, *name, label);
                }
            });
    }

    fn melody(&mut self, ui: &mut Ui, grid_divisions: usize) {
        let mut hover = None;
        ui.add(NoteView::new(
            &self.pitches,
            &mut hover,
            self.id("melody_view"),
            self.audio_state.beat(),
            grid_divisions,
        ));
        self.hover = hover.cloned();
    }

    fn handle_recompile(&mut self) {
        let mut dirty = false;

        self.buffer.update(&mut self.edits, |_name, code| {
            match mm_eval::compile::<Pitch, _, _>(&mut Heap, &mut self.names, self.id.clone(), code)
            {
                Ok(program) => {
                    self.entries = program.public.clone();
                    self.entry = self
                        .entries
                        .first()
                        .copied()
                        .expect("no public names reported by checking pass");
                    self.prev_entry = self.entry;
                    self.program = program;
                    dirty = true;

                    Ok(())
                }

                Err(es) => Err(es
                    .into_iter()
                    .map(|error| Self::report(&self.names, error))
                    .collect()),
            }
        });

        if dirty || self.prev_entry != self.entry {
            let eval: Evaluator<_, _, Heap> =
                Evaluator::new(&self.program.defs, self.entry).with_max_depth(5);
            self.pitches = eval.iter().take(1000).collect();
            self.send_events();
        }
    }

    fn send_events(&self) {
        let mut events = BinaryHeap::new();
        for (id, (pitch, _, start, length)) in self.pitches.iter().enumerate() {
            let Length::Bounded(length) = length else { unreachable!() };
            let end = (&start.0 + length).to_f64().unwrap().into();
            let start = start.0.to_f64().unwrap().into();
            let frequency = pitch.to_frequency(440.0).into();

            events.push(Event {
                kind: EventKind::Start { frequency },
                beat: start,
                id: id as u32,
            });

            events.push(Event {
                kind: EventKind::Stop,
                beat: end,
                id: id as u32,
            });
        }

        let events = events.into_sorted_vec();
        let events = EventList::new(events);
        self.events.send(events);
    }

    fn report(names: &Names, error: mm_eval::Error<Id>) -> (String, Span<Id>) {
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
