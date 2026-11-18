//! Navigation components with caller-owned selection and application-owned actions.
//!
//! [`ButtonGroup`] and [`SegmentedControl`] present an exclusive choice among labeled options.
//! [`Breadcrumb`] presents a navigation path whose earlier entries remain interactive. None of
//! these widgets change application routes or commands: they update the values passed in and
//! return the activated index as a proposal.

mod breadcrumb;
mod group;
mod segmented;

pub use breadcrumb::{Breadcrumb, BreadcrumbResponse};
pub use group::ButtonGroup;
pub use segmented::SegmentedControl;

use egui::Response;

/// The result of an exclusive navigation control.
pub struct SelectionResponse {
    /// Container response, including focus and enabled state.
    pub response: Response,
    /// The index activated by click or keyboard this frame.
    pub activated: Option<usize>,
}

#[cfg(test)]
mod tests;
