//! Tests for the mod rules ("Making It Yours", pp. 120–144), one per rule,
//! named after it, with the book's examples where they add mods.

use crate::catalog::Catalog;
use crate::design::{Design, Improvements, InstalledMod};
use crate::files;
use crate::mods::install_test;
use crate::{Report, evaluate};

fn design(chassis: &str, powertrain: &str, console: &str) -> Design {
    Design { chassis: chassis.into(), powertrain: powertrain.into(), console: console.into(), ..Design::default() }
}

fn run(d: &Design) -> Report {
    evaluate(d, Catalog::builtin()).expect("the design's ids are in the catalog")
}

/// A design with these mods: (id, rating, count, built in).
fn with_installed(mut d: Design, mods: &[(&str, i32, i32, bool)]) -> Design {
    d.mods = mods
        .iter()
        .map(|&(id, rating, count, built_in)| InstalledMod {
            id: id.into(),
            rating,
            count,
            built_in,
            note: String::new(),
        })
        .collect();
    d
}

/// One of a mod, installed afterward, at its lowest rating.
fn with(d: Design, ids: &[&str]) -> Design {
    let mods: Vec<(&str, i32, i32, bool)> = ids.iter().map(|&id| (id, 0, 1, false)).collect();
    with_installed(d, &mods)
}

/// A car as designed (Body 10, Armor 4): Acceleration 11, Top Speed 160.
fn car() -> Design {
    design("car", "wheeled", "basic")
}

fn has_problem(r: &Report, start: &str) -> bool {
    r.problems.iter().any(|p| p.starts_with(start))
}

// The table.

#[test]
fn every_mod_loads_with_a_unique_id_and_sensible_ranges() {
    let c = Catalog::builtin();
    assert!(c.mods.len() > 90);
    for m in &c.mods {
        assert_eq!(c.mods.iter().filter(|other| other.id == m.id).count(), 1, "{}", m.id);
        if let (Some(min), Some(max)) = (m.rating_min, m.rating_max) {
            assert!(min <= max, "{}", m.id);
        }
        if !m.levels.is_empty() && m.effect.as_deref() != Some("gecko") {
            assert_eq!(m.levels.len() as i32, m.rating_max.unwrap(), "{}: a level per rating", m.id);
        }
        assert!(!(m.hardpoint.is_some() && m.provides.is_some()), "{}", m.id);
    }
}

#[test]
fn every_mod_calculates_on_a_car_and_a_drone() {
    for m in &Catalog::builtin().mods {
        for d in [car(), design("drone-medium", "wheeled", "basic")] {
            let rating = m.rating_max.unwrap_or(0).min(2);
            run(&with_installed(d, &[(&m.id, rating, 1, false)]));
        }
    }
}

#[test]
fn an_unknown_mod_is_left_out_and_flagged() {
    assert!(has_problem(&run(&with(car(), &["rocket-skates"])), "Unknown mod"));
}

// The book's examples.

#[test]
fn example_1_with_its_mods_prices_its_weapon_mounts() {
    // Example 1's accessories and two standard weapon mounts. The book's
    // 7,300¥ leaves the mounts out and prices run-flat racing tires at
    // 1,800¥; both decided otherwise: 4,000 + 1,500 + 1,500 + 2 × 2,500.
    let mut d = design("car", "wheeled", "intermediate");
    d.modifications = vec!["racing-engine".into()];
    d.improvements = Improvements { body: -1, speed_interval: 2, ..Improvements::default() };
    let d = with_installed(
        d,
        &[
            ("distinctive-features", 0, 1, false),
            ("racing-tires-run-flat", 0, 1, false),
            ("spoof-kit", 0, 1, false),
            ("standard-weapon-mount", 0, 2, false),
        ],
    );
    let r = run(&d);
    assert_eq!(r.cost, 53_000);
    assert_eq!(r.mods.aftermarket_cost, 12_000);
    assert_eq!(r.total_cost, 65_000);
    // Accessories get no do-it-yourself discount; the mounts are halved.
    assert_eq!(r.mods.aftermarket_diy_cost, 7_000 + 2_500);
    // Body 9 gives 3 free standard hardpoints; the mounts use 2.
    assert_eq!((r.mods.hardpoints.free, r.mods.hardpoints.used), (3.0, 2.0));
    // The highest of the vehicle's 2 and the mods' 4(I).
    assert_eq!(r.availability, "4(I)");
    assert_eq!(r.stat_line.values[10], "65,000¥");
    assert_eq!(r.factory_stat_line.values[10], "53,000¥");
    assert!(r.notes.iter().any(|n| n.starts_with("Racing tires")));
    assert!(r.problems.is_empty(), "{:?}", r.problems);
}

