//! Stateless layout algorithms for reusable Faseela application chrome.
//!
//! Applications own their layout trees, persisted sizing, commands and pane contents. These
//! helpers return bounded geometry or a fit proposal; they do not mutate application state.
//!
//! The split and toolbar algorithms are adapted from the MIT OR Apache-2.0 licensed
//! `storytold/craft-libs` project. See the attribution notes in the repository's `UPSTREAM.md`.

mod geometry;
mod toolbar;

pub use geometry::{SplitAxis, SplitLayout, SplitSize, split_rect};
pub use toolbar::{ToolbarFit, fit_toolbar};
