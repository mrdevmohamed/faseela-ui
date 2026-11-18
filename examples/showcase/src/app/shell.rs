//! Responsive desktop/mobile shells and bounded scrolling content.
use super::{Showcase, helpers::padded};
use eframe::egui::{self, Ui, Vec2, vec2};
use faseela_ui::direction;

pub(super) const MOBILE_BREAKPOINT: f32 = 680.0;
const SIDEBAR_WIDTH: f32 = 214.0;
const CONTENT_MAX_WIDTH: f32 = 1120.0;

impl Showcase {
    pub(super) fn mobile_shell(&mut self, ui: &mut Ui, available: Vec2) {
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

    pub(super) fn desktop_shell(&mut self, ui: &mut Ui, available: Vec2) {
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

    pub(super) fn desktop_content(&mut self, ui: &mut Ui, height: f32) {
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

    pub(super) fn scroll_content(&mut self, ui: &mut Ui, salt: &'static str) {
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
}
