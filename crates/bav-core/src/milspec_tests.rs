//! Tests for Deadly Arts: its mil-spec design options (pp. 150–152), its
//! mods, and turning the book on and off. Each test is named after the rule
//! it checks.

use crate::block::{self, BlockLine};
use crate::catalog::Catalog;
use crate::design::{Design, InstalledMod, MilSpec, MilspecHardpoint, MountKind, Weapon};
use crate::files;
use crate::milspec::{grade, minimum_body};
use crate::{Report, evaluate};

fn run(d: &Design) -> Report {
    evaluate(d, Catalog::builtin()).expect("the design's ids are in the catalog")
}

/// A truck (Body 14, Armor 12) with Deadly Arts on.
fn truck() -> Design {
    Design {
        chassis: "truck".into(),
        powertrain: "wheeled".into(),
        console: "basic".into(),
        books: vec!["deadly-arts".into()],
        ..Design::default()
    }
}

/// The truck with these mil-spec options.
fn with(milspec: MilSpec) -> Design {
    Design { milspec, ..truck() }
}

fn hardpoint(size: i32) -> MilspecHardpoint {
    MilspecHardpoint { size, ..MilspecHardpoint::default() }
}

fn points(r: &Report, label: &str) -> Option<i32> {
    r.build_points.iter().find(|l| l.label.starts_with(label)).map(|l| l.points)
}

fn has_problem(r: &Report, start: &str) -> bool {
    r.problems.iter().any(|p| p.starts_with(start))
}

fn mods(d: Design, ids: &[(&str, i32)]) -> Design {
    let mods =
        ids.iter().map(|&(id, rating)| InstalledMod { id: id.into(), rating, ..InstalledMod::default() }).collect();
    Design { mods, ..d }
}

// Turning the book on and off.

#[test]
fn deadly_arts_is_off_unless_the_design_turns_it_on() {
    assert!(Design::default().books.is_empty());
    let catalog = Catalog::builtin();
    assert!(catalog.book.iter().any(|b| b.id == "deadly-arts" && b.name == "Deadly Arts"));
}

#[test]
fn with_deadly_arts_off_its_options_are_left_out_and_flagged() {
    let mut d = with(MilSpec { frame: 4, comms: 1, ..MilSpec::default() });
    d.books.clear();
    let r = run(&d);
    assert!(has_problem(&r, "Deadly Arts is off"), "{:?}", r.problems);
    assert_eq!(points(&r, "Mil-spec"), None);
    assert_eq!(r.design_test.build_threshold, 20);
}

#[test]
fn with_deadly_arts_off_its_mods_are_left_out_and_flagged() {
    let mut d = mods(truck(), &[("mds", 0)]);
    d.books.clear();
    let r = run(&d);
    assert!(has_problem(&r, "MDS (Missile Defense System) is from Deadly Arts, which is off"), "{:?}", r.problems);
    assert!(r.mods.lines.is_empty());
    assert!(r.block.lines.iter().all(|l| l.label != "Standard Equipment"));
}

#[test]
fn a_design_file_keeps_its_books_and_milspec_options() {
    let d = with(MilSpec { frame: 2, hardpoints: vec![hardpoint(1)], ..MilSpec::default() });
    assert_eq!(files::from_text(&files::to_text(&d)).unwrap(), d);
    let older = files::from_text("format = 1\nchassis = \"car\"\n").unwrap();
    assert!(older.books.is_empty() && older.milspec.is_empty());
}

// The design options.

#[test]
fn design_points_are_build_points() {
    // Mil-spec comms I is 5 Design Points: 5 more Build Points, 5,000¥ more.
    let plain = run(&truck());
    let r = run(&with(MilSpec { comms: 1, ..MilSpec::default() }));
    assert_eq!(r.base_points, plain.base_points + 5);
    assert_eq!(r.cost, plain.cost + 5_000);
}

#[test]
fn mil_spec_comms_cost_5_20_or_30() {
    for (level, cost) in [(1, 5), (2, 20), (3, 30)] {
        assert_eq!(points(&run(&with(MilSpec { comms: level, ..MilSpec::default() })), "Mil-spec comms"), Some(cost));
    }
    assert!(has_problem(&run(&with(MilSpec { comms: 4, ..MilSpec::default() })), "Mil-spec comms: I, II, or III"));
}

#[test]
fn a_mil_spec_frame_costs_10_plus_5_a_rating_and_adds_body_but_not_slots() {
    let plain = run(&truck());
    let r = run(&with(MilSpec { frame: 6, ..MilSpec::default() }));
    assert_eq!(points(&r, "Mil-spec frame 6"), Some(40));
    assert_eq!(r.stats.body, plain.stats.body + 6);
    assert_eq!(r.stats.mod_slots.chassis, plain.stats.mod_slots.chassis);
    assert_eq!(r.factory_stat_line.values[4], (plain.stats.body + 6).to_string());
    assert!(r.problems.is_empty(), "{:?}", r.problems);
}

