//! Reusable, theme-aware presentation widgets.
//!
//! Every widget in this module reads its colors from [`crate::theme::active_tokens`], so a theme
//! switch takes effect on the next frame without callers rebuilding their styles. Widgets return
//! proposals (for example which toolbar item was clicked); applications keep ownership of state,
//! commands, and selection.

mod badge;
mod feedback;
mod surface;
mod toolbar;

pub use badge::{Badge, BadgeTone};
pub use feedback::{
    EmptyState, EmptyStateResponse, LoadingIndicator, ProgressBar, StatusIndicator,
};
pub use surface::{Card, SectionHeader, Separator, SeparatorOrientation, Tooltip};
pub use toolbar::{ToolbarItem, ToolbarResponse, ToolbarState, toolbar, toolbar_response};

#[cfg(test)]
mod tests;
