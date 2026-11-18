//! Shared UI foundations for native and WebAssembly Faseela applications.
//!
//! Application-specific state and business logic should remain outside this crate.
//! Widgets expose focused APIs and use egui primitives so applications can compose them.

#![forbid(unsafe_code)]

pub mod buttons;
pub mod direction;
pub mod forms;
pub mod icons;
pub mod layout;
pub mod navigation;
pub mod tabs;
pub mod text;
pub mod theme;
pub mod widgets;

pub use egui;
