//! Vehicle mods, by Double Clutch's "Making It Yours" rules (pp. 120–144):
//! each mod's slots, Cargo Factors, hardpoints, price, Availability, and
//! install test, what it changes on the vehicle, and the mod slot budget.
//!
//! `calc::evaluate` works out the designed vehicle first (its "factory"
//! stats), then hands them here. Where the book contradicts itself, the
//! choice made is recorded in RULINGS.md.

use serde::Serialize;

use crate::catalog::{Catalog, Chassis, DroneSize, Form, Hardpoint, Mod, Powertrain, SlotType};
use crate::design::InstalledMod;

/// The numbers mods change. `calc` fills it with the designed vehicle's;
/// `apply` returns a changed copy.
#[derive(Debug, Clone, PartialEq)]
pub struct Vehicle {
    pub body: i32,
    pub armor: i32,
    pub handling_on_road: i32,
    pub handling_off_road: i32,
    pub acceleration: i32,
    pub speed_interval: i32,
    pub top_speed: i32,
    pub pilot: i32,
    pub sensor: i32,
    pub seat: i32,
    pub cargo_cf: f64,
}

/// What a mod's rules need to know about the vehicle beyond its stats.
pub struct Context<'a> {
    pub chassis: &'a Chassis,
    pub powertrain: &'a Powertrain,
    pub size_class: i32,
    pub is_aircraft: bool,
    /// Easily Modified's universal slots.
    pub universal_slots: i32,
}

impl Context<'_> {
    fn is_drone(&self) -> bool {
        self.chassis.drone_size.is_some()
    }
}

