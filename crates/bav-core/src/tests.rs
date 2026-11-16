//! Tests for the design rules, one per rule, named after it. The three
//! build examples from Double Clutch (pp. 155–156) are here as the book's
//! numbers, corrected where RULINGS.md says the example
//! is wrong.

use crate::calc::{cargo_text, thousands};
use crate::catalog::Catalog;
use crate::design::{Design, Improvements};
use crate::files;
use crate::pdf;
use crate::{Report, evaluate};

fn design(chassis: &str, powertrain: &str, console: &str) -> Design {
    Design {
        chassis: chassis.into(),
        powertrain: powertrain.into(),
        console: console.into(),
        ..Design::default() // the rest from `Default`
    }
}

fn run(d: &Design) -> Report {
    evaluate(d, Catalog::builtin()).expect("the design's ids are in the catalog")
}

fn with_mods(mut d: Design, mods: &[&str]) -> Design {
    d.modifications = mods.iter().map(|m| m.to_string()).collect();
    d
}

fn with_qualities(mut d: Design, qualities: &[&str]) -> Design {
    d.qualities = qualities.iter().map(|q| q.to_string()).collect();
    d
}

// The book's examples.

#[test]
fn example_1_street_racer_with_body_lowered_to_9() {
    // A car (7), wheeled (10), intermediate console (5), Body −1 (−4),
    // Improved Speed Interval ×2 (10), racing engine (25): 53 Build Points.
    // Its accessories (tires, spoof kit, paint) are mods, not design options.
    let mut d = with_mods(design("car", "wheeled", "intermediate"), &["racing-engine"]);
    d.improvements = Improvements { body: -1, speed_interval: 2, ..Improvements::default() };
    let r = run(&d);
    assert_eq!(r.base_points, 53);
    assert_eq!(r.build_point_value, 53);
    assert_eq!(r.cost, 53_000);
    assert_eq!(r.stats.body, 9);
    assert_eq!(r.stats.speed_modifier, 8); // 9 ÷ 2 + 4
    assert_eq!(r.stats.acceleration, 17); // 15 − 4 = 11, + 6
    // The final Body sets the speed modifier (decided), so 250 − 80 + 100;
    // the book's 260 used Body 10.
    assert_eq!(r.stats.top_speed, 270);
    assert_eq!(r.stats.speed_interval, 30);
    assert_eq!(r.design_test.day, 11); // 53 ÷ 5, rounded up
    assert_eq!(r.availability, "2");
}

#[test]
fn example_2_lav_rounds_the_acceleration_loss_down() {
    // VTOL (15), vectored thrust (100), high performance engine (15, as its
    // entry says; the example's 173 charges 25, decided as a slip),
    // multi-engine (25), expert console (8): 163.
    let d = with_mods(design("vtol", "vectored-thrust", "expert"), &["high-performance-engine", "multi-engine"]);
    let r = run(&d);
    assert_eq!(r.stats.speed_modifier, 19); // 14 ÷ 2 + 12
    // 40 − 9 (19 ÷ 2 rounded down, decided) = 31; + 3 = 34; × 2 = 68.
    assert_eq!(r.stats.acceleration, 68);
    // 500 − 190 = 310; + 50 = 360; × 2 = 720.
    assert_eq!(r.stats.top_speed, 720);
    assert_eq!(r.stats.pilot, 3);
    assert_eq!(r.stats.sensor, 3);
    assert_eq!(r.base_points, 15 + 100 + 15 + 25 + 8);
    assert_eq!(r.build_point_value, 163); // no quality factors
    assert_eq!(r.cost, 163_000);
    assert_eq!(r.parts_cost, 81_500);
    assert_eq!(r.availability, "3"); // 2, +1 for an aircraft
    // 163 ÷ 5 = 32.6, rounded up to 33; ÷ 2 for a week, 17; ÷ 4 for a month, 9.
    assert_eq!((r.design_test.day, r.design_test.week, r.design_test.month), (33, 17, 9));
}

