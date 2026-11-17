use eframe::egui;
use faseela_ui::{
    buttons::{ButtonVisuals, IconButton, IconButtonStyle},
    direction::{self, Direction},
    layout::{ResizeHandle, ResizeHandleStyle, SplitAxis, SplitSize, split_rect},
    tabs::Tab,
    text,
    theme::{self, ThemePreset, ThemeTokens, palette},
    widgets::{Badge, BadgeTone, Card, EmptyState, SectionHeader, ToolbarItem, toolbar},
};

/// Commands demonstrated by the interactive toolbar.
const TOOLBAR_COMMANDS: [&str; 8] = [
    "New", "Open", "Save", "Run", "Build", "Test", "Debug", "Deploy",
];

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Faseela",
        eframe::NativeOptions::default(),
        Box::new(|cc| {
            theme::apply_preset(&cc.egui_ctx, ThemePreset::FaseelaDark);
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
                        theme::apply_preset(&cc.egui_ctx, ThemePreset::FaseelaDark);
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
    split_size: Option<SplitSize>,
    last_command: Option<usize>,
    applied_theme: Option<ThemePreset>,
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Apply the preset at the very start of the frame so every widget painted below reads
        // the new tokens on the same frame the selection changes.
        if self.applied_theme != Some(self.selected_theme) {
            theme::apply_preset(ui.ctx(), self.selected_theme);
            // egui's Ui caches its style when created (before App::ui runs). Reset it so
            // built-in widgets use the new theme on this very frame, not one frame later.
            ui.reset_style();
            self.applied_theme = Some(self.selected_theme);
            ui.ctx().request_repaint();
        }

        // eframe's WebAssembly canvas is transparent. Paint the active egui background
        // explicitly so the browser's default canvas color never leaks through the UI.
        let tokens = theme::active_tokens(ui.ctx());
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, tokens.background);

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
        let tokens = theme::active_tokens(ui.ctx());
        ui.vertical(|ui| {
            ui.set_min_size(available);
            direction::horizontal(ui, |ui| {
                ui.heading(egui::RichText::new("Faseela").size(25.0).color(tokens.text));
                ui.with_layout(direction::trailing(ui), |ui| {
                    ui.label(
                        egui::RichText::new("LIVE PREVIEW")
                            .size(9.0)
                            .strong()
                            .color(tokens.secondary),
                    );
                });
            });
            ui.label(
                egui::RichText::new("Your workspace, running in WebAssembly")
                    .size(12.0)
                    .color(tokens.secondary),
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
            ui.allocate_ui_with_layout(nav_size, direction::row(ui), |ui| {
                let button_width = (ui.available_width() / 3.0).max(0.0);
                for (index, label) in ["Home", "Appearance", "Settings"].iter().enumerate() {
                    let size = egui::vec2(button_width, 48.0);
                    self.nav_tab(ui, "mobile-nav", index, label, size);
                }
            });
        });
    }

    fn desktop_shell(&mut self, ui: &mut egui::Ui, available: egui::Vec2) {
        let tokens = theme::active_tokens(ui.ctx());
        let rtl = self.rtl;
        let selected_tab = self.selected_tab;
        ui.with_layout(direction::row(ui), |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(218.0, available.y),
                egui::Layout::top_down(direction::start_align(ui)),
                |ui| {
                    Card::new()
                        .fill(tokens.surface)
                        .padding(egui::Margin::same(14))
                        .show(ui, |ui| {
                            ui.set_min_width(190.0);
                            ui.set_min_height((available.y - 12.0).max(0.0));
                            ui.add_space(6.0);
                            ui.heading(
                                egui::RichText::new("Faseela").size(25.0).color(tokens.text),
                            );
                            ui.label(
                                egui::RichText::new("UI WORKSPACE")
                                    .small()
                                    .color(tokens.secondary),
                            );
                            ui.add_space(28.0);
                            for (index, label) in
                                ["Overview", "Appearance", "Preferences"].iter().enumerate()
                            {
                                let size = egui::vec2(ui.available_width().max(0.0), 44.0);
                                self.nav_tab(ui, "desktop-nav", index, label, size);
                                ui.add_space(5.0);
                            }
                            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                                ui.label(
                                    egui::RichText::new("Rust · egui · WebAssembly")
                                        .small()
                                        .color(tokens.muted_text),
                                );
                                ui.label(
                                    egui::RichText::new("Faseela UI 0.1")
                                        .small()
                                        .color(tokens.secondary),
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
                    let heading = match selected_tab {
                        0 => "Overview",
                        1 => "Appearance",
                        _ => "Preferences",
                    };
                    direction::horizontal(ui, |ui| {
                        ui.heading(egui::RichText::new(heading).color(tokens.text));
                        // The caption owns the remaining width. Align its text inside that
                        // rectangle instead of placing an LTR run at an RTL layout edge.
                        let caption_width = ui.available_width().max(0.0);
                        let caption_align = if rtl {
                            egui::Align::LEFT
                        } else {
                            egui::Align::RIGHT
                        };
                        ui.add_sized(
                            egui::vec2(caption_width, ui.spacing().interact_size.y),
                            egui::Label::new(
                                egui::RichText::new("INTERACTIVE SHOWCASE")
                                    .small()
                                    .strong()
                                    .color(tokens.secondary),
                            )
                            .halign(caption_align)
                            .truncate(),
                        );
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.page_content(ui, false));
                },
            );
        });
    }

    /// A real interactive tab from the shared library, with caller-owned painting.
    fn nav_tab(
        &mut self,
        ui: &mut egui::Ui,
        salt: &'static str,
        index: usize,
        label: &str,
        size: egui::Vec2,
    ) {
        let tokens = theme::active_tokens(ui.ctx());
        let selected = self.selected_tab == index;
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let id = egui::Id::unique(salt).with(index);
        let response = Tab::new(id, label, selected).show_at(ui, rect, |ui, response| {
            let fill = if selected {
                tokens.selection
            } else if response.hovered() {
                tokens.hover
            } else {
                tokens.surface
            };
            let foreground = if selected {
                ThemeTokens::on_color(tokens.selection)
            } else {
                tokens.text
            };
            let painter = ui.painter();
            painter.rect_filled(rect, egui::CornerRadius::same(8), fill);
            painter.rect_stroke(
                rect,
                egui::CornerRadius::same(8),
                egui::Stroke::new(1.0, tokens.border),
                egui::StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(13.0),
                foreground,
            );
        });
        if response.clicked() {
            self.selected_tab = index;
        }
    }

    fn page_content(&mut self, ui: &mut egui::Ui, is_mobile: bool) {
        match self.selected_tab {
            0 => self.overview(ui, is_mobile),
            1 => self.appearance(ui, is_mobile),
            _ => self.preferences(ui, is_mobile),
        }

        let tokens = theme::active_tokens(ui.ctx());
        ui.add_space(18.0);
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new("Faseela UI")
                    .strong()
                    .color(tokens.secondary),
            );
            ui.label(
                egui::RichText::new("A real Rust application · compiled to WebAssembly")
                    .small()
                    .color(tokens.muted_text),
            );
        });
        ui.add_space(20.0);
    }

    fn overview(&mut self, ui: &mut egui::Ui, is_mobile: bool) {
        let tokens = theme::active_tokens(ui.ctx());
        let rtl = self.rtl;
        Card::new().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new("WELCOME BACK")
                    .small()
                    .strong()
                    .color(tokens.secondary),
            );
            ui.add_space(6.0);
            ui.heading(
                egui::RichText::new(if is_mobile {
                    "Your interface"
                } else {
                    "Your Faseela workspace"
                })
                .color(tokens.text),
            );
            ui.label(egui::RichText::new(
                "Explore shared components running as a real interactive application.",
            ));
            ui.add_space(10.0);
            // display_text has already put short RTL labels in visual word order. Keep the
            // widget in a normal LTR allocation and align its text inside a full-width rect.
            let arabic_line =
                text::display_text("مرحبًا بك في واجهة فسيلة. الحروف العربية متصلة الآن.");
            ui.add_sized(
                egui::vec2(ui.available_width().max(0.0), ui.spacing().interact_size.y),
                egui::Label::new(egui::RichText::new(arabic_line).color(tokens.secondary))
                    .halign(egui::Align::RIGHT)
                    .truncate(),
            );
            ui.add_space(14.0);
            ui.horizontal_wrapped(|ui| {
                self.stat_card(ui, "Components", "04", "Ready to use");
                self.stat_card(
                    ui,
                    "Direction",
                    if rtl { "RTL" } else { "LTR" },
                    "Reading layout",
                );
            });
        });

        ui.add_space(14.0);
        self.layout_preview(ui);

        ui.add_space(14.0);
        Card::new().show(ui, |ui| {
            ui.set_width(ui.available_width());
            SectionHeader::new("Direction & language")
                .caption("Switch the reading direction and see the layout respond immediately.")
                .show(ui);
            ui.add_space(10.0);
            if ui
                .add_sized(
                    egui::vec2(ui.available_width().max(0.0), 44.0),
                    egui::Button::new(if self.rtl {
                        "Switch to left-to-right"
                    } else {
                        "التبديل إلى RTL"
                    }),
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
                    .color(tokens.secondary),
                );
            });
        });

        ui.add_space(14.0);
        Card::new().show(ui, |ui| {
            ui.set_width(ui.available_width());
            SectionHeader::new("Quick action")
                .caption("A reusable icon button with hover, selected and focus states.")
                .show(ui);
            ui.add_space(8.0);
            let style = Self::button_style(tokens);
            let activated = self.icon_activated;
            let response = IconButton::new("Toggle sample action", egui::vec2(46.0, 46.0), &style)
                .selected(activated)
                .show(ui, |ui, rect, tint| {
                    ui.painter().circle_filled(rect.center(), 9.0, tint);
                });
            if response.clicked() {
                self.icon_activated = !self.icon_activated;
            }
            ui.horizontal(|ui| {
                ui.label(if activated {
                    "Action is active"
                } else {
                    "Action is inactive"
                });
                ui.add(Badge::new(
                    if activated { "ON" } else { "OFF" },
                    if activated {
                        BadgeTone::Success
                    } else {
                        BadgeTone::Neutral
                    },
                ));
            });
        });
    }

    fn layout_preview(&mut self, ui: &mut egui::Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        let rtl = self.rtl;
        Card::new().show(ui, |ui| {
            ui.set_width(ui.available_width());
            SectionHeader::new("Responsive layout")
                .caption(
                    "Drag the divider, or focus it and use the arrow keys to resize the panes.",
                )
                .show(ui);
            ui.add_space(8.0);

            let width = ui.available_width().max(0.0);
            let (outer, _) = ui.allocate_exact_size(egui::vec2(width, 88.0), egui::Sense::hover());
            let current_size = self.split_size.unwrap_or(SplitSize::Ratio(0.64));

            if let Some(split) = split_rect(outer, SplitAxis::Horizontal, current_size, 8.0, 64.0) {
                // Geometry stays physical; the app maps logical leading/trailing panes for RTL.
                let (leading, trailing) = if rtl {
                    (split.second, split.first)
                } else {
                    (split.first, split.second)
                };

                let painter = ui.painter();
                painter.rect_filled(leading, egui::CornerRadius::same(8), tokens.primary);
                painter.rect_filled(split.divider, egui::CornerRadius::same(3), tokens.border);
                painter.rect_filled(
                    trailing,
                    egui::CornerRadius::same(8),
                    tokens.elevated_surface,
                );
                painter.text(
                    leading.center(),
                    egui::Align2::CENTER_CENTER,
                    "LEADING",
                    egui::FontId::proportional(11.0),
                    ThemeTokens::on_color(tokens.primary),
                );
                painter.text(
                    trailing.center(),
                    egui::Align2::CENTER_CENTER,
                    "TRAILING",
                    egui::FontId::proportional(11.0),
                    tokens.text,
                );

                let handle_style = ResizeHandleStyle {
                    hovered: tokens.hover,
                    dragged: tokens.active,
                    focused: tokens.focus,
                };
                let mut handle = ResizeHandle::new(
                    egui::Id::unique("showcase-workspace-split"),
                    SplitAxis::Horizontal,
                    split.divider,
                    "Resize workspace panes",
                    handle_style,
                )
                .hit_margin(8.0)
                .keyboard_step(8.0)
                .reversed(rtl);

                // Expose the logical leading pane size to screen readers. In RTL that is the
                // physical second pane, so its accessible value grows opposite to physical drag.
                if split.available() >= 128.0 {
                    let leading_size = if rtl {
                        split.available() - split.first_size()
                    } else {
                        split.first_size()
                    };
                    handle = handle.value(leading_size, 64.0..=(split.available() - 64.0));
                }

                let resize = handle.show(ui);
                if resize.delta != 0.0
                    && let Some(next_size) = split.resized(current_size, resize.delta, 64.0)
                {
                    self.split_size = Some(next_size);
                    ui.ctx().request_repaint();
                }
            }

            ui.add_space(10.0);
            SectionHeader::new("Toolbar fitting")
                .caption("Trailing commands move into an accessible overflow menu.")
                .show(ui);
            let items: Vec<ToolbarItem<'_>> = TOOLBAR_COMMANDS
                .iter()
                .map(|label| ToolbarItem::new(label))
                .collect();
            let clicked = toolbar(ui, egui::Id::unique("showcase-toolbar"), &items);
            if let Some(index) = clicked {
                self.last_command = Some(index);
            }
            let status = match self.last_command {
                Some(index) => format!("Activated: {}", TOOLBAR_COMMANDS[index]),
                None => "Click a command to change state".to_owned(),
            };
            ui.label(egui::RichText::new(status).small().color(tokens.muted_text));
        });
    }

    fn appearance(&mut self, ui: &mut egui::Ui, _is_mobile: bool) {
        let tokens = theme::active_tokens(ui.ctx());
        let selected_theme = self.selected_theme;
        Card::new().show(ui, |ui| {
            ui.set_width(ui.available_width());
            SectionHeader::new("Appearance")
                .caption("Choose a visual preset for your application.")
                .show(ui);
            ui.add_space(12.0);

            // Immediate preset switching: every preset is one click away, plus the standard
            // selection control below.
            ui.horizontal_wrapped(|ui| {
                for preset in ThemePreset::ALL {
                    let preset_tokens = preset.tokens();
                    let active = preset == selected_theme;
                    let button = egui::Button::new(
                        egui::RichText::new(preset.label())
                            .color(ThemeTokens::on_color(preset_tokens.primary)),
                    )
                    .fill(preset_tokens.primary)
                    .stroke(egui::Stroke::new(
                        if active { 2.0 } else { 1.0 },
                        if active { tokens.focus } else { tokens.border },
                    ));
                    if ui.add(button).clicked() {
                        self.selected_theme = preset;
                    }
                }
            });
            ui.add_space(12.0);

            egui::ComboBox::from_label("Color theme")
                .width(ui.available_width().min(320.0))
                .selected_text(selected_theme.label())
                .show_ui(ui, |ui| {
                    for preset in ThemePreset::ALL {
                        ui.selectable_value(&mut self.selected_theme, preset, preset.label());
                    }
                });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(format!("Active theme: {}", selected_theme.label()));
                ui.add(Badge::new(selected_theme.label(), BadgeTone::Primary));
            });
            ui.add_space(16.0);

            SectionHeader::new("Faseela color palette")
                .caption("The official identity that anchors the default preset.")
                .show(ui);
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
                        ui.label(egui::RichText::new(name).small().color(tokens.text));
                    });
                }
            });
        });
    }

    fn preferences(&mut self, ui: &mut egui::Ui, is_mobile: bool) {
        let tokens = theme::active_tokens(ui.ctx());
        let padding = if self.compact_mode {
            egui::Margin::same(8)
        } else {
            egui::Margin::same(14)
        };
        Card::new().padding(padding).show(ui, |ui| {
            ui.set_width(ui.available_width());
            SectionHeader::new("Preferences")
                .caption("Application behavior and accessibility options.")
                .show(ui);
            ui.add_space(12.0);
            ui.checkbox(&mut self.notifications, "Enable notifications");
            ui.checkbox(&mut self.compact_mode, "Compact interface spacing");
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.add(Badge::new("Success", BadgeTone::Success));
                ui.add(Badge::new("Warning", BadgeTone::Warning));
                ui.add(Badge::new("Error", BadgeTone::Error));
            });
            ui.add_space(8.0);

            SectionHeader::new("Notifications")
                .caption("Empty and populated states use the same shared components.")
                .show(ui);
            if self.notifications {
                for (title, tone) in [
                    ("Build finished", BadgeTone::Success),
                    ("Update available", BadgeTone::Warning),
                    ("Sync failed", BadgeTone::Error),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(title).color(tokens.text));
                        ui.add(Badge::new(
                            match tone {
                                BadgeTone::Success => "OK",
                                BadgeTone::Warning => "LATER",
                                _ => "RETRY",
                            },
                            tone,
                        ));
                    });
                }
            } else {
                let response = EmptyState::new("No notifications")
                    .description("Enable notifications to see the populated state.")
                    .action("Enable notifications")
                    .show(ui);
                if response.inner.action_clicked {
                    self.notifications = true;
                }
            }
            if is_mobile {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Mobile layout is active").color(tokens.muted_text));
            }
        });
    }

    fn stat_card(&self, ui: &mut egui::Ui, title: &str, value: &str, detail: &str) {
        let tokens = theme::active_tokens(ui.ctx());
        Card::new().padding(egui::Margin::same(10)).show(ui, |ui| {
            ui.set_min_width(104.0);
            ui.set_min_height(76.0);
            ui.label(egui::RichText::new(title).small().color(tokens.secondary));
            ui.heading(egui::RichText::new(value).size(24.0).color(tokens.text));
            ui.label(egui::RichText::new(detail).small().color(tokens.muted_text));
        });
    }

    fn button_style(tokens: ThemeTokens) -> IconButtonStyle {
        IconButtonStyle {
            normal: ButtonVisuals {
                fill: tokens.surface,
                stroke: egui::Stroke::new(1.0, tokens.border),
                icon: tokens.text,
            },
            hovered: ButtonVisuals {
                fill: tokens.hover,
                stroke: egui::Stroke::new(1.0, tokens.primary),
                icon: tokens.text,
            },
            pressed: Some(ButtonVisuals {
                fill: tokens.active,
                stroke: egui::Stroke::new(1.0, tokens.focus),
                icon: ThemeTokens::on_color(tokens.active),
            }),
            selected: ButtonVisuals {
                fill: tokens.selection,
                stroke: egui::Stroke::new(1.5, tokens.focus),
                icon: ThemeTokens::on_color(tokens.selection),
            },
            corner_radius: egui::CornerRadius::same(9),
            focus_stroke: egui::Stroke::new(1.5, tokens.focus),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(
        size: egui::Vec2,
        rtl: bool,
        preset: ThemePreset,
        mut show: impl FnMut(&mut Showcase, &mut egui::Ui),
    ) {
        let ctx = egui::Context::default();
        theme::apply_preset(&ctx, preset);
        direction::apply(&ctx, if rtl { Direction::Rtl } else { Direction::Ltr });
        let mut app = Showcase::default();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                ..Default::default()
            },
            |ui| show(&mut app, ui),
        );
        output.textures_delta.clear();
        assert!(!output.shapes.is_empty(), "showcase must paint content");
    }

    #[test]
    fn every_page_renders_in_desktop_and_mobile_layouts() {
        for theme in ThemePreset::ALL {
            for tab in 0..3 {
                for rtl in [false, true] {
                    let preset = theme;
                    let render_page = |app: &mut Showcase, _ui: &mut egui::Ui| {
                        app.selected_theme = preset;
                        app.selected_tab = tab;
                        app.rtl = rtl;
                        app.notifications = tab % 2 == 0;
                    };

                    let desktop = egui::vec2(1280.0, 800.0);
                    render(desktop, rtl, preset, |app, ui| {
                        render_page(app, ui);
                        app.desktop_shell(ui, desktop);
                    });
                    let mobile = egui::vec2(390.0, 780.0);
                    render(mobile, rtl, preset, |app, ui| {
                        render_page(app, ui);
                        app.mobile_shell(ui, mobile);
                    });
                }
            }
        }
    }

    #[test]
    fn button_style_is_derived_from_tokens() {
        let tokens = ThemePreset::Light.tokens();
        let style = Showcase::button_style(tokens);
        assert_eq!(style.normal.fill, tokens.surface);
        assert_eq!(style.hovered.fill, tokens.hover);
        assert_eq!(style.selected.fill, tokens.selection);
        assert_eq!(style.focus_stroke.color, tokens.focus);
    }
}