/// Everything the report shows about the mods.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ModsReport {
    pub lines: Vec<ModLine>,
    /// Chassis, powertrain, and electronics slots: used and available.
    pub slots: Vec<SlotUse>,
    pub universal_slots: i32,
    pub hardpoints: HardpointUse,
    /// Cargo Factors the CF mods take.
    pub cargo_used: f64,
    /// Build Points for the mods built into the design (2 a slot).
    pub built_in_points: i32,
    /// The built-in mods' price, part of the vehicle's.
    pub built_in_cost: i64,
    /// The mods installed afterward: at a shop, or doing it yourself.
    pub aftermarket_cost: i64,
    pub aftermarket_diy_cost: i64,
    /// The highest Availability among the mods, and its legality code.
    pub availability: i32,
    pub legality: Option<String>,
    /// A secondary propulsion system's stats while deployed.
    pub secondary: Option<SecondaryLine>,
    /// The form the skin ends up with.
    pub form: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModLine {
    pub id: String,
    /// The mod as listed: name, rating, count, and note.
    pub label: String,
    pub slots: f64,
    pub cf: f64,
    pub availability: String,
    pub price: i64,
    /// Doing it yourself (half, except accessories); 0 when built in.
    pub diy_price: i64,
    pub built_in: bool,
    /// Build Points when built in (2 a slot, rounded up).
    pub build_points: i32,
    /// Engineering + Logic (threshold, interval), and the tools needed.
    pub install_test: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotUse {
    pub category: String,
    pub used: f64,
    pub available: i32,
}

/// Hardpoints, counted in standard hardpoints (a small is half, a large
/// two, a huge three; p. 124).
#[derive(Debug, Clone, Default, Serialize)]
pub struct HardpointUse {
    pub free: f64,
    pub added: f64,
    pub used: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecondaryLine {
    pub name: String,
    pub handling: String,
    pub acceleration: i32,
    pub speed_interval: i32,
    pub top_speed: i32,
}

/// One installed mod worked out, before it's added to the totals.
struct Worked<'a> {
    def: &'a Mod,
    installed: &'a InstalledMod,
    rating: i32,
    count: i32,
    slots: f64,
    cf: f64,
    availability: Option<(i32, Option<String>)>,
    price: i64,
}

/// Works out the installed mods on a vehicle whose designed stats are
/// `factory`, returning the changed stats and the report. Problems (rules a
/// mod breaks) and notes (what comes with one) are added to the lists given.
pub fn apply(
    installed: &[InstalledMod],
    catalog: &Catalog,
    ctx: &Context,
    factory: &Vehicle,
    problems: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> (Vehicle, ModsReport) {
    let mut v = factory.clone();
    let mut report = ModsReport { universal_slots: ctx.universal_slots, ..ModsReport::default() };

    // The mods the catalog knows, each paired with its definition.
    // `filter_map` keeps the `Some`s and drops the `None`s, after noting them.
    let known: Vec<(&InstalledMod, &Mod)> = installed
        .iter()
        .filter_map(|inst| match catalog.vehicle_mod(&inst.id) {
            Some(def) => Some((inst, def)),
            None => {
                problems.push(format!("Unknown mod “{}” left out.", inst.id));
                None
            }
        })
        .collect();
    let has = |id: &str| known.iter().any(|(inst, _)| inst.id == id);
    let rating_of = |id: &str| known.iter().find(|(inst, _)| inst.id == id).map(|(inst, _)| inst.rating.max(1));

    // Standard armor and the form come first: other mods' slots, prices, and
    // rules read the final Armor and form.
    // Its rating can be negative (armor removed), so it's read as it is,
    // held to its range, rather than through `rating_of`.
    let armor_added = known
        .iter()
        .find(|(inst, _)| inst.id == "standard-armor")
        .map_or(0, |(inst, _)| inst.rating.clamp(-factory.armor, 2 * factory.body));
    v.armor = factory.armor + armor_added;
    let form = known
        .iter()
        .find_map(|(_, def)| match def.id.as_str() {
            "form-open" => Some(Form::Open),
            "form-standard" => Some(Form::Standard),
            "form-enclosed" => Some(Form::Enclosed),
            _ => None,
        })
        .unwrap_or(ctx.chassis.form);
    report.form = form.name().into();

    // Each mod's own numbers.
    let mut worked = Vec::new();
    for &(inst, def) in &known {
        worked.push(work_out(inst, def, ctx, factory, &v, form, problems));
    }

    // Rules between mods.
    check_exclusive(&worked, problems);
    for w in &worked {
        check_requirements(w, ctx, form, &known, problems);
    }

    // What the mods change, in the order the rules build on each other.
    let mut steps = Effects { v: &mut v, factory, problems: &mut *problems, notes: &mut *notes };
    for w in &worked {
        steps.apply(w, ctx, &mut report);
    }
    if has("amenities-squatter") {
        v.seat *= 2;
    }
    floor_stats(&mut v, factory, ctx, problems);
    if let Some(r) = rating_of("pilot-by-wire")
        && v.pilot < 2 * r
    {
        problems.push(format!("Pilot-By-Wire {r} needs Pilot {} or more (this has {}).", 2 * r, v.pilot));
    }

    // Totals: slots, Cargo Factors, hardpoints, prices, Availability.
    let mut used = [0.0_f64; 3]; // chassis, powertrain, electronics
    let mut either = 0.0;
    for w in &worked {
        let total = w.slots * f64::from(w.count);
        match w.def.slot_type {
            SlotType::Chassis => used[0] += total,
            SlotType::Powertrain => used[1] += total,
            SlotType::Electronics => used[2] += total,
            SlotType::Either => either += total,
            SlotType::None => {}
        }
        report.cargo_used += w.cf * f64::from(w.count);
        if let Some(size) = w.def.provides {
            report.hardpoints.added += size.standard_hardpoints() * f64::from(w.count);
        }
        if let Some(size) = w.def.hardpoint {
            report.hardpoints.used += size.standard_hardpoints() * f64::from(w.count);
        }
        // Two steps: `line` borrows `report` mutably, which can't overlap with
        // the borrow `report.lines.push` takes.
        let mod_line = line(w, &mut report);
        report.lines.push(mod_line);
    }
    // A suncell goes where there's more room left.
    let body = f64::from(factory.body);
    if body - used[1] >= body - used[2] {
        used[1] += either;
    } else {
        used[2] += either;
    }
    report.slots = ["Chassis", "Powertrain", "Electronics"]
        .iter()
        .zip(used)
        .map(|(category, used)| SlotUse { category: (*category).into(), used, available: factory.body })
        .collect();
    check_slots(&report, problems);

    report.hardpoints.free = free_hardpoints(ctx, factory.body);
    let hp = &report.hardpoints;
    if hp.used > hp.free + hp.added {
        problems.push(format!(
            "The hardpoint devices need {} standard hardpoints' worth; the vehicle has {} ({} free for its Body, {} added).",
            number(hp.used),
            number(hp.free + hp.added),
            number(hp.free),
            number(hp.added)
        ));
    }

    if report.cargo_used > v.cargo_cf {
        problems.push(format!(
            "The CF mods take {} CF; the vehicle has {} CF. Increased Cargo Space adds 2 CF a rating.",
            number(report.cargo_used),
            number(v.cargo_cf)
        ));
    }
    v.cargo_cf = (v.cargo_cf - report.cargo_used).max(0.0);

    (v, report)
}

/// One mod's slots, Cargo Factors, Availability, and price.
fn work_out<'a>(
    inst: &'a InstalledMod,
    def: &'a Mod,
    ctx: &Context,
    factory: &Vehicle,
    v: &Vehicle,
    form: Form,
    problems: &mut Vec<String>,
) -> Worked<'a> {
    // The rating: checked against its range (some ranges depend on the
    // vehicle), and a missing one counts as the lowest.
    let mut rating = 0;
    if def.effect.as_deref() == Some("gecko") {
        if ctx.size_class > 4 {
            problems.push(format!("Gecko Tips work up to Size Class 4 (this is {}).", ctx.size_class));
        }
        rating = ctx.size_class.clamp(1, 4);
    } else if def.rated() {
        let (min, max) = match def.id.as_str() {
            // Up to half the factory Body, so the Body ends up at most 1.5 × (decided).
            "structural-integrity" => (1, factory.body / 2),
            // Up to twice the Body added, or all of it removed.
            "standard-armor" => (-factory.armor, 2 * factory.body),
            _ => (def.rating_min.unwrap_or(1), def.rating_max.unwrap_or(1)),
        };
        rating = if inst.rating == 0 && min > 0 { min } else { inst.rating };
        if rating < min || rating > max {
            problems.push(format!("{}: rating {min} to {max} here (it's {rating}).", def.name));
            rating = rating.clamp(min, max.max(min));
        }
    }
    let count = if def.many { inst.count.max(1) } else { 1 };
    if !def.many && inst.count > 1 {
        problems.push(format!("{} is installed once; the count is left out.", def.name));
    }
    let level = if def.levels.is_empty() { None } else { def.levels.get((rating.max(1) - 1) as usize) };

    let body = f64::from(factory.body);
    let mut slots = match level.and_then(|l| l.slots) {
        Some(s) => s,
        None => {
            def.slots
                + def.slots_per_rating * f64::from(rating)
                + (def.slots_per_body * body).ceil()
                + def.slots_per_armor * f64::from(v.armor)
        }
    };
    // Easy assembly costs a micro drone no slots (p. 123).
    if def.id == "easy-assembly" && ctx.chassis.drone_size == Some(DroneSize::Micro) {
        slots = 0.0;
    }

    let cf = def.cf + def.cf_per_rating * f64::from(rating);

    let availability_text = level.and_then(|l| l.availability.clone()).unwrap_or_else(|| def.availability.clone());
    let mut availability =
        parse_availability(&availability_text).map(|(n, code)| (n + def.availability_per_rating * rating, code));

    let small_vehicle = ctx.is_drone() || ctx.chassis.id == "bike";
    let mut price = match level.and_then(|l| l.cost) {
        Some(c) => c,
        None => {
            let base = match def.cost_small {
                Some(small) if small_vehicle => small,
                _ => def.cost,
            };
            base + def.cost_per_rating * i64::from(rating)
                + def.cost_per_body * i64::from(factory.body)
                + def.cost_per_armor * i64::from(v.armor)
                + def.cost_per_rating_body * i64::from(rating) * i64::from(factory.body)
        }
    };
    // Removed armor costs nothing (the book prices only armor added).
    if def.id == "standard-armor" {
        price = price.max(0);
    }

    // A form change: a slot and a price per level, from the chassis's form.
    if def.effect.as_deref() == Some("form") {
        let change = form.level() - ctx.chassis.form.level();
        slots = f64::from(change);
        if change == 0 {
            problems.push(format!("The chassis already has {}.", form.name()));
            price = 0;
            availability = None;
        } else if change < 0 {
            price = 4000 * i64::from(-change);
            availability = Some((2, None));
        } else {
            price = 6000 * i64::from(change);
            availability = Some((3, None));
            // More skin needs the armor for it (p. 131): Armor at least the
            // Body for standard, 1.5 × Body (rounded up) for enclosed.
            let need = match form {
                Form::Enclosed => (3 * factory.body + 1) / 2,
                _ => factory.body,
            };
            if v.armor < need {
                let name = form.name();
                // Capitalize the first letter: `chars()` yields it, and the
                // rest of the &str is sliced after its byte length.
                let first = name.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or(' ');
                problems.push(format!("{first}{} needs Armor {need} or more (this has {}).", &name[1..], v.armor));
            }
        }
    }

    Worked { def, installed: inst, rating, count, slots, cf, availability, price: price * i64::from(count) }
}

