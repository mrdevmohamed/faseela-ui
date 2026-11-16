use eframe::egui;
use faseela_ui::{
    buttons::{ButtonVisuals, IconButton, IconButtonStyle},
    direction::{self, Direction},
    text,
    theme::{self, palette},
};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Faseela",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            theme::apply_dark(&cc.egui_ctx);
            theme::install_arabic_font(&cc.egui_ctx);
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
                        theme::install_arabic_font(&cc.egui_ctx);
                        Ok(Box::<Showcase>::default())
                    }),
                )
                .await;

            if let Err(error) = result {
                web_sys::console::error_1(&JsValue::from_str(&format!(
                    "Faseela failed to start: {error:?}"
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
    notifications: bool,
    compact_mode: bool,
    applied_theme: usize,
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.applied_theme != self.selected_theme {
            theme::apply_preset(ui.ctx(), theme::ThemePreset::from_index(self.selected_theme));
            self.applied_theme = self.selected_theme;
        }

        let available = ui.available_size();
        let is_mobile = available.x < 680.0;
        ui.set_min_size(available);
        direction::apply(
            ui.ctx(),
            if self.rtl {
                Direction::Rtl
            } else {
                Direction::Ltr
            },
        );

        if is_mobile {
            self.mobile_shell(ui, available);
        } else {
            self.desktop_shell(ui, available);
        }
    }
}

impl Showcase {
    fn mobile_shell(&mut self, ui: &mut egui::Ui, available: egui::Vec2) {
        ui.vertical(|ui| {
            ui.set_min_size(available);
            direction::horizontal(ui, |ui| {
                ui.heading(egui::RichText::new("Faseela").size(25.0));
                let trailing = direction::trailing(ui);
                ui.with_layout(trailing, |ui| {
                    ui.label(
                        egui::RichText::new("LIVE PREVIEW")
                            .size(9.0)
                            .strong()
                            .color(ui.visuals().hyperlink_color),
                    );
                });
            });
            ui.label(
                egui::RichText::new("Your workspace, running in WebAssembly")
                    .size(12.0)
                    .color(ui.visuals().hyperlink_color),
            );
            ui.add_space(10.0);

            let content_height = (ui.available_height() - 62.0).max(120.0);
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), content_height),
                egui::Layout::top_down(direction::start_align(ui)),
                |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.page_content(ui, true));
                },
            );

            ui.separator();
            let nav_size = egui::vec2(ui.available_width(), 52.0);
            let nav_layout = direction::row(ui);
            ui.allocate_ui_with_layout(nav_size, nav_layout, |ui| {
                let button_width = ui.available_width() / 3.0;
                for (index, label) in ["Home", "Appearance", "Settings"].iter().enumerate() {
                    let selected = self.selected_tab == index;
                    let fill = if selected {
                        palette::PRIMARY
                    } else {
                        ui.visuals().widgets.inactive.bg_fill
                    };
                    let response = ui.add_sized(
                        egui::vec2(button_width, 48.0),
                        egui::Button::new(egui::RichText::new(*label).size(11.0).color(
                            if selected {
                                palette::SURFACE
                            } else {
                                ui.visuals().hyperlink_color
                            },
                        ))
                        .fill(fill)
                        .stroke(egui::Stroke::NONE),
                    );
                    if response.clicked() {
                        self.selected_tab = index;
                    }
                }
            });
        });
    }

    fn desktop_shell(&mut self, ui: &mut egui::Ui, available: egui::Vec2) {
        let shell_layout = direction::row(ui);
        ui.with_layout(shell_layout, |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(218.0, available.y),
                egui::Layout::top_down(direction::start_align(ui)),
                |ui| {
                    egui::Frame::group(ui.style())
                        .fill(ui.visuals().panel_fill)
                        .show(ui, |ui| {
                            ui.set_min_width(190.0);
                            ui.set_min_height(available.y - 12.0);
                            ui.add_space(12.0);
                            ui.heading(egui::RichText::new("Faseela").size(25.0));
                            ui.label(
                                egui::RichText::new("UI WORKSPACE")
                                    .small()
                                    .color(ui.visuals().hyperlink_color),
                            );
                            ui.add_space(28.0);
                            for (index, label) in
                                ["Overview", "Appearance", "Preferences"].iter().enumerate()
                            {
                                let selected = self.selected_tab == index;
                                let response = ui.add_sized(
                                    egui::vec2(ui.available_width(), 44.0),
                                    egui::Button::new(egui::RichText::new(*label).color(
                                        if selected {
                                            palette::SURFACE
                                        } else {
                                            ui.visuals().hyperlink_color
                                        },
                                    ))
                                    .fill(if selected {
                                        palette::PRIMARY
                                    } else {
                                        ui.visuals().widgets.inactive.bg_fill
                                    })
                                    .stroke(egui::Stroke::NONE),
                                );
                                if response.clicked() {
                                    self.selected_tab = index;
                                }
                                ui.add_space(5.0);
                            }
                            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                                ui.label(
                                    egui::RichText::new("Rust · egui · WebAssembly")
                                        .small()
                                        .color(egui::Color32::GRAY),
                                );
                                ui.label(
                                    egui::RichText::new("Faseela UI 0.1")
                                        .small()
                                        .color(ui.visuals().hyperlink_color),
                                );
                            });
                        });
                },
            );

            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2((available.x - 232.0).max(260.0), available.y),
                egui::Layout::top_down(direction::start_align(ui)),
                |ui| {
                    direction::horizontal(ui, |ui| {
                        ui.heading(match self.selected_tab {
                            0 => "Overview",
                            1 => "Appearance",
                            _ => "Preferences",
                        });
                        let trailing = direction::trailing(ui);
                        ui.with_layout(trailing, |ui| {
                            ui.label(
                                egui::RichText::new("INTERACTIVE SHOWCASE")
                                    .small()
                                    .strong()
                                    .color(ui.visuals().hyperlink_color),
                            );
                        });
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.page_content(ui, false));
                },
            );
        });
    }

    fn page_content(&mut self, ui: &mut egui::Ui, is_mobile: bool) {
        match self.selected_tab {
            0 => self.overview(ui, is_mobile),
            1 => self.appearance(ui, is_mobile),
            _ => self.preferences(ui, is_mobile),
        }

        ui.add_space(18.0);
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new("Faseela UI")
                    .strong()
                    .color(ui.visuals().hyperlink_color),
            );
            ui.label(
                egui::RichText::new("A real Rust application · compiled to WebAssembly")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
        ui.add_space(20.0);
    }
}

