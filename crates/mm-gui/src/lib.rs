mod audio;
mod clip_editor;
mod code;
mod node_editor;
mod structures;
mod timeline;
mod ui;

use egui::{pos2, vec2, CentralPanel, DragValue, Key, Modifiers, Pos2, TopBottomPanel, Ui, Vec2};

use crate::audio::AudioThread;
use crate::clip_editor::Editor;
use crate::node_editor::{GraphEditor, GraphView};
use crate::timeline::{Timeline, Tracks};

pub struct Gui {
    editor: Editor<()>,
    stream: AudioThread,
    graph: GraphEditor,
    tracks: Tracks,
    test: TestCourse,

    tab: Tab,

    divisions: usize,
}

impl Default for Gui {
    fn default() -> Self {
        Self::new()
    }
}

impl Gui {
    pub fn new() -> Self {
        let (stream, events) = audio::play();

        let mut graph = GraphEditor::new();
        let a = graph.add_node("aa".into(), pos2(50.0, 30.0));
        let b = graph.add_node("bb".into(), pos2(340.0, 125.0));
        graph.toggle_edge(a, b);

        let mut tracks = Tracks::new();
        let a = tracks.add_track("first".into()).unwrap();
        tracks.add_clip(a, 2.0.into());
        tracks.add_clip(a, 5.0.into());

        let b = tracks.add_track("second".into()).unwrap();
        tracks.add_clip(b, 0.0.into());
        tracks.add_clip(b, 1.0.into());

        Self {
            editor: Editor::new((), stream.state().shallow_copy(), events),
            stream,
            graph,
            tracks,
            test: TestCourse::new(),

            tab: Tab::Test,

            divisions: 4,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        TopBottomPanel::top("tab_select")
            .resizable(false)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.tab, Tab::Graph, "Audio");
                    ui.radio_value(&mut self.tab, Tab::Editor, "Melody");
                    ui.radio_value(&mut self.tab, Tab::Test, "Test");
                })
            });

        TopBottomPanel::bottom("play_options")
            .resizable(false)
            .show_separator_line(false)
            .show_inside(ui, |ui| self.draw_play_options(ui));

        TopBottomPanel::bottom("timeline")
            .default_height(200.0)
            .resizable(true)
            .show_separator_line(true)
            .show_inside(ui, |ui| {
                ui.add(Timeline::new(&mut self.tracks, self.stream.state().beat()))
            });

        CentralPanel::default().show_inside(ui, |ui| match self.tab {
            Tab::Graph => {
                ui.add(GraphView::new("graph_editor", &mut self.graph));
            }

            Tab::Editor => {
                self.editor.draw(ui, self.divisions);
            }

            Tab::Test => {
                crate::ui::plate::Plate::new("beep boop".into()).show(ui, &mut self.test);
            }
        });

        if self.stream.state().is_playing() {
            ui.ctx().request_repaint();
        }
    }

    fn draw_play_options(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let text = if self.stream.state().is_playing() {
                "Pause"
            } else {
                "Play"
            };

            let pressed_shortcut = ui.input(|input| {
                input.modifiers.matches(Modifiers::COMMAND) && input.key_pressed(Key::Space)
            });

            if ui.button(text).clicked() || pressed_shortcut {
                self.stream.state().toggle_playing();
            }

            if ui.button("Stop").clicked() {
                self.stream.state().stop();
            }

            ui.add(
                DragValue::from_get_set(|value| match value {
                    None => self.stream.state().bpm().into(),
                    Some(value) => {
                        self.stream.state().set_bpm(value.into());
                        value
                    }
                })
                .clamp_range(1.0..=1000.0)
                .suffix(" bpm")
                .speed(0.5),
            );

            ui.add(
                DragValue::from_get_set(|value| match value {
                    None => self.divisions as f64,
                    Some(value) => {
                        let value = value.max(1.0).round();
                        self.divisions = value as usize;
                        value
                    }
                })
                .max_decimals(0)
                .suffix(" / beat"),
            );

            ui.label(format!("{:.1}", self.stream.state().time()));
        });
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Tab {
    Graph,
    Editor,
    Test,
}

struct TestCourse {
    square: (Pos2, Vec2),
}

impl TestCourse {
    pub fn new() -> Self {
        Self {
            square: (pos2(50.0, 50.0), vec2(400.0, 200.0)),
        }
    }
}

impl crate::ui::plate::Course for TestCourse {
    type MealId = ();

    fn meals(&self) -> Vec<(Self::MealId, ui::plate::Meal)> {
        vec![(
            (),
            crate::ui::plate::Meal {
                top_left: self.square.0,
                size: self.square.1,
            },
        )]
    }

    fn drag(&mut self, (): &Self::MealId, delta: Vec2) -> bool {
        self.square.0 += delta;
        true
    }
}