/// "4(I)" → (4, Some("I")); "" → None (the book's "--": no Availability).
fn parse_availability(text: &str) -> Option<(i32, Option<String>)> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    // `split_once('(')` splits "4(I)" into "4" and "I)".
    let (number, code) = match text.split_once('(') {
        Some((n, rest)) => (n.trim(), Some(rest.trim_end_matches(')').trim().to_uppercase())),
        None => (text, None),
    };
    number.parse().ok().map(|n| (n, code))
}

fn availability_text(availability: &Option<(i32, Option<String>)>) -> String {
    match availability {
        None => "—".into(),
        Some((n, Some(code))) => format!("{n}({code})"),
        Some((n, None)) => n.to_string(),
    }
}

/// Only one mod from each `exclusive` group (one set of tires, one form).
fn check_exclusive(worked: &[Worked], problems: &mut Vec<String>) {
    let mut seen: Vec<(&str, Vec<&str>)> = Vec::new();
    for w in worked {
        if let Some(group) = w.def.exclusive.as_deref() {
            match seen.iter_mut().find(|(g, _)| *g == group) {
                Some((_, names)) => names.push(&w.def.name),
                None => seen.push((group, vec![&w.def.name])),
            }
        }
    }
    for (_, names) in seen {
        if names.len() > 1 {
            problems.push(format!("Choose one of these: {}.", names.join(", ")));
        }
    }
    let ids: Vec<&str> = worked.iter().map(|w| w.def.id.as_str()).collect();
    if ids.contains(&"improved-handling") && ids.contains(&"pilot-by-wire") {
        problems.push("Improved Handling and Pilot-By-Wire don't stack (p. 126): choose one.".into());
    }
    if ids.contains(&"racing-suspension") && (ids.contains(&"improved-stability") || ids.contains(&"tuned-suspension"))
    {
        problems.push("Racing Suspension doesn't work with Improved Stability or Tuned Suspension (p. 137).".into());
    }
}