#[test]
fn example_3_walker_drone_uses_the_powertrain_table() {
    // Medium drone (10), walker (15, the table's), advanced console (14),
    // Body +2 (8), Handling +1 (5), Integrated Armor (20), One-off Custom.
    let mut d = with_qualities(
        with_mods(design("drone-medium", "walker", "advanced"), &["integrated-armor"]),
        &["one-off-custom"],
    );
    d.improvements = Improvements { body: 2, handling: 1, ..Improvements::default() };
    let r = run(&d);
    assert_eq!(r.stats.body, 6);
    assert_eq!(r.stats.armor, 6);
    assert_eq!(r.stats.handling, "2");
    assert_eq!(r.stats.speed_modifier, 6); // 6 ÷ 2 + 6 ÷ 2
    assert_eq!(r.stats.acceleration, 7); // 10 − 3
    assert_eq!(r.stats.speed_interval, 15); // the table's (the example says 10)
    assert_eq!(r.stats.top_speed, 30); // a walker loses none (the example says 40)
    // One-off Custom × 2 and Drone × 0.5 cancel.
    assert_eq!(r.multiplier, 1.0);
    assert_eq!(r.base_points, 10 + 15 + 14 + 8 + 5 + 20);
    assert_eq!(r.build_point_value, 72);
    assert_eq!(r.cost, 72_000);
    assert_eq!(r.parts_cost, 36_000);
    assert_eq!(r.availability, "2");
    // 72 ÷ 5 = 14.4, rounded up to 15 (decided: the example rounds its
    // 67 ÷ 5 down to 13, where Examples 1 and 2 round up); 8 by the week.
    assert_eq!((r.design_test.day, r.design_test.week, r.design_test.month), (15, 8, 4));
    assert_eq!(r.stat_line.values[8], "—"); // a drone's Seat
}

// Tables.

#[test]
fn every_table_loads() {
    let c = Catalog::builtin();
    assert_eq!(c.size_class.len(), 21);
    assert_eq!(c.chassis.len(), 18);
    assert_eq!(c.powertrain.len(), 13);
    assert_eq!(c.console.len(), 4);
    assert_eq!(c.improvement.len(), 12);
    assert_eq!(c.modification.len(), 11);
    assert_eq!(c.quality.len(), 12);
    assert_eq!(c.environments.len(), 5);
    for (i, row) in c.size_class.iter().enumerate() {
        assert_eq!(row.class, i as i32, "size classes are in order");
    }
}

#[test]
fn every_chassis_powertrain_and_console_combination_calculates() {
    let c = Catalog::builtin();
    for chassis in &c.chassis {
        for powertrain in &c.powertrain {
            for console in &c.console {
                let r = run(&design(&chassis.id, &powertrain.id, &console.id));
                assert!(r.stats.acceleration >= 0 && r.stats.top_speed >= 0);
            }
        }
    }
}

#[test]
fn every_improvement_in_the_design_has_a_catalog_entry() {
    for (id, _) in Improvements::default().levels() {
        assert!(Catalog::builtin().improvement(id).is_some(), "{id}");
    }
}

// Chassis, powertrain, console.

#[test]
fn a_basic_car_has_the_tables_values() {
    let r = run(&design("car", "wheeled", "basic"));
    let s = &r.stats;
    assert_eq!((s.body, s.armor, s.handling.as_str(), s.seat, s.size_class), (10, 4, "3", 4, 6));
    assert_eq!((s.pilot, s.sensor), (1, 1));
    assert_eq!(s.speed_interval, 20);
    assert_eq!(r.base_points, 7 + 10 + 3);
}

#[test]
fn tracked_is_two_better_off_road() {
    let r = run(&design("truck", "tracked", "basic"));
    assert_eq!(r.stats.handling, "4/2");
}

#[test]
fn hover_is_two_better_everywhere() {
    let r = run(&design("car", "hover", "basic"));
    assert_eq!(r.stats.handling, "1");
}

#[test]
fn a_truck_has_4_cf_more_cargo() {
    // Size Class 8's 4 CF, plus 4.
    assert_eq!(run(&design("truck", "wheeled", "basic")).stats.cargo_cf, 8.0);
}

#[test]
fn a_submersible_is_built_for_underwater_use() {
    let r = run(&design("submersible", "propeller-water", "basic"));
    assert!(r.notes.iter().any(|n| n.contains("underwater")));
}

// Speed.