impl Showcase {
    fn overview(&mut self, ui: &mut egui::Ui, is_mobile: bool) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new("WELCOME BACK")
                    .small()
                    .strong()
                    .color(ui.visuals().hyperlink_color),
            );
            ui.add_space(6.0);
            ui.heading(if is_mobile {
                "Your interface"
            } else {
                "Your Faseela workspace"
            });
            ui.label("Explore shared components running as a real interactive application.");
            ui.add_space(10.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.label(
                    egui::RichText::new(text::display_text(
                        "مرحبًا بك في واجهة فسيلة. الحروف العربية متصلة الآن.",
                    ))
                    .color(ui.visuals().hyperlink_color),
                );
            });
            ui.add_space(14.0);
            ui.horizontal_wrapped(|ui| {
                self.stat_card(ui, "Components", "04", "Ready to use");
                self.stat_card(
                    ui,
                    "Direction",
                    if self.rtl { "RTL" } else { "LTR" },
                    "Reading layout",
                );
            });
        });

        ui.add_space(14.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading("Direction & language");
            ui.label("Switch the reading direction and see the layout respond immediately.");
            ui.add_space(10.0);
            if ui
                .add_sized(
                    egui::vec2(ui.available_width(), 44.0),
                    egui::Button::new(text::display_text(if self.rtl {
                        "Switch to left-to-right"
                    } else {
                        "التبديل إلى RTL"
                    })),
                )
                .clicked()
            {
                self.rtl = !self.rtl;
            }
            direction::horizontal(ui, |ui| {
                ui.label("Reading direction");
                ui.label(
                    egui::RichText::new(if self.rtl {
                        "Right to left"
                    } else {
                        "Left to right"
                    })
                    .strong()
                    .color(ui.visuals().hyperlink_color),
                );
            });
        });

        ui.add_space(14.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading("Quick action");
            ui.label("A reusable icon button with hover, selected and focus states.");
            ui.add_space(8.0);
            let style = self.button_style();
            let response = IconButton::new("Toggle sample action", egui::vec2(46.0, 46.0), &style)
                .selected(self.icon_activated)
                .show(ui, |ui, rect, tint| {
                    ui.painter().circle_filled(rect.center(), 9.0, tint);
                });
            if response.clicked() {
                self.icon_activated = !self.icon_activated;
            }
            ui.label(if self.icon_activated {
                "Action is active"
            } else {
                "Action is inactive"
            });
        });
    }

    fn appearance(&mut self, ui: &mut egui::Ui, _is_mobile: bool) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading("Appearance");
            ui.label("Choose a visual preset for your application.");
            ui.add_space(12.0);
            egui::ComboBox::from_label("Color theme")
                .width(ui.available_width().min(320.0))
                .selected_text(theme::ThemePreset::from_index(self.selected_theme).label())
                .show_ui(ui, |ui| {
                    for (index, preset) in theme::ThemePreset::ALL.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_theme, index, preset.label());
                    }
                });
            ui.add_space(8.0);
            ui.label(format!(
                "Active theme: {}",
                theme::ThemePreset::from_index(self.selected_theme).label()
            ));
            ui.add_space(16.0);
            ui.label("Faseela color palette");
            ui.horizontal_wrapped(|ui| {
                for (name, color) in [
                    ("Deep", egui::Color32::from_rgb(0x0A, 0x1D, 0x1A)),
                    ("Green", egui::Color32::from_rgb(0x2F, 0xA3, 0x6B)),
                    ("Mint", egui::Color32::from_rgb(0x9B, 0xEB, 0xB8)),
                    ("Gold", egui::Color32::from_rgb(0xF2, 0xC1, 0x4E)),
                ] {
                    ui.vertical(|ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(48.0, 38.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 7.0, color);
                        ui.label(egui::RichText::new(name).small());
                    });
                }
            });
        });
    }

    fn preferences(&mut self, ui: &mut egui::Ui, is_mobile: bool) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading("Preferences");
            ui.label("Application behavior and accessibility options.");
            ui.add_space(12.0);
            ui.checkbox(&mut self.notifications, "Enable notifications");
            ui.checkbox(&mut self.compact_mode, "Compact interface spacing");
            ui.separator();
            ui.label("Input support");
            ui.label("• Touch and pointer interaction");
            ui.label("• Keyboard focus and navigation");
            ui.label("• Responsive layout for phones and desktops");
            if is_mobile {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Mobile layout is active").color(ui.visuals().hyperlink_color),
                );
            }
        });
    }

    fn stat_card(&self, ui: &mut egui::Ui, title: &str, value: &str, detail: &str) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_min_width(104.0);
            ui.set_min_height(76.0);
            ui.label(
                egui::RichText::new(title)
                    .small()
                    .color(ui.visuals().hyperlink_color),
            );
            ui.heading(egui::RichText::new(value).size(24.0));
            ui.label(
                egui::RichText::new(detail)
                    .small()
                    .color(egui::Color32::GRAY),
            );
        });
    }

    fn button_style(&self) -> IconButtonStyle {
        IconButtonStyle {
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
            corner_radius: egui::CornerRadius::same(9),
            focus_stroke: egui::Stroke::new(1.0, palette::ACCENT),
        }
    }
}