/// The vehicles each mod is for.
fn check_requirements(
    w: &Worked,
    ctx: &Context,
    form: Form,
    known: &[(&InstalledMod, &Mod)],
    problems: &mut Vec<String>,
) {
    let name = &w.def.name;
    let size = ctx.chassis.drone_size;
    let ok = match w.def.requires.as_deref() {
        Some("passenger") => !ctx.is_drone(),
        Some("wheeled") => {
            ctx.powertrain.id == "wheeled" || known.iter().any(|(inst, _)| inst.id == "secondary-wheeled")
        }
        Some("medevac-drone") => matches!(size, Some(DroneSize::Medium | DroneSize::Large)),
        Some("small-drone") => matches!(size, Some(s) if s <= DroneSize::Medium),
        Some("valkyrie") => {
            !ctx.is_drone() || (size == Some(DroneSize::Large) && known.iter().any(|(inst, _)| inst.id == "medevac"))
        }
        _ => true,
    };
    if !ok {
        let what = match w.def.requires.as_deref() {
            Some("passenger") => "passenger vehicles, not drones",
            Some("wheeled") => "wheeled vehicles",
            Some("medevac-drone") => "medium and large drones",
            Some("small-drone") => "medium drones and smaller",
            _ => "passenger vehicles, or large drones with Medevac Modifications",
        };
        problems.push(format!("{name} is only for {what}."));
    }
    match w.def.effect.as_deref() {
        Some("closed-form") if form == Form::Open => {
            problems.push(format!("{name} needs standard or totally enclosed construction."));
        }
        Some("enclosed-form") if form != Form::Enclosed => {
            problems.push(format!("{name} needs totally enclosed construction."));
        }
        _ => {}
    }
    if w.def.provides.is_some() {
        match size {
            Some(DroneSize::Micro) => problems.push("A microdrone can't add hardpoints (p. 124).".into()),
            Some(DroneSize::Mini | DroneSize::Small) if w.def.provides != Some(Hardpoint::Small) => {
                problems.push("Small drones and minidrones can only add small hardpoints (p. 124).".into());
            }
            _ => {}
        }
    }
}

/// Applies what mods change to the stats, in turn.
struct Effects<'a, 'b> {
    v: &'a mut Vehicle,
    factory: &'a Vehicle,
    problems: &'b mut Vec<String>,
    notes: &'b mut Vec<String>,
}

