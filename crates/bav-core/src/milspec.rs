//! Deadly Arts' mil-spec design options ("Mil-Spec Mayhem", pp. 150–152),
//! an expansion of Double Clutch's design options: airframe ceramic plate,
//! heavy mil-spec plate, mil-spec comms, a mil-spec frame, and mil-spec
//! hardpoints. The book prices them in Design Points, which are Build Points
//! (RULINGS.md R21).
//!
//! They're offered only when a design turns Deadly Arts on.

use serde::Serialize;

use crate::calc::Line;
use crate::design::{MilSpec, MilspecHardpoint, MountKind};

/// Deadly Arts' id, as `data/design.toml` lists it.
pub const BOOK: &str = "deadly-arts";

/// What the mil-spec options add to a design.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Outcome {
    /// Their Build Points, a line each.
    pub lines: Vec<Line>,
    /// The frame's rating: added to Body, as Increased Structural Integrity
    /// is, without adding mod slots.
    pub frame_body: i32,
    /// Levels of mil-spec plate whose Speed Interval loss the frame offsets
    /// (one per 2 points of frame).
    pub plate_offset: i32,
    /// Speed Interval lost to heavy mil-spec plate, after the frame's offset.
    pub speed_interval_loss: i32,
    /// Each mil-spec hardpoint's Grade, for the weapons fitted to it.
    pub hardpoints: Vec<i32>,
    /// Any mil-spec option makes it a military vehicle: +8 to the build
    /// test's threshold (decided).
    pub military: bool,
}

/// The comms' names and Build Points: I, II, III.
const COMMS: [(&str, i32); 3] = [("I", 5), ("II", 20), ("III", 30)];

/// A mil-spec hardpoint's Grade: the weapon's Size, −2 inline or open, +1
/// pop-out, +1 twin-linked or +2 quad-linked, and never below 0.
pub fn grade(hp: &MilspecHardpoint) -> i32 {
    let mount = match hp.mount {
        MountKind::Turreted => 0,
        MountKind::Inline | MountKind::Open => -2,
    };
    let linked = match hp.linked {
        2 => 1,
        4 => 2,
        _ => 0,
    };
    (hp.size + mount + i32::from(hp.pop_out) + linked).max(0)
}

/// The smallest Body that carries a hardpoint of this Grade: 5 + 5 × Grade;
/// a dedicated primary weapon's is a Grade smaller's (p. 151).
pub fn minimum_body(grade: i32, primary: bool) -> i32 {
    let grade = if primary { (grade - 1).max(0) } else { grade };
    5 + 5 * grade
}

/// A hardpoint's Build Points: 5 × Grade, another 5 × Grade for pop-out,
/// and +3 twin-linked or +5 quad-linked.
fn hardpoint_points(hp: &MilspecHardpoint, grade: i32) -> i32 {
    let pop_out = if hp.pop_out { 5 * grade } else { 0 };
    let linked = match hp.linked {
        2 => 3,
        4 => 5,
        _ => 0,
    };
    5 * grade + pop_out + linked
}

/// How a hardpoint is mounted, for its lines: "pop-out turret",
/// "twin-linked", and its note.
pub fn mounting(hp: &MilspecHardpoint) -> Vec<String> {
    let mount = match hp.mount {
        MountKind::Turreted => "turret",
        MountKind::Inline => "inline",
        MountKind::Open => "open",
    };
    let mut parts = vec![if hp.pop_out { format!("pop-out {mount}") } else { mount.into() }];
    match hp.linked {
        2 => parts.push("twin-linked".into()),
        4 => parts.push("quad-linked".into()),
        _ => {}
    }
    if !hp.note.trim().is_empty() {
        parts.push(hp.note.trim().into());
    }
    parts
}

/// The hardpoint as its Build Points line names it:
/// "Mil-spec hardpoint, Grade 3 (Size 2, pop-out turret, twin-linked)".
fn hardpoint_label(hp: &MilspecHardpoint, grade: i32) -> String {
    let mut parts = vec![format!("Size {}", hp.size)];
    if hp.primary {
        parts.push("primary weapon".into());
    }
    parts.extend(mounting(hp));
    format!("Mil-spec hardpoint, Grade {grade} ({})", parts.join(", "))
}

