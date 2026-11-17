use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Id, Margin, Rect, RichText, Sense, Stroke,
    StrokeKind, Ui, Vec2, pos2, vec2,
};
use faseela_ui::{
    buttons::{Button, ButtonVariant, ControlSize, IconButton, IconButtonStyle},
    direction::{self, Direction},
    forms::{Checkbox, RadioGroup, SearchField, Select, Switch, TextField},
    layout::{ResizeHandle, ResizeHandleStyle, SplitAxis, SplitSize, split_rect},
    navigation::{Breadcrumb, ButtonGroup, SegmentedControl},
    tabs::Tab,
    text,
    theme::{self, ThemePreset, ThemeTokens, palette},
    widgets::{
        Badge, BadgeTone, Card, EmptyState, LoadingIndicator, ProgressBar, SectionHeader,
        StatusIndicator, ToolbarItem, ToolbarState, Tooltip, toolbar_response,
    },
};

/// Navigation entries for the ten showcase sections.
const SECTIONS: [(&str, &str); 10] = [
    (
        "Overview",
        "A live application built from the shared Faseela UI crate.",
    ),
    (
        "Foundations",
        "Semantic color, typography, spacing, and shape tokens.",
    ),
    (
        "Buttons",
        "Every variant, size, icon, selection, and disabled state.",
    ),
    (
        "Form Controls",
        "Application-owned values with labels, help, errors, and keyboard input.",
    ),
    (
        "Navigation",
        "Breadcrumbs, segmented controls, button groups, and tabs.",
    ),
    (
        "Layout",
        "Responsive panes, resize handles, and toolbar fitting.",
    ),
    (
        "Feedback & Status",
        "Progress, loading, status, badges, empty states, and tooltips.",
    ),
    (
        "Appearance",
        "Switch themes and inspect the measured contrast of their tokens.",
    ),
    (
        "Accessibility",
        "Contrast evidence, focus behavior, disabled controls, and keyboard review.",
    ),
    (
        "RTL & Arabic",
        "Right-to-left layout, mirrored controls, and Arabic display text.",
    ),
];

const MOBILE_BREAKPOINT: f32 = 680.0;
const SIDEBAR_WIDTH: f32 = 214.0;
const CONTENT_MAX_WIDTH: f32 = 1120.0;

const TOOLBAR_COMMANDS: [&str; 9] = [
    "New", "Open", "Save", "Run", "Build", "Test", "Debug", "Deploy", "Publish",
];
const DISABLED_TOOLBAR_COMMAND: usize = 7;
const FILTERABLE_COMPONENTS: [&str; 4] = [
    "Buttons",
    "Form controls",
    "Navigation",
    "Feedback & status",
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

struct Showcase {
    rtl: bool,
    selected_section: usize,
    selected_theme: ThemePreset,
    applied_theme: Option<ThemePreset>,
    mobile_nav_open: bool,
    scroll_to_top: bool,
    last_action: String,

    // Form-owned values. The library only reads and proposes changes to these.
    name: String,
    email: String,
    search: String,
    arabic_input: String,
    notifications: bool,
    compact_mode: bool,
    density: usize,
    plan: usize,

    // Component-specific caller-owned state.
    action_selected: bool,
    bookmarked: bool,
    nav_segment: usize,
    nav_group: usize,
    nav_tab: usize,
    breadcrumb_last: Option<usize>,
    split_axis: usize,
    split_size: Option<SplitSize>,
    last_command: Option<usize>,
    progress: f32,
    show_loading: bool,
    keyboard_review: bool,
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            rtl: false,
            selected_section: 0,
            selected_theme: ThemePreset::FaseelaDark,
            applied_theme: None,
            mobile_nav_open: false,
            scroll_to_top: false,
            last_action: "Ready — every control below changes real application state.".to_owned(),
            name: String::new(),
            email: String::new(),
            search: String::new(),
            arabic_input: "فسيلة".to_owned(),
            notifications: false,
            compact_mode: false,
            density: 1,
            plan: 1,
            action_selected: false,
            bookmarked: false,
            nav_segment: 0,
            nav_group: 1,
            nav_tab: 0,
            breadcrumb_last: None,
            split_axis: 0,
            split_size: None,
            last_command: None,
            progress: 0.45,
            show_loading: true,
            keyboard_review: false,
        }
    }
}

impl eframe::App for Showcase {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

impl Showcase {
    fn show(&mut self, ui: &mut egui::Ui) {
        self.sync_theme(ui);

        let available = ui.available_size();
        let tokens = theme::active_tokens(ui.ctx());
        // The WebAssembly canvas is transparent; always paint the themed background.
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, tokens.background);
        direction::apply(
            ui.ctx(),
            if self.rtl {
                Direction::Rtl
            } else {
                Direction::Ltr
            },
        );
        ui.set_min_size(available);
        ui.set_max_width(available.x);

        if available.x < MOBILE_BREAKPOINT {
            self.mobile_shell(ui, available);
        } else {
            self.desktop_shell(ui, available);
        }
    }

    fn sync_theme(&mut self, ui: &mut Ui) {
        if self.applied_theme != Some(self.selected_theme) {
            theme::apply_preset(ui.ctx(), self.selected_theme);
            // Ui styles are cached when the Ui is created, before App::ui runs.
            ui.reset_style();
            self.applied_theme = Some(self.selected_theme);
            ui.ctx().request_repaint();
        }
    }

    fn select_section(&mut self, index: usize) {
        let index = index.min(SECTIONS.len() - 1);
        if index != self.selected_section {
            self.scroll_to_top = true;
        }
        self.selected_section = index;
        self.mobile_nav_open = false;
    }

    // ---------------------------------------------------------------- shells

    fn mobile_shell(&mut self, ui: &mut Ui, available: Vec2) {
        ui.vertical(|ui| {
            ui.set_width(available.x);
            padded(ui, 14, 8, |ui| self.mobile_header(ui));

            if self.mobile_nav_open {
                padded(ui, 14, 0, |ui| self.mobile_navigation(ui));
            }

            let height = ui.available_height().max(0.0);
            ui.allocate_ui_with_layout(
                vec2(available.x, height),
                egui::Layout::top_down(direction::start_align(ui)),
                |ui| self.scroll_content(ui, "showcase-mobile-content"),
            );
        });
    }

