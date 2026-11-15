//! The design tables from Double Clutch, read from `data/design.toml`.
//!
//! The file is built into the program with `include_str!`, so the app and
//! the tests always use the same tables and there's no file to lose.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// Every table the design rules use.
///
/// `#[derive(Deserialize)]` lets `toml` build this straight from the file:
/// each `[[chassis]]` table becomes one `Chassis` in the `chassis` list, and
/// so on. `Serialize` lets the shell send it to the UI as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub environments: Vec<String>,
    pub size_class: Vec<SizeClass>,
    pub chassis: Vec<Chassis>,
    pub powertrain: Vec<Powertrain>,
    pub console: Vec<Console>,
    pub improvement: Vec<Improvement>,
    pub modification: Vec<Modification>,
    pub quality: Vec<Quality>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeClass {
    pub class: i32,
    pub length: String,
    pub weight: String,
    /// Cargo space in Cargo Factors (1 CF = half a cubic meter, 200 kg).
    pub cargo_cf: f64,
}

/// What a chassis is, which decides a few rules (aircraft Availability,
/// a drone's Acceleration multiplier and size changes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Ground,
    Water,
    Air,
    Drone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DroneSize {
    Micro,
    Mini,
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chassis {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    /// Only drones have one. `Option` is Rust's "maybe": `None` when the
    /// file leaves it out, which `#[serde(default)]` allows.
    #[serde(default)]
    pub drone_size: Option<DroneSize>,
    pub body: i32,
    pub armor: i32,
    pub handling: i32,
    pub seat: i32,
    pub build_points: i32,
    pub size_class: i32,
    /// Cargo on top of the size class's (a truck's +4 CF).
    #[serde(default)]
    pub extra_cargo_cf: f64,
    /// Built for underwater use (a submersible).
    #[serde(default)]
    pub underwater: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Powertrain {
    pub id: String,
    pub name: String,
    /// Ground, water, or air (never drone).
    pub medium: Kind,
    pub acceleration: i32,
    pub speed_interval: i32,
    pub top_speed: i32,
    pub build_points: i32,
    /// Added to Handling on and off road (hover: −2, which is better).
    #[serde(default)]
    pub handling: i32,
    /// Added to off-road Handling only (tracked: −2).
    #[serde(default)]
    pub off_road_handling: i32,
    /// A walker's Top Speed isn't lowered by Body and Armor.
    #[serde(default)]
    pub no_top_speed_loss: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Console {
    pub id: String,
    pub name: String,
    pub pilot: i32,
    pub sensor: i32,
    pub build_points: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Improvement {
    pub id: String,
    pub name: String,
    pub per_level: i32,
    /// The book's fixed limit, or 0 when the limit depends on the vehicle.
    pub max_level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modification {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub group: Option<String>,
    pub build_points: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quality {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub group: Option<String>,
    pub multiplier: f64,
    #[serde(default)]
    pub availability: i32,
    #[serde(default)]
    pub legality: Option<String>,
}

/// The tables' source, built into the program.
const DESIGN_TOML: &str = include_str!("../../../data/design.toml");

impl Catalog {
    /// The built-in tables, parsed once and shared.
    ///
    /// `OnceLock` runs the closure the first time it's asked and keeps the
    /// result for the life of the program; later calls get the same
    /// `&'static Catalog` (a reference that's valid until the program ends).
    /// The `expect` can only fail if `data/design.toml` is broken, which the
    /// tests catch before anyone runs the app.
    pub fn builtin() -> &'static Catalog {
        static CATALOG: OnceLock<Catalog> = OnceLock::new();
        CATALOG.get_or_init(|| toml::from_str(DESIGN_TOML).expect("data/design.toml is valid"))
    }

    // Lookups by id. Each returns `Option<&T>`: a reference into the catalog
    // (no copy), or `None` if there's no such id. `iter().find(...)` walks
    // the list and stops at the first match.

    pub fn chassis(&self, id: &str) -> Option<&Chassis> {
        self.chassis.iter().find(|c| c.id == id)
    }

    pub fn powertrain(&self, id: &str) -> Option<&Powertrain> {
        self.powertrain.iter().find(|p| p.id == id)
    }

    pub fn console(&self, id: &str) -> Option<&Console> {
        self.console.iter().find(|c| c.id == id)
    }

    pub fn improvement(&self, id: &str) -> Option<&Improvement> {
        self.improvement.iter().find(|i| i.id == id)
    }

    pub fn modification(&self, id: &str) -> Option<&Modification> {
        self.modification.iter().find(|m| m.id == id)
    }

    pub fn quality(&self, id: &str) -> Option<&Quality> {
        self.quality.iter().find(|q| q.id == id)
    }

    /// A size class's row, with classes past the table clamped to its ends.
    pub fn size_class(&self, class: i32) -> &SizeClass {
        let last = self.size_class.len() as i32 - 1;
        &self.size_class[class.clamp(0, last) as usize]
    }
}
