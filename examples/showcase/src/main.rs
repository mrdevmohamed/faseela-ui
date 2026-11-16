use eframe::egui;
use faseela_ui::{
    buttons::{ButtonVisuals, IconButton, IconButtonStyle},
    direction::{self, Direction},
    layout::{fit_toolbar, split_rect, SplitAxis, SplitSize},
    text,
    theme::{self, ThemePreset, palette},
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
    selected_theme: ThemePreset,
    icon_activated: bool,
    notifications: bool,
    compact_mode: bool,
    applied_theme: Option<ThemePreset>,
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.applied_theme != Some(self.selected_theme) {
            theme::apply_preset(ui.ctx(), self.selected_theme);
            self.applied_theme = Some(self.selected_theme);
            ui.ctx().request_repaint();
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
                        ui.visuals().selection.bg_fill
                    } else {
                        ui.visuals().widgets.inactive.bg_fill
                    };
                    let foreground = if selected {
                        ui.visuals().widgets.active.fg_stroke.color
                    } else {
                        ui.visuals().widgets.inactive.fg_stroke.color
                    };
                    let response = ui.add_sized(
                        egui::vec2(button_width, 48.0),
                        egui::Button::new(egui::RichText::new(*label).size(11.0).color(foreground))
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
                        .fill(ui.visuals().widgets.noninteractive.bg_fill)
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
                                let foreground = if selected {
                                    ui.visuals().widgets.active.fg_stroke.color
                                } else {
                                    ui.visuals().widgets.inactive.fg_stroke.color
                                };
                                let fill = if selected {
                                    ui.visuals().selection.bg_fill
                                } else {
                                    ui.visuals().widgets.inactive.bg_fill
                                };
                                let response = ui.add_sized(
                                    egui::vec2(ui.available_width(), 44.0),
                                    egui::Button::new(
                                        egui::RichText::new(*label).color(foreground),
                                    )
                                    .fill(fill)
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
                                        .color(self.selected_theme.muted_text()),
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
                    .color(self.selected_theme.muted_text()),
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
        self.layout_preview(ui);

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
            let style = Self::button_style(ui.visuals());
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

    fn layout_preview(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.heading("Responsive layout");
            ui.label(
                "Split geometry and toolbar overflow are calculated by shared layout helpers.",
            );
            ui.add_space(8.0);

            let width = ui.available_width().max(0.0);
            let (outer, _) =
                ui.allocate_exact_size(egui::vec2(width, 88.0), egui::Sense::hover());
            if let Some(split) = split_rect(
                outer,
                SplitAxis::Horizontal,
                SplitSize::Ratio(0.64),
                8.0,
                64.0,
            ) {
                // Geometry is physical; the app maps its logical leading pane according to RTL.
                let (leading, trailing) = if self.rtl {
                    (split.second, split.first)
                } else {
                    (split.first, split.second)
                };
                let painter = ui.painter();
                let text_color = ui.visuals().text_color();
                painter.rect_filled(leading, egui::CornerRadius::same(8), palette::PRIMARY);
                painter.rect_filled(
                    split.divider,
                    egui::CornerRadius::same(3),
                    ui.visuals().widgets.noninteractive.bg_stroke.color,
                );
                painter.rect_filled(
                    trailing,
                    egui::CornerRadius::same(8),
                    ui.visuals().widgets.inactive.bg_fill,
                );
                painter.text(
                    leading.center(),
                    egui::Align2::CENTER_CENTER,
                    "LEADING",
                    egui::FontId::proportional(11.0),
                    text_color,
                );
                painter.text(
                    trailing.center(),
                    egui::Align2::CENTER_CENTER,
                    "TRAILING",
                    egui::FontId::proportional(11.0),
                    text_color,
                );
            }

            ui.add_space(10.0);
            ui.label("Toolbar fitting");
            let widths = [72.0, 72.0, 72.0, 72.0];
            let labels = ["File", "Edit", "View", "Run"];
            let toolbar_width = ui.available_width().max(0.0);
            let (toolbar, _) = ui.allocate_exact_size(
                egui::vec2(toolbar_width, 32.0),
                egui::Sense::hover(),
            );
            if let Some(fit) = fit_toolbar(&widths, toolbar_width, 6.0, 42.0) {
                let overflow_visible = !fit.overflow.is_empty();
                let visible_items = fit.shown.len() + usize::from(overflow_visible);
                let mut cursor = if self.rtl {
                    toolbar.right()
                } else {
                    toolbar.left()
                };
                let painter = ui.painter();
                let foreground = ui.visuals().text_color();

                for (index, item_width) in &fit.shown {
                    let item_rect = if self.rtl {
                        cursor -= *item_width;
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(cursor, toolbar.top()),
                            egui::vec2(*item_width, toolbar.height()),
                        );
                        cursor -= 6.0;
                        rect
                    } else {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(cursor, toolbar.top()),
                            egui::vec2(*item_width, toolbar.height()),
                        );
                        cursor += *item_width + 6.0;
                        rect
                    };
                    painter.rect_filled(
                        item_rect,
                        egui::CornerRadius::same(6),
                        ui.visuals().widgets.inactive.bg_fill,
                    );
                    painter.text(
                        item_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        labels[*index],
                        egui::FontId::proportional(11.0),
                        foreground,
                    );
                }

                if overflow_visible {
                    let overflow_rect = if self.rtl {
                        cursor -= fit.overflow_width;
                        egui::Rect::from_min_size(
                            egui::pos2(cursor, toolbar.top()),
                            egui::vec2(fit.overflow_width, toolbar.height()),
                        )
                    } else {
                        egui::Rect::from_min_size(
                            egui::pos2(cursor, toolbar.top()),
                            egui::vec2(fit.overflow_width, toolbar.height()),
                        )
                    };
                    painter.rect_filled(
                        overflow_rect,
                        egui::CornerRadius::same(6),
                        palette::PRIMARY,
                    );
                    painter.text(
                        overflow_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("+{}", fit.overflow.len()),
                        egui::FontId::proportional(11.0),
                        foreground,
                    );
                }

            } else {
                ui.label("This width needs a scrollable or wrapping toolbar fallback.");
            }
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
                .selected_text(self.selected_theme.label())
                .show_ui(ui, |ui| {
                    for preset in ThemePreset::ALL {
                        ui.selectable_value(&mut self.selected_theme, preset, preset.label());
                    }
                });
            ui.add_space(8.0);
            ui.label(format!("Active theme: {}", self.selected_theme.label()));
            ui.add_space(16.0);
            ui.label("Faseela color palette");
            ui.horizontal_wrapped(|ui| {
                for (name, color) in [
                    ("Deep", palette::BACKGROUND),
                    ("Green", palette::PRIMARY),
                    ("Mint", palette::PRIMARY_LIGHT),
                    ("Gold", palette::ACCENT),
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
                    egui::RichText::new("Mobile layout is active")
                        .color(self.selected_theme.muted_text()),
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
                    .color(self.selected_theme.muted_text()),
            );
        });
    }

    fn button_style(visuals: &egui::Visuals) -> IconButtonStyle {
        IconButtonStyle {
            normal: ButtonVisuals {
                fill: visuals.widgets.inactive.bg_fill,
                stroke: visuals.widgets.inactive.bg_stroke,
                icon: visuals.widgets.inactive.fg_stroke.color,
            },
            hovered: ButtonVisuals {
                fill: visuals.widgets.hovered.bg_fill,
                stroke: visuals.widgets.hovered.bg_stroke,
                icon: visuals.widgets.hovered.fg_stroke.color,
            },
            pressed: Some(ButtonVisuals {
                fill: visuals.widgets.active.bg_fill,
                stroke: visuals.widgets.active.bg_stroke,
                icon: visuals.widgets.active.fg_stroke.color,
            }),
            selected: ButtonVisuals {
                fill: visuals.selection.bg_fill,
                stroke: visuals.selection.stroke,
                icon: visuals.widgets.active.fg_stroke.color,
            },
            corner_radius: egui::CornerRadius::same(9),
            focus_stroke: visuals.selection.stroke,
        }
    }
}
