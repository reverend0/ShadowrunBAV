//! ShadowrunBAV's rules: designing vehicles and drones from scratch by
//! Shadowrun Sixth World's Double Clutch ("Build Your Dream", pp. 147–157).
//!
//! - `catalog`: the book's tables (chassis, powertrains, consoles, design
//!   options, quality factors), from `data/design.toml`.
//! - `design`: a design, the player's choices.
//! - `calc`: works a design out into stats, Build Points, cost, and tests.
//! - `files`: reading and writing design files.
//! - `mods`: vehicle mods ("Making It Yours"), on top of a design.
//! - `pdf`: a design as a PDF vehicle sheet, laid out like the book's entries.

pub mod calc;
pub mod catalog;
pub mod design;
pub mod files;
pub mod mods;
pub mod pdf;

#[cfg(test)]
mod mod_tests;
#[cfg(test)]
mod tests;

pub use calc::{Report, evaluate};
pub use catalog::Catalog;
pub use design::{Design, Improvements};