    fn mobile_header(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        direction::horizontal(ui, |ui| {
            ui.heading(RichText::new("Faseela").size(24.0).color(tokens.text));
            ui.with_layout(direction::trailing(ui), |ui| {
                let label = if self.mobile_nav_open {
                    "Close"
                } else {
                    "Sections"
                };
                let response = Button::new(label)
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small)
                    .id(Id::unique("showcase-mobile-menu"))
                    .show(ui);
                if response.clicked() {
                    self.mobile_nav_open = !self.mobile_nav_open;
                }
            });
        });
        ui.label(
            RichText::new(format!(
                "{} · {} · {}",
                SECTIONS[self.selected_section].0,
                self.selected_theme.label(),
                if self.rtl { "RTL" } else { "LTR" }
            ))
            .size(11.0)
            .color(tokens.muted_text),
        );
    }

    fn mobile_navigation(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            ui.label(
                RichText::new("SHOWCASE SECTIONS")
                    .size(9.5)
                    .strong()
                    .color(tokens.secondary),
            );
            ui.add_space(6.0);
            egui::ScrollArea::vertical()
                .id_salt("showcase-mobile-nav-scroll")
                .max_height(238.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (index, (title, _)) in SECTIONS.iter().enumerate() {
                        let selected = self.selected_section == index;
                        let response = Button::new(*title)
                            .variant(if selected {
                                ButtonVariant::Primary
                            } else {
                                ButtonVariant::Ghost
                            })
                            .selected(selected)
                            .size(ControlSize::Small)
                            .full_width()
                            .id(Id::unique("showcase-mobile-nav").with(index))
                            .show(ui);
                        if response.clicked() {
                            self.select_section(index);
                        }
                    }
                });
        });
    }

    fn desktop_shell(&mut self, ui: &mut Ui, available: Vec2) {
        ui.horizontal(|ui| {
            let sidebar_width = if available.x >= 1100.0 {
                232.0
            } else {
                SIDEBAR_WIDTH
            };
            ui.allocate_ui_with_layout(
                vec2(sidebar_width, available.y),
                egui::Layout::top_down(direction::start_align(ui)),
                |ui| {
                    ui.add_space(8.0);
                    self.sidebar(ui, available.y - 16.0);
                },
            );
            ui.allocate_ui_with_layout(
                vec2(ui.available_width().max(0.0), available.y),
                egui::Layout::top_down(direction::start_align(ui)),
                |ui| self.desktop_content(ui, available.y),
            );
        });
    }

    fn sidebar(&mut self, ui: &mut Ui, height: f32) {
        let tokens = theme::active_tokens(ui.ctx());
        Card::new()
            .outer_margin(Margin {
                left: 8,
                right: 8,
                top: 8,
                bottom: 8,
            })
            .padding(Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height((height - 44.0).max(0.0));
                ui.label(
                    RichText::new("Faseela")
                        .size(24.0)
                        .strong()
                        .color(tokens.text),
                );
                ui.label(
                    RichText::new("UI COMPONENT LIBRARY")
                        .size(9.0)
                        .strong()
                        .color(tokens.secondary),
                );
                ui.add_space(18.0);

                for (index, (title, _)) in SECTIONS.iter().enumerate() {
                    let selected = self.selected_section == index;
                    let response = Button::new(*title)
                        .variant(if selected {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Ghost
                        })
                        .selected(selected)
                        .size(ControlSize::Small)
                        .full_width()
                        .id(Id::unique("showcase-nav").with(index))
                        .show(ui);
                    if response.clicked() {
                        self.select_section(index);
                    }
                    ui.add_space(2.0);
                }

                ui.with_layout(egui::Layout::bottom_up(direction::start_align(ui)), |ui| {
                    ui.label(
                        RichText::new("Rust · egui · WebAssembly")
                            .size(9.5)
                            .color(tokens.muted_text),
                    );
                    ui.label(
                        RichText::new("Faseela UI 0.1")
                            .size(9.5)
                            .color(tokens.secondary),
                    );
                });
            });
    }

    fn desktop_content(&mut self, ui: &mut Ui, height: f32) {
        padded(ui, 18, 14, |ui| {
            let width = ui.available_width().min(CONTENT_MAX_WIDTH);
            let offset = ((ui.available_width() - width) * 0.5).max(0.0);
            ui.horizontal(|ui| {
                ui.add_space(offset);
                ui.allocate_ui_with_layout(
                    vec2(width, height),
                    egui::Layout::top_down(direction::start_align(ui)),
                    |ui| {
                        self.section_intro(ui);
                        ui.add_space(6.0);
                        ui.separator();
                        let remaining = ui.available_height().max(0.0);
                        ui.allocate_ui_with_layout(
                            vec2(ui.available_width(), remaining),
                            egui::Layout::top_down(direction::start_align(ui)),
                            |ui| self.scroll_content(ui, "showcase-desktop-content"),
                        );
                    },
                );
            });
        });
    }

    fn scroll_content(&mut self, ui: &mut Ui, salt: &'static str) {
        let mut area = egui::ScrollArea::vertical()
            .id_salt(salt)
            .auto_shrink([false, false]);
        if self.scroll_to_top {
            area = area.vertical_scroll_offset(0.0);
            self.scroll_to_top = false;
        }
        area.show(ui, |ui| {
            padded(ui, 2, 4, |ui| self.section_content(ui));
        });
    }

    // ------------------------------------------------------------- sections

    fn section_intro(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        let index = self.selected_section.min(SECTIONS.len() - 1);
        let (title, caption) = SECTIONS[index];
        let section_number = format!("{:02} / 10", index + 1);
        let direction_label = if self.rtl { "RTL" } else { "LTR" };
        ui.horizontal_wrapped(|ui| {
            ui.heading(RichText::new(title).size(25.0).color(tokens.text));
            ui.add(Badge::new(&section_number, BadgeTone::Neutral));
            ui.add(Badge::new(self.selected_theme.label(), BadgeTone::Primary));
            ui.add(Badge::new(direction_label, BadgeTone::Neutral));
        });
        ui.label(RichText::new(caption).size(12.0).color(tokens.muted_text));
    }

    fn section_content(&mut self, ui: &mut Ui) {
        let index = self.selected_section.min(SECTIONS.len() - 1);
        match index {
            0 => self.overview(ui),
            1 => self.foundations(ui),
            2 => self.buttons(ui),
            3 => self.form_controls(ui),
            4 => self.navigation(ui),
            5 => self.layout(ui),
            6 => self.feedback(ui),
            7 => self.appearance(ui),
            8 => self.accessibility(ui),
            _ => self.rtl_arabic(ui),
        }

        let tokens = theme::active_tokens(ui.ctx());
        ui.add_space(18.0);
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Faseela UI").strong().color(tokens.secondary));
            ui.label(
                RichText::new(
                    "Shared components · application-owned state · native and WebAssembly",
                )
                .size(10.5)
                .color(tokens.muted_text),
            );
        });
        ui.add_space(16.0);
    }

    fn overview(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            ui.label(
                RichText::new("WELCOME BACK")
                    .size(9.5)
                    .strong()
                    .color(tokens.secondary),
            );
            ui.add_space(4.0);
            ui.heading(
                RichText::new("A production-minded egui foundation")
                    .size(23.0)
                    .color(tokens.text),
            );
            ui.label(
                RichText::new(
                    "Every section is this application: the same library widgets painted in a \
                     responsive shell with real state, keyboard input, and four themes.",
                )
                .color(tokens.muted_text),
            );
            ui.add_space(8.0);
            let arabic = text::display_text("مرحبًا بك في مكتبة فسيلة للواجهات.");
            ui.add(
                egui::Label::new(RichText::new(arabic).size(13.0).color(tokens.secondary)).wrap(),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Quick actions")
                .caption("These buttons change the selected section, theme, or direction.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let open_buttons = Button::new("Browse buttons")
                    .variant(ButtonVariant::Primary)
                    .id(Id::unique("showcase-overview-buttons"))
                    .show(ui);
                if open_buttons.clicked() {
                    self.select_section(2);
                    self.last_action = "Opened the Buttons section.".to_owned();
                }

                let open_forms = Button::new("Try forms")
                    .variant(ButtonVariant::Secondary)
                    .id(Id::unique("showcase-overview-forms"))
                    .show(ui);
                if open_forms.clicked() {
                    self.select_section(3);
                    self.last_action = "Opened the Form Controls section.".to_owned();
                }

                let cycle_theme = Button::new("Cycle theme")
                    .variant(ButtonVariant::Outline)
                    .id(Id::unique("showcase-overview-theme"))
                    .show(ui);
                if cycle_theme.clicked() {
                    let index = ThemePreset::ALL
                        .iter()
                        .position(|preset| *preset == self.selected_theme)
                        .unwrap_or(0);
                    self.selected_theme = ThemePreset::ALL[(index + 1) % ThemePreset::ALL.len()];
                    self.last_action = format!("Theme set to {}.", self.selected_theme.label());
                }

                let toggle_direction = Button::new("Toggle direction")
                    .variant(ButtonVariant::Ghost)
                    .id(Id::unique("showcase-overview-direction"))
                    .show(ui);
                if toggle_direction.clicked() {
                    self.rtl = !self.rtl;
                    self.last_action = format!(
                        "Reading direction set to {}.",
                        if self.rtl { "RTL" } else { "LTR" }
                    );
                }
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new(&self.last_action)
                    .size(11.0)
                    .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Live state")
                .caption("Values reported by this running instance.")
                .show(ui);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                stat_tile(ui, "SECTIONS", "10");
                stat_tile(ui, "THEMES", "4");
                stat_tile(ui, "DIRECTION", if self.rtl { "RTL" } else { "LTR" });
                stat_tile(ui, "FORM PLAN", plan_label(self.plan));
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!(
                    "Theme {} · direction {} · notifications {} · filter “{}”",
                    self.selected_theme.label(),
                    if self.rtl {
                        "right-to-left"
                    } else {
                        "left-to-right"
                    },
                    if self.notifications { "on" } else { "off" },
                    if self.search.trim().is_empty() {
                        "none"
                    } else {
                        self.search.trim()
                    }
                ))
                .size(11.0)
                .color(tokens.muted_text),
            );
        });
    }

    fn foundations(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Semantic color tokens")
                .caption("Widgets paint from roles, not fixed colors, so presets switch cleanly.")
                .show(ui);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                for (name, color) in [
                    ("background", tokens.background),
                    ("surface", tokens.surface),
                    ("elevated_surface", tokens.elevated_surface),
                    ("primary", tokens.primary),
                    ("secondary", tokens.secondary),
                    ("accent", tokens.accent),
                    ("text", tokens.text),
                    ("muted_text", tokens.muted_text),
                    ("border", tokens.border),
                    ("control_border", tokens.control_border),
                    ("focus", tokens.focus),
                    ("error", tokens.error),
                    ("warning", tokens.warning),
                    ("success", tokens.success),
                ] {
                    token_swatch(ui, name, color, tokens);
                }
            });
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Typography")
                .caption("The crate uses egui's proportional family and application-scaled sizes.")
                .show(ui);
            ui.add_space(8.0);
            ui.label(
                RichText::new("Display heading")
                    .size(23.0)
                    .strong()
                    .color(tokens.text),
            );
            ui.label(
                RichText::new("Body copy supports the theme's primary text token.")
                    .size(13.0)
                    .color(tokens.text),
            );
            ui.label(
                RichText::new("Caption and supporting text use the muted token.")
                    .size(10.5)
                    .color(tokens.muted_text),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new("0123456789 · ABCDEFGHIJKLM · مرحبًا")
                    .size(12.0)
                    .color(tokens.secondary),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Spacing & shape")
                .caption("Shared widgets use a compact 4-point rhythm and 4–10 point radii.")
                .show(ui);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                for (label, width) in [("4", 4.0), ("8", 8.0), ("12", 12.0), ("20", 20.0)] {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(width + 44.0, 26.0), Sense::hover());
                    let bar =
                        Rect::from_min_size(rect.left_center() - vec2(0.0, 4.0), vec2(width, 8.0));
                    ui.painter()
                        .rect_filled(bar, CornerRadius::same(2), tokens.primary);
                    ui.painter().text(
                        pos2(bar.right() + 6.0, bar.center().y),
                        Align2::LEFT_CENTER,
                        format!("{label} pt"),
                        FontId::proportional(9.5),
                        tokens.muted_text,
                    );
                }
            });
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for radius in [2.0_f32, 4.0, 7.0, 10.0] {
                    let (rect, _) = ui.allocate_exact_size(vec2(58.0, 36.0), Sense::hover());
                    let shape = Rect::from_min_size(rect.min, vec2(42.0, 30.0));
                    ui.painter().rect(
                        shape,
                        CornerRadius::same(radius as u8),
                        tokens.elevated_surface,
                        Stroke::new(1.0, tokens.control_border),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        pos2(rect.right() - 10.0, rect.center().y),
                        Align2::RIGHT_CENTER,
                        format!("{}", radius as u8),
                        FontId::proportional(9.5),
                        tokens.muted_text,
                    );
                }
            });
        });
    }

    fn buttons(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Variants")
                .caption("Primary, secondary, outline, ghost, and destructive actions.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                for (label, variant, id) in [
                    ("Primary", ButtonVariant::Primary, "showcase-action-primary"),
                    (
                        "Secondary",
                        ButtonVariant::Secondary,
                        "showcase-action-secondary",
                    ),
                    ("Outline", ButtonVariant::Outline, "showcase-action-outline"),
                    ("Ghost", ButtonVariant::Ghost, "showcase-action-ghost"),
                    (
                        "Destructive",
                        ButtonVariant::Destructive,
                        "showcase-action-destructive",
                    ),
                ] {
                    let response = Button::new(label)
                        .variant(variant)
                        .id(Id::unique(id))
                        .show(ui);
                    if response.clicked() {
                        self.last_action = format!("{label} button activated.");
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new(&self.last_action)
                    .size(11.0)
                    .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Sizes & icons")
                .caption("Small, medium, and large controls; leading glyphs and icon-only actions.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    Button::new("Small")
                        .size(ControlSize::Small)
                        .variant(ButtonVariant::Secondary),
                );
                ui.add(
                    Button::new("Medium")
                        .size(ControlSize::Medium)
                        .variant(ButtonVariant::Secondary),
                );
                ui.add(
                    Button::new("Large")
                        .size(ControlSize::Large)
                        .variant(ButtonVariant::Secondary),
                );
                ui.add(Button::new("With icon").icon("+"));
            });
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                let selected = Button::new("Selected state")
                    .variant(ButtonVariant::Outline)
                    .selected(self.action_selected)
                    .id(Id::unique("showcase-action-selected"))
                    .show(ui);
                if selected.clicked() {
                    self.action_selected = !self.action_selected;
                    self.last_action = format!(
                        "Selected button is {}.",
                        if self.action_selected { "on" } else { "off" }
                    );
                }

                let disabled = Button::new("Disabled")
                    .variant(ButtonVariant::Primary)
                    .enabled(false)
                    .id(Id::unique("showcase-action-disabled"))
                    .show(ui);
                let _ = disabled;

                let icon_style = icon_style(tokens);
                let icon = IconButton::new("Toggle bookmark", vec2(38.0, 38.0), &icon_style)
                    .selected(self.bookmarked)
                    .id(Id::unique("showcase-icon-bookmark"))
                    .show(ui, |ui, rect, tint| {
                        let center = rect.center();
                        let points = vec![
                            pos2(center.x, center.y - 8.0),
                            pos2(center.x + 2.6, center.y - 2.4),
                            pos2(center.x + 8.4, center.y - 1.8),
                            pos2(center.x + 4.0, center.y + 2.0),
                            pos2(center.x + 5.4, center.y + 8.0),
                            pos2(center.x, center.y + 4.9),
                            pos2(center.x - 5.4, center.y + 8.0),
                            pos2(center.x - 4.0, center.y + 2.0),
                            pos2(center.x - 8.4, center.y - 1.8),
                            pos2(center.x - 2.6, center.y - 2.4),
                        ];
                        ui.painter()
                            .add(egui::Shape::convex_polygon(points, tint, Stroke::NONE));
                    });
                if icon.clicked() {
                    self.bookmarked = !self.bookmarked;
                    self.last_action = format!(
                        "Bookmark icon is {}.",
                        if self.bookmarked { "on" } else { "off" }
                    );
                }
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new("Disabled controls remain focusable to assistive technology but never activate.")
                    .size(10.5)
                    .color(tokens.muted_text),
            );
        });
    }

    fn form_controls(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Text & search")
                .caption("The application owns every value; the widgets report changes.")
                .show(ui);
            ui.add_space(8.0);
            TextField::new(&mut self.name)
                .label("Full name")
                .hint("e.g. Faseela Rahman")
                .helper("Used only inside this demo.")
                .id(Id::unique("showcase-name"))
                .show(ui);
            ui.add_space(8.0);

            let invalid_email = !self.email.trim().is_empty() && !self.email.contains('@');
            let mut email = TextField::new(&mut self.email)
                .label("Email")
                .hint("name@example.com")
                .id(Id::unique("showcase-email"));
            email = if invalid_email {
                email.error("Include an @ sign to make this look like an email.")
            } else {
                email.helper("A small validation rule runs on the application side.")
            };
            email.show(ui);

            ui.add_space(8.0);
            SearchField::new(&mut self.search)
                .label("Filter components")
                .hint("Try “form” or “rtl”")
                .id(Id::unique("showcase-search"))
                .show(ui);
            let query = self.search.trim().to_lowercase();
            let matches: Vec<&str> = FILTERABLE_COMPONENTS
                .iter()
                .copied()
                .filter(|name| query.is_empty() || name.to_lowercase().contains(&query))
                .collect();
            ui.label(
                RichText::new(if query.is_empty() {
                    format!("{} components available.", FILTERABLE_COMPONENTS.len())
                } else {
                    format!(
                        "{} match{} for “{}”: {}",
                        matches.len(),
                        if matches.len() == 1 { "" } else { "es" },
                        self.search.trim(),
                        if matches.is_empty() {
                            "none".to_owned()
                        } else {
                            matches.join(", ")
                        }
                    )
                })
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Choices")
                .caption(
                    "Checkbox, switch, radio group, and select all share tokens and focus rules.",
                )
                .show(ui);
            ui.add_space(8.0);
            Checkbox::new("Product notifications", &mut self.notifications)
                .helper("Also updates the Feedback & Status section.")
                .id(Id::unique("showcase-notify"))
                .show(ui);
            Switch::new("Compact spacing", &mut self.compact_mode)
                .helper("Changes the preview padding below.")
                .id(Id::unique("showcase-compact"))
                .show(ui);
            ui.add_space(8.0);
            RadioGroup::new(&["Compact", "Comfortable", "Spacious"], &mut self.density)
                .label("Density")
                .helper("Arrow keys follow the reading direction.")
                .id(Id::unique("showcase-density"))
                .show(ui);
            ui.add_space(8.0);
            Select::new("Plan", &["Starter", "Team", "Enterprise"], &mut self.plan)
                .helper("The selected index stays in application state.")
                .id(Id::unique("showcase-plan"))
                .show(ui);
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Application state")
                .caption("A snapshot of the values the library has proposed so far.")
                .show(ui);
            ui.add_space(8.0);
            let padding = if self.compact_mode { 5 } else { 11 };
            Card::new()
                .elevated(true)
                .padding(Margin::same(padding))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(if self.compact_mode {
                            "Compact preview"
                        } else {
                            "Comfortable preview"
                        })
                        .strong()
                        .color(tokens.text),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{} · {} plan · {} density",
                            if self.name.trim().is_empty() {
                                "Anonymous"
                            } else {
                                self.name.trim()
                            },
                            plan_label(self.plan),
                            density_label(self.density)
                        ))
                        .size(11.0)
                        .color(tokens.muted_text),
                    );
                });
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!(
                    "Notifications {} · search “{}”",
                    if self.notifications { "on" } else { "off" },
                    self.search.trim()
                ))
                .size(10.5)
                .color(tokens.muted_text),
            );
        });
    }

    fn navigation(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Breadcrumb")
                .caption("Earlier path entries stay interactive; the current page does not.")
                .show(ui);
            ui.add_space(8.0);
            let crumbs = ["Workspace", "Projects", "Faseela UI", "Navigation"];
            let result = Breadcrumb::new(&crumbs)
                .id(Id::unique("showcase-breadcrumb"))
                .show(ui);
            if let Some(index) = result.activated {
                self.breadcrumb_last = Some(index);
                self.last_action = format!("Breadcrumb opened “{}”.", crumbs[index]);
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(match self.breadcrumb_last {
                    Some(index) => format!("Last opened crumb: {}", crumbs[index]),
                    None => "No breadcrumb has been activated.".to_owned(),
                })
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Modes & views")
                .caption(
                    "Exclusive controls update application-owned indices and report activation.",
                )
                .show(ui);
            ui.add_space(8.0);
            let segmented =
                SegmentedControl::new(&["Design", "Develop", "Ship"], &mut self.nav_segment)
                    .label("Workspace view")
                    .full_width()
                    .id(Id::unique("showcase-segmented"))
                    .show(ui);
            if let Some(index) = segmented.activated {
                self.last_action = format!(
                    "Workspace view set to {}.",
                    ["Design", "Develop", "Ship"][index]
                );
            }
            ui.add_space(10.0);
            let group = ButtonGroup::new(&["Day", "Week", "Month"], &mut self.nav_group)
                .accessible_label("Reporting period")
                .id(Id::unique("showcase-period"))
                .show(ui);
            if let Some(index) = group.activated {
                self.last_action = format!(
                    "Reporting period set to {}.",
                    ["Day", "Week", "Month"][index]
                );
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!(
                    "View {} · period {}",
                    ["Design", "Develop", "Ship"][self.nav_segment.min(2)],
                    ["Day", "Week", "Month"][self.nav_group.min(2)]
                ))
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Tabs")
                .caption("The Tab primitive paints with caller-owned selection and focus.")
                .show(ui);
            ui.add_space(8.0);
            self.tab_strip(ui);
            ui.add_space(8.0);
            ui.label(
                RichText::new(match self.nav_tab {
                    0 => "Preview: responsive content area.",
                    _ => "Details: keyboard focus and selected state are managed by the caller.",
                })
                .size(11.0)
                .color(tokens.muted_text),
            );
        });
    }

    fn tab_strip(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        let labels = ["Preview", "Details"];
        let width = ui.available_width().max(1.0);
        let gap = 4.0;
        let tab_width = ((width - gap) * 0.5).max(1.0);
        let (outer, _) = ui.allocate_exact_size(vec2(width, 34.0), Sense::hover());
        for (index, label) in labels.iter().enumerate() {
            let rect = Rect::from_min_size(
                pos2(outer.left() + index as f32 * (tab_width + gap), outer.top()),
                vec2(tab_width, 34.0),
            );
            let selected = self.nav_tab == index;
            let response = Tab::new(Id::unique("showcase-tabs").with(index), label, selected)
                .show_at(ui, rect, |ui, response| {
                    let fill = if selected {
                        tokens.selection
                    } else if response.hovered() {
                        tokens.hover
                    } else {
                        tokens.elevated_surface
                    };
                    let fore = if selected {
                        tokens.on_primary
                    } else {
                        tokens.text
                    };
                    ui.painter().rect_filled(rect, CornerRadius::same(7), fill);
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        *label,
                        FontId::proportional(12.5),
                        fore,
                    );
                });
            if response.clicked() {
                self.nav_tab = index;
            }
        }
    }

    fn layout(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Resizable panes")
                .caption("Drag the divider, or focus it and use the arrow keys.")
                .show(ui);
            ui.add_space(8.0);
            let mut axis = self.split_axis;
            let axis_control = ButtonGroup::new(&["Columns", "Rows"], &mut axis)
                .accessible_label("Pane axis")
                .id(Id::unique("showcase-split-axis"))
                .show(ui);
            if axis_control.activated.is_some() {
                self.split_axis = axis;
            }
            ui.add_space(8.0);
            self.split_preview(ui);
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Toolbar fitting")
                .caption("Trailing commands move into an accessible overflow menu.")
                .show(ui);
            ui.add_space(8.0);
            let items: Vec<ToolbarItem<'_>> = TOOLBAR_COMMANDS
                .iter()
                .enumerate()
                .map(|(index, label)| {
                    let item = ToolbarItem::new(label);
                    if index == DISABLED_TOOLBAR_COMMAND {
                        item.disabled()
                    } else {
                        item
                    }
                })
                .collect();
            let result = toolbar_response(ui, Id::unique("showcase-toolbar"), &items);
            if let Some(index) = result.activated {
                self.last_command = Some(index);
            }
            ui.add_space(6.0);
            let state = match result.state {
                ToolbarState::Empty => "empty",
                ToolbarState::Fits => "all commands fit",
                ToolbarState::Overflow => "overflow menu active",
                ToolbarState::NoSpace => "not enough room",
            };
            ui.label(
                RichText::new(format!(
                    "{state} · {} visible · {} in overflow",
                    result.shown.len(),
                    result.overflow.len()
                ))
                .size(10.5)
                .color(tokens.muted_text),
            );
            if let Some(index) = self.last_command {
                ui.label(
                    RichText::new(format!(
                        "Last command: {}",
                        TOOLBAR_COMMANDS.get(index).copied().unwrap_or("unknown")
                    ))
                    .size(10.5)
                    .color(tokens.secondary),
                );
            }
        });
    }

    fn split_preview(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        let rtl = self.rtl;
        let horizontal = self.split_axis == 0;
        let axis = if horizontal {
            SplitAxis::Horizontal
        } else {
            SplitAxis::Vertical
        };
        let height = if horizontal { 92.0 } else { 182.0 };
        let width = ui.available_width().max(1.0);
        let (outer, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let current = self.split_size.unwrap_or(SplitSize::Ratio(0.62));

        if let Some(split) = split_rect(outer, axis, current, 8.0, 56.0) {
            let swap = rtl && horizontal;
            let (leading, trailing) = if swap {
                (split.second, split.first)
            } else {
                (split.first, split.second)
            };
            let painter = ui.painter();
            painter.rect_filled(leading, CornerRadius::same(8), tokens.primary);
            painter.rect_filled(split.divider, CornerRadius::same(3), tokens.control_border);
            painter.rect_filled(trailing, CornerRadius::same(8), tokens.elevated_surface);
            painter.text(
                leading.center(),
                Align2::CENTER_CENTER,
                "PRIMARY",
                FontId::proportional(9.5),
                tokens.on_primary,
            );
            painter.text(
                trailing.center(),
                Align2::CENTER_CENTER,
                "SECONDARY",
                FontId::proportional(9.5),
                tokens.text,
            );

            let handle_style = ResizeHandleStyle {
                hovered: tokens.hover,
                dragged: tokens.active,
                focused: tokens.focus,
            };
            let mut handle = ResizeHandle::new(
                Id::unique("showcase-split-handle"),
                axis,
                split.divider,
                "Resize preview panes",
                handle_style,
            )
            .hit_margin(8.0)
            .keyboard_step(8.0)
            .reversed(swap);
            if split.available() >= 128.0 {
                let leading_size = if swap {
                    split.available() - split.first_size()
                } else {
                    split.first_size()
                };
                handle = handle.value(leading_size, 56.0..=(split.available() - 56.0));
            }
            let resize = handle.show(ui);
            if resize.delta != 0.0
                && let Some(next) = split.resized(current, resize.delta, 56.0)
            {
                self.split_size = Some(next);
                ui.ctx().request_repaint();
            }
        }

        ui.label(
            RichText::new("Physical geometry stays stable; logical panes mirror in RTL.")
                .size(10.5)
                .color(tokens.muted_text),
        );
    }

    fn feedback(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Progress")
                .caption("A determinate bar with visible and accessible percent value.")
                .show(ui);
            ui.add_space(8.0);
            ProgressBar::new(self.progress)
                .label("Release upload")
                .show_value(true)
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if Button::new("Advance 10%")
                    .variant(ButtonVariant::Secondary)
                    .size(ControlSize::Small)
                    .id(Id::unique("showcase-progress-advance"))
                    .show(ui)
                    .clicked()
                {
                    self.progress = (self.progress + 0.1).min(1.0);
                }
                if Button::new("Reset")
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small)
                    .id(Id::unique("showcase-progress-reset"))
                    .show(ui)
                    .clicked()
                {
                    self.progress = 0.0;
                }
                ui.label(
                    RichText::new(format!("{:.0}% complete", self.progress * 100.0))
                        .size(10.5)
                        .color(tokens.muted_text),
                );
            });
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Loading & status")
                .caption("Spinners and status rows use semantic tones from the active theme.")
                .show(ui);
            ui.add_space(8.0);
            if Button::new(if self.show_loading {
                "Stop loading"
            } else {
                "Start loading"
            })
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small)
            .id(Id::unique("showcase-loading-toggle"))
            .show(ui)
            .clicked()
            {
                self.show_loading = !self.show_loading;
            }
            ui.add_space(8.0);
            if self.show_loading {
                LoadingIndicator::new().label("Syncing workspace…").show(ui);
            } else {
                StatusIndicator::new("Idle", BadgeTone::Neutral)
                    .detail("No background work")
                    .show(ui);
            }
            ui.add_space(8.0);
            StatusIndicator::new("API connected", BadgeTone::Success)
                .detail("42 ms")
                .show(ui);
            StatusIndicator::new("Token expires soon", BadgeTone::Warning)
                .detail("in 2 days")
                .show(ui);
            StatusIndicator::new("Sync failed", BadgeTone::Error)
                .detail("retry queued")
                .show(ui);
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Badges, empty state & tooltip")
                .caption("Tone foregrounds and measurements come from the same tokens.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.add(Badge::new("Neutral", BadgeTone::Neutral));
                ui.add(Badge::new("Primary", BadgeTone::Primary));
                ui.add(Badge::new("Success", BadgeTone::Success));
                ui.add(Badge::new("Warning", BadgeTone::Warning));
                ui.add(Badge::new("Error", BadgeTone::Error));
            });
            ui.add_space(10.0);
            if self.notifications {
                for (title, tone, detail) in [
                    ("Build finished", BadgeTone::Success, "2 minutes ago"),
                    ("Update available", BadgeTone::Warning, "version 0.2"),
                    ("Sync failed", BadgeTone::Error, "retry scheduled"),
                ] {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(title).size(11.5).color(tokens.text));
                        ui.add(Badge::new(detail, tone));
                    });
                }
                if Button::new("Clear notifications")
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small)
                    .id(Id::unique("showcase-clear-notifications"))
                    .show(ui)
                    .clicked()
                {
                    self.notifications = false;
                }
            } else {
                let empty = EmptyState::new("Inbox is clear")
                    .description("Turn on notifications to see the populated state.")
                    .action("Show sample notifications")
                    .show(ui);
                if empty.inner.action_clicked {
                    self.notifications = true;
                }
            }
            ui.add_space(8.0);
            let tooltip_response = Button::new("Hover for tooltip")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small)
                .id(Id::unique("showcase-tooltip"))
                .show(ui);
            let _ = Tooltip::new(
                "Tooltips are keyboard-adjacent supporting text; the visible label still names the control.",
            )
            .show(tooltip_response);
        });
    }

    fn appearance(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Theme presets")
                .caption("Switching a preset re-applies semantic tokens on the next frame.")
                .show(ui);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                for (index, preset) in ThemePreset::ALL.iter().enumerate() {
                    let selected = *preset == self.selected_theme;
                    let response = Button::new(preset.label())
                        .variant(if selected {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Outline
                        })
                        .selected(selected)
                        .id(Id::unique("showcase-theme").with(index))
                        .show(ui);
                    if response.clicked() {
                        self.selected_theme = *preset;
                        self.last_action = format!("Theme set to {}.", preset.label());
                        ui.ctx().request_repaint();
                    }
                }
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!(
                    "Selected: {} · painted: {}",
                    self.selected_theme.label(),
                    theme::active_preset(ui.ctx()).label()
                ))
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Faseela identity")
                .caption("The official palette anchors the default preset.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                for (name, color) in [
                    ("Deep", palette::BACKGROUND),
                    ("Green", palette::PRIMARY),
                    ("Mint", palette::PRIMARY_LIGHT),
                    ("Gold", palette::ACCENT),
                ] {
                    palette_swatch(ui, name, color, tokens);
                }
            });
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Measured contrast")
                .caption("Computed with WCAG relative luminance from the selected preset.")
                .show(ui);
            ui.add_space(8.0);
            let checks = contrast_checks(self.selected_theme.tokens());
            for check in checks.iter().take(7) {
                contrast_row(ui, check);
            }
            let failures = checks.iter().filter(|check| !check.passes()).count();
            ui.add_space(6.0);
            ui.add(Badge::new(
                if failures == 0 {
                    "All sampled pairs pass"
                } else {
                    "Some sampled pairs fail"
                },
                if failures == 0 {
                    BadgeTone::Success
                } else {
                    BadgeTone::Error
                },
            ));
        });
    }

    fn accessibility(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Contrast evidence")
                .caption("WCAG AA: 4.5:1 for normal text and 3:1 for non-text indicators.")
                .show(ui);
            ui.add_space(8.0);
            let checks = contrast_checks(tokens);
            let text: Vec<ContrastCheck> = checks
                .iter()
                .copied()
                .filter(|check| check.minimum >= 4.5)
                .collect();
            let non_text: Vec<ContrastCheck> = checks
                .iter()
                .copied()
                .filter(|check| check.minimum < 4.5)
                .collect();
            let text_pass = text.iter().filter(|check| check.passes()).count();
            let non_text_pass = non_text.iter().filter(|check| check.passes()).count();
            ui.horizontal_wrapped(|ui| {
                ui.add(Badge::new(
                    &format!("Text {text_pass}/{}", text.len()),
                    if text_pass == text.len() {
                        BadgeTone::Success
                    } else {
                        BadgeTone::Error
                    },
                ));
                ui.add(Badge::new(
                    &format!("Non-text {non_text_pass}/{}", non_text.len()),
                    if non_text_pass == non_text.len() {
                        BadgeTone::Success
                    } else {
                        BadgeTone::Error
                    },
                ));
            });
            ui.add_space(8.0);
            for check in checks.iter().take(7) {
                contrast_row(ui, check);
            }
            ui.label(
                RichText::new(
                    "Ratios are sampled pairs, not a complete audit of every custom composition.",
                )
                .size(10.0)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Keyboard focus")
                .caption("Focus can be moved programmatically and is painted by the widgets.")
                .show(ui);
            ui.add_space(8.0);
            let trigger_id = Id::unique("showcase-a11y-focus-trigger");
            let target_id = Id::unique("showcase-a11y-target");
            ui.horizontal_wrapped(|ui| {
                if Button::new("Focus the target")
                    .variant(ButtonVariant::Secondary)
                    .size(ControlSize::Small)
                    .id(trigger_id)
                    .show(ui)
                    .clicked()
                {
                    ui.memory_mut(|memory| memory.request_focus(target_id));
                }
                ui.add(
                    Button::new("Focus target")
                        .variant(ButtonVariant::Outline)
                        .size(ControlSize::Small)
                        .id(target_id),
                );
            });
            let focused = ui.ctx().memory(|memory| memory.focused()) == Some(target_id);
            ui.add_space(4.0);
            ui.label(
                RichText::new(if focused {
                    "Focus is on the target. Space or Enter would activate it."
                } else {
                    "Focus is elsewhere. Use the button above or press Tab."
                })
                .size(10.5)
                .color(if focused {
                    tokens.secondary
                } else {
                    tokens.muted_text
                }),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Disabled & reviewed")
                .caption("Disabled widgets stay named and visible without accepting activation.")
                .show(ui);
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    Button::new("Disabled action")
                        .variant(ButtonVariant::Primary)
                        .enabled(false),
                );
                let mut disabled_value = false;
                Checkbox::new("Disabled checkbox", &mut disabled_value)
                    .enabled(false)
                    .show(ui);
            });
            ui.add_space(8.0);
            Checkbox::new(
                "I can reach every interactive control with Tab",
                &mut self.keyboard_review,
            )
            .helper("A real checkbox, not an automated claim.")
            .id(Id::unique("showcase-keyboard-review"))
            .show(ui);
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Known limitation: eframe's web canvas does not expose per-widget AccessKit nodes \
                     to browsers; native AccessKit support depends on the platform backend.",
                )
                .size(10.0)
                .color(tokens.muted_text),
            );
        });
    }

    fn rtl_arabic(&mut self, ui: &mut Ui) {
        let tokens = theme::active_tokens(ui.ctx());
        card(ui, |ui| {
            SectionHeader::new("Reading direction")
                .caption("The shared layout helpers mirror navigation and pane order.")
                .show(ui);
            ui.add_space(8.0);
            let mut selected = usize::from(self.rtl);
            let result = ButtonGroup::new(&["LTR", "RTL"], &mut selected)
                .accessible_label("Reading direction")
                .id(Id::unique("showcase-rtl"))
                .show(ui);
            if result.activated.is_some() {
                self.rtl = selected == 1;
                self.last_action = format!(
                    "Reading direction set to {}.",
                    if self.rtl { "RTL" } else { "LTR" }
                );
            }
            ui.add_space(8.0);
            ui.add(Badge::new(
                if self.rtl {
                    "RIGHT TO LEFT"
                } else {
                    "LEFT TO RIGHT"
                },
                BadgeTone::Primary,
            ));
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!(
                    "Active direction: {} · segmented controls and split panes follow it.",
                    if self.rtl { "RTL" } else { "LTR" }
                ))
                .size(10.5)
                .color(tokens.muted_text),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Arabic display text")
                .caption("The display helper applies Arabic shaping for painting only.")
                .show(ui);
            ui.add_space(8.0);
            let headline = text::display_text("فسيلة: واجهة عربية أنيقة وسريعة");
            ui.add(
                egui::Label::new(RichText::new(headline).size(18.0).color(tokens.text))
                    .halign(if self.rtl { Align::RIGHT } else { Align::LEFT })
                    .wrap(),
            );
            let body = text::display_text(
                "تعرض هذه الصفحة نصًا عربيًا متصل الحروف، مع أرقام لاتينية مثل 2026 و3.5.",
            );
            ui.add(
                egui::Label::new(RichText::new(body).size(12.0).color(tokens.muted_text))
                    .halign(if self.rtl { Align::RIGHT } else { Align::LEFT })
                    .wrap(),
            );
        });

        ui.add_space(12.0);
        card(ui, |ui| {
            SectionHeader::new("Logical text stays logical")
                .caption("Edit the source below; the display line is generated separately.")
                .show(ui);
            ui.add_space(8.0);
            TextField::new(&mut self.arabic_input)
                .label("Arabic source")
                .hint("Type logical Unicode text")
                .helper("Stored as typed; display shaping is a separate step.")
                .id(Id::unique("showcase-arabic-input"))
                .show(ui);
            ui.add_space(6.0);
            let shaped = text::display_text(&self.arabic_input);
            ui.add(
                egui::Label::new(RichText::new(shaped).size(13.0).color(tokens.secondary))
                    .halign(if self.rtl { Align::RIGHT } else { Align::LEFT })
                    .wrap(),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(
                    "Mirrored: navigation order, split panes, directional arrows, and control alignment. \
                     Logical: numbers, URLs, code, and stored text. Full bidi shaping in editable text \
                     is not provided; the helper is for display labels.",
                )
                .size(10.0)
                .color(tokens.muted_text),
            );
        });
    }
}

