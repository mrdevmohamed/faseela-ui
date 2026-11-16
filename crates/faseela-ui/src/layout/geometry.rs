//! Bounded geometry for a two-pane split.
//!
//! Adapted from https://github.com/storytold/craft-libs (MIT OR Apache-2.0).

use egui::Rect;

/// The direction in which the children of a split are placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitAxis {
    /// First child on the left, second child on the right.
    ///
    /// For a right-to-left interface, applications can keep this geometry and map their logical
    /// leading/trailing panes to the returned rectangles.
    Horizontal,
    /// First child above the second child.
    Vertical,
}

/// A transient sizing request. Applications may adapt their own persisted sizing enum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitSize {
    /// Fraction of the space left after the divider is removed, assigned to the first child.
    Ratio(f32),
    /// First child size in logical points.
    FixedFirst(f32),
    /// Second child size in logical points.
    FixedSecond(f32),
}

impl SplitSize {
    fn value(self) -> f32 {
        match self {
            Self::Ratio(value) | Self::FixedFirst(value) | Self::FixedSecond(value) => value,
        }
    }
}

/// Contained, non-overlapping rectangles of a single split, including its divider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitLayout {
    /// Rectangle allocated to the first child.
    pub first: Rect,
    /// Rectangle allocated to the divider.
    pub divider: Rect,
    /// Rectangle allocated to the second child.
    pub second: Rect,
    first_size: f32,
    available: f32,
}

impl SplitLayout {
    /// The first child's size along the split axis, in logical points.
    pub fn first_size(self) -> f32 {
        self.first_size
    }

    /// Space for both children, excluding the divider, in logical points.
    pub fn available(self) -> f32 {
        self.available
    }

    /// Propose a new size after moving the divider by `delta` points.
    ///
    /// The sizing mode is retained. `minimum` may differ from the minimum used for layout, e.g.
    /// a pane can have a 20-point layout minimum but a 40-point drag minimum. When both minima
    /// cannot fit, each child receives half the available space. Invalid inputs and zero available
    /// space return `None`, so the caller can leave its persisted value unchanged.
    pub fn resized(self, size: SplitSize, delta: f32, minimum: f32) -> Option<SplitSize> {
        if !size.value().is_finite()
            || !delta.is_finite()
            || !valid_nonnegative(minimum)
            || self.available <= 0.0
        {
            return None;
        }

        let available = f64::from(self.available);
        let first = clamp_first(
            f64::from(self.first_size) + f64::from(delta),
            available,
            minimum,
        );

        Some(match size {
            SplitSize::Ratio(_) => SplitSize::Ratio((first / available) as f32),
            SplitSize::FixedFirst(_) => SplitSize::FixedFirst(first as f32),
            SplitSize::FixedSecond(_) => SplitSize::FixedSecond((available - first) as f32),
        })
    }
}

/// Divide `rect` into two panes and a divider.
///
/// `gap` and `minimum` are logical points. The divider is capped to the available span; equal
/// pane minima relax to half the remaining span in tiny areas. Zero-sized rectangles are valid.
/// Finite sizing requests outside the available range are clamped. Inverted rectangles,
/// non-finite coordinates or extents, negative/non-finite gap or minimum, and non-finite sizing
/// requests return `None`. No state is stored or changed.
///
/// For horizontal splits, the first pane is on the left and the second is on the right.
pub fn split_rect(
    rect: Rect,
    axis: SplitAxis,
    size: SplitSize,
    gap: f32,
    minimum: f32,
) -> Option<SplitLayout> {
    if !valid_rect(rect)
        || !valid_nonnegative(gap)
        || !valid_nonnegative(minimum)
        || !size.value().is_finite()
    {
        return None;
    }

    let total = match axis {
        SplitAxis::Horizontal => rect.width(),
        SplitAxis::Vertical => rect.height(),
    };
    let gap = gap.min(total);
    let available = total - gap;
    let space = f64::from(available);

    // Do intermediate arithmetic in f64: two finite f32 inputs can overflow when multiplied.
    let wanted = match size {
        SplitSize::Ratio(ratio) => space * f64::from(ratio),
        SplitSize::FixedFirst(points) => f64::from(points),
        SplitSize::FixedSecond(points) => space - f64::from(points),
    };
    let first_size = clamp_first(wanted, space, minimum) as f32;

    let mut first = rect;
    let mut divider = rect;
    let mut second = rect;
    match axis {
        SplitAxis::Horizontal => {
            first.max.x = (rect.min.x + first_size).min(rect.max.x);
            divider.min.x = first.max.x;
            divider.max.x = (divider.min.x + gap).min(rect.max.x);
            second.min.x = divider.max.x;
        }
        SplitAxis::Vertical => {
            first.max.y = (rect.min.y + first_size).min(rect.max.y);
            divider.min.y = first.max.y;
            divider.max.y = (divider.min.y + gap).min(rect.max.y);
            second.min.y = divider.max.y;
        }
    }

    Some(SplitLayout {
        first,
        divider,
        second,
        first_size,
        available,
    })
}