#[test]
fn speed_modifier_is_half_body_plus_armor_rounded_down() {
    // Bike: 4 ÷ 2 + 2 = 4; Acceleration 15 − 2, Top Speed 250 − 40.
    let r = run(&design("bike", "wheeled", "basic"));
    assert_eq!(r.stats.speed_modifier, 4);
    assert_eq!(r.stats.acceleration, 13);
    assert_eq!(r.stats.top_speed, 210);
}

#[test]
fn speed_interval_isnt_lowered_by_body_and_armor() {
    assert_eq!(run(&design("bus", "wheeled", "basic")).stats.speed_interval, 20);
}

#[test]
fn integrated_armor_counts_half_the_armor() {
    // Van: 12 ÷ 2 + 10 ÷ 2 = 11.
    let r = run(&with_mods(design("van", "wheeled", "basic"), &["integrated-armor"]));
    assert_eq!(r.stats.speed_modifier, 11);
}

#[test]
fn cargo_hauler_and_integrated_armor_dont_stack() {
    let r = run(&with_mods(design("van", "wheeled", "basic"), &["integrated-armor", "cargo-hauler"]));
    assert_eq!(r.stats.speed_modifier, 11);
}

#[test]
fn small_drones_multiply_acceleration_before_the_engine() {
    // Small drone: 2 ÷ 2 + 1 = 2; 15 − 1 = 14; × 0.75 = 10; racing + 6 = 16.
    let r = run(&with_mods(design("drone-small", "wheeled", "basic"), &["racing-engine"]));
    assert_eq!(r.stats.acceleration, 16);
    // Mini: 1 ÷ 2 + 0 = 0; 15 × 0.5 = 7. Micro: 15 × 0.25 = 3.
    assert_eq!(run(&design("drone-mini", "wheeled", "basic")).stats.acceleration, 7);
    assert_eq!(run(&design("drone-micro", "wheeled", "basic")).stats.acceleration, 3);
    // Medium and large drones aren't multiplied: 15 − (4 ÷ 2 + 6) ÷ 2.
    assert_eq!(run(&design("drone-medium", "wheeled", "basic")).stats.acceleration, 11);
}

#[test]
fn engine_types_change_acceleration_and_top_speed() {
    let base = run(&design("car", "wheeled", "basic")).stats;
    let hp = run(&with_mods(design("car", "wheeled", "basic"), &["high-performance-engine"])).stats;
    let economy = run(&with_mods(design("car", "wheeled", "basic"), &["economy-engine"])).stats;
    assert_eq!((hp.acceleration - base.acceleration, hp.top_speed - base.top_speed), (3, 50));
    assert_eq!((economy.acceleration - base.acceleration, economy.top_speed - base.top_speed), (-3, -50));
}

#[test]
fn a_cargo_hauler_gets_half_the_high_performance_and_multi_engine_bonuses() {
    let plain = run(&with_mods(design("truck", "wheeled", "basic"), &["cargo-hauler"])).stats;
    let both = run(&with_mods(
        design("truck", "wheeled", "basic"),
        &["cargo-hauler", "high-performance-engine", "multi-engine"],
    ))
    .stats;
    assert_eq!(both.acceleration, (plain.acceleration + 1) * 3 / 2);
    assert_eq!(both.top_speed, (plain.top_speed + 25) * 3 / 2);
}

#[test]
fn underpowered_halves_top_speed_only() {
    let base = run(&design("car", "wheeled", "basic")).stats;
    let under = run(&with_mods(design("car", "wheeled", "basic"), &["underpowered"])).stats;
    assert_eq!(under.top_speed, base.top_speed / 2);
    assert_eq!(under.acceleration, base.acceleration);
    assert_eq!(under.speed_interval, base.speed_interval);
}

#[test]
fn off_road_chassis_moves_a_handling_point_and_costs_10_top_speed() {
    let base = run(&design("truck", "wheeled", "basic")).stats;
    let off = run(&with_mods(design("truck", "wheeled", "basic"), &["off-road-chassis"])).stats;
    assert_eq!(off.handling, "5/3"); // the book's example
    assert_eq!(off.top_speed, base.top_speed - 10);
}

#[test]
fn too_heavy_a_vehicle_is_flagged_and_shown_as_0() {
    // A ship on sails: 20 ÷ 2 + 12 = 22; 20 − 220.
    let r = run(&design("ship", "sail", "basic"));
    assert_eq!(r.stats.top_speed, 0);
    assert!(r.problems.iter().any(|p| p.starts_with("Top Speed is")));
}

