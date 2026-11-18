//! Reusable button primitives: text buttons, icon buttons, variants, and sizes.
//!
//! The original icon-only primitive is adapted from Craft's reusable egui controls.
//!
//! Upstream: <https://github.com/storytold/craft-libs>
//! The application owns icon assets, commands and selection state.

mod icon;
mod style;
mod text;

pub use icon::IconButton;
pub use style::{ButtonVariant, ButtonVisuals, ControlSize, IconButtonStyle};
pub use text::Button;

#[cfg(test)]
mod tests;
