//! Application-owned state and frame orchestration.
mod demos;
mod helpers;
mod navigation;
mod shell;

use eframe::egui::{self, Ui};
use faseela_ui::{
    direction::{self, Direction},
    layout::SplitSize,
    theme::{self, ThemePreset},
};
use navigation::SECTIONS;
use shell::MOBILE_BREAKPOINT;

pub(crate) struct Showcase {
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
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
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
}

#[cfg(test)]
mod tests;