impl Effects<'_, '_> {
    fn apply(&mut self, w: &Worked, ctx: &Context, report: &mut ModsReport) {
        let r = w.rating;
        let f = self.factory;
        let v = &mut *self.v;
        match w.def.effect.as_deref() {
            Some("structure") => v.body += r,
            Some("armor") => {
                // Every 2 points of armor added cost 1 Acceleration, every
                // point 10 Top Speed; removing armor gives them back (p. 133).
                // Rust's `/` rounds toward zero, so −3 / 2 is −1, as wanted.
                v.acceleration -= r / 2;
                v.top_speed -= 10 * r;
                if v.top_speed <= 0 && !ctx.is_aircraft {
                    self.problems.push("Standard Armor: this much armor leaves the vehicle unable to move.".into());
                }
                if ctx.is_aircraft && v.top_speed * 2 < f.top_speed {
                    self.problems
                        .push("Standard Armor: an aircraft needs at least half its Top Speed to take off.".into());
                }
            }
            Some("milspec") => v.speed_interval -= 5 * r,
            Some("improved-handling" | "pilot-by-wire") => {
                v.handling_on_road -= r;
                v.handling_off_road -= r;
            }
            Some("multi-terrain") => {
                // Moves points of off-road Handling to on-road, until they're
                // level at most (p. 126: 5/7 becomes 6/6, then 7/5).
                let points = r.min((v.handling_off_road - v.handling_on_road).max(0));
                if points == 0 {
                    self.notes
                        .push(format!("{}: no effect on a vehicle whose off-road Handling is no worse.", w.def.name));
                }
                v.handling_on_road += points;
                v.handling_off_road -= points;
            }
            Some("multi-terrain-active") => self.notes.push(format!(
                "{}: moves up to {r} points of off-road Handling to on-road, or back, on command (a Minor Action, \
                 two combat rounds).",
                w.def.name
            )),
            // Each rank adds 30% (Acceleration) or 20% (Top Speed) of the
            // vehicle's own value, rounded up (pp. 134, 138).
            Some("acceleration") => v.acceleration += r * ceil_percent(f.acceleration, 30),
            Some("top-speed") => v.top_speed += r * ceil_percent(f.top_speed, 20),
            Some("pilot") => v.pilot = r,
            Some("sensor") => v.sensor = r,
            Some("seats") => v.seat += r * w.count,
            Some("cargo") => v.cargo_cf += 2.0 * f64::from(r),
            Some("racing-tires") => self
                .notes
                .push("Racing tires: Acceleration +5 and Speed Interval +10 on road; −5 and −10 off road.".into()),
            Some("off-road-tires") => self
                .notes
                .push("Off-road tires: Acceleration +5 and Speed Interval +10 off road; −5 and −10 on road.".into()),
            Some("smart-tires") => {
                self.notes.push("Smart tires: Acceleration +5 and Speed Interval +10 on road and off road.".into())
            }
            _ => {}
        }
        if let Some(s) = &w.def.secondary {
            report.secondary = Some(SecondaryLine {
                name: w.def.name.trim_start_matches("Secondary Propulsion: ").into(),
                handling: s.handling.clone(),
                acceleration: s.acceleration,
                speed_interval: s.speed_interval,
                top_speed: s.top_speed,
            });
        }
        // Squatter amenities double the seats (p. 128); last, after any
        // seats added.
        if w.def.effect.as_deref() == Some("squatter") {
            self.notes.push("Squatter amenities: twice the seats, cramped.".into());
        }
    }
}

/// `percent`% of `value`, rounded up.
fn ceil_percent(value: i32, percent: i32) -> i32 {
    // Integer arithmetic rounds down; adding 99 first rounds up instead.
    (value.max(0) * percent + 99) / 100
}

/// The floors after mods: Handling and Speed Interval at least 1 (p. 125
/// for Handling), Acceleration and Top Speed at least 0, and squatter seats.
fn floor_stats(v: &mut Vehicle, factory: &Vehicle, ctx: &Context, problems: &mut Vec<String>) {
    for (value, was, road) in [
        (&mut v.handling_on_road, factory.handling_on_road, "on road"),
        (&mut v.handling_off_road, factory.handling_off_road, "off road"),
    ] {
        if *value < 1 {
            // Already flagged by the design if it started below 1.
            if was >= 1 {
                problems
                    .push(format!("Handling {road} is {value} with the mods: it can't be below 1. It's shown as 1."));
            }
            *value = 1;
        }
    }
    if v.speed_interval < 1 {
        problems.push(format!("Speed Interval is {} with the mods. It's shown as 1.", v.speed_interval));
        v.speed_interval = 1;
    }
    v.acceleration = v.acceleration.max(0);
    v.top_speed = v.top_speed.max(0);
    if ctx.is_drone() {
        v.seat = 0;
    }
}