/// Works out the mil-spec options on a design whose Body and Armor (as
/// designed, before mods) are given. Rules they break go to `problems`, and
/// what comes with them to `notes`.
pub fn work_out(ms: &MilSpec, body: i32, armor: i32, problems: &mut Vec<String>, notes: &mut Vec<String>) -> Outcome {
    let mut out = Outcome { military: !ms.is_empty(), ..Outcome::default() };

    // Mil-spec frame (1–12): 10 + 5 × rating, up to half the Body again.
    if ms.frame != 0 {
        let most = (body / 2).min(12);
        let frame = ms.frame.clamp(0, most.max(0));
        if ms.frame < 1 || ms.frame > most {
            problems.push(format!(
                "Mil-spec frame: rating 1 to {most} here (at most half the Body, {body}, and 12); it's {}.",
                ms.frame
            ));
        }
        if frame > 0 {
            out.lines.push(Line { label: format!("Mil-spec frame {frame}"), points: 10 + 5 * frame });
            out.frame_body = frame;
            out.plate_offset = frame / 2;
            notes.push(format!(
                "Mil-spec frame {frame}: Body +{frame} for resisting damage, and {} level{} of mil-spec plate \
                 without its Speed Interval loss.",
                frame / 2,
                if frame / 2 == 1 { "" } else { "s" }
            ));
        }
    }

    // Airframe ceramic plate: 2 × Armor.
    if ms.airframe_plate {
        out.lines.push(Line { label: format!("Airframe ceramic plate (Armor {armor})"), points: 2 * armor });
        notes.push(format!(
            "Airframe ceramic plate: half the Armor ({}) as bonus dice; damaged panels need factory \
             replacements or a nanomaintenance repair system.",
            armor / 2
        ));
    }

    // Heavy mil-spec plate (4–6): 5 × rating, and 5 Speed Interval a level
    // not offset by the frame.
    if ms.heavy_plate != 0 {
        let rating = ms.heavy_plate.clamp(4, 6);
        if rating != ms.heavy_plate {
            problems.push(format!("Heavy mil-spec plate: rating 4 to 6 (it's {}).", ms.heavy_plate));
        }
        out.lines.push(Line { label: format!("Heavy mil-spec plate {rating}"), points: 5 * rating });
        out.speed_interval_loss = 5 * (rating - out.plate_offset).max(0);
        notes.push(format!("Heavy mil-spec plate {rating}: incoming attacks' DV −{rating}."));
    }

    // Mil-spec comms I, II, III.
    if ms.comms != 0 {
        // `usize::try_from` fails for a negative number (a hand-edited
        // file), where `as usize` would wrap it into a huge index.
        match usize::try_from(ms.comms - 1).ok().and_then(|i| COMMS.get(i)) {
            Some((name, points)) => out.lines.push(Line { label: format!("Mil-spec comms {name}"), points: *points }),
            None => problems.push(format!("Mil-spec comms: I, II, or III (it's {}).", ms.comms)),
        }
    }

    // Mil-spec hardpoints.
    for hp in &ms.hardpoints {
        let grade = grade(hp);
        out.hardpoints.push(grade);
        out.lines.push(Line { label: hardpoint_label(hp, grade), points: hardpoint_points(hp, grade) });
        if !(0..=20).contains(&hp.size) {
            problems.push(format!("A mil-spec hardpoint's weapon Size is 0 to 20 (it's {}).", hp.size));
        }
        if hp.pop_out && hp.mount == MountKind::Inline {
            problems.push("Pop-out goes on a turreted or open mount, not an inline one.".into());
        }
        if ![1, 2, 4].contains(&hp.linked) {
            problems.push(format!(
                "A mil-spec hardpoint holds 1 weapon, 2 twin-linked, or 4 quad-linked (it's {}).",
                hp.linked
            ));
        }
        let need = minimum_body(grade, hp.primary);
        if body < need {
            problems.push(format!("A Grade {grade} mil-spec hardpoint needs Body {need} or more (this has {body})."));
        }
    }
    out
}