#[test]
fn example_3_builds_its_stealth_mods_into_the_design() {
    // Realistic Features 4 and Chameleon Coating 2 built in, at 2 Build
    // Points a slot (8 and 4; decided: the example's 67 left them out), and
    // a concealed standard turret installed afterward.
    let mut d = design("drone-medium", "walker", "advanced");
    d.modifications = vec!["integrated-armor".into()];
    d.qualities = vec!["one-off-custom".into()];
    d.improvements = Improvements { body: 2, handling: 1, ..Improvements::default() };
    let d = with_installed(
        d,
        &[
            ("realistic-features", 4, 1, true),
            ("chameleon-coating", 2, 1, true),
            ("standard-turreted-weapon-mount", 0, 1, false),
            ("large-pop-out", 0, 1, false),
        ],
    );
    let r = run(&d);
    assert_eq!(r.mods.built_in_points, 12);
    assert_eq!(r.base_points, 72 + 12);
    assert_eq!(r.build_point_value, 84);
    // Realistic Features 4 × Body 6 × 1,000 = 24,000; chameleon 2 × 4,500.
    assert_eq!(r.mods.built_in_cost, 33_000);
    assert_eq!(r.cost, 84_000 + 33_000);
    assert_eq!(r.parts_cost, 58_500);
    assert_eq!(r.mods.aftermarket_cost, 8_500 + 4_500);
    // Built-in mods use slots too (decided): chassis 4 + 2 of 6, electronics 2.
    assert_eq!(r.mods.slots[0].used, 6.0);
    assert_eq!(r.mods.slots[2].used, 2.0);
    // Body 6: 2 free standard hardpoints, which a turreted standard mount
    // (on a large hardpoint) takes.
    assert_eq!((r.mods.hardpoints.free, r.mods.hardpoints.used), (2.0, 2.0));
    assert_eq!(r.availability, "8(I)");
    assert_eq!(r.design_test.day, 17); // 84 ÷ 5, rounded up
    assert!(r.problems.is_empty(), "{:?}", r.problems);
}

// Slots, prices, and tests.

#[test]
fn mod_slots_are_the_designed_body_in_each_category() {
    let available: Vec<i32> = run(&car()).mods.slots.iter().map(|s| s.available).collect();
    assert_eq!(available, [10, 10, 10]);
}

#[test]
fn slots_past_a_category_shift_from_spare_ones_at_2_for_1() {
    // Chassis 15 of 10: 5 over, covered by 10 spare slots elsewhere.
    assert!(run(&with_installed(car(), &[("extreme-environmental", 0, 15, false)])).problems.is_empty());
    // Chassis 22 of 10: 12 over needs 24 spare; there are 20.
    let r = run(&with_installed(car(), &[("extreme-environmental", 0, 22, false)]));
    assert!(has_problem(&r, "The mods need 2 more mod slots"), "{:?}", r.problems);
}

#[test]
fn universal_slots_cover_any_category_one_for_one() {
    let mut d = with_installed(car(), &[("extreme-environmental", 0, 22, false)]);
    d.improvements.easily_modified = 1; // 2 universal slots
    // 12 over: 2 universal, then 10 more need 20 spare, exactly enough.
    let r = run(&d);
    assert!(r.problems.is_empty(), "{:?}", r.problems);
}

#[test]
fn a_suncell_takes_powertrain_or_electronics_slots_where_there_is_room() {
    let r = run(&with_installed(car(), &[("suncell", 0, 1, false), ("nitro-boost", 0, 1, false)]));
    assert_eq!((r.mods.slots[1].used, r.mods.slots[2].used), (2.0, 1.0));
}

#[test]
fn installing_a_mod_takes_slots_times_8_and_slots_hours() {
    assert_eq!(install_test(0.0), "Engineering + Logic (8, 1 hour), toolkit");
    assert_eq!(install_test(0.5), "Engineering + Logic (4, 0.5 hours), toolkit");
    assert_eq!(install_test(2.0), "Engineering + Logic (16, 2 hours), shop");
    assert_eq!(install_test(6.0), "Engineering + Logic (48, 6 hours), facility");
}

