use super::*;
use eframe::egui::{Align2, CornerRadius, FontId, Id, RichText, Sense, Ui, vec2};
use faseela_ui::{
    layout::{ResizeHandle, ResizeHandleStyle, SplitAxis, SplitSize, split_rect},
    navigation::ButtonGroup,
    theme,
    widgets::{SectionHeader, ToolbarItem, ToolbarState, toolbar_response},
};

const TOOLBAR_COMMANDS: [&str; 9] = [
    "New", "Open", "Save", "Run", "Build", "Test", "Debug", "Deploy", "Publish",
];
const DISABLED_TOOLBAR_COMMAND: usize = 7;

impl Showcase {
    pub(super) fn layout(&mut self, ui: &mut Ui) {
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

    pub(super) fn split_preview(&mut self, ui: &mut Ui) {
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
}
