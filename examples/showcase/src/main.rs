use eframe::egui;
use faseela_ui::{
    buttons::{ButtonVisuals, IconButton, IconButtonStyle},
    theme::{self, palette},
};

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

            let button_style = IconButtonStyle {
                normal: ButtonVisuals {
                    fill: palette::PRIMARY,
                    stroke: egui::Stroke::NONE,
                    icon: palette::SURFACE,
                },
                hovered: ButtonVisuals {
                    fill: palette::PRIMARY_LIGHT,
                    stroke: egui::Stroke::NONE,
                    icon: palette::BACKGROUND,
                },
                pressed: None,
                selected: ButtonVisuals {
                    fill: palette::ACCENT,
                    stroke: egui::Stroke::NONE,
                    icon: palette::BACKGROUND,
                },
                corner_radius: egui::CornerRadius::same(8),
                focus_stroke: egui::Stroke::new(1.0, palette::ACCENT),
            };
            let response = IconButton::new(
                "Example icon action",
                egui::vec2(40.0, 40.0),
                &button_style,
            )
                .show(ui, |ui, rect, tint| {
                    ui.painter().circle_filled(rect.center(), 8.0, tint);
                });
            if response.clicked() {
                ui.label("Icon button activated");
            }
        });
    }
}
