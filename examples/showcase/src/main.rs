use eframe::egui;
use faseela_ui::{
    buttons::{ButtonVisuals, IconButton, IconButtonStyle},
    direction::{self, Direction},
    tabs::Tab,
    theme::{self, palette},
};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Faseela UI Showcase",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            theme::apply_dark(&cc.egui_ctx);
            Ok(Box::<Showcase>::default())
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("browser document is unavailable"))?;
        let canvas = document
            .get_element_by_id("faseela-ui-canvas")
            .ok_or_else(|| JsValue::from_str("showcase canvas was not found"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()?;

        wasm_bindgen_futures::spawn_local(async move {
            let result = eframe::WebRunner::new()
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|cc| {
                        theme::apply_dark(&cc.egui_ctx);
                        Ok(Box::<Showcase>::default())
                    }),
                )
                .await;

            if let Err(error) = result {
                web_sys::console::error_1(&JsValue::from_str(&format!(
                    "Faseela UI failed to start: {error:?}"
                )));
            }
        });

        Ok(())
    }
}

#[derive(Default)]
struct Showcase {
    rtl: bool,
    selected_tab: usize,
    selected_theme: usize,
    icon_activated: bool,
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        {
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

            ui.add_space(16.0);
            ui.separator();
            ui.heading("Tabs");
            ui.horizontal(|ui| {
                for (index, label) in ["General", "Appearance", "Keyboard"].iter().enumerate() {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(112.0, 36.0), egui::Sense::hover());
                    let response = Tab::new(
                        egui::Id::unique(("showcase-tab", index)),
                        label,
                        index == self.selected_tab,
                    )
                    .show_at(ui, rect, |ui, response| {
                        let fill = if index == self.selected_tab {
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
                            if index == self.selected_tab {
                                palette::SURFACE
                            } else {
                                palette::PRIMARY_LIGHT
                            },
                        );
                    });

                    if response.clicked() {
                        self.selected_tab = index;
                    }
                }
            });
            ui.label(format!(
                "Selected: {}",
                ["General", "Appearance", "Keyboard"][self.selected_tab]
            ));

            ui.add_space(16.0);
            ui.separator();
            ui.heading("Selection");
            egui::ComboBox::from_label("Theme preset")
                .selected_text(["Faseela Dark", "Midnight", "Forest"][self.selected_theme])
                .show_ui(ui, |ui| {
                    for (index, name) in ["Faseela Dark", "Midnight", "Forest"].iter().enumerate() {
                        ui.selectable_value(&mut self.selected_theme, index, *name);
                    }
                });

            ui.add_space(16.0);
            ui.separator();
            ui.heading("Buttons");
            let style = IconButtonStyle {
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
            let response = IconButton::new("Example icon action", egui::vec2(40.0, 40.0), &style)
                .show(ui, |ui, rect, tint| {
                    ui.painter().circle_filled(rect.center(), 8.0, tint);
                });

            if response.clicked() {
                self.icon_activated = !self.icon_activated;
            }
            ui.label(if self.icon_activated {
                "Action activated"
            } else {
                "Try the icon button"
            });
        }
    }
}