// Design options.

#[test]
fn improvements_add_their_levels_and_build_points() {
    let mut d = design("car", "wheeled", "basic");
    d.improvements =
        Improvements { body: 2, armor: 3, acceleration: 1, speed_interval: 1, top_speed: 1, ..Improvements::default() };
    let r = run(&d);
    assert_eq!(r.stats.body, 12);
    assert_eq!(r.stats.armor, 7);
    // 12 ÷ 2 + 7 = 13: Acceleration 20 − 6, Top Speed 275 − 130.
    assert_eq!(r.stats.acceleration, 14);
    assert_eq!(r.stats.top_speed, 145);
    assert_eq!(r.stats.speed_interval, 25);
    assert_eq!(r.base_points, 20 + 2 * 4 + 3 * 2 + 5 + 5 + 5);
}

#[test]
fn improvements_in_reverse_lower_build_points() {
    let mut d = design("car", "wheeled", "basic");
    d.improvements.top_speed = -2;
    let r = run(&d);
    assert_eq!(r.base_points, 20 - 10);
    assert!(r.problems.is_empty());
}

#[test]
fn levels_past_the_limit_are_flagged() {
    let mut d = design("car", "wheeled", "basic");
    d.improvements = Improvements { body: 5, acceleration: -3, ..Improvements::default() };
    let problems = run(&d).problems;
    assert!(problems.iter().any(|p| p.starts_with("Improved Body: at most 4 levels")));
    assert!(problems.iter().any(|p| p.starts_with("Improved Acceleration: at most 2 levels in reverse")));
}

#[test]
fn armor_above_twice_the_body_is_flagged() {
    let mut d = design("bike", "wheeled", "basic");
    d.improvements.armor = 7; // 9 > 2 × 4
    assert!(run(&d).problems.iter().any(|p| p.contains("more than twice the Body")));
}

#[test]
fn improved_handling_is_one_better_a_level_and_on_or_off_road_alone_costs_3() {
    let mut d = design("truck", "wheeled", "basic");
    d.improvements = Improvements { handling: 1, handling_off_road: 2, ..Improvements::default() };
    let r = run(&d);
    assert_eq!(r.stats.handling, "3/1");
    assert_eq!(r.base_points, 21 + 5 + 6);
    d.improvements.handling_off_road = 3; // 4 off-road levels in all
    assert!(run(&d).problems.iter().any(|p| p.starts_with("Improved Handling (off road)")));
}

#[test]
fn improved_seating_adds_at_most_half_the_seats() {
    let mut d = design("van", "wheeled", "basic");
    d.improvements.seating = 3;
    let r = run(&d);
    assert_eq!(r.stats.seat, 9);
    assert!(r.problems.is_empty());
    d.improvements.seating = 4;
    assert!(run(&d).problems.iter().any(|p| p.starts_with("Improved Seating")));
}

#[test]
fn improved_size_adds_a_class_two_body_and_half_the_seats() {
    let mut d = design("car", "wheeled", "basic");
    d.improvements.size = 1;
    let r = run(&d);
    assert_eq!((r.stats.size_class, r.stats.body, r.stats.seat), (7, 12, 6));
    assert_eq!(r.stats.cargo, "1 CF"); // Size Class 7's
    d.improvements.size = -3; // at most 2 levels down
    assert!(run(&d).problems.iter().any(|p| p.starts_with("Improved Size")));
}

#[test]
fn a_drones_size_comes_from_its_chassis_except_a_large_drone_grows() {
    let mut d = design("drone-small", "wheeled", "basic");
    d.improvements.size = 1;
    assert!(run(&d).problems.iter().any(|p| p.starts_with("A drone's size")));
    let mut large = design("drone-large", "wheeled", "basic");
    large.improvements.size = 1;
    assert!(run(&large).problems.is_empty());
}

#[test]
fn improved_cargo_adds_2_cf_at_most_3_levels_or_doubling() {
    let mut d = design("truck", "wheeled", "basic"); // 8 CF: 3 levels
    d.improvements.cargo = 3;
    let r = run(&d);
    assert_eq!(r.stats.cargo_cf, 14.0);
    assert!(r.problems.is_empty());
    let mut car = design("car", "wheeled", "basic"); // 1/2 CF: no room
    car.improvements.cargo = 1;
    assert!(run(&car).problems.iter().any(|p| p.starts_with("Improved Cargo Space")));
}

