//! Tab-strip width fitting without owning a tab model, painter, or overflow menu.
//!
//! The caller owns tab identity, selection, rendering, and overflow behavior.
//! Adapted from https://github.com/storytold/craft-libs (MIT OR Apache-2.0).

/// Tabs visible on a strip and tabs that need an overflow menu, in original order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TabFit {
    /// `(input index, width)` for visible tabs.
    pub shown: Vec<(usize, f32)>,
    pub overflow: Vec<usize>,
    /// Reserved width for the overflow button, zero when no tabs overflow.
    pub overflow_width: f32,
}

/// Largest meaningful extent for one UI component, in logical points.
const MAX_EXTENT: f32 = 1_000_000.0;

fn extent(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, MAX_EXTENT)
    } else {
        0.0
    }
}

/// Fit tab widths into `available`, shrinking the widest tabs first.
///
/// Tabs normally retain at least `min(natural_width, minimum_width)`. If that cannot fit,
/// the selected tab is retained first, followed by as many other tabs as fit beside the
/// overflow button. The selected tab is retained even when its width must become zero.
///
/// Negative and non-finite dimensions become zero; finite dimensions are capped at one million
/// points. The widths of all shown tabs plus `overflow_width` never exceed sanitized available
/// width. Work and storage are linear in the input count.
pub fn fit(
    natural: &[f32],
    selected: usize,
    available: f32,
    minimum_width: f32,
    overflow_width: f32,
) -> TabFit {
    let available = extent(available);
    let minimum_width = extent(minimum_width);
    let natural: Vec<f32> = natural.iter().map(|&width| extent(width)).collect();
    let all: Vec<usize> = (0..natural.len()).collect();

    if natural.iter().map(|&width| f64::from(width)).sum::<f64>() <= f64::from(available) {
        return TabFit {
            shown: all.into_iter().zip(natural).collect(),
            ..Default::default()
        };
    }
    if natural.len() == 1 {
        return TabFit {
            shown: vec![(0, available)],
            ..Default::default()
        };
    }

    let floor = |width: f32| width.min(minimum_width);
    if natural
        .iter()
        .map(|&width| f64::from(floor(width)))
        .sum::<f64>()
        <= f64::from(available)
    {
        return TabFit {
            shown: squeeze(&natural, &all, available, minimum_width),
            ..Default::default()
        };
    }

    let overflow_width = extent(overflow_width).min(available);
    let budget = available - overflow_width;
    let selected = selected.min(natural.len().saturating_sub(1));
    let mut keep = vec![false; natural.len()];
    let mut used = 0.0_f64;

    for index in std::iter::once(selected).chain(all.iter().copied().filter(|&i| i != selected)) {
        let Some(&width) = natural.get(index) else {
            continue;
        };
        let width = f64::from(floor(width));
        if index == selected || used + width <= f64::from(budget) {
            if let Some(keep) = keep.get_mut(index) {
                *keep = true;
            }
            used += width;
        }
    }

    let (shown, overflow): (Vec<usize>, Vec<usize>) = all
        .into_iter()
        .partition(|&index| keep.get(index).copied().unwrap_or(false));
    let shown = squeeze(&natural, &shown, budget, minimum_width);
    TabFit {
        shown,
        overflow,
        overflow_width,
    }
}

/// Fit tabs at natural widths; tabs that do not fit go to an overflow menu.
///
/// Tabs are shown in input order up to the first one that does not fit beside the selected tab
/// and the overflow button. The selected tab is always shown in its logical place. A selected
/// tab wider than the strip minus the button is clipped to the remaining width by its caller.
pub fn fit_whole(natural: &[f32], selected: usize, available: f32, overflow_width: f32) -> TabFit {
    let available = extent(available);
    let natural: Vec<f32> = natural.iter().map(|&width| extent(width)).collect();

    if natural.iter().map(|&width| f64::from(width)).sum::<f64>() <= f64::from(available) {
        return TabFit {
            shown: natural.into_iter().enumerate().collect(),
            ..Default::default()
        };
    }
    if let [width] = natural[..] {
        return TabFit {
            shown: vec![(0, width.min(available))],
            ..Default::default()
        };
    }

    let overflow_width = extent(overflow_width).min(available);
    let budget = available - overflow_width;
    let selected = selected.min(natural.len().saturating_sub(1));
    let active = natural.get(selected).copied().unwrap_or(0.0).min(budget);
    let mut room = f64::from(budget - active);
    let mut shown = Vec::new();
    let mut overflow = Vec::new();
    let mut full = false;

    for (index, &width) in natural.iter().enumerate() {
        if index == selected {
            shown.push((index, active));
        } else if !full && f64::from(width) <= room {
            room -= f64::from(width);
            shown.push((index, width));
        } else {
            full = true;
            overflow.push(index);
        }
    }

    TabFit {
        shown,
        overflow,
        overflow_width,
    }
}