fn valid_rect(rect: Rect) -> bool {
    rect.min.is_finite()
        && rect.max.is_finite()
        && rect.width().is_finite()
        && rect.height().is_finite()
        && rect.min.x <= rect.max.x
        && rect.min.y <= rect.max.y
}

fn valid_nonnegative(value: f32) -> bool {
    value.is_finite() && value >= 0.0
}

fn clamp_first(wanted: f64, available: f64, minimum: f32) -> f64 {
    let low = f64::from(minimum).min(available * 0.5);
    wanted.clamp(low, available - low)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Pos2, Rect, Vec2, pos2, vec2};

    fn sample() -> Rect {
        Rect::from_min_max(pos2(10.0, 20.0), pos2(410.0, 220.0))
    }

    #[test]
    fn ordinary_splits_keep_relative_and_fixed_point_sizes() {
        for (request, first, second) in [
            (SplitSize::Ratio(0.25), 99.0, 297.0),
            (SplitSize::FixedFirst(150.0), 150.0, 246.0),
            (SplitSize::FixedSecond(150.0), 246.0, 150.0),
        ] {
            let layout = split_rect(sample(), SplitAxis::Horizontal, request, 4.0, 20.0)
                .expect("valid split");
            assert_eq!(layout.first.width(), first);
            assert_eq!(layout.second.width(), second);
            assert_eq!(layout.divider.width(), 4.0);
            assert_eq!(layout.first.max.x, layout.divider.min.x);
            assert_eq!(layout.divider.max.x, layout.second.min.x);
            assert_eq!(layout.first.min, sample().min);
            assert_eq!(layout.second.max, sample().max);
        }

        let layout = split_rect(
            sample(),
            SplitAxis::Vertical,
            SplitSize::Ratio(0.5),
            2.0,
            0.0,
        )
        .expect("valid vertical split");
        assert_eq!(layout.first.height(), 99.0);
        assert_eq!(layout.second.height(), 99.0);
        assert_eq!(layout.divider.height(), 2.0);
    }

    #[test]
    fn tiny_panes_never_invert_or_escape_the_parent() {
        for span in [0.0, 1.0, 15.0, 20.0, 35.0, 39.9, 40.0, 80.0, 1000.0] {
            for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
                let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(span));
                for gap in [0.0, 4.0, 2000.0] {
                    for size in [
                        SplitSize::Ratio(-10.0),
                        SplitSize::Ratio(0.5),
                        SplitSize::Ratio(10.0),
                        SplitSize::FixedFirst(300.0),
                        SplitSize::FixedSecond(300.0),
                    ] {
                        let layout = split_rect(rect, axis, size, gap, 20.0)
                            .expect("finite requests are clamped");
                        for child in [layout.first, layout.divider, layout.second] {
                            assert!(rect.contains_rect(child), "{rect:?}: {child:?}");
                            assert!(child.width() >= 0.0 && child.height() >= 0.0);
                            assert!(child.min.is_finite() && child.max.is_finite());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            for size in [
                SplitSize::Ratio(invalid),
                SplitSize::FixedFirst(invalid),
                SplitSize::FixedSecond(invalid),
            ] {
                assert!(split_rect(sample(), SplitAxis::Horizontal, size, 4.0, 20.0).is_none());
            }
            assert!(
                split_rect(
                    sample(),
                    SplitAxis::Horizontal,
                    SplitSize::Ratio(0.5),
                    invalid,
                    20.0,
                )
                .is_none()
            );
            assert!(
                split_rect(
                    sample(),
                    SplitAxis::Horizontal,
                    SplitSize::Ratio(0.5),
                    4.0,
                    invalid,
                )
                .is_none()
            );

            let mut rect = sample();
            rect.min.x = invalid;
            assert!(
                split_rect(
                    rect,
                    SplitAxis::Horizontal,
                    SplitSize::Ratio(0.5),
                    4.0,
                    20.0,
                )
                .is_none()
            );
        }

        assert!(
            split_rect(
                sample(),
                SplitAxis::Horizontal,
                SplitSize::Ratio(0.5),
                -1.0,
                20.0,
            )
            .is_none()
        );
        assert!(
            split_rect(
                sample(),
                SplitAxis::Horizontal,
                SplitSize::Ratio(0.5),
                4.0,
                -1.0,
            )
            .is_none()
        );
        assert!(
            split_rect(
                Rect { min: sample().max, max: sample().min },
                SplitAxis::Horizontal,
                SplitSize::Ratio(0.5),
                4.0,
                20.0,
            )
            .is_none()
        );

        let huge = Rect::from_min_max(pos2(-f32::MAX, 0.0), pos2(f32::MAX, 100.0));
        assert!(
            split_rect(
                huge,
                SplitAxis::Horizontal,
                SplitSize::Ratio(0.5),
                4.0,
                20.0,
            )
            .is_none()
        );
    }

    #[test]
    fn extreme_finite_requests_stay_inside_large_rectangles() {
        let huge = Rect::from_min_max(pos2(0.0, 0.0), pos2(f32::MAX, 100.0));
        for request in [
            SplitSize::Ratio(f32::MAX),
            SplitSize::FixedFirst(f32::MAX),
            SplitSize::FixedSecond(-f32::MAX),
        ] {
            let layout = split_rect(huge, SplitAxis::Horizontal, request, 4.0, 20.0)
                .expect("finite dimensions are accepted");
            for child in [layout.first, layout.divider, layout.second] {
                assert!(huge.contains_rect(child));
                assert!(child.min.is_finite() && child.max.is_finite());
            }
        }
    }

    #[test]
    fn resizing_preserves_sizing_mode_and_minimums() {
        let layout = split_rect(
            sample(),
            SplitAxis::Horizontal,
            SplitSize::FixedFirst(100.0),
            4.0,
            20.0,
        )
        .expect("valid split");

        assert_eq!(
            layout.resized(SplitSize::Ratio(0.25), 20.0, 40.0),
            Some(SplitSize::Ratio(120.0 / 396.0))
        );
        assert_eq!(
            layout.resized(SplitSize::FixedFirst(100.0), 20.0, 40.0),
            Some(SplitSize::FixedFirst(120.0))
        );
        assert_eq!(
            layout.resized(SplitSize::FixedSecond(296.0), 20.0, 40.0),
            Some(SplitSize::FixedSecond(276.0))
        );
        assert_eq!(
            layout.resized(SplitSize::FixedFirst(100.0), -1000.0, 40.0),
            Some(SplitSize::FixedFirst(40.0))
        );
        assert_eq!(
            layout.resized(SplitSize::FixedFirst(100.0), 1000.0, 40.0),
            Some(SplitSize::FixedFirst(356.0))
        );

        let tiny = split_rect(
            Rect::from_min_size(Pos2::ZERO, vec2(19.0, 100.0)),
            SplitAxis::Horizontal,
            SplitSize::Ratio(0.5),
            4.0,
            20.0,
        )
        .expect("valid tiny split");
        assert_eq!(
            tiny.resized(SplitSize::FixedFirst(7.5), 100.0, 40.0),
            Some(SplitSize::FixedFirst(7.5))
        );
        assert_eq!(
            tiny.resized(SplitSize::FixedSecond(7.5), -100.0, 40.0),
            Some(SplitSize::FixedSecond(7.5))
        );
        assert!(tiny.resized(SplitSize::Ratio(0.5), f32::NAN, 40.0).is_none());

        let zero = split_rect(
            Rect::ZERO,
            SplitAxis::Horizontal,
            SplitSize::Ratio(0.5),
            4.0,
            20.0,
        )
        .expect("zero-sized rectangles are valid");
        assert!(zero.resized(SplitSize::Ratio(0.5), 10.0, 40.0).is_none());
    }
}