#[test]
fn easily_modified_adds_two_universal_slots_a_level() {
    let mut d = design("car", "wheeled", "basic");
    d.improvements.easily_modified = 2;
    let r = run(&d);
    assert_eq!(r.stats.mod_slots.universal, 4);
    assert_eq!(r.stats.mod_slots.chassis, 10);
    let mut mini = design("drone-mini", "wheeled", "basic");
    mini.improvements.easily_modified = 2; // at most its Body, 1
    assert!(run(&mini).problems.iter().any(|p| p.starts_with("Easily Modified")));
}

// Design modifications.

#[test]
fn heavy_duty_and_lightweight_change_body_by_half() {
    let heavy = run(&with_mods(design("car", "wheeled", "basic"), &["heavy-duty"]));
    let light = run(&with_mods(design("bike", "wheeled", "basic"), &["lightweight"]));
    assert_eq!(heavy.stats.body, 15);
    assert_eq!(light.stats.body, 2);
    assert_eq!(heavy.base_points, 30);
    assert_eq!(light.base_points, 16 - 10);
}

#[test]
fn cargo_hauler_doubles_cargo_and_halves_seats_to_at_least_two() {
    let r = run(&with_mods(design("van", "wheeled", "basic"), &["cargo-hauler"]));
    assert_eq!(r.stats.cargo_cf, 2.0);
    assert_eq!(r.stats.seat, 3);
    let bike = run(&with_mods(design("bike", "wheeled", "basic"), &["cargo-hauler"]));
    assert_eq!(bike.stats.seat, 2);
}

#[test]
fn mass_transit_triples_the_seats() {
    assert_eq!(run(&with_mods(design("bus", "wheeled", "basic"), &["mass-transit"])).stats.seat, 36);
}

#[test]
fn two_engine_types_are_flagged() {
    let r = run(&with_mods(design("car", "wheeled", "basic"), &["racing-engine", "economy-engine"]));
    assert!(r.problems.iter().any(|p| p.starts_with("Choose one engine type")));
}

// Quality factors, cost, Availability.

#[test]
fn quality_factors_multiply_the_build_points() {
    let r = run(&with_qualities(design("car", "wheeled", "basic"), &["luxury", "security"]));
    assert_eq!(r.multiplier, 3.0);
    assert_eq!(r.build_point_value, 60);
    assert_eq!(r.cost, 60_000);
    assert_eq!(r.parts_cost, 30_000);
}

#[test]
fn the_build_point_value_rounds_down() {
    // A bike: 3 + 10 + 3 = 16; × 1.5 = 24. A small drone: 8 + 10 + 3 = 21; × 0.5 = 10.
    assert_eq!(run(&with_qualities(design("bike", "wheeled", "basic"), &["luxury"])).build_point_value, 24);
    assert_eq!(run(&design("drone-small", "wheeled", "basic")).build_point_value, 10);
}

#[test]
fn a_drone_chassis_has_the_drone_quality_once() {
    let r = run(&with_qualities(design("drone-large", "wheeled", "basic"), &["drone"]));
    assert_eq!(r.multipliers.len(), 1);
    assert_eq!(r.multiplier, 0.5);
}

#[test]
fn availability_adds_aircraft_luxury_security_and_military() {
    assert_eq!(run(&design("car", "wheeled", "basic")).availability, "2");
    assert_eq!(run(&design("fixed-wing", "propeller-air", "basic")).availability, "3");
    // A rotor drone is an aircraft too.
    assert_eq!(run(&design("drone-small", "rotor", "basic")).availability, "3");
    assert_eq!(run(&with_qualities(design("car", "wheeled", "basic"), &["luxury", "security"])).availability, "4(L)");
    assert_eq!(run(&with_qualities(design("car", "wheeled", "basic"), &["military"])).availability, "4(I)");
}

#[test]
fn security_and_military_together_are_flagged() {
    let r = run(&with_qualities(design("car", "wheeled", "expert"), &["security", "military"]));
    assert!(r.problems.iter().any(|p| p.contains("Security or Military")));
}

