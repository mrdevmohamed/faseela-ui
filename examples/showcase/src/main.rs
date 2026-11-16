use eframe::egui;
use faseela_ui::theme::{self, palette};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Faseela UI Showcase",
        options,
        Box::new(|cc| {
            theme::apply_dark(&cc.egui_ctx);
            Ok(Box::<Showcase>::default())
        }),
    )
}

#[derive(Default)]
struct Showcase;

impl eframe::App for Showcase {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Faseela UI");
            ui.label("A shared interface foundation for Faseela applications.");
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label("Primary color");
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(72.0, 28.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(rect, 6.0, palette::PRIMARY);
            });
            if ui.button("Example action").clicked() {
                ui.label("Action triggered");
            }
        });
    }
}