#[test]
fn a_mil_spec_frame_adds_at_most_half_the_body() {
    // Body 14: at most 7.
    let r = run(&with(MilSpec { frame: 8, ..MilSpec::default() }));
    assert!(has_problem(&r, "Mil-spec frame: rating 1 to 7"), "{:?}", r.problems);
    assert_eq!(points(&r, "Mil-spec frame 7"), Some(45));
}

#[test]
fn a_mil_spec_frame_and_increased_structural_integrity_dont_go_together() {
    let d = mods(with(MilSpec { frame: 2, ..MilSpec::default() }), &[("structural-integrity", 2)]);
    assert!(has_problem(&run(&d), "A mil-spec frame and Increased Structural Integrity"));
}

#[test]
fn heavy_mil_spec_plate_costs_5_a_rating_and_5_speed_interval_a_rating() {
    let plain = run(&truck());
    let r = run(&with(MilSpec { heavy_plate: 5, ..MilSpec::default() }));
    assert_eq!(points(&r, "Heavy mil-spec plate 5"), Some(25));
    assert_eq!(r.stats.speed_interval, (plain.stats.speed_interval - 25).max(1));
    assert!(has_problem(
        &run(&with(MilSpec { heavy_plate: 3, ..MilSpec::default() })),
        "Heavy mil-spec plate: rating 4 to 6"
    ));
}

#[test]
fn every_2_points_of_frame_offset_a_level_of_mil_spec_plate() {
    let plain = run(&truck());
    // Frame 6 offsets 3 of heavy plate 4's levels: 5 Speed Interval lost.
    let r = run(&with(MilSpec { heavy_plate: 4, frame: 6, ..MilSpec::default() }));
    assert_eq!(r.stats.speed_interval, plain.stats.speed_interval - 5);
    // Double Clutch's milspec plate too: frame 4 offsets rating 2 entirely.
    let d = mods(with(MilSpec { frame: 4, ..MilSpec::default() }), &[("milspec-plate", 2)]);
    assert_eq!(run(&d).stats.speed_interval, plain.stats.speed_interval);
}

#[test]
fn airframe_ceramic_plate_costs_2_a_point_of_armor() {
    let r = run(&with(MilSpec { airframe_plate: true, ..MilSpec::default() }));
    assert_eq!(points(&r, "Airframe ceramic plate"), Some(24)); // Armor 12
    assert!(r.notes.iter().any(|n| n.starts_with("Airframe ceramic plate: half the Armor (6)")));
}

#[test]
fn only_one_kind_of_specialty_armor() {
    let both = with(MilSpec { airframe_plate: true, heavy_plate: 4, ..MilSpec::default() });
    assert!(has_problem(&run(&both), "Only one kind of specialty armor"));
    let with_mod = mods(with(MilSpec { heavy_plate: 4, ..MilSpec::default() }), &[("ceramic-plate", 0)]);
    assert!(has_problem(&run(&with_mod), "Only one kind of specialty armor"));
    // Double Clutch alone doesn't have the rule.
    let mut dc = mods(truck(), &[("ceramic-plate", 0), ("milspec-plate", 1)]);
    dc.books.clear();
    assert!(!has_problem(&run(&dc), "Only one kind of specialty armor"));
}

#[test]
fn a_hardpoints_grade_is_the_weapons_size_changed_by_its_mounting() {
    assert_eq!(grade(&hardpoint(3)), 3);
    assert_eq!(grade(&MilspecHardpoint { mount: MountKind::Inline, ..hardpoint(3) }), 1);
    assert_eq!(grade(&MilspecHardpoint { mount: MountKind::Open, ..hardpoint(1) }), 0);
    assert_eq!(grade(&MilspecHardpoint { pop_out: true, linked: 2, ..hardpoint(3) }), 5);
    assert_eq!(grade(&MilspecHardpoint { linked: 4, ..hardpoint(3) }), 5);
}

#[test]
fn a_hardpoint_needs_body_5_plus_5_a_grade_or_a_grade_less_for_a_primary_weapon() {
    assert_eq!(minimum_body(3, false), 20);
    assert_eq!(minimum_body(3, true), 15);
    // Body 14 carries Grade 1 (Body 10), not Grade 2 (15) unless primary.
    assert!(run(&with(MilSpec { hardpoints: vec![hardpoint(1)], ..MilSpec::default() })).problems.is_empty());
    let r = run(&with(MilSpec { hardpoints: vec![hardpoint(2)], ..MilSpec::default() }));
    assert!(has_problem(&r, "A Grade 2 mil-spec hardpoint needs Body 15"), "{:?}", r.problems);
    let primary = MilspecHardpoint { primary: true, ..hardpoint(2) };
    assert!(run(&with(MilSpec { hardpoints: vec![primary], ..MilSpec::default() })).problems.is_empty());
}