#[test]
fn doing_it_yourself_halves_the_price_except_accessories() {
    let r = run(&with(car(), &["nitro-boost", "spoof-kit"]));
    assert_eq!(r.mods.aftermarket_cost, 5_000 + 1_500);
    assert_eq!(r.mods.aftermarket_diy_cost, 2_500 + 1_500);
}

#[test]
fn a_portable_facility_halves_only_past_its_fixed_50000() {
    let d = with_installed(
        design("truck", "wheeled", "basic"),
        &[("portable-facility", 0, 1, false), ("cargo-space", 25, 1, false)],
    );
    assert_eq!(run(&d).mods.lines[0].diy_price, 62_500);
}

#[test]
fn a_built_in_mod_costs_2_build_points_a_slot_rounded_up() {
    let r = run(&with_installed(car(), &[("gun-port", 0, 1, true)]));
    assert_eq!(r.mods.built_in_points, 1); // half a slot
    assert!(r.build_points.iter().any(|l| l.label == "Built in: Gun Port" && l.points == 1));
    assert_eq!(r.mods.aftermarket_cost, 0);
    assert_eq!(r.cost, 1_000 * i64::from(r.build_point_value) + 1_000);
}

#[test]
fn availability_is_the_highest_of_the_vehicle_and_its_mods() {
    let r = run(&with(car(), &["emergency-lights", "oil-slick-sprayer"]));
    assert_eq!(r.availability, "4(I)"); // I outranks L
    assert_eq!(r.factory_stat_line.values[9], "2");
}

#[test]
fn medevac_modifications_cost_a_flat_3500() {
    let r = run(&with(design("drone-medium", "wheeled", "basic"), &["medevac"]));
    assert_eq!(r.mods.aftermarket_cost, 3_500);
}

#[test]
fn a_mod_installed_once_ignores_a_count() {
    let r = run(&with_installed(car(), &[("nitro-boost", 0, 3, false)]));
    assert_eq!(r.mods.aftermarket_cost, 5_000);
    assert!(has_problem(&r, "Nitro Boost is installed once"));
}

// What mods change.

#[test]
fn increased_structural_integrity_adds_body_but_not_slots() {
    let r = run(&with_installed(car(), &[("structural-integrity", 5, 1, false)]));
    assert_eq!(r.stats.body, 15);
    assert_eq!(r.mods.slots[0].available, 10);
    assert_eq!(r.mods.slots[0].used, 5.0);
    // At most half the factory Body (decided).
    let r = run(&with_installed(car(), &[("structural-integrity", 6, 1, false)]));
    assert!(has_problem(&r, "Increased Structural Integrity: rating 1 to 5"));
}

#[test]
fn standard_armor_costs_acceleration_and_top_speed() {
    // +4 Armor: Acceleration −2, Top Speed −40.
    let r = run(&with_installed(car(), &[("standard-armor", 4, 1, false)]));
    assert_eq!((r.stats.armor, r.stats.acceleration, r.stats.top_speed), (8, 9, 120));
    assert_eq!(r.mods.aftermarket_cost, 3_200);
    // Removing 2 gives back 1 and 20, and costs nothing.
    let r = run(&with_installed(car(), &[("standard-armor", -2, 1, false)]));
    assert_eq!((r.stats.armor, r.stats.acceleration, r.stats.top_speed), (2, 12, 180));
    assert_eq!(r.mods.aftermarket_cost, 0);
    // Up to twice the Body.
    assert!(has_problem(&run(&with_installed(car(), &[("standard-armor", 21, 1, false)])), "Standard Armor: rating"));
}

#[test]
fn ceramic_plate_is_priced_by_the_armor() {
    let r = run(&with(car(), &["ceramic-plate"]));
    assert_eq!((r.mods.lines[0].slots, r.mods.lines[0].price), (1.0, 6_000)); // Armor 4
}

#[test]
fn milspec_plate_lowers_the_speed_interval_by_5_a_rating() {
    let r = run(&with_installed(car(), &[("milspec-plate", 2, 1, false)]));
    assert_eq!(r.stats.speed_interval, 10);
    let line = &r.mods.lines[0];
    assert_eq!((line.slots, line.availability.as_str(), line.price), (4.0, "10(I)", 40_000));
}

#[test]
fn improved_acceleration_and_top_speed_add_a_share_of_the_vehicles_own() {
    // 30% of 11, rounded up, is 4 a rank; 20% of 160 is 32.
    let d = with_installed(car(), &[("improved-acceleration", 2, 1, false), ("top-speed-increase", 1, 1, false)]);
    let r = run(&d);
    assert_eq!((r.stats.acceleration, r.stats.top_speed), (19, 192));
}