#[test]
fn security_below_an_expert_console_gets_a_note() {
    let r = run(&with_qualities(design("car", "wheeled", "basic"), &["security"]));
    assert!(r.notes.iter().any(|n| n.contains("at least an Expert console")));
    let expert = run(&with_qualities(design("car", "wheeled", "expert"), &["security"]));
    assert!(!expert.notes.iter().any(|n| n.contains("at least an Expert console")));
}

#[test]
fn superstructure_needs_size_class_15() {
    let r = run(&with_qualities(design("ship", "jet-water", "basic"), &["superstructure"]));
    assert!(r.problems.iter().any(|p| p.contains("Size Class 15")));
}

#[test]
fn environmental_adaptation_needs_an_environment() {
    let mut d = with_qualities(design("car", "wheeled", "basic"), &["environmental-adaptation"]);
    assert!(run(&d).problems.iter().any(|p| p.contains("choose the environment")));
    d.environment = "Arctic".into();
    let r = run(&d);
    assert!(r.problems.is_empty());
    assert_eq!(r.multipliers[0].label, "Environmental Adaptation (Arctic)");
}

#[test]
fn anthropomorphic_vehicles_walk() {
    let r = run(&with_qualities(design("drone-medium", "wheeled", "basic"), &["anthropomorphic"]));
    assert!(r.problems.iter().any(|p| p.contains("Walker")));
}

// Tests and output.

#[test]
fn design_test_threshold_is_a_fifth_rounded_up_and_halved_per_week() {
    // The book's Example 2: 173 → 35, 18 with a week's interval.
    let mut d = with_mods(design("vtol", "vectored-thrust", "expert"), &["high-performance-engine", "multi-engine"]);
    d.improvements.armor = 5; // + 10 points: 163 + 10 = 173
    let r = run(&d);
    assert_eq!(r.build_point_value, 173);
    assert_eq!((r.design_test.day, r.design_test.week, r.design_test.month), (35, 18, 9));
}

#[test]
fn stat_line_has_the_sr6_vehicle_columns() {
    let r = run(&with_mods(design("truck", "wheeled", "basic"), &["off-road-chassis"]));
    assert_eq!(r.stat_line.headers.join(" "), "Hand Accel Spd Int Top Spd Body Armor Pilot Sensor Seat Avail Cost");
    assert_eq!(r.stat_line.values[0], "5/3");
    assert_eq!(r.stat_line.values[10], "26,000¥");
}

#[test]
fn numbers_and_cargo_are_written_as_the_book_does() {
    assert_eq!(thousands(1_234_567), "1,234,567");
    assert_eq!(thousands(999), "999");
    assert_eq!(cargo_text(0.0), "none");
    assert_eq!(cargo_text(0.025), "5 kg");
    assert_eq!(cargo_text(0.5), "1/2 CF");
    assert_eq!(cargo_text(4000.0), "4,000 CF");
}

#[test]
fn unknown_ids_are_an_error_not_a_crash() {
    assert!(evaluate(&design("hovertank", "wheeled", "basic"), Catalog::builtin()).is_err());
    let r = run(&with_mods(design("car", "wheeled", "basic"), &["flux-capacitor"]));
    assert!(r.problems.iter().any(|p| p.contains("flux-capacitor")));
}

// Files.

#[test]
fn a_design_saves_and_reopens_unchanged() {
    let mut d = with_qualities(with_mods(design("vtol", "jet-air", "expert"), &["multi-engine"]), &["military"]);
    d.name = "Thunderbird “Raptor”".into();
    d.improvements = Improvements { armor: 2, top_speed: -1, ..Improvements::default() };
    d.notes = "Two lines\nof notes".into();
    let text = files::to_text(&d);
    assert!(text.starts_with("format = 1"));
    assert_eq!(files::from_text(&text), Ok(d));
}

#[test]
fn a_file_missing_fields_opens_with_defaults() {
    let d = files::from_text("format = 1\nchassis = \"bike\"\n").unwrap();
    assert_eq!(d.chassis, "bike");
    assert_eq!(d.powertrain, "wheeled");
    assert_eq!(d.improvements, Improvements::default());
}

#[test]
fn a_newer_or_broken_file_gives_a_clear_error() {
    assert!(files::from_text("format = 99\n").unwrap_err().contains("newer"));
    assert!(files::from_text("this is [not toml").unwrap_err().starts_with("This isn't a design file"));
}