// ------------------------------------------------------------------ helpers

fn padded<R>(ui: &mut Ui, horizontal: i8, vertical: i8, add: impl FnOnce(&mut Ui) -> R) -> R {
    egui::Frame::new()
        .inner_margin(Margin::symmetric(horizontal, vertical))
        .show(ui, add)
        .inner
}

fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    Card::new()
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

fn stat_tile(ui: &mut Ui, label: &str, value: &str) {
    let tokens = theme::active_tokens(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(116.0, 58.0), Sense::hover());
    ui.painter().rect(
        rect,
        CornerRadius::same(8),
        tokens.elevated_surface,
        Stroke::new(1.0, tokens.border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.left_top() + vec2(9.0, 8.0),
        Align2::LEFT_TOP,
        label,
        FontId::proportional(8.5),
        tokens.muted_text,
    );
    ui.painter().text(
        rect.left_top() + vec2(9.0, 24.0),
        Align2::LEFT_TOP,
        value,
        FontId::proportional(15.0),
        tokens.text,
    );
}

fn token_swatch(ui: &mut Ui, name: &str, color: Color32, tokens: ThemeTokens) {
    let (rect, _) = ui.allocate_exact_size(vec2(112.0, 58.0), Sense::hover());
    let swatch = Rect::from_min_size(rect.min, vec2(112.0, 29.0));
    ui.painter().rect(
        swatch,
        CornerRadius::same(5),
        color,
        Stroke::new(1.0, tokens.control_border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        swatch.left_bottom() + vec2(0.0, 3.0),
        Align2::LEFT_TOP,
        name,
        FontId::proportional(9.5),
        tokens.text,
    );
    ui.painter().text(
        swatch.left_bottom() + vec2(0.0, 14.0),
        Align2::LEFT_TOP,
        hex(color),
        FontId::proportional(8.5),
        tokens.muted_text,
    );
}

fn palette_swatch(ui: &mut Ui, name: &str, color: Color32, tokens: ThemeTokens) {
    let (rect, _) = ui.allocate_exact_size(vec2(64.0, 52.0), Sense::hover());
    let swatch = Rect::from_min_size(rect.min, vec2(64.0, 31.0));
    ui.painter().rect(
        swatch,
        CornerRadius::same(6),
        color,
        Stroke::new(1.0, tokens.control_border),
        StrokeKind::Inside,
    );
    ui.painter().text(
        swatch.left_bottom() + vec2(0.0, 2.0),
        Align2::LEFT_TOP,
        name,
        FontId::proportional(9.5),
        tokens.text,
    );
    ui.painter().text(
        swatch.left_bottom() + vec2(0.0, 13.0),
        Align2::LEFT_TOP,
        hex(color),
        FontId::proportional(8.0),
        tokens.muted_text,
    );
}

#[derive(Clone, Copy, Debug)]
struct ContrastCheck {
    label: &'static str,
    foreground: Color32,
    background: Color32,
    minimum: f32,
}

impl ContrastCheck {
    fn ratio(self) -> f32 {
        theme::contrast_ratio(self.foreground, self.background)
    }

    fn passes(self) -> bool {
        self.ratio() + f32::EPSILON >= self.minimum
    }
}

fn contrast_checks(tokens: ThemeTokens) -> Vec<ContrastCheck> {
    vec![
        ContrastCheck {
            label: "text / background",
            foreground: tokens.text,
            background: tokens.background,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "text / surface",
            foreground: tokens.text,
            background: tokens.surface,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "muted text / background",
            foreground: tokens.muted_text,
            background: tokens.background,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "secondary / background",
            foreground: tokens.secondary,
            background: tokens.background,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "on primary / primary",
            foreground: tokens.on_primary,
            background: tokens.primary,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "on warning / warning",
            foreground: tokens.on_warning,
            background: tokens.warning,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "on error / error",
            foreground: tokens.on_error,
            background: tokens.error,
            minimum: 4.5,
        },
        ContrastCheck {
            label: "focus / background",
            foreground: tokens.focus,
            background: tokens.background,
            minimum: 3.0,
        },
        ContrastCheck {
            label: "control border / surface",
            foreground: tokens.control_border,
            background: tokens.surface,
            minimum: 3.0,
        },
    ]
}

fn contrast_row(ui: &mut Ui, check: &ContrastCheck) {
    let tokens = theme::active_tokens(ui.ctx());
    let ratio = check.ratio();
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(check.label).size(10.5).color(tokens.text));
        ui.add(Badge::new(
            &format!("{ratio:.2}:1"),
            if check.passes() {
                BadgeTone::Success
            } else {
                BadgeTone::Error
            },
        ));
        ui.label(
            RichText::new(format!("minimum {:.1}:1", check.minimum))
                .size(9.5)
                .color(tokens.muted_text),
        );
    });
}

