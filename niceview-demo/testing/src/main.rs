use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use niceview_lib::{KeyboardDisplay, color::Color};
use virtual_display::VirtualDisplay;

pub mod virtual_display;

fn main() {
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

        display.draw_pixel(40, 30, Color::Black);
        display.draw_pixel(41, 30, Color::Black);
        display.draw_pixel(40, 31, Color::Black);

        display.draw_pixel(40, 40, Color::White);
        display.draw_pixel(41, 40, Color::White);
        display.draw_pixel(40, 41, Color::White);

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

            // let painter = ui.painter();
            // let draw_area_size = Vec2::new(
            //     (VirtualDisplay::WIDTH * 4) as f32,
            //     (VirtualDisplay::HEIGHT * 4) as f32,
            // ) + Vec2::splat(1.0);
            // let (_response, painter) = ui.allocate_painter(draw_area_size, Sense::hover());

            // painter.rect_filled(egui::Rect::EVERYTHING, 0.0, Color32::from_rgb(40, 55, 20));
        });
    }
}
