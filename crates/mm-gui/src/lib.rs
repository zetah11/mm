mod audio;
mod clip_editor;
mod code;
mod grid;
mod node_editor;
mod structures;

use egui::{CentralPanel, DragValue, Key, Modifiers, TopBottomPanel, Ui};

use crate::audio::AudioThread;
use crate::clip_editor::Editor;

pub struct Gui {
    editor: Editor<()>,
    stream: AudioThread,

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

        Self {
            editor: Editor::new((), stream.state().shallow_copy(), events),
            stream,

            divisions: 4,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        TopBottomPanel::bottom("play_options")
            .resizable(false)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
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

                    ui.label(format!("{:.1} s", self.stream.state().time()));

                    ui.add(
                        DragValue::from_get_set(|value| match value {
                            None => self.stream.state().bpm(),
                            Some(value) => {
                                self.stream.state().set_bpm(value);
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
                });
            });

        CentralPanel::default().show_inside(ui, |ui| self.editor.draw(ui, self.divisions));

        if self.stream.state().is_playing() {
            ui.ctx().request_repaint();
        }
    }
}