fn squeeze(natural: &[f32], indices: &[usize], total: f32, minimum: f32) -> Vec<(usize, f32)> {
    let width = |index: usize| natural.get(index).copied().unwrap_or(0.0);
    let sum_at = |cap: f32| {
        indices
            .iter()
            .map(|&index| f64::from(width(index).min(cap.max(width(index).min(minimum)))))
            .sum::<f64>()
    };
    let (mut low, mut high) = (
        minimum,
        indices
            .iter()
            .map(|&index| width(index))
            .fold(minimum, f32::max),
    );

    if sum_at(high) > f64::from(total) {
        for _ in 0..32 {
            let middle = low + (high - low) * 0.5;
            if sum_at(middle) <= f64::from(total) {
                low = middle;
            } else {
                high = middle;
            }
        }
    } else {
        low = high;
    }

    let mut remaining = total;
    indices
        .iter()
        .map(|&index| {
            let natural = width(index);
            let width = natural.min(low.max(natural.min(minimum))).min(remaining);
            remaining = (remaining - width).max(0.0);
            (index, width)
        })
        .collect()
}


/// An accessible tab interaction primitive with caller-owned painting and selection state.
///
/// IDs must be unique within the parent UI scope and must not depend on translated labels.
/// This widget does not allocate space: pass the rectangle assigned by the tab-strip layout.
/// Use `Sense::drag()` when the parent implements reorder or tear-off behavior.
pub struct Tab<'a> {
    id: egui::Id,
    label: &'a str,
    selected: bool,
    sense: egui::Sense,
    focus_stroke: Option<egui::Stroke>,
}

impl<'a> Tab<'a> {
    pub fn new(id: egui::Id, label: &'a str, selected: bool) -> Self {
        Self {
            id,
            label,
            selected,
            sense: egui::Sense::click(),
            focus_stroke: None,
        }
    }

    /// Override the default click interaction, for example with `Sense::drag()`.
    pub fn sense(mut self, sense: egui::Sense) -> Self {
        self.sense = sense;
        self
    }

    /// Set the inset keyboard-focus outline. By default egui's selection stroke is used.
    /// Passing `Stroke::NONE` disables the outline.
    pub fn focus_stroke(mut self, stroke: egui::Stroke) -> Self {
        self.focus_stroke = Some(stroke);
        self
    }

    /// Register the tab in the UI and invoke the caller's painter.
    ///
    /// Invalid, empty, inverted, non-finite, or excessively large rectangles produce an inert
    /// response and do not invoke the painter.
    pub fn show_at(
        self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        paint: impl FnOnce(&egui::Ui, &egui::Response),
    ) -> egui::Response {
        let size = rect.size();
        let valid = rect.is_finite()
            && size.x > 0.0
            && size.y > 0.0
            && size.x <= 1_000_000.0
            && size.y <= 1_000_000.0;
        let safe_rect = if valid {
            rect
        } else {
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::ZERO)
        };
        let response = ui.interact(
            safe_rect,
            self.id,
            if valid { self.sense } else { egui::Sense::hover() },
        );
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::Role::Button,
                valid && ui.is_enabled(),
                self.selected,
                self.label,
            )
        });

        if !valid {
            return response;
        }

        paint(ui, &response);

        if response.has_focus() && rect.width() > 4.0 && rect.height() > 4.0 {
            let focus = rect.shrink(2.0);
            let stroke = self.focus_stroke.unwrap_or(ui.visuals().selection.stroke);
            if stroke.width.is_finite() && stroke.width > 0.0 {
                let stroke = egui::Stroke::new(
                    stroke.width.min(focus.width().min(focus.height()) * 0.5),
                    stroke.color,
                );
                ui.painter().rect_stroke(
                    focus,
                    0.0,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn total(fit: &TabFit) -> f32 {
        fit.shown.iter().map(|(_, width)| width).sum::<f32>() + fit.overflow_width
    }

    #[test]
    fn natural_widths_are_kept_when_they_fit() {
        let result = fit(&[80.0, 100.0, 60.0], 1, 300.0, 40.0, 24.0);
        assert_eq!(result.shown, vec![(0, 80.0), (1, 100.0), (2, 60.0)]);
        assert!(result.overflow.is_empty());
        assert_eq!(result.overflow_width, 0.0);
    }

    #[test]
    fn tabs_shrink_before_overflow_is_needed() {
        let result = fit(&[120.0, 100.0, 90.0], 1, 240.0, 48.0, 24.0);
        assert!(result.overflow.is_empty());
        assert!(result.shown.iter().all(|(_, width)| *width >= 48.0));
        assert!(total(&result) <= 240.0);
    }

    #[test]
    fn selected_tab_is_retained_when_other_tabs_overflow() {
        let result = fit(&[100.0, 100.0, 100.0], 2, 130.0, 80.0, 24.0);
        assert!(result.shown.iter().any(|(index, _)| *index == 2));
        assert!(!result.overflow.contains(&2));
        assert!(total(&result) <= 130.0);
    }

    #[test]
    fn whole_tabs_keep_order_and_selected_tab() {
        let result = fit_whole(&[60.0, 70.0, 80.0, 90.0], 2, 190.0, 24.0);
        assert!(result.shown.iter().any(|(index, _)| *index == 2));
        assert_eq!(result.overflow, vec![1, 3]);
        assert!(total(&result) <= 190.0);
    }

    #[test]
    fn invalid_dimensions_are_sanitized() {
        let result = fit(
            &[f32::NAN, -5.0, f32::INFINITY],
            usize::MAX,
            80.0,
            30.0,
            24.0,
        );
        assert!(total(&result) <= 80.0);
        assert!(
            result
                .shown
                .iter()
                .all(|(_, width)| width.is_finite() && *width >= 0.0)
        );
    }
}
