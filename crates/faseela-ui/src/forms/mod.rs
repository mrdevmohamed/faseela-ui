//! Theme-aware form controls with application-owned values.
//!
//! Every control stores no hidden application state: callers pass `&mut` values for text fields,
//! checkboxes, switches, radio groups, and selects. The widgets provide labels, helper/error
//! text, disabled and focus presentation, AccessKit roles, and keyboard activation through
//! egui's native interaction model.
//!
//! ```
//! use faseela_ui::forms::{Checkbox, Switch, TextField};
//!
//! egui::__run_test_ui(|ui| {
//!     let mut name = "Faseela".to_owned();
//!     let mut enabled = true;
//!     TextField::new(&mut name).label("Name").show(ui);
//!     Checkbox::new("Show notifications", &mut enabled).show(ui);
//!     Switch::new("Live updates", &mut enabled).show(ui);
//! });
//! ```

mod common;
mod options;
mod selection;
mod text;
mod toggles;

pub use selection::{RadioButton, RadioGroup, Select};
pub use text::{SearchField, TextField};
pub use toggles::{Checkbox, Switch};

#[cfg(test)]
mod tests;
