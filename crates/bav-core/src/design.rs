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
    /// Mods ("Making It Yours"), in the order they were added.
    pub mods: Vec<InstalledMod>,
    /// Weapons, each in one of the vehicle's weapon mounts.
    pub weapons: Vec<Weapon>,
    /// How the stat block is laid out, as in ShadowrunEdit.
    pub layout: Layout,
    /// Whether a mod built into the design uses the vehicle's mod slots
    /// (RULINGS.md R19). The book doesn't say; on by default, and a file
    /// saved before the choice existed gets the default.
    pub built_in_uses_slots: bool,
    /// Sourcebooks beyond Double Clutch turned on for this design, by id
    /// ("deadly-arts"). Their options are offered only when they're on.
    pub books: Vec<String>,
    /// Deadly Arts' mil-spec design options (pp. 150–152).
    pub milspec: MilSpec,
    /// A picture of the vehicle, shown above its stat block: a PNG or JPEG
    /// as a data URL ("data:image/jpeg;base64,…"), kept in the file so it
    /// travels with the design. Empty for none.
    pub image: String,
}

/// The longest picture a design keeps: the window shrinks pictures well
/// below this, so only a hand-edited file comes near it.
pub const IMAGE_LIMIT: usize = 3_000_000;

impl Design {
    /// What's wrong with the design's picture, if anything: only a PNG or
    /// JPEG data URL, and not too big.
    pub fn image_problem(&self) -> Option<String> {
        let image = self.image.as_str();
        if image.is_empty() {
            None
        } else if !(image.starts_with("data:image/png;base64,") || image.starts_with("data:image/jpeg;base64,")) {
            Some("The picture isn't a PNG or JPEG; it's left out.".into())
        } else if image.len() > IMAGE_LIMIT {
            Some("The picture is too big; it's left out. Load a smaller one.".into())
        } else {
            None
        }
    }
}

/// Deadly Arts' mil-spec design options (pp. 150–152), an expansion of
/// Double Clutch's design options. Worked out in `milspec.rs`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MilSpec {
    pub airframe_plate: bool,
    /// Heavy mil-spec plate's rating: 4 to 6, or 0 for none.
    pub heavy_plate: i32,
    /// Mil-spec comms I, II, or III (1 to 3), or 0 for none.
    pub comms: i32,
    /// Mil-spec frame's rating, 1 to 12, or 0 for none.
    pub frame: i32,
    pub hardpoints: Vec<MilspecHardpoint>,
}

impl MilSpec {
    /// Whether the design uses any of them.
    pub fn is_empty(&self) -> bool {
        // Comparing with the default compares every field (derived `PartialEq`).
        *self == MilSpec::default()
    }
}

/// A mil-spec hardpoint (MHP), sized for the weapon it carries: its Grade is
/// the weapon's Size, changed by how it's mounted (p. 151).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MilspecHardpoint {
    /// The weapon's Size.
    pub size: i32,
    pub mount: MountKind,
    /// Hidden inside the body until it's needed (turreted or open only).
    pub pop_out: bool,
    /// 1, or 2 (twin-linked) or 4 (quad-linked) identical weapons.
    pub linked: i32,
    /// A dedicated primary weapon, built in at construction: its minimum
    /// Body is a Grade smaller's.
    pub primary: bool,
    /// Where it is, or what for ("bow turret").
    pub note: String,
}

impl Default for MilspecHardpoint {
    fn default() -> Self {
        MilspecHardpoint {
            size: 1,
            mount: MountKind::Turreted,
            pop_out: false,
            linked: 1,
            primary: false,
            note: String::new(),
        }
    }
}

/// How a weapon sits on a mil-spec hardpoint (p. 151).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountKind {
    /// The baseline, a full turret.
    #[default]
    Turreted,
    /// Fixed, firing straight ahead: Grade −2.
    Inline,
    /// Outside the armor: Grade −2.
    Open,
}