/// The mod's line in the report, adding its price and Availability to the
/// totals.
fn line(w: &Worked, report: &mut ModsReport) -> ModLine {
    let total_slots = w.slots * f64::from(w.count);
    let mut label = w.def.name.clone();
    if w.def.rated() && w.def.effect.as_deref() != Some("gecko") {
        label = format!("{label} {}", w.rating);
    }
    if w.count > 1 {
        label = format!("{label} ×{}", w.count);
    }
    if !w.installed.note.trim().is_empty() {
        label = format!("{label} ({})", w.installed.note.trim());
    }

    if let Some((n, code)) = &w.availability {
        if *n > report.availability {
            report.availability = *n;
        }
        // Illegal (I) outranks Licensed (L).
        if let Some(code) = code
            && (report.legality.is_none() || code == "I")
        {
            report.legality = Some(code.clone());
        }
    }

    let (build_points, diy_price, install_test) = if w.installed.built_in {
        let points = (2.0 * total_slots.max(0.0)).ceil() as i32;
        report.built_in_points += points;
        report.built_in_cost += w.price;
        (points, 0, "built in".to_string())
    } else {
        let diy = if w.def.is_accessory() {
            w.price
        } else if let Some(fixed) = w.def.diy_fixed {
            // Only the part past the fixed price is halved (a portable
            // facility's 50,000¥ is the facility itself, p. 129).
            fixed + (w.price - fixed) / 2
        } else {
            w.price / 2
        };
        report.aftermarket_cost += w.price;
        report.aftermarket_diy_cost += diy;
        (0, diy, install_test(w.slots))
    };

    ModLine {
        id: w.def.id.clone(),
        label,
        slots: total_slots,
        cf: w.cf * f64::from(w.count),
        availability: availability_text(&w.availability),
        price: w.price,
        diy_price,
        built_in: w.installed.built_in,
        build_points,
        install_test,
    }
}

/// Installing a mod yourself (p. 120): Engineering + Logic, threshold mod
/// slots × 8, interval mod slots × hours; a mod without slots counts as 1.
/// A toolkit does under 2 slots, a shop 2 to 5, a facility 6 or more.
pub fn install_test(slots: f64) -> String {
    let n = if slots > 0.0 { slots } else { 1.0 };
    let threshold = (n * 8.0).ceil();
    let hours = if n == 1.0 { "1 hour".to_string() } else { format!("{} hours", number(n)) };
    let tools = if n < 2.0 {
        "toolkit"
    } else if n < 6.0 {
        "shop"
    } else {
        "facility"
    };
    format!("Engineering + Logic ({threshold}, {hours}), {tools}")
}

/// Mod slots used past each category's, covered first by universal slots
/// and then by spare slots elsewhere at 2 for 1 (p. 120).
fn check_slots(report: &ModsReport, problems: &mut Vec<String>) {
    let over: f64 = report.slots.iter().map(|s| (s.used - f64::from(s.available)).max(0.0)).sum();
    let spare: f64 = report.slots.iter().map(|s| (f64::from(s.available) - s.used).max(0.0)).sum();
    let short = (over - f64::from(report.universal_slots)).max(0.0);
    if short > 0.0 && 2.0 * short > spare {
        let missing = short - spare / 2.0;
        problems.push(format!(
            "The mods need {} more mod slots than the vehicle has, after universal slots and shifting spare \
             slots at 2 for 1.",
            number(missing)
        ));
    }
}

/// Free hardpoints, in standard hardpoints (p. 123): one per 3 full points
/// of Body; a small drone over Body 3 gets one small one instead.
fn free_hardpoints(ctx: &Context, body: i32) -> f64 {
    match ctx.chassis.drone_size {
        Some(DroneSize::Small) => {
            if body > 3 {
                0.5
            } else {
                0.0
            }
        }
        _ => f64::from(body.max(0) / 3),
    }
}

/// A count as the book writes it: whole, or with its fraction (1.5).
pub fn number(n: f64) -> String {
    if n.fract() == 0.0 { format!("{n:.0}") } else { format!("{n}") }
}