#[test]
fn handling_mods_never_take_handling_below_1() {
    let mut d = with_installed(car(), &[("improved-handling", 2, 1, false)]);
    assert_eq!(run(&d).stats.handling, "1");
    d.improvements.handling = 1;
    let r = run(&d);
    assert_eq!(r.stats.handling, "1");
    // 3, less 1 designed in and 2 for the mod.
    assert!(has_problem(&r, "Handling on road is 0 with the mods"), "{:?}", r.problems);
}

#[test]
fn improved_handling_and_pilot_by_wire_dont_stack() {
    let r = run(&with_installed(car(), &[("improved-handling", 1, 1, false), ("pilot-by-wire", 1, 1, false)]));
    assert!(has_problem(&r, "Improved Handling and Pilot-By-Wire"));
}

#[test]
fn pilot_by_wire_needs_pilot_twice_its_rating() {
    // A basic console's Pilot 1 is too low for rating 1.
    assert!(has_problem(&run(&with(car(), &["pilot-by-wire"])), "Pilot-By-Wire 1 needs Pilot 2"));
    assert!(run(&with(design("car", "wheeled", "intermediate"), &["pilot-by-wire"])).problems.is_empty());
}

#[test]
fn multi_terrain_suspension_moves_off_road_handling_on_road_until_level() {
    // On-road-only Improved Handling makes the car 2/3; rating 3 can move
    // only 1 point, to 3/2.
    let mut d = with_installed(car(), &[("multi-terrain-static", 3, 1, false)]);
    d.improvements.handling_on_road = 1;
    assert_eq!(run(&d).stats.handling, "3/2");
    // No effect when off road is no worse.
    let r = run(&with(car(), &["multi-terrain-static"]));
    assert_eq!(r.stats.handling, "3");
    assert!(r.notes.iter().any(|n| n.contains("no effect")));
}

#[test]
fn improved_pilot_and_enhanced_sensors_replace_the_consoles() {
    let r = run(&with_installed(car(), &[("improved-pilot", 4, 1, false), ("enhanced-sensors", 6, 1, false)]));
    assert_eq!((r.stats.pilot, r.stats.sensor), (4, 6));
    assert_eq!(r.mods.slots[2].used, 10.0);
    assert_eq!(r.mods.aftermarket_cost, 20_000 + 30_000);
}

// Hardpoints, cargo, and seats.

#[test]
fn hardpoint_devices_need_hardpoints() {
    // Body 10: 3 free standard hardpoints; two large mounts need 4.
    assert!(has_problem(
        &run(&with_installed(car(), &[("large-weapon-mount", 0, 2, false)])),
        "The hardpoint devices need 4"
    ));
    let d = with_installed(car(), &[("large-weapon-mount", 0, 2, false), ("standard-hardpoint", 0, 1, false)]);
    let r = run(&d);
    assert!(r.problems.is_empty(), "{:?}", r.problems);
}

#[test]
fn small_drones_get_a_small_hardpoint_and_microdrones_none() {
    let mut d = design("drone-small", "wheeled", "basic");
    d.improvements.body = 2; // Body 4, over 3
    assert_eq!(run(&d).mods.hardpoints.free, 0.5);
    let r = run(&with(design("drone-small", "wheeled", "basic"), &["standard-hardpoint"]));
    assert!(has_problem(&r, "Small drones and minidrones can only add small"));
    let r = run(&with(design("drone-micro", "wheeled", "basic"), &["small-hardpoint"]));
    assert!(has_problem(&r, "A microdrone can't add hardpoints"));
}

#[test]
fn cf_mods_take_cargo_space() {
    // A car has 1/2 CF: a rigger cocoon (1 CF) doesn't fit until
    // Increased Cargo Space adds 2.
    assert!(has_problem(&run(&with(car(), &["rigger-cocoon"])), "The CF mods take 1 CF"));
    let r = run(&with_installed(car(), &[("rigger-cocoon", 0, 1, false), ("cargo-space", 1, 1, false)]));
    assert!(r.problems.is_empty(), "{:?}", r.problems);
    assert_eq!(r.stats.cargo_cf, 1.5);
}

