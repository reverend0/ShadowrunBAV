//! Works out a design's stats, Build Points, cost, Availability, and build
//! tests, by Double Clutch's "Build Your Dream" rules (pp. 147–157).
//!
//! Where the book contradicts itself, or leaves something open, the choice
//! made here is recorded in DESIGN.md ("Rules decisions"). All division
//! rounds down unless a comment says otherwise.

use serde::Serialize;

use crate::catalog::{Catalog, DroneSize, Kind};
use crate::design::Design;

/// Everything the UI shows for a design.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub stats: Stats,
    /// Where the Build Points come from, one line per choice.
    pub build_points: Vec<Line>,
    /// The sum of `build_points`, before quality factors.
    pub base_points: i32,
    pub multipliers: Vec<Factor>,
    /// The quality factors multiplied together (1 with none).
    pub multiplier: f64,
    /// The Build Point Value: base points × multiplier, rounded down.
    pub build_point_value: i32,
    /// The market price in nuyen (Build Point Value × 1,000).
    pub cost: i64,
    /// Parts for building it yourself: half the price.
    pub parts_cost: i64,
    /// "3", or "4(L)" with a legality code.
    pub availability: String,
    pub design_test: DesignTest,
    /// The speed calculation, step by step, for the player to follow.
    pub speed_steps: Vec<String>,
    /// Rules the design breaks. It still calculates; these say what to fix.
    pub problems: Vec<String>,
    /// Features that come with the choices (underwater adaptation, say).
    pub notes: Vec<String>,
    /// The SR6 vehicle stat line, as a header row and a value row.
    pub stat_line: StatLine,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub body: i32,
    pub armor: i32,
    pub handling_on_road: i32,
    pub handling_off_road: i32,
    /// "3", or "5/3" when on-road and off-road differ.
    pub handling: String,
    pub seat: i32,
    pub size_class: i32,
    pub length: String,
    pub weight: String,
    pub cargo_cf: f64,
    /// Cargo for people: "1/2 CF", or kilograms for the smallest classes.
    pub cargo: String,
    pub acceleration: i32,
    pub speed_interval: i32,
    pub top_speed: i32,
    pub speed_modifier: i32,
    pub pilot: i32,
    pub sensor: i32,
    pub mod_slots: ModSlots,
}

