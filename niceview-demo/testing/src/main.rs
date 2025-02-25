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
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut display = VirtualDisplay::new();

        display.draw_text("TK-Pie", 2, 20, niceview_lib::TextVariant::LargeBold);

        display.draw_text(
            "Current layout: Layer 1x2",
            4,
            28,
            niceview_lib::TextVariant::Regular,
        );

        display.draw_text("55%", 138, 14, niceview_lib::TextVariant::Regular);

        Self { display }
    }
}

impl eframe::App for DisplayApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
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
