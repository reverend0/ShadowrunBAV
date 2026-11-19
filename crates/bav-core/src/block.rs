//! The vehicle's stat block, laid out as ShadowrunEdit's `sr6-vehicle`
//! block is: the title and type, the main row, a gamemaster's derived row,
//! then labeled lines (Standard Equipment or Equipment for what's built in,
//! Modifications for what's installed afterward, and Weapons). The
//! report shows it, Copy pastes it into ShadowrunEdit, and the PDF prints it.

use serde::Serialize;

use crate::calc::Report;
use crate::catalog::{Catalog, Chassis, Kind};
use crate::design::{Design, Layout, Weapon};
use crate::milspec::{self, Outcome};

#[derive(Debug, Clone, Default, Serialize)]
pub struct StatBlock {
    pub title: String,
    /// The type, for the title's parentheses ("car", "medium drone").
    pub detail: String,
    pub main: Row,
    /// A gamemaster's DR, I/ID, AC, CM row.
    pub derived: Option<Row>,
    pub lines: Vec<BlockLine>,
    /// The whole block as text, for Copy.
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Row {
    pub headers: Vec<String>,
    pub values: Vec<String>,
}

/// A labeled line: "Standard Equipment: …", or Weapons, one per line.
#[derive(Debug, Clone, Serialize)]
pub struct BlockLine {
    pub label: String,
    pub entries: Vec<String>,
    /// Each entry on a line of its own under the label (weapons).
    pub one_per_line: bool,
}

/// The stat block for a design and its report.
pub fn build(design: &Design, report: &Report, catalog: &Catalog, chassis: &Chassis) -> StatBlock {
    let gear = design.layout == Layout::Gear;
    let is_drone = chassis.kind == Kind::Drone;
    let s = &report.stats;

    // The main row: the stat line, without Avail and Cost for a gamemaster.
    let mut main = Row { headers: report.stat_line.headers.clone(), values: report.stat_line.values.clone() };
    if !gear {
        main.headers.truncate(9);
        main.values.truncate(9);
    }

    // A gamemaster's row (the core rules; as ShadowrunEdit works it out): DR
    // `+Armor (Armor + Pilot)`, a drone's own Initiative Pilot × 2 with 4D6
    // (a vehicle uses its driver's), and one Condition Monitor, Body ÷ 2
    // rounded up, + 8.
    let derived = (!gear).then(|| Row {
        headers: ["DR", "I/ID", "AC", "CM"].map(String::from).to_vec(),
        values: vec![
            format!("+{} ({})", s.armor, s.armor + s.pilot),
            if is_drone { format!("{}/4", s.pilot * 2) } else { "—".into() },
            if is_drone { "A1, I5".into() } else { "—".into() },
            ((s.body + 1) / 2 + 8).to_string(),
        ],
    });

    let mut lines = Vec::new();
    // What's built into the design is its standard equipment; mods
    // installed afterward are its modifications (decided), as ShadowrunEdit's
    // vehicle block has both lines.
    let equipment = equipment_entries(design, catalog, &report.milspec, true);
    if !equipment.is_empty() {
        let label = if gear { "Standard Equipment" } else { "Equipment" };
        lines.push(BlockLine { label: label.into(), entries: equipment, one_per_line: false });
    }
    let modifications = equipment_entries(design, catalog, &report.milspec, false);
    if !modifications.is_empty() {
        lines.push(BlockLine { label: "Modifications".into(), entries: modifications, one_per_line: false });
    }
    let mut weapons: Vec<String> = design.weapons.iter().map(weapon_line).collect();
    weapons.sort_by_key(|w| w.to_lowercase());
    if !weapons.is_empty() {
        lines.push(BlockLine { label: "Weapons".into(), entries: weapons, one_per_line: true });
    }

    let mut detail = vehicle_type(chassis);
    if !design.specialized_function.trim().is_empty() {
        detail = format!("{detail}, {}", design.specialized_function.trim());
    }
    let title = if design.name.trim().is_empty() { "Untitled vehicle" } else { design.name.trim() };
    let mut block = StatBlock { title: title.into(), detail, main, derived, lines, text: String::new() };
    block.text = text(&block);
    block
}

/// The mods as a stat block lists them: in name order, in lowercase but for
/// the first (and acronyms and names), with a rating after and a count or
/// note in parentheses: "Realistic features 4, standard weapon mount (2)".
/// `built_in` picks the mods built into the design (with Deadly Arts'
/// mil-spec options, when it's on) or those installed afterward.
fn equipment_entries(design: &Design, catalog: &Catalog, milspec: &Outcome, built_in: bool) -> Vec<String> {
    let mut entries: Vec<String> = design
        .mods
        .iter()
        .filter(|inst| inst.built_in == built_in)
        .filter_map(|inst| {
            let def = catalog.vehicle_mod(&inst.id)?;
            // A mod from a book the design has turned off is left out.
            if def.book.as_ref().is_some_and(|book| !design.books.contains(book)) {
                return None;
            }
            let mut entry = lowercase_words(&def.name);
            if def.rated() && def.effect.as_deref() != Some("gecko") {
                entry = format!("{entry} {}", inst.rating);
            }
            let mut extra = Vec::new();
            if def.many && inst.count > 1 {
                extra.push(inst.count.to_string());
            }
            if !inst.note.trim().is_empty() {
                extra.push(inst.note.trim().to_string());
            }
            if !extra.is_empty() {
                entry = format!("{entry} ({})", extra.join(", "));
            }
            Some(entry)
        })
        .collect();
    if built_in && milspec.military {
        entries.extend(milspec_entries(design, milspec));
    }
    entries.sort_by_key(|e| e.to_lowercase());
    if let Some(first) = entries.first_mut() {
        *first = capitalize(first);
    }
    entries
}

/// Deadly Arts' mil-spec options as equipment: "mil-spec frame 6",
/// "mil-spec hardpoint (Grade 3, pop-out turret, twin-linked)".
fn milspec_entries(design: &Design, milspec: &Outcome) -> Vec<String> {
    let ms = &design.milspec;
    let mut entries = Vec::new();
    if ms.airframe_plate {
        entries.push("airframe ceramic plate".to_string());
    }
    if ms.heavy_plate != 0 {
        entries.push(format!("heavy mil-spec plate {}", ms.heavy_plate.clamp(4, 6)));
    }
    if (1..=3).contains(&ms.comms) {
        entries.push(format!("mil-spec comms {}", ["I", "II", "III"][(ms.comms - 1) as usize]));
    }
    if milspec.frame_body > 0 {
        entries.push(format!("mil-spec frame {}", milspec.frame_body));
    }
    // `zip` pairs each hardpoint with its Grade, worked out in the same order.
    for (hp, grade) in ms.hardpoints.iter().zip(&milspec.hardpoints) {
        let mut parts = vec![format!("Grade {grade}")];
        parts.extend(milspec::mounting(hp));
        entries.push(format!("mil-spec hardpoint ({})", parts.join(", ")));
    }
    entries
}

/// Words capitalized only because they're in a name, in lowercase:
/// "Standard Weapon Mount" → "standard weapon mount", keeping acronyms and
/// names with other capitals (RAM, ECM, C3, GridLink, xGuide), and Valkyrie.
pub(crate) fn lowercase_words(name: &str) -> String {
    name.split_inclusive([' ', '-'])
        .map(|word| {
            let letters: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
            let other_capitals = letters.chars().skip(1).any(|c| c.is_uppercase());
            let starts_lower = letters.chars().next().is_some_and(|c| c.is_lowercase());
            if other_capitals || starts_lower || letters.starts_with("Valkyrie") {
                word.to_string()
            } else {
                word.to_lowercase()
            }
        })
        .collect()
}

fn capitalize(text: &str) -> String {
    // `chars()` gives the first character; the rest is the &str after its
    // byte length, since a character can be more than one byte.
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// A weapon mount's size and turret, by its mod id: its ammunition
/// (p. 142) and a short name for the weapon's line.
pub fn mount(id: &str) -> Option<(&'static str, &'static str)> {
    Some(match id {
        "small-weapon-mount" => ("100(c)", "small mount"),
        "small-turreted-weapon-mount" => ("100(c)", "small turret"),
        "standard-weapon-mount" => ("250(c)", "standard mount"),
        "standard-turreted-weapon-mount" => ("250(c)", "standard turret"),
        "large-weapon-mount" => ("500(b)", "large mount"),
        "large-turreted-weapon-mount" => ("500(b)", "large turret"),
        _ => return None,
    })
}

/// A mil-spec hardpoint's number from a weapon's mount ("milspec-2" → 2).
pub fn milspec_mount(id: &str) -> Option<usize> {
    // `strip_prefix` gives the rest of the text after the prefix, if it
    // starts with it; `parse` then reads the number, `ok()` dropping the error.
    id.strip_prefix("milspec-")?.parse().ok().filter(|n| *n > 0)
}

/// A weapon's line as ShadowrunEdit writes one, with the mount's
/// ammunition and its +2 to every Attack Rating in range (p. 142):
/// `Ares Alpha [Assault Rifles, DV 4P, SA/BF/FA, 250(c), 5/12/11/8/4, w/ smartgun system] (standard turret)`.
/// On a mil-spec hardpoint (Deadly Arts) it keeps its own ammunition and
/// Attack Ratings.
pub fn weapon_line(w: &Weapon) -> String {
    let mut parts = Vec::new();
    let mut push = |text: &str| {
        if !text.trim().is_empty() {
            parts.push(text.trim().to_string());
        }
    };
    push(&w.weapon_type);
    if !w.dv.trim().is_empty() {
        push(&format!("DV {}", w.dv.trim()));
    }
    push(&w.mode);
    let mount = mount(&w.mount);
    let (place, bonus) = match (mount, milspec_mount(&w.mount)) {
        (Some((ammo, place)), _) => {
            push(ammo);
            (Some(place.to_string()), 2)
        }
        (None, Some(n)) => {
            push(&w.ammo);
            (Some(format!("mil-spec hardpoint {n}")), 0)
        }
        (None, None) => {
            push(&w.ammo);
            (None, 0)
        }
    };
    let ratings: Vec<String> = (0..5)
        .map(|i| w.ar.get(i).copied().unwrap_or(0))
        .map(|ar| if ar > 0 { (ar + bonus).to_string() } else { "—".into() })
        .collect();
    push(&ratings.join("/"));
    if !w.mods.trim().is_empty() {
        push(&format!("w/ {}", w.mods.trim()));
    }
    let name = if w.name.trim().is_empty() { "Unnamed weapon" } else { w.name.trim() };
    match place {
        Some(place) => format!("{name} [{}] ({place})", parts.join(", ")),
        None => format!("{name} [{}]", parts.join(", ")),
    }
}

/// Weapons need a weapon mount, and a mount holds one weapon (p. 142). A
/// mil-spec hardpoint (Deadly Arts) holds one, or 2 or 4 linked.
pub fn check_weapons(design: &Design, milspec: &Outcome, problems: &mut Vec<String>) {
    for w in &design.weapons {
        let name = if w.name.trim().is_empty() { "A weapon" } else { w.name.trim() };
        match milspec_mount(&w.mount) {
            Some(n) if n > milspec.hardpoints.len() => {
                problems.push(format!("{name} is on mil-spec hardpoint {n}, which the vehicle doesn't have."));
            }
            Some(_) => {}
            None if mount(&w.mount).is_none() => {
                problems.push(format!("{name} needs a weapon mount: a vehicle's weapons are fitted in one (p. 142)."));
            }
            None => {}
        }
    }
    for id in design.weapons.iter().map(|w| w.mount.as_str()).filter(|id| mount(id).is_some()) {
        let weapons = design.weapons.iter().filter(|w| w.mount == id).count() as i32;
        let mounts: i32 = design.mods.iter().filter(|m| m.id == id).map(|m| m.count.max(1)).sum();
        if weapons > mounts {
            let place = mount(id).map_or("", |(_, place)| place);
            let problem = format!(
                "{weapons} weapons are in a {place}, and the vehicle has {mounts}: add a {}.",
                place.replace("mount", "weapon mount").replace("turret", "turreted weapon mount")
            );
            if !problems.contains(&problem) {
                problems.push(problem);
            }
        }
    }
    // `enumerate` counts from 0; hardpoints are numbered from 1.
    for (i, hp) in design.milspec.hardpoints.iter().enumerate().take(milspec.hardpoints.len()) {
        let n = i + 1;
        let weapons = design.weapons.iter().filter(|w| milspec_mount(&w.mount) == Some(n)).count() as i32;
        let holds = if [2, 4].contains(&hp.linked) { hp.linked } else { 1 };
        if weapons > holds {
            problems.push(format!("{weapons} weapons are on mil-spec hardpoint {n}, which holds {holds}."));
        }
    }
}

/// The type in a stat block's parentheses: the chassis in lowercase, keeping
/// acronyms ("van/SUV", "LTA (lighter than air)"), and "medium drone" for
/// "Drone, Medium".
pub fn vehicle_type(chassis: &Chassis) -> String {
    if chassis.kind == Kind::Drone
        && let Some((_, size)) = chassis.name.split_once(", ")
    {
        return format!("{} drone", size.to_lowercase());
    }
    // `split_inclusive` keeps each separator with the word before it, so
    // joining the pieces back gives the original spacing.
    chassis
        .name
        .split_inclusive([' ', '/'])
        .map(|word| {
            let letters: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
            let acronym = letters.len() > 1 && letters.iter().all(|c| c.is_uppercase());
            if acronym { word.to_string() } else { word.to_lowercase() }
        })
        .collect()
}

/// The block as text to paste into ShadowrunEdit (or anywhere): the title,
/// tab-separated rows, and the labeled lines.
pub fn text(block: &StatBlock) -> String {
    let mut out = format!("{} ({})\n", block.title, block.detail);
    for row in std::iter::once(&block.main).chain(block.derived.as_ref()) {
        out.push_str(&row.headers.join("\t"));
        out.push('\n');
        out.push_str(&row.values.join("\t"));
        out.push('\n');
    }
    for line in &block.lines {
        if line.one_per_line {
            out.push_str(&format!("{}:\n", line.label));
            for entry in &line.entries {
                out.push_str(&format!("\t{entry}\n"));
            }
        } else {
            out.push_str(&format!("{}: {}\n", line.label, line.entries.join(", ")));
        }
    }
    out
}