#[test]
fn increased_seating_takes_a_cf_a_seat_and_squatter_amenities_double_them() {
    let r = run(&with_installed(car(), &[("cargo-space", 1, 1, false), ("increased-seating", 2, 1, false)]));
    assert_eq!(r.stats.seat, 6);
    assert_eq!(run(&with(car(), &["amenities-squatter"])).stats.seat, 8);
}

// Which vehicles a mod is for.

#[test]
fn passenger_mods_arent_for_drones() {
    let r = run(&with(design("drone-medium", "wheeled", "basic"), &["rigger-cocoon"]));
    assert!(has_problem(&r, "Rigger Cocoon is only for passenger vehicles"));
}

#[test]
fn tires_are_for_wheeled_vehicles_and_cheaper_for_bikes_and_drones() {
    assert_eq!(run(&with(design("bike", "wheeled", "basic"), &["racing-tires"])).mods.aftermarket_cost, 500);
    assert_eq!(run(&with(car(), &["racing-tires"])).mods.aftermarket_cost, 1_000);
    let r = run(&with(design("car", "hover", "basic"), &["racing-tires"]));
    assert!(has_problem(&r, "Racing Tires is only for wheeled"));
}

#[test]
fn one_set_of_tires_at_a_time() {
    let r = run(&with(car(), &["racing-tires", "off-road-tires"]));
    assert!(has_problem(&r, "Choose one of these: Racing Tires, Off-Road Tires"));
}

#[test]
fn changing_the_form_takes_or_frees_a_slot_a_level() {
    // Stripping a car to open construction frees a chassis slot, 4,000¥.
    let r = run(&with(car(), &["form-open"]));
    assert_eq!((r.mods.lines[0].slots, r.mods.lines[0].price), (-1.0, 4_000));
    assert_eq!(r.mods.form, "open construction");
    // Totally enclosed takes a slot and 6,000¥, and Armor at least 1.5 × Body.
    let r = run(&with(car(), &["form-enclosed"]));
    assert_eq!((r.mods.lines[0].slots, r.mods.lines[0].price), (1.0, 6_000));
    assert!(has_problem(&r, "Totally enclosed construction needs Armor 15"), "{:?}", r.problems);
    // A bike is already open.
    let r = run(&with(design("bike", "wheeled", "basic"), &["form-open"]));
    assert!(has_problem(&r, "The chassis already has open construction"));
}

#[test]
fn life_safety_needs_a_closed_form() {
    let r = run(&with(design("bike", "wheeled", "basic"), &["life-safety-1"]));
    assert!(has_problem(&r, "Life Safety System (Level 1) needs standard or totally enclosed"));
}

#[test]
fn secondary_propulsion_has_its_own_stats_and_is_priced_by_body() {
    let r = run(&with(car(), &["secondary-hovercraft"]));
    let s = r.mods.secondary.as_ref().expect("a secondary line");
    assert_eq!((s.handling.as_str(), s.acceleration, s.speed_interval, s.top_speed), ("4", 10, 15, 90));
    assert_eq!(r.mods.aftermarket_cost, 30_000); // Body 10 × 3,000
    assert_eq!(r.mods.slots[1].used, 3.0);
}

#[test]
fn gecko_tips_go_by_size_class() {
    // A medium drone is Size Class 3: 2 slots, 6,500¥.
    let r = run(&with(design("drone-medium", "walker", "basic"), &["gecko-tips"]));
    assert_eq!((r.mods.lines[0].slots, r.mods.lines[0].price), (2.0, 6_500));
    assert!(has_problem(&run(&with(car(), &["gecko-tips"])), "Gecko Tips work up to Size Class 4"));
}

// Files.

#[test]
fn a_design_file_keeps_its_mods_and_an_older_file_has_none() {
    let mut d = with_installed(car(), &[("standard-weapon-mount", 0, 2, true)]);
    d.mods[0].note = "roof".into();
    assert_eq!(files::from_text(&files::to_text(&d)).unwrap(), d);
    assert!(files::from_text("format = 1\nchassis = \"car\"\n").unwrap().mods.is_empty());
}

// The PDF sheet.

#[test]
fn the_sheet_lists_the_mods_as_standard_equipment() {
    let d = with_installed(car(), &[("spoof-kit", 0, 1, false), ("standard-weapon-mount", 0, 2, false)]);
    let bytes = crate::pdf::vehicle_sheet(&d, Catalog::builtin()).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    // The × is byte 0xD7 in the fonts' encoding, written as octal 327.
    assert!(text.contains(r"(Standard Equipment: Spoof Kit, Standard Weapon Mount \3272) Tj"));
}
