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
    /// The vehicle mods, from `data/mods.toml`.
    #[serde(default)]
    pub mods: Vec<Mod>,
    /// Sourcebooks beyond Double Clutch that a design can turn on.
    #[serde(default)]
    pub book: Vec<Book>,
}

/// A sourcebook with vehicle rules of its own (Deadly Arts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub name: String,
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

/// Drone sizes, smallest first, so they compare (`Small < Medium`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    /// How much skin it has (p. 131): standard unless the file says open (a
    /// bike) or enclosed (a drone).
    #[serde(default)]
    pub form: Form,
}

/// A skin form (p. 131), in order from least to most skin, so they compare:
/// `Form::Open < Form::Enclosed`. `#[derive(PartialOrd, Ord)]` orders an
/// enum's variants as they're written.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Form {
    Open,
    #[default]
    Standard,
    Enclosed,
}

impl Form {
    pub fn level(self) -> i32 {
        match self {
            Form::Open => 1,
            Form::Standard => 2,
            Form::Enclosed => 3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Form::Open => "open construction",
            Form::Standard => "standard construction",
            Form::Enclosed => "totally enclosed construction",
        }
    }
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

/// A vehicle mod ("Making It Yours", pp. 120–144). `data/mods.toml`
/// explains each field; what an `effect` does is in `mods.rs`.
///
/// `#[serde(default)]` on the struct fills every field the file leaves out
/// from `Default` (zero, empty, `None`), so each entry lists only what it has.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub group: String,
    pub slot_type: SlotType,
    pub rating_min: Option<i32>,
    pub rating_max: Option<i32>,
    pub slots: f64,
    pub slots_per_rating: f64,
    pub slots_per_body: f64,
    pub slots_per_armor: f64,
    pub availability: String,
    pub availability_per_rating: i32,
    pub cost: i64,
    pub cost_per_rating: i64,
    pub cost_per_body: i64,
    pub cost_per_armor: i64,
    pub cost_per_rating_body: i64,
    pub cost_small: Option<i64>,
    pub diy_fixed: Option<i64>,
    pub levels: Vec<Level>,
    pub cf: f64,
    pub cf_per_rating: f64,
    pub hardpoint: Option<Hardpoint>,
    pub provides: Option<Hardpoint>,
    pub many: bool,
    pub exclusive: Option<String>,
    pub requires: Option<String>,
    pub effect: Option<String>,
    pub secondary: Option<Secondary>,
    /// The sourcebook it's from, when it isn't Double Clutch ("deadly-arts");
    /// offered only when the design turns that book on.
    pub book: Option<String>,
}

impl Mod {
    /// Whether it has a rating to choose.
    pub fn rated(&self) -> bool {
        self.rating_max.is_some()
    }

    /// Accessories use no slots and get no do-it-yourself discount (p. 120).
    pub fn is_accessory(&self) -> bool {
        self.group == "accessory"
    }
}

/// Which mod slots a mod takes. `Either` is powertrain or electronics (a
/// suncell), whichever has room.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlotType {
    Chassis,
    Powertrain,
    Electronics,
    Either,
    #[default]
    None,
}

/// One rating's row when the book tabulates a mod by rating (milspec plate,
/// anti-theft). A field left out falls back to the mod's own.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Level {
    pub slots: Option<f64>,
    pub availability: Option<String>,
    pub cost: Option<i64>,
}

/// Hardpoint sizes (p. 124). A standard hardpoint splits into two small,
/// and two or three combine into a large or a huge, so each size counts as
/// some number of standard hardpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Hardpoint {
    Small,
    Standard,
    Large,
    Huge,
}

impl Hardpoint {
    pub fn standard_hardpoints(self) -> f64 {
        match self {
            Hardpoint::Small => 0.5,
            Hardpoint::Standard => 1.0,
            Hardpoint::Large => 2.0,
            Hardpoint::Huge => 3.0,
        }
    }
}

/// A secondary propulsion system's own stats while deployed (p. 137).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secondary {
    pub handling: String,
    pub acceleration: i32,
    pub speed_interval: i32,
    pub top_speed: i32,
}

/// The tables' sources, built into the program.
const DESIGN_TOML: &str = include_str!("../../../data/design.toml");
const MODS_TOML: &str = include_str!("../../../data/mods.toml");

/// `data/mods.toml`'s shape: its `[[mod]]` tables. `mod` is a Rust keyword,
/// so the field has another name and serde maps it.
#[derive(Deserialize)]
struct ModsFile {
    #[serde(rename = "mod")]
    mods: Vec<Mod>,
}

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
        CATALOG.get_or_init(|| {
            let mut catalog: Catalog = toml::from_str(DESIGN_TOML).expect("data/design.toml is valid");
            let mods: ModsFile = toml::from_str(MODS_TOML).expect("data/mods.toml is valid");
            catalog.mods = mods.mods;
            catalog
        })
    }

    pub fn vehicle_mod(&self, id: &str) -> Option<&Mod> {
        self.mods.iter().find(|m| m.id == id)
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