/// A stat block's layout, ShadowrunEdit's two modes: a gear book's (Avail
/// and Cost, "Standard Equipment") or a gamemaster's (a DR, I/ID, AC, CM
/// row, "Equipment").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    #[default]
    Gear,
    Gamemaster,
}

/// A weapon on the vehicle, typed in as ShadowrunEdit's weapon rows are.
/// The mount sets its ammunition and adds to its Attack Ratings (p. 142).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Weapon {
    pub name: String,
    /// The weapon type ("Assault Rifles", "LMG").
    pub weapon_type: String,
    /// The Damage Value as written ("5P").
    pub dv: String,
    /// Firing modes ("SA/BF/FA"); blank for none.
    pub mode: String,
    /// The five Attack Ratings, close to extreme, before the mount's +2; 0
    /// is an em dash (out of range).
    pub ar: Vec<i32>,
    /// Its own modifications ("smartgun system"), as written.
    pub mods: String,
    /// The weapon mount's mod id ("standard-turreted-weapon-mount"), or a
    /// mil-spec hardpoint's number ("milspec-1").
    pub mount: String,
    /// Its ammunition ("200(br)"), for a mount that doesn't set it: a
    /// mil-spec hardpoint carries the weapon's own magazine.
    pub ammo: String,
}

impl Default for Weapon {
    fn default() -> Self {
        Weapon {
            name: String::new(),
            weapon_type: String::new(),
            dv: String::new(),
            mode: String::new(),
            ar: vec![0; 5],
            mods: String::new(),
            mount: String::new(),
            ammo: String::new(),
        }
    }
}

/// One mod on the vehicle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InstalledMod {
    /// An id from `data/mods.toml`.
    pub id: String,
    /// Its rating, for a mod that has one (0 otherwise).
    pub rating: i32,
    /// How many, for a mod that can be installed more than once.
    pub count: i32,
    /// Built into the design (2 Build Points a slot, its price added to the
    /// vehicle's), rather than installed afterward.
    pub built_in: bool,
    /// What it's for, as the player wants it noted ("cold", "rear turret").
    pub note: String,
}

impl Default for InstalledMod {
    /// One of a mod, installed afterward. (A count of 0 would mean none.)
    fn default() -> Self {
        InstalledMod { id: String::new(), rating: 0, count: 1, built_in: false, note: String::new() }
    }
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
            mods: Vec::new(),
            weapons: Vec::new(),
            layout: Layout::Gear,
            built_in_uses_slots: true,
            books: Vec::new(),
            milspec: MilSpec::default(),
            image: String::new(),
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

    /// A copy with every level held within ±`limit`.
    ///
    /// A hand-edited file can hold any number, and multiplying a level near
    /// `i32::MAX` would overflow (a panic in a debug build, a wrong answer in
    /// a release one). `Self { … }` builds the copy field by field;
    /// `clamp` returns the value moved into the range.
    pub fn clamped(&self, limit: i32) -> Self {
        let c = |level: i32| level.clamp(-limit, limit);
        Self {
            body: c(self.body),
            armor: c(self.armor),
            handling: c(self.handling),
            handling_on_road: c(self.handling_on_road),
            handling_off_road: c(self.handling_off_road),
            seating: c(self.seating),
            size: c(self.size),
            acceleration: c(self.acceleration),
            speed_interval: c(self.speed_interval),
            top_speed: c(self.top_speed),
            cargo: c(self.cargo),
            easily_modified: c(self.easily_modified),
        }
    }
}

impl Design {
    pub fn has_modification(&self, id: &str) -> bool {
        self.modifications.iter().any(|m| m == id)
    }

    pub fn has_quality(&self, id: &str) -> bool {
        self.qualities.iter().any(|q| q == id)
    }

    /// Whether a sourcebook is turned on for this design.
    pub fn uses_book(&self, id: &str) -> bool {
        self.books.iter().any(|b| b == id)
    }
}
