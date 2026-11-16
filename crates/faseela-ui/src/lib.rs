//! Shared UI foundations for Faseela desktop applications.
//!
//! Application-specific state and business logic should remain outside this crate.
//! Widgets expose focused APIs and use egui primitives so applications can compose them.

pub mod buttons;
pub mod direction;
pub mod icons;
pub mod layout;
pub mod tabs;
pub mod text;
pub mod theme;

pub use egui;
