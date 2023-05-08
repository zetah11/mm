mod audio;
mod code;
mod melody;
mod structures;

use egui::{CentralPanel, DragValue, TopBottomPanel, Ui};

use crate::audio::AudioThread;
use crate::melody::Editor;

pub struct Gui {
    editor: Editor<()>,
    stream: AudioThread,
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
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        TopBottomPanel::bottom("play_options")
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let text = if self.stream.state().is_playing() {
                        "Pause"
                    } else {
                        "Play"
                    };

                    if ui.button(text).clicked() {
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
                        .speed(0.5),
                    );
                });
            });

        CentralPanel::default().show_inside(ui, |ui| self.editor.ui(ui));

        if self.stream.state().is_playing() {
            ui.ctx().request_repaint();
        }
    }
}
