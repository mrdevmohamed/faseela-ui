use eframe::egui;
use faseela_ui::{
    buttons::{ButtonVisuals, IconButton, IconButtonStyle},
    direction::{self, Direction},
    tabs::Tab,
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
struct Showcase {
    rtl: bool,
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Faseela UI");
        ui.label("A shared interface foundation for Faseela applications.");
        ui.add_space(12.0);

        if ui
            .button(if self.rtl {
                "Switch to LTR"
            } else {
                "Switch to RTL"
            })
            .clicked()
        {
            self.rtl = !self.rtl;
        }
        direction::apply(
            ui.ctx(),
            if self.rtl {
                Direction::Rtl
            } else {
                Direction::Ltr
            },
        );
        direction::horizontal(ui, |ui| {
            ui.label("This row follows the interface reading direction.");
            ui.label(if self.rtl { "RTL" } else { "LTR" });
        });
        ui.add_space(8.0);

        ui.label("Accessible tabs");
        let tab_layout = direction::row(ui);
        ui.with_layout(tab_layout, |ui| {
            for (index, label) in ["General", "Appearance", "Keyboard"].iter().enumerate() {
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(112.0, 34.0), egui::Sense::hover());
                let _response = Tab::new(egui::Id::new(("showcase-tab", index)), label, index == 0)
                    .show_at(ui, rect, |ui, response| {
                        let fill = if index == 0 {
                            palette::PRIMARY
                        } else if response.hovered() {
                            palette::PRIMARY_LIGHT
                        } else {
                            egui::Color32::from_rgb(0x17, 0x35, 0x2D)
                        };
                        ui.painter().rect_filled(rect, 6.0, fill);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(13.0),
                            if index == 0 {
                                palette::SURFACE
                            } else {
                                palette::PRIMARY_LIGHT
                            },
                        );
                    });
            }
        });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label("Primary color");
            let (rect, _) = ui.allocate_exact_size(egui::vec2(72.0, 28.0), egui::Sense::hover());
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
        let response =
            IconButton::new("Example icon action", egui::vec2(40.0, 40.0), &button_style).show(
                ui,
                |ui, rect, tint| {
                    ui.painter().circle_filled(rect.center(), 8.0, tint);
                },
            );
        if response.clicked() {
            ui.label("Icon button activated");
        }
    }
}
