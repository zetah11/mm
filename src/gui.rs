use eframe::NativeOptions;
use mm_gui::Gui;

pub fn run() -> eframe::Result<()> {
    let options = NativeOptions {
        ..Default::default()
    };

    eframe::run_native("mmm", options, Box::new(|cc| Box::new(App::new(cc))))
}

struct App {
    state: Gui,
}

impl App {
    fn new(_cc: &eframe::CreationContext) -> Self {
        Self { state: Gui::new() }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            self.state.ui(ui);
        });
    }
}