#[test]
fn only_bav_files_are_saved_or_opened() {
    let dir = std::env::temp_dir();
    assert!(files::save(&dir.join("notes.txt"), &Design::default()).is_err());
    assert!(files::open(&dir.join("notes.txt")).is_err());
}

#[test]
fn saving_writes_the_file_and_leaves_no_temporary_file() {
    let dir = std::env::temp_dir().join(format!("bav-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("drone.bav");
    let d = design("drone-small", "rotor", "expert");
    files::save(&path, &d).unwrap();
    assert_eq!(files::open(&path), Ok(d));
    assert!(!dir.join("drone.bav.tmp").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

// Floors.

#[test]
fn handling_is_at_least_1() {
    // Car 3, hover −2, Improved Handling 3: −2, shown as 1.
    let mut d = design("car", "hover", "basic");
    d.improvements.handling = 3;
    let r = run(&d);
    assert_eq!(r.stats.handling, "1");
    assert!(r.problems.iter().any(|p| p.starts_with("Handling on road is -2")));
}

#[test]
fn handling_off_road_alone_is_at_least_1() {
    // Truck 4, tracked −2 off road, Improved Handling 3, Off Road Chassis −1:
    // 4 − 3 + 1 = 2 on road, and 4 − 2 − 3 − 1 = −2 off road.
    let mut d = with_mods(design("truck", "tracked", "basic"), &["off-road-chassis"]);
    d.improvements.handling = 3;
    let r = run(&d);
    assert_eq!(r.stats.handling, "2/1");
    assert!(r.problems.iter().any(|p| p.starts_with("Handling off road is -2")));
    assert!(!r.problems.iter().any(|p| p.starts_with("Handling on road")));
}

#[test]
fn speed_interval_is_at_least_1() {
    // Sail 5, Improved Speed Interval −3: −10, shown as 1.
    let mut d = design("boat", "sail", "basic");
    d.improvements.speed_interval = -3;
    let r = run(&d);
    assert_eq!(r.stats.speed_interval, 1);
    assert!(r.problems.iter().any(|p| p.starts_with("Speed Interval is -10")));
}

#[test]
fn a_huge_level_from_a_hand_edited_file_is_flagged_without_overflowing() {
    let text = "format = 1\nchassis = \"car\"\npowertrain = \"wheeled\"\nconsole = \"basic\"\n\n\
                [improvements]\nbody = 2000000000\ntop_speed = -2000000000\n";
    let d = files::from_text(text).expect("the file reads");
    let r = run(&d);
    assert!(r.problems.iter().any(|p| p.starts_with("Improved Body: at most 4 levels")));
    assert!(r.problems.iter().any(|p| p.starts_with("Improved Top Speed: at most 3 levels in reverse")));
}

// The PDF vehicle sheet.

fn sheet_text(d: &Design) -> String {
    let bytes = pdf::vehicle_sheet(d, Catalog::builtin()).expect("the sheet is made");
    // The sheet is ASCII throughout (other characters are octal escapes).
    String::from_utf8(bytes).expect("the sheet is ASCII")
}

#[test]
fn the_sheet_is_a_pdf_whose_cross_references_point_at_its_objects() {
    let text = sheet_text(&design("car", "wheeled", "basic"));
    assert!(text.starts_with("%PDF-1.4\n"));
    assert!(text.ends_with("%%EOF\n"));
    let start: usize = text.rsplit("startxref\n").next().unwrap().lines().next().unwrap().parse().unwrap();
    assert!(text[start..].starts_with("xref\n"));
    // Each "n" entry in the table is the byte where object 1, 2, … starts.
    let entries = text[start..].lines().skip(3).take_while(|l| l.ends_with(" n "));
    for (i, entry) in entries.enumerate() {
        let offset: usize = entry[..10].parse().unwrap();
        assert!(text[offset..].starts_with(&format!("{} 0 obj\n", i + 1)), "object {}", i + 1);
    }
}

#[test]
fn the_sheet_prints_the_books_stat_table() {
    let mut d = design("car", "wheeled", "basic");
    d.name = "Street Racer".into();
    let text = sheet_text(&d);
    let r = run(&d);
    assert!(text.contains("(Street Racer) Tj"));
    // Parentheses inside a PDF string are escaped with a backslash; `r"…"`
    // is a raw string, where a backslash is just a backslash.
    assert!(text.contains(r"(\(car\)) Tj"));
    for header in ["HAND", "ACC", "SPD", "INT", "TOP", "BODY", "ARM", "PILOT", "SENS", "SEAT", "AVAIL", "COST"] {
        assert!(text.contains(&format!("({header}) Tj")), "{header}");
    }
    // The cost's ¥ is byte 0xA5 in the fonts' encoding, octal 245.
    let cost = format!(r"({},000\245) Tj", r.build_point_value);
    assert!(text.contains(&cost), "{cost}");
    assert!(text.contains(&format!("({}) Tj", r.stats.top_speed)));
}

#[test]
fn the_sheet_has_the_notes_as_its_description_and_the_design_after() {
    let mut d = design("car", "wheeled", "basic");
    d.notes = "A runner's getaway car.".into();
    let text = sheet_text(&d);
    assert!(text.contains("(A runner's getaway car.) Tj"));
    assert!(text.contains("(Design) Tj"));
    assert!(text.contains("(Build Points: Chassis: Car"));
    assert!(text.contains("(Design it: Engineering + Logic"));
}

#[test]
fn the_sheet_lists_the_designs_problems() {
    let mut d = design("car", "hover", "basic");
    d.improvements.handling = 3;
    assert!(sheet_text(&d).contains("Handling on road is -2"));
}

#[test]
fn long_notes_go_on_to_a_second_page() {
    let mut d = design("car", "wheeled", "basic");
    d.notes = "A long description of the vehicle. ".repeat(400);
    let text = sheet_text(&d);
    assert!(text.contains("/Count 2") || text.contains("/Count 3"));
}

#[test]
fn the_entrys_type_is_the_chassis_in_lowercase_keeping_acronyms() {
    let c = Catalog::builtin();
    let kind = |id: &str| crate::block::vehicle_type(c.chassis(id).unwrap());
    assert_eq!(kind("car"), "car");
    assert_eq!(kind("van"), "van/SUV");
    assert_eq!(kind("fixed-wing"), "fixed wing");
    assert_eq!(kind("lta"), "LTA (lighter than air)");
    assert_eq!(kind("drone-medium"), "medium drone");
}

#[test]
fn the_entrys_type_adds_a_specialized_function() {
    let mut d = with_qualities(design("van", "wheeled", "basic"), &["specialized-function"]);
    d.specialized_function = "ambulance".into();
    assert!(sheet_text(&d).contains(r"(\(van/SUV, ambulance\)) Tj"));
}

#[test]
fn text_is_encoded_for_the_pdf_fonts() {
    assert_eq!(pdf::win_ansi('A'), b'A');
    assert_eq!(pdf::win_ansi('¥'), 0xA5);
    assert_eq!(pdf::win_ansi('—'), 0x97);
    assert_eq!(pdf::win_ansi('’'), 0x92);
    assert_eq!(pdf::win_ansi('★'), b'?');
}

#[test]
fn wrapped_lines_fit_the_width() {
    let text = "The quick brown fox jumps over the lazy dog. ".repeat(10);
    let lines = pdf::wrap(&text, pdf::Font::Regular, 10.0, 200.0);
    assert!(lines.len() > 5);
    // Helvetica 10 pt averages about 5 points a character, so 200 points
    // holds about 40.
    assert!(lines.iter().all(|l| l.len() <= 45), "{lines:?}");
    assert_eq!(lines.join(" "), text.trim_end());
}

#[test]
fn a_sheet_is_only_written_as_a_pdf() {
    let d = design("car", "wheeled", "basic");
    let err = pdf::export(std::path::Path::new("car.bav"), &d, Catalog::builtin()).unwrap_err();
    assert!(err.contains("isn't a PDF"));
}

#[test]
fn a_sheet_is_written_to_its_file() {
    let dir = std::env::temp_dir().join(format!("bav-pdf-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("Car.pdf");
    pdf::export(&path, &design("car", "wheeled", "basic"), Catalog::builtin()).unwrap();
    assert!(std::fs::read(&path).unwrap().starts_with(b"%PDF"));
    std::fs::remove_dir_all(&dir).unwrap();
}