#[test]
fn a_hardpoint_costs_5_a_grade_plus_pop_out_and_linking() {
    let hp = MilspecHardpoint { pop_out: true, linked: 2, ..hardpoint(1) }; // Grade 3
    let mut d = with(MilSpec { hardpoints: vec![hp], ..MilSpec::default() });
    d.chassis = "ship".into(); // Body 20
    let r = run(&d);
    assert_eq!(points(&r, "Mil-spec hardpoint, Grade 3"), Some(15 + 15 + 3));
    let quad = MilspecHardpoint { linked: 4, ..hardpoint(0) }; // Grade 2
    d.milspec.hardpoints = vec![quad];
    assert_eq!(points(&run(&d), "Mil-spec hardpoint, Grade 2"), Some(10 + 5));
}

#[test]
fn pop_out_isnt_for_an_inline_mount() {
    let hp = MilspecHardpoint { pop_out: true, mount: MountKind::Inline, ..hardpoint(2) };
    assert!(has_problem(&run(&with(MilSpec { hardpoints: vec![hp], ..MilSpec::default() })), "Pop-out goes on"));
}

#[test]
fn any_mil_spec_option_makes_it_military_plus_8_to_build() {
    assert_eq!(run(&truck()).design_test.build_threshold, 20);
    let r = run(&with(MilSpec { comms: 1, ..MilSpec::default() }));
    assert_eq!(r.design_test.build_threshold, 28);
    assert!(r.notes.iter().any(|n| n.starts_with("Military vehicle")));
}

// Its mods.

#[test]
fn the_mds_takes_2_electronics_slots() {
    let r = run(&mods(truck(), &[("mds", 0)]));
    assert_eq!(r.mods.slots[2].used, 2.0);
    assert_eq!((r.mods.lines[0].availability.as_str(), r.mods.lines[0].price), ("6(L)", 15_000));
}

#[test]
fn a_mobile_magic_lodge_takes_50_cf() {
    let r = run(&mods(truck(), &[("mobile-magic-lodge", 0)]));
    assert_eq!(r.mods.cargo_used, 50.0);
    assert_eq!(r.mods.lines[0].price, 80_000);
}

// Weapons on mil-spec hardpoints, and the stat block.

fn weapon(mount: &str) -> Weapon {
    Weapon {
        name: "Railgun".into(),
        weapon_type: "Linear Kinetics".into(),
        dv: "26P".into(),
        mode: "SS".into(),
        ar: vec![0, 8, 10, 12, 14],
        ammo: "200(br)".into(),
        mount: mount.into(),
        ..Weapon::default()
    }
}

#[test]
fn a_weapon_on_a_mil_spec_hardpoint_keeps_its_own_ammunition_and_attack_ratings() {
    assert_eq!(
        block::weapon_line(&weapon("milspec-1")),
        "Railgun [Linear Kinetics, DV 26P, SS, 200(br), —/8/10/12/14] (mil-spec hardpoint 1)"
    );
}

#[test]
fn a_mil_spec_hardpoint_holds_one_weapon_or_its_linked_ones() {
    let mut d = with(MilSpec { hardpoints: vec![hardpoint(1)], ..MilSpec::default() });
    d.weapons = vec![weapon("milspec-1"), weapon("milspec-1")];
    assert!(has_problem(&run(&d), "2 weapons are on mil-spec hardpoint 1, which holds 1"));
    d.milspec.hardpoints[0].linked = 2;
    d.milspec.hardpoints[0].size = 0; // Grade 1 with twin-linking
    let r = run(&d);
    assert!(r.problems.is_empty(), "{:?}", r.problems);
    d.weapons = vec![weapon("milspec-2")];
    assert!(has_problem(&run(&d), "Railgun is on mil-spec hardpoint 2, which the vehicle doesn't have"));
}

#[test]
fn mil_spec_options_are_listed_as_equipment() {
    let hp = MilspecHardpoint { pop_out: true, note: "roof".into(), ..hardpoint(0) };
    let r = run(&with(MilSpec { frame: 4, comms: 2, hardpoints: vec![hp], ..MilSpec::default() }));
    let line: &BlockLine = r.block.lines.iter().find(|l| l.label == "Standard Equipment").unwrap();
    assert_eq!(
        line.entries,
        ["Mil-spec comms II", "mil-spec frame 4", "mil-spec hardpoint (Grade 1, pop-out turret, roof)"]
    );
}
