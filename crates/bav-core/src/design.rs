//! A vehicle design: the choices the player makes, as saved in a file.
//!
//! It holds only choices, never results; `calc::evaluate` works out the
//! stats each time, so a file always follows the current rules.

use serde::{Deserialize, Serialize};

/// The player's choices.
///
/// `#[serde(default)]` fills any field a file leaves out with its default
/// (empty text, zero levels), so an older or hand-edited file still opens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Design {
    pub name: String,
    /// Ids from the catalog's tables.
    pub chassis: String,
    pub powertrain: String,
    pub console: String,
    pub improvements: Improvements,
    /// Design modification ids (Cargo Hauler, Racing Engine, …).
    pub modifications: Vec<String>,
    /// Quality factor ids (One-off Custom, Luxury, …). The Drone quality
    /// comes with a drone chassis and isn't listed here.
    pub qualities: Vec<String>,
    /// The extreme environment, for Environmental Adaptation.
    pub environment: String,
    /// What a Specialized Function vehicle is for (an ambulance, say).
    pub specialized_function: String,
    pub notes: String,
}

impl Default for Design {
    /// A new design starts as a basic car, so every pick has a value.
    fn default() -> Self {
        Design {
            name: String::new(),
            chassis: "car".into(),
            powertrain: "wheeled".into(),
            console: "basic".into(),
            improvements: Improvements::default(),
            modifications: Vec::new(),
            qualities: Vec::new(),
            environment: String::new(),
            specialized_function: String::new(),
            notes: String::new(),
        }
    }
}

/// Levels of each functional improvement. A negative level is the
/// improvement in reverse (less Body for fewer Build Points, say).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Improvements {
    pub body: i32,
    pub armor: i32,
    pub handling: i32,
    pub handling_on_road: i32,
    pub handling_off_road: i32,
    pub seating: i32,
    pub size: i32,
    pub acceleration: i32,
    pub speed_interval: i32,
    pub top_speed: i32,
    pub cargo: i32,
    pub easily_modified: i32,
}

impl Improvements {
    /// Each improvement's id (as in the catalog) and level, in the book's order.
    ///
    /// Returning an array of pairs lets callers loop over the fields by name
    /// without repeating the list.
    pub fn levels(&self) -> [(&'static str, i32); 12] {
        [
            ("body", self.body),
            ("armor", self.armor),
            ("handling", self.handling),
            ("handling_on_road", self.handling_on_road),
            ("handling_off_road", self.handling_off_road),
            ("seating", self.seating),
            ("size", self.size),
            ("acceleration", self.acceleration),
            ("speed_interval", self.speed_interval),
            ("top_speed", self.top_speed),
            ("cargo", self.cargo),
            ("easily_modified", self.easily_modified),
        ]
    }
}

impl Design {
    pub fn has_modification(&self, id: &str) -> bool {
        self.modifications.iter().any(|m| m == id)
    }

    pub fn has_quality(&self, id: &str) -> bool {
        self.qualities.iter().any(|q| q == id)
    }
}