fn icon_style(tokens: ThemeTokens) -> IconButtonStyle {
    IconButtonStyle::themed(tokens, ButtonVariant::Outline, ControlSize::Medium)
}

fn plan_label(index: usize) -> &'static str {
    match index {
        1 => "Team",
        2 => "Enterprise",
        _ => "Starter",
    }
}

fn density_label(index: usize) -> &'static str {
    match index {
        1 => "Comfortable",
        2 => "Spacious",
        _ => "Compact",
    }
}

fn hex(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, Key, Modifiers, PointerButton, Pos2, RawInput};

    const VIEWPORTS: [(f32, f32); 7] = [
        (320.0, 568.0),
        (360.0, 640.0),
        (390.0, 844.0),
        (680.0, 800.0),
        (768.0, 1024.0),
        (1280.0, 800.0),
        (1920.0, 1080.0),
    ];

    fn frame(
        ctx: &egui::Context,
        size: Vec2,
        app: &mut Showcase,
        events: Vec<Event>,
    ) -> egui::FullOutput {
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                focused: true,
                events,
                ..Default::default()
            },
            |ui| app.show(ui),
        );
        output.textures_delta.clear();
        output
    }

    fn setup(preset: ThemePreset, rtl: bool) -> egui::Context {
        let ctx = egui::Context::default();
        theme::apply_preset(&ctx, preset);
        direction::apply(&ctx, if rtl { Direction::Rtl } else { Direction::Ltr });
        ctx
    }

    fn click_at(pos: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ]
    }

    fn response_rect(ctx: &egui::Context, id: Id) -> Rect {
        ctx.read_response(id)
            .unwrap_or_else(|| panic!("missing response for {id:?}"))
            .rect
    }

    #[test]
    fn every_section_renders_without_horizontal_overflow_at_target_viewports() {
        for (width, height) in VIEWPORTS {
            for section in 0..SECTIONS.len() {
                let ctx = setup(ThemePreset::FaseelaDark, false);
                let mut app = Showcase {
                    selected_section: section,
                    ..Default::default()
                };
                let size = vec2(width, height);
                let output = frame(&ctx, size, &mut app, vec![]);
                assert!(
                    !output.shapes.is_empty(),
                    "{width}x{height} section {section} painted nothing"
                );
                let used = ctx.globally_used_rect();
                assert!(
                    used.width() <= width + 1.0,
                    "{width}x{height} section {section} used width {}",
                    used.width()
                );
                assert!(
                    used.right() <= width + 1.0,
                    "{width}x{height} section {section} right edge {}",
                    used.right()
                );
            }
        }
    }

    #[test]
    fn every_section_renders_in_all_themes_and_directions() {
        for preset in ThemePreset::ALL {
            for rtl in [false, true] {
                for section in 0..SECTIONS.len() {
                    let ctx = setup(preset, rtl);
                    let mut app = Showcase {
                        rtl,
                        selected_theme: preset,
                        selected_section: section,
                        ..Default::default()
                    };
                    let output = frame(&ctx, vec2(390.0, 844.0), &mut app, vec![]);
                    assert!(
                        !output.shapes.is_empty(),
                        "{preset:?} rtl={rtl} section {section} painted nothing"
                    );
                    assert!(
                        ctx.globally_used_rect().width() <= 391.0,
                        "{preset:?} rtl={rtl} section {section} overflowed"
                    );
                }
            }
        }
    }

    #[test]
    fn desktop_navigation_clicks_change_the_section() {
        let ctx = setup(ThemePreset::FaseelaDark, false);
        let mut app = Showcase::default();
        let size = vec2(1280.0, 800.0);
        let _ = frame(&ctx, size, &mut app, vec![]);
        let rect = response_rect(&ctx, Id::unique("showcase-nav").with(4));
        let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
        assert_eq!(app.selected_section, 4);
    }

    #[test]
    fn mobile_menu_opens_and_navigates_to_a_section() {
        let ctx = setup(ThemePreset::Light, false);
        let mut app = Showcase::default();
        let size = vec2(360.0, 640.0);
        let _ = frame(&ctx, size, &mut app, vec![]);
        assert!(!app.mobile_nav_open);
        let menu_rect = response_rect(&ctx, Id::unique("showcase-mobile-menu"));
        let _ = frame(&ctx, size, &mut app, click_at(menu_rect.center()));
        assert!(app.mobile_nav_open);

        let _ = frame(&ctx, size, &mut app, vec![]);
        let item_rect = response_rect(&ctx, Id::unique("showcase-mobile-nav").with(6));
        let _ = frame(&ctx, size, &mut app, click_at(item_rect.center()));
        assert_eq!(app.selected_section, 6);
        assert!(!app.mobile_nav_open);
    }

    #[test]
    fn theme_buttons_apply_the_preset_on_the_following_frame() {
        let ctx = setup(ThemePreset::FaseelaDark, false);
        let mut app = Showcase {
            selected_section: 7,
            ..Default::default()
        };
        let size = vec2(1280.0, 800.0);
        let _ = frame(&ctx, size, &mut app, vec![]);
        let rect = response_rect(&ctx, Id::unique("showcase-theme").with(2));
        let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
        assert_eq!(app.selected_theme, ThemePreset::Forest);
        let _ = frame(&ctx, size, &mut app, vec![]);
        assert_eq!(theme::active_preset(&ctx), ThemePreset::Forest);
    }

    #[test]
    fn rtl_control_switches_direction_and_persists() {
        let ctx = setup(ThemePreset::FaseelaDark, false);
        let mut app = Showcase {
            selected_section: 9,
            ..Default::default()
        };
        let size = vec2(390.0, 844.0);
        let _ = frame(&ctx, size, &mut app, vec![]);
        let rect = response_rect(&ctx, Id::unique("showcase-rtl").with(("group-item", 1)));
        let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
        assert!(app.rtl);
        let _ = frame(&ctx, size, &mut app, vec![]);
        assert!(direction::is_rtl(&ctx));
    }

    #[test]
    fn button_and_form_interactions_update_application_state() {
        let ctx = setup(ThemePreset::FaseelaDark, false);
        let mut app = Showcase {
            selected_section: 2,
            ..Default::default()
        };
        let size = vec2(1280.0, 800.0);
        let _ = frame(&ctx, size, &mut app, vec![]);
        let rect = response_rect(&ctx, Id::unique("showcase-action-primary"));
        let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
        assert!(app.last_action.contains("Primary"));

        app.selected_section = 3;
        let _ = frame(&ctx, size, &mut app, vec![]);
        let check_id = Id::unique("showcase-notify");
        let check_rect = response_rect(&ctx, check_id);
        let _ = frame(&ctx, size, &mut app, click_at(check_rect.center()));
        assert!(app.notifications);

        let _ = frame(&ctx, size, &mut app, vec![]);
        let field_id = Id::unique("showcase-name");
        let _ = frame(&ctx, size, &mut app, vec![]);
        let _ = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                focused: true,
                ..Default::default()
            },
            |ui| {
                app.show(ui);
                ui.memory_mut(|memory| memory.request_focus(field_id));
            },
        );
        let _ = frame(
            &ctx,
            size,
            &mut app,
            vec![Event::Text("Faseela".to_owned())],
        );
        assert_eq!(app.name, "Faseela");
    }

    #[test]
    fn progress_controls_update_the_bar_value() {
        let ctx = setup(ThemePreset::Light, false);
        let mut app = Showcase {
            selected_section: 6,
            ..Default::default()
        };
        let size = vec2(390.0, 844.0);
        let _ = frame(&ctx, size, &mut app, vec![]);
        let before = app.progress;
        let rect = response_rect(&ctx, Id::unique("showcase-progress-advance"));
        let _ = frame(&ctx, size, &mut app, click_at(rect.center()));
        assert!(app.progress > before);
        assert!(app.progress <= 1.0);
    }

    #[test]
    fn contrast_checks_pass_for_every_preset() {
        for preset in ThemePreset::ALL {
            let checks = contrast_checks(preset.tokens());
            for check in checks {
                assert!(
                    check.passes(),
                    "{preset:?} {} was {:.2}:1 below {}:1",
                    check.label,
                    check.ratio(),
                    check.minimum
                );
            }
        }
    }

    #[test]
    fn colors_are_reported_as_uppercase_hex() {
        assert_eq!(hex(Color32::from_rgb(0x0A, 0x1D, 0x1A)), "#0A1D1A");
    }

    #[test]
    fn keyboard_review_checkbox_is_handled_without_errors() {
        let ctx = setup(ThemePreset::FaseelaDark, false);
        let mut app = Showcase {
            selected_section: 8,
            ..Default::default()
        };
        let size = vec2(390.0, 844.0);
        let _ = frame(&ctx, size, &mut app, vec![]);
        let key = Event::Key {
            key: Key::Tab,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        };
        let _ = frame(&ctx, size, &mut app, vec![key]);
        assert!(ctx.globally_used_rect().width() <= 391.0);
    }
}