/// Mod slots for later customization (p. 120): the vehicle's Body in each
/// category, plus Easily Modified's universal slots.
#[derive(Debug, Clone, Serialize)]
pub struct ModSlots {
    pub chassis: i32,
    pub powertrain: i32,
    pub electronics: i32,
    pub universal: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Line {
    pub label: String,
    pub points: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Factor {
    pub label: String,
    pub factor: f64,
}

/// The Engineering + Logic Extended test to design the vehicle (p. 155):
/// its threshold with a 1-day, 1-week, or 1-month interval. Building it is
/// always threshold 20 with a 1-week interval, in a facility.
#[derive(Debug, Clone, Serialize)]
pub struct DesignTest {
    pub day: i32,
    pub week: i32,
    pub month: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatLine {
    pub headers: Vec<String>,
    pub values: Vec<String>,
}

/// Division rounding down, also for negative numbers (Rust's `/` rounds
/// toward zero, so −7 / 2 is −3; `div_euclid` gives −4).
fn floor_div(a: i32, b: i32) -> i32 {
    a.div_euclid(b)
}

/// Division rounding up, for test thresholds (p. 155's examples round up).
fn ceil_div(a: i32, b: i32) -> i32 {
    -floor_div(-a, b)
}

/// Works out a design. Fails only if it names a chassis, powertrain, or
/// console the catalog doesn't have.
///
/// `Result<Report, String>` is Rust's way to return either a value (`Ok`)
/// or an error (`Err`); the `?`-free `ok_or_else` calls below turn a missing
/// lookup (`None`) into an `Err` with a message.
pub fn evaluate(design: &Design, catalog: &Catalog) -> Result<Report, String> {
    let chassis = catalog.chassis(&design.chassis).ok_or_else(|| format!("Unknown chassis “{}”", design.chassis))?;
    let powertrain =
        catalog.powertrain(&design.powertrain).ok_or_else(|| format!("Unknown powertrain “{}”", design.powertrain))?;
    let console = catalog.console(&design.console).ok_or_else(|| format!("Unknown console “{}”", design.console))?;

    // Levels past ±100 (no option goes past 20) only come from a hand-edited
    // file; they're held there so the arithmetic can't overflow, and the
    // limit checks below still flag them. `clamped` returns a new value, and
    // `&` borrows it: Rust keeps the temporary alive as long as `imp` is used.
    let imp = &design.improvements.clamped(100);
    let has = |id: &str| design.has_modification(id);
    let is_drone = chassis.kind == Kind::Drone;
    let mut problems = Vec::new();
    let mut notes = Vec::new();

    // A closure that checks one improvement's level against a limit. It
    // borrows `problems` mutably, so it's declared `mut` and can't be used
    // while anything else touches `problems`; each check below is one call.
    let mut limit = |name: &str, level: i32, min: i32, max: i32| {
        if level > max {
            problems.push(format!("{name}: at most {max} level{} here.", if max == 1 { "" } else { "s" }));
        } else if level < min {
            problems.push(format!(
                "{name}: at most {} level{} in reverse here.",
                -min,
                if min == -1 { "" } else { "s" }
            ));
        }
    };

    // Size class (p. 151): +1 per level, up to 20, and at most 2 levels down.
    let size_class = chassis.size_class + imp.size;
    // (`(-2).max(…)` needs its parentheses: `-2.max(…)` would be `-(2.max(…))`.)
    limit("Improved Size", imp.size, (-2).max(-chassis.size_class), 20 - chassis.size_class);

    // Body: the chassis, Improved Body (4 levels either way), Improved Size
    // (+2 a level), then Heavy Duty or Lightweight (±50%).
    limit("Improved Body", imp.body, -4, 4);
    let mut body = chassis.body + imp.body + 2 * imp.size;
    if has("heavy-duty") {
        body = floor_div(body * 3, 2);
    } else if has("lightweight") {
        body = floor_div(body, 2);
    }

    // Armor: the chassis plus Improved Armor, never above twice the Body.
    let armor = chassis.armor + imp.armor;

    // Handling (lower is better): the powertrain's change, then Improved
    // Handling overall and on or off road only, then Off Road Chassis, which
    // moves a point from on road to off road.
    limit("Improved Handling", imp.handling, -3, 3);
    limit("Improved Handling (on road)", imp.handling + imp.handling_on_road, -3, 3);
    limit("Improved Handling (off road)", imp.handling + imp.handling_off_road, -3, 3);
    let base_handling = chassis.handling + powertrain.handling;
    let mut handling_on_road = base_handling - imp.handling - imp.handling_on_road;
    let mut handling_off_road = base_handling + powertrain.off_road_handling - imp.handling - imp.handling_off_road;
    if has("off-road-chassis") {
        handling_on_road += 1;
        handling_off_road -= 1;
    }

    // Seating: Improved Seating adds up to 50% of the chassis's seats; each
    // level of Improved Size adds 50% more; Cargo Hauler halves it (to no
    // fewer than 2) and Mass Transit triples it.
    let extra_seats = chassis.seat / 2;
    limit("Improved Seating", imp.seating, -extra_seats, extra_seats);
    let mut seat = floor_div((chassis.seat + imp.seating) * (2 + imp.size), 2).max(0);
    if has("cargo-hauler") && seat > 0 {
        seat = (seat / 2).max(2).min(seat);
    }
    if has("mass-transit") {
        seat *= 3;
    }

    // Cargo: the size class's, plus a truck's extra. Improved Cargo Space
    // adds 2 CF a level, at most 3 levels and no more than doubling it.
    let size_row = catalog.size_class(size_class);
    let start_cargo = size_row.cargo_cf + chassis.extra_cargo_cf;
    let cargo_levels = ((start_cargo / 2.0).floor() as i32).min(3);
    limit("Improved Cargo Space", imp.cargo, -cargo_levels, cargo_levels);
    let mut cargo_cf = (start_cargo + 2.0 * f64::from(imp.cargo)).max(0.0);
    if has("cargo-hauler") {
        cargo_cf *= 2.0;
    }

    // Easily Modified: 2 universal mod slots a level, at most 4 levels or the
    // chassis's Body, whichever is lower. It has no reverse.
    limit("Easily Modified", imp.easily_modified, 0, chassis.body.min(4));
    limit("Improved Acceleration", imp.acceleration, -2, 2);
    limit("Improved Speed Interval", imp.speed_interval, -3, 3);
    limit("Improved Top Speed", imp.top_speed, -3, 3);

    // The closure's borrow of `problems` ends with its last use above, so
    // `problems` can be used directly from here on.

    // Handling is a test's threshold, so it's at least 1 (decided; see
    // DESIGN.md). `&mut` lets the loop change both values in place.
    for (value, road) in [(&mut handling_on_road, "on road"), (&mut handling_off_road, "off road")] {
        if *value < 1 {
            problems.push(format!("Handling {road} is {}: it can't be below 1. It's shown as 1.", *value));
            *value = 1;
        }
    }

    if body < 0 {
        problems.push(format!("Body can't go below 0 (it's {body})."));
        body = 0;
    }
    if armor > 2 * body {
        problems.push(format!("Armor {armor} is more than twice the Body ({body})."));
    }
    let armor = if armor < 0 {
        problems.push(format!("Armor can't go below 0 (it's {armor})."));
        0
    } else {
        armor
    };
    if is_drone && imp.size != 0 && !(chassis.drone_size == Some(DroneSize::Large) && imp.size > 0) {
        problems.push(
            "A drone's size comes from its chassis: choose a bigger or smaller drone chassis instead \
             (only a large drone can be made bigger)."
                .into(),
        );
    }

    // Speed (p. 154). The speed modifier is Body ÷ 2 + Armor; Integrated
    // Armor or Cargo Hauler count half the Armor (they don't stack).
    let mut speed_steps = Vec::new();
    let half_armor = has("integrated-armor") || has("cargo-hauler");
    let armor_for_speed = if half_armor { armor / 2 } else { armor };
    let speed_modifier = body / 2 + armor_for_speed;
    speed_steps.push(if half_armor {
        format!("Speed modifier: Body {body} ÷ 2 + Armor {armor} ÷ 2 = {speed_modifier}")
    } else {
        format!("Speed modifier: Body {body} ÷ 2 + Armor {armor} = {speed_modifier}")
    });

    let mut acceleration = powertrain.acceleration + 5 * imp.acceleration;
    let mut speed_interval = powertrain.speed_interval + 5 * imp.speed_interval;
    if speed_interval < 1 {
        problems.push(format!("Speed Interval is {speed_interval}: too little for this powertrain. It's shown as 1."));
        speed_interval = 1;
    }
    let mut top_speed = powertrain.top_speed + 25 * imp.top_speed;
    if imp.acceleration != 0 || imp.top_speed != 0 {
        speed_steps.push(format!("Improved: Acceleration {acceleration}, Top Speed {top_speed}"));
    }

    acceleration -= speed_modifier / 2;
    if powertrain.no_top_speed_loss {
        speed_steps.push(format!(
            "Acceleration − {} = {acceleration}; Top Speed {top_speed} (no loss for a {})",
            speed_modifier / 2,
            powertrain.name.to_lowercase()
        ));
    } else {
        top_speed -= speed_modifier * 10;
        speed_steps.push(format!(
            "Acceleration − {} = {acceleration}; Top Speed − {} = {top_speed}",
            speed_modifier / 2,
            speed_modifier * 10
        ));
    }

    // Small drones multiply their Acceleration, before the engine steps
    // (decided; see DESIGN.md).
    let drone_factor = match chassis.drone_size {
        Some(DroneSize::Small) => Some((3, 4, "× 0.75 (small drone)")),
        Some(DroneSize::Mini) => Some((1, 2, "× 0.5 (mini drone)")),
        Some(DroneSize::Micro) => Some((1, 4, "× 0.25 (micro drone)")),
        _ => None,
    };
    // `if let` runs the block only when the pattern matches: here, when
    // `drone_factor` is `Some`, binding the tuple's parts to names.
    if let Some((num, den, label)) = drone_factor {
        acceleration = floor_div(acceleration * num, den);
        speed_steps.push(format!("Acceleration {label} = {acceleration}"));
    }

    // Engine type. A Cargo Hauler gets half the High Performance bonus.
    let engine = if has("racing-engine") {
        Some(("Racing Engine", 6, 100))
    } else if has("high-performance-engine") {
        let halved = has("cargo-hauler");
        Some(if halved {
            ("High Performance Engine (half, Cargo Hauler)", 1, 25)
        } else {
            ("High Performance Engine", 3, 50)
        })
    } else if has("economy-engine") {
        Some(("Economy Engine", -3, -50))
    } else {
        None
    };
    if let Some((label, accel, top)) = engine {
        acceleration += accel;
        top_speed += top;
        speed_steps.push(format!("{label}: Acceleration {accel:+} = {acceleration}, Top Speed {top:+} = {top_speed}"));
    }
    if has("off-road-chassis") {
        top_speed -= 10;
        speed_steps.push(format!("Off Road Chassis: Top Speed − 10 = {top_speed}"));
    }
    if has("multi-engine") {
        if has("cargo-hauler") {
            acceleration = floor_div(acceleration * 3, 2);
            top_speed = floor_div(top_speed * 3, 2);
            speed_steps.push(format!(
                "Multi-Engine (half bonus, Cargo Hauler): × 1.5 = Acceleration {acceleration}, Top Speed {top_speed}"
            ));
        } else {
            acceleration *= 2;
            top_speed *= 2;
            speed_steps.push(format!("Multi-Engine: × 2 = Acceleration {acceleration}, Top Speed {top_speed}"));
        }
    }
    if has("underpowered") {
        top_speed = floor_div(top_speed, 2);
        speed_steps.push(format!("Underpowered: Top Speed ÷ 2 = {top_speed}"));
    }
    if acceleration < 0 {
        problems.push(format!(
            "Acceleration is {acceleration}: too much Body and Armor for this powertrain. It's shown as 0."
        ));
        acceleration = 0;
    }
    if top_speed < 0 {
        problems
            .push(format!("Top Speed is {top_speed}: too much Body and Armor for this powertrain. It's shown as 0."));
        top_speed = 0;
    }

    // Choices that exclude each other.
    for (group, what) in [("engine", "engine type"), ("build", "Heavy Duty or Lightweight")] {
        let chosen: Vec<&str> = design
            .modifications
            .iter()
            .filter_map(|id| catalog.modification(id))
            .filter(|m| m.group.as_deref() == Some(group))
            .map(|m| m.name.as_str())
            .collect();
        if chosen.len() > 1 {
            problems.push(format!("Choose one {what}: {}.", chosen.join(", ")));
        }
    }
    for (group, what) in [("production", "way it's made"), ("service", "of Security or Military")] {
        let chosen: Vec<&str> = design
            .qualities
            .iter()
            .filter_map(|id| catalog.quality(id))
            .filter(|q| q.group.as_deref() == Some(group))
            .map(|q| q.name.as_str())
            .collect();
        if chosen.len() > 1 {
            problems.push(format!("Choose one {what}: {}.", chosen.join(", ")));
        }
    }

    // Build Points.
    let mut build_points = vec![
        Line { label: format!("Chassis: {}", chassis.name), points: chassis.build_points },
        Line { label: format!("Powertrain: {}", powertrain.name), points: powertrain.build_points },
        Line { label: format!("Console: {}", console.name), points: console.build_points },
    ];
    for (id, level) in imp.levels() {
        if level == 0 {
            continue;
        }
        if let Some(improvement) = catalog.improvement(id) {
            let label = if level > 0 {
                format!("{} ×{level}", improvement.name)
            } else {
                format!("{} in reverse ×{}", improvement.name, -level)
            };
            build_points.push(Line { label, points: level * improvement.per_level });
        }
    }
    for id in &design.modifications {
        match catalog.modification(id) {
            Some(m) => build_points.push(Line { label: m.name.clone(), points: m.build_points }),
            None => problems.push(format!("Unknown design modification “{id}” left out.")),
        }
    }
    let base_points: i32 = build_points.iter().map(|line| line.points).sum();

    // Quality factors multiply together. A drone chassis always has the
    // Drone quality.
    let mut multipliers = Vec::new();
    if is_drone && let Some(drone) = catalog.quality("drone") {
        multipliers.push(Factor { label: format!("{} (drone chassis)", drone.name), factor: drone.multiplier });
    }
    let mut availability = 2;
    let mut legality: Option<&str> = None;
    for id in &design.qualities {
        if id == "drone" {
            continue; // comes from the chassis
        }
        let Some(quality) = catalog.quality(id) else {
            problems.push(format!("Unknown quality factor “{id}” left out."));
            continue;
        };
        let label = match id.as_str() {
            "environmental-adaptation" if !design.environment.is_empty() => {
                format!("{} ({})", quality.name, design.environment)
            }
            "specialized-function" if !design.specialized_function.is_empty() => {
                format!("{} ({})", quality.name, design.specialized_function)
            }
            _ => quality.name.clone(),
        };
        multipliers.push(Factor { label, factor: quality.multiplier });
        availability += quality.availability;
        // Illegal (I) outranks Licensed (L).
        if let Some(code) = quality.legality.as_deref()
            && (legality.is_none() || code == "I")
        {
            legality = Some(code);
        }
    }
    let multiplier: f64 = multipliers.iter().map(|f| f.factor).product();
    let build_point_value = (f64::from(base_points) * multiplier).floor() as i32;
    if build_point_value < 0 {
        problems.push(format!("The Build Point Value is {build_point_value}; it can't be below 0."));
    }
    let cost = i64::from(build_point_value.max(0)) * 1000;

    // Availability (p. 155): 2, +1 for an aircraft, plus the qualities'.
    if chassis.kind == Kind::Air || powertrain.medium == Kind::Air {
        availability += 1;
    }
    let availability = match legality {
        Some(code) => format!("{availability}({code})"),
        None => availability.to_string(),
    };

    // Quality factors' own rules and features.
    let quality = |id: &str| design.has_quality(id);
    if chassis.underwater {
        notes.push("Built for underwater use.".into());
    }
    if powertrain.no_top_speed_loss {
        notes.push(format!("A {} loses no Top Speed for Body and Armor.", powertrain.name.to_lowercase()));
    }
    if quality("anthropomorphic") {
        if powertrain.id != "walker" {
            problems.push("An anthropomorphic vehicle walks: give it the Walker powertrain.".into());
        }
        notes.push(if size_class <= 4 {
            "Anthropomorphic: built like a person, with a pair of arms and legs. Small enough to fit through \
             spaces made for people, and to use some of their gear (p. 153)."
                .into()
        } else {
            "Anthropomorphic: built like a person, with a pair of arms and legs (p. 153).".into()
        });
    }
    if quality("environmental-adaptation") {
        if design.environment.is_empty() {
            problems.push("Environmental Adaptation: choose the environment.".into());
        } else if chassis.underwater && design.environment == "Underwater" {
            problems.push("A submersible is already built for underwater use.".into());
        } else {
            notes.push(format!("Adapted to {}, with basic life support.", design.environment.to_lowercase()));
        }
    }
    if quality("specialized-function") {
        if design.specialized_function.is_empty() {
            problems.push("Specialized Function: say what it's for.".into());
        } else {
            notes.push(format!(
                "Specialized function ({}): comes with the mods that function needs.",
                design.specialized_function
            ));
        }
    }
    if quality("luxury") {
        notes.push("Luxury: High Amenities.".into());
    }
    let below_expert = console.pilot < 3;
    if quality("security") {
        notes.push("Security: Ceramic Plate foundation.".into());
        if below_expert {
            notes.push("Security vehicles usually have at least an Expert console.".into());
        }
    }
    if quality("military") {
        notes.push("Military: Milspec Plate foundation.".into());
        if below_expert {
            notes.push("Military vehicles usually have at least an Expert console.".into());
        }
    }
    if quality("superstructure") && size_class < 15 {
        problems.push(format!("A superstructure must be Size Class 15 or more (this is {size_class})."));
    }
    if has("cargo-hauler") {
        notes.push("Cargo Hauler: tows as if 2 Size Classes larger.".into());
    }

    // The design test's threshold rounds up (the book's examples do).
    let day = ceil_div(build_point_value.max(0), 5);
    let design_test = DesignTest { day, week: ceil_div(day, 2), month: ceil_div(day, 4) };

    let handling = if handling_on_road == handling_off_road {
        handling_on_road.to_string()
    } else {
        format!("{handling_on_road}/{handling_off_road}")
    };
    let stat_line = StatLine {
        headers: ["Hand", "Accel", "Spd Int", "Top Spd", "Body", "Armor", "Pilot", "Sensor", "Seat", "Avail", "Cost"]
            .map(String::from)
            .to_vec(),
        values: vec![
            handling.clone(),
            acceleration.to_string(),
            speed_interval.to_string(),
            top_speed.to_string(),
            body.to_string(),
            armor.to_string(),
            console.pilot.to_string(),
            console.sensor.to_string(),
            if is_drone { "—".into() } else { seat.to_string() },
            availability.clone(),
            format!("{}¥", thousands(cost)),
        ],
    };

    Ok(Report {
        stats: Stats {
            body,
            armor,
            handling_on_road,
            handling_off_road,
            handling,
            seat,
            size_class,
            length: size_row.length.clone(),
            weight: size_row.weight.clone(),
            cargo_cf,
            cargo: cargo_text(cargo_cf),
            acceleration,
            speed_interval,
            top_speed,
            speed_modifier,
            pilot: console.pilot,
            sensor: console.sensor,
            mod_slots: ModSlots {
                chassis: body,
                powertrain: body,
                electronics: body,
                universal: 2 * imp.easily_modified.max(0),
            },
        },
        build_points,
        base_points,
        multipliers,
        multiplier,
        build_point_value,
        cost,
        parts_cost: cost / 2,
        availability,
        design_test,
        speed_steps,
        problems,
        notes,
        stat_line,
    })
}

/// A number with thousands separators: 173000 → "173,000".
pub fn thousands(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("–{out}") } else { out }
}

/// Cargo as the book writes it: kilograms below a quarter CF, fractions for
/// the quarter and half, else a whole number of CF.
pub fn cargo_text(cf: f64) -> String {
    if cf <= 0.0 {
        "none".into()
    } else if cf < 0.25 {
        format!("{} kg", (cf * 200.0).round())
    } else if cf.fract() == 0.0 {
        format!("{} CF", thousands(cf as i64))
    } else if cf == 0.25 {
        "1/4 CF".into()
    } else if cf == 0.5 {
        "1/2 CF".into()
    } else {
        format!("{cf} CF")
    }
}
