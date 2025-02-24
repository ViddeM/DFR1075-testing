use eframe::egui;
use niceview_lib::KeyboardDisplay;
use virtual_display::VirtualDisplay;

pub mod virtual_display;

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Display testing",
        native_options,
        Box::new(|cc| Ok(Box::new(DisplayApp::new(cc)))),
    )
    .expect("EGUI error");
}

struct DisplayApp {
    display: VirtualDisplay,
}

impl DisplayApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut display = VirtualDisplay::new();

        let text = "HELLO WORLD!";
        display.draw_text(text, 10, 10);
        // for (i, c) in text.chars().enumerate() {
        // display.draw_char_at(c, 10 + i * 6, 10);
        // }

        Self { display }
    }
}

impl eframe::App for DisplayApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!(
                "Display (size: {}x{})",
                VirtualDisplay::WIDTH,
                VirtualDisplay::HEIGHT
            ));

            self.display.update(ui);
        });
    }
}
