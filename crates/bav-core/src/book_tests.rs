//! Tests for the smaller books: Smooth Operations' drone-mounted tag
//! sprayer, Explosive Decompression's Spaceworthy, and Sixth World
//! Companion's vehicle packs. Each test is named after the rule it checks.

use crate::catalog::Catalog;
use crate::design::{Design, InstalledMod};
use crate::{Report, evaluate};

fn run(d: &Design) -> Report {
    evaluate(d, Catalog::builtin()).expect("the design's ids are in the catalog")
}

fn design(chassis: &str, books: &[&str], mods: &[(&str, bool)]) -> Design {
    Design {
        chassis: chassis.into(),
        powertrain: "wheeled".into(),
        console: "basic".into(),
        books: books.iter().map(|b| b.to_string()).collect(),
        mods: mods
            .iter()
            .map(|&(id, built_in)| InstalledMod { id: id.into(), built_in, ..InstalledMod::default() })
            .collect(),
        ..Design::default()
    }
}

fn has_problem(r: &Report, start: &str) -> bool {
    r.problems.iter().any(|p| p.starts_with(start))
}

#[test]
fn each_book_can_be_turned_on() {
    let ids: Vec<&str> = Catalog::builtin().book.iter().map(|b| b.id.as_str()).collect();
    for id in ["deadly-arts", "smooth-operations", "explosive-decompression", "sixth-world-companion"] {
        assert!(ids.contains(&id), "{id}");
    }
}

#[test]
fn every_book_mod_and_pack_names_a_listed_book() {
    let catalog = Catalog::builtin();
    let listed = |id: &str| catalog.book.iter().any(|b| b.id == id);
    for m in catalog.mods.iter().filter_map(|m| m.book.as_deref()) {
        assert!(listed(m), "{m}");
    }
    for p in &catalog.pack {
        assert!(listed(&p.book), "{}", p.id);
    }
}

// Smooth Operations.

#[test]
fn the_drone_mounted_tag_sprayer_takes_a_chassis_slot_and_costs_1000() {
    let r = run(&design("drone-mini", &["smooth-operations"], &[("drone-tag-sprayer", false)]));
    assert!(r.problems.is_empty(), "{:?}", r.problems);
    let line = &r.mods.lines[0];
    assert_eq!((line.slots, line.availability.as_str(), line.price), (1.0, "—", 1_000));
    assert_eq!(r.mods.slots[0].used, 1.0);
    assert!(r.notes.iter().any(|n| n.contains("10,000 pixel tags")));
}

#[test]
fn a_microdrones_tag_sprayer_holds_1000_tags() {
    let r = run(&design("drone-micro", &["smooth-operations"], &[("drone-tag-sprayer", false)]));
    assert!(r.notes.iter().any(|n| n.contains("1,000 pixel tags")));
}

#[test]
fn the_tag_sprayer_is_for_microdrones_and_minidrones() {
    let r = run(&design("drone-small", &["smooth-operations"], &[("drone-tag-sprayer", false)]));
    assert!(has_problem(&r, "Drone-Mounted Tag Sprayer is only for microdrones and minidrones"));
}

#[test]
fn with_smooth_operations_off_the_tag_sprayer_is_left_out() {
    let r = run(&design("drone-mini", &[], &[("drone-tag-sprayer", false)]));
    assert!(has_problem(&r, "Drone-Mounted Tag Sprayer is from Smooth Operations, which is off"));
}

// Explosive Decompression.

#[test]
fn spaceworthy_costs_the_drones_price_so_it_ends_up_twice_as_much() {
    let plain = run(&design("drone-medium", &["explosive-decompression"], &[]));
    let r = run(&design("drone-medium", &["explosive-decompression"], &[("spaceworthy", false)]));
    assert!(r.problems.is_empty(), "{:?}", r.problems);
    assert_eq!(r.mods.lines[0].price, plain.cost);
    assert_eq!(r.mods.lines[0].diy_price, plain.cost);
    assert_eq!(r.total_cost, 2 * plain.cost);
    assert_eq!(r.mods.lines[0].availability, "4(L)");
}

#[test]
fn spaceworthy_built_in_doubles_the_drones_price() {
    let plain = run(&design("drone-medium", &["explosive-decompression"], &[]));
    let r = run(&design("drone-medium", &["explosive-decompression"], &[("spaceworthy", true)]));
    assert_eq!(r.cost, 2 * plain.cost);
    assert_eq!(r.mods.built_in_points, 0); // no slots
}

#[test]
fn spaceworthy_is_for_drones() {
    let r = run(&design("car", &["explosive-decompression"], &[("spaceworthy", false)]));
    assert!(has_problem(&r, "Spaceworthy is only for drones"));
}

// Sixth World Companion.

#[test]
fn every_packs_mods_are_in_the_catalog_with_ratings_in_range() {
    let catalog = Catalog::builtin();
    assert_eq!(catalog.pack.len(), 4);
    for pack in &catalog.pack {
        for inst in &pack.mods {
            let def = catalog.vehicle_mod(&inst.id).unwrap_or_else(|| panic!("{}: {}", pack.id, inst.id));
            if def.rated() {
                let (min, max) = (def.rating_min.unwrap_or(1), def.rating_max.unwrap_or(1));
                assert!((min..=max).contains(&inst.rating), "{}: {} {}", pack.id, inst.id, inst.rating);
            } else {
                assert_eq!(inst.rating, 0, "{}: {}", pack.id, inst.id);
            }
            assert!(inst.count == 1 || def.many, "{}: {}", pack.id, inst.id);
        }
    }
}

#[test]
fn a_packs_mods_fit_a_vehicle_its_size() {
    // Rigger's Baby needs Body 10: a car. Combat Biker, Body 4: a bike.
    let catalog = Catalog::builtin();
    for (pack, chassis) in
        [("riggers-baby", "car"), ("combat-biker", "bike"), ("dirty-tricks", "car"), ("gun-ports", "car")]
    {
        let pack = catalog.pack.iter().find(|p| p.id == pack).unwrap();
        let d = Design { mods: pack.mods.clone(), ..design(chassis, &["sixth-world-companion"], &[]) };
        let r = run(&d);
        assert_eq!(r.mods.lines.len(), pack.mods.len(), "{}", pack.id);
        assert!(
            !r.problems.iter().any(|p| p.contains(": rating") || p.contains("Unknown") || p.contains("only for")),
            "{}: {:?}",
            pack.id,
            r.problems
        );
    }
}

#[test]
fn a_pack_lists_its_bodys_minimum() {
    let catalog = Catalog::builtin();
    let min = |id: &str| catalog.pack.iter().find(|p| p.id == id).and_then(|p| p.min_body);
    assert_eq!((min("riggers-baby"), min("combat-biker"), min("gun-ports")), (Some(10), Some(4), None));
}
