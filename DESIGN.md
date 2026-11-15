# ShadowrunBAV — Design Overview

A Windows app for designing vehicles and drones from scratch by the rules of Shadowrun Sixth World's *Double Clutch*, "Build Your Dream" (pp. 147–157): pick a chassis, powertrain, and console, add design options, design modifications, and quality factors, and the app works out the vehicle's stats, Build Points, price, Availability, and the tests to design and build it.

## Goals

- A tool to share with other players and gamemasters. So it holds only the names and numbers the calculations need, never the book's rules text or descriptions; the book stays the reference for what things do.
- Windows first (Tauri also builds for macOS and Linux).
- The rules live in Rust, which the owner is learning; TypeScript is only the window's UI.
- Every rule has a test, and the book's three build examples are tests, as decided below where the book contradicts itself.

## Scope

- Version 1: designing from scratch ("Build Your Dream").
- Not yet: "Making It Yours" (pp. 119–146): mod slots and the roughly 80 aftermarket mods (accessories, chassis, powertrain, electronics, weapon mounts, secondary propulsion, vehicle cyberware), on a design or on a stock vehicle typed in. The design rules let a mod be built in as a design option at 2 Build Points per mod slot, plus the mod's price, so this comes next (see "Future features").
- Not covered: the book's stock vehicles. Their stats would be the book's data; a stock vehicle would be typed in when mods arrive.

## Stack

- Tauri 2 with a Rust core, the same layout as ShadowrunEdit:
  - `crates/bav-core/`: the rules, with no Tauri dependency, so `cargo test` is quick. `catalog.rs` reads the tables, `design.rs` is a design (the player's choices), `calc.rs` works it out, `files.rs` reads and writes design files.
  - `data/design.toml`: the book's tables, built into the program (`include_str!`).
  - `src-tauri/`: the window, exposing four commands (`catalog`, `evaluate_design`, `open_design`, `save_design`).
  - `src/`: the UI in strict TypeScript with no framework.
- A design file (`.bav`) is TOML holding the choices only, never results, so a file always follows the current rules: `format = 1`, then the design's fields. A field left out takes its default, so older or hand-edited files open. A file from a newer format is refused with a message. Saving writes a temporary file, flushes it, and renames it into place. Only `.bav` files are read or written.

## The window

- Left: the design. Name; Base (chassis grouped by ground, water, air, drones; powertrain grouped by medium; console), each option's label carrying its numbers; Design options, a level per functional improvement (a negative level is the improvement in reverse); Design modifications (engine type and Heavy Duty/Lightweight as radio buttons, the rest checkboxes); Quality factors (production and Security/Military as radio buttons; Environmental Adaptation asks for its environment and Specialized Function for what it's for); Notes.
- Right, recalculated on every change: the SR6 vehicle stat line (`Hand Accel Spd Int Top Spd Body Armor Pilot Sensor Seat Avail Cost`, a drone's Seat an em dash), with Copy (tab-separated, header and values, to paste into a document or ShadowrunEdit); problems in red; Size Class, cargo, speed modifier, and mod slots; notes on what comes with the choices; the speed calculation step by step; the Build Points line by line with the multipliers; and price, Availability, the design test, the parts' price, and the build test.
- A design that breaks a rule still calculates; the problems say what to fix.
- New, Open, Save, Save As (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Shift+S). Unsaved changes show as a dot after the file name, and New or Open asks before discarding them.

## The rules as calculated

All division rounds down unless noted.

- Chassis, powertrain, console: the tables' values (p. 150).
- Size Class: the chassis's, + Improved Size, up to 20, at most 2 levels down. A drone's size comes from its chassis (a different drone chassis), except that a large drone can be made bigger.
- Body: the chassis's + Improved Body (4 levels either way) + 2 per level of Improved Size, then Heavy Duty × 1.5 or Lightweight × 0.5. Below 0 is a problem.
- Armor: the chassis's + Improved Armor; more than twice the Body is a problem.
- Handling (lower is better): the chassis's, the powertrain's change (hover −2; tracked −2 off road only), Improved Handling overall (5 BP a level) and on or off road only (3 BP a level), at most 3 levels better or worse on each, then Off Road Chassis (+1 on road, −1 off road). Shown `on/off` when they differ.
- Seats: the chassis's + Improved Seating (at most half the chassis's seats either way), × 1.5 per level of Improved Size, then Cargo Hauler halves (to no fewer than 2) and Mass Transit triples.
- Cargo: the Size Class's (the Size Class table), plus a truck's 4 CF, + 2 CF per level of Improved Cargo Space (at most 3 levels, and no more than doubling it), then Cargo Hauler doubles it.
- Speed, in this order:
  1. The powertrain's Acceleration, Speed Interval, and Top Speed, plus Improved Acceleration (5 a level, 2 levels), Speed Interval (5, 3 levels), Top Speed (25, 3 levels).
  2. The speed modifier: final Body ÷ 2 + Armor (half the Armor with Integrated Armor or Cargo Hauler; they don't stack).
  3. Acceleration − speed modifier ÷ 2; Top Speed − speed modifier × 10, except for a walker. Speed Interval isn't lowered.
  4. Small drones' Acceleration × 0.75, mini × 0.5, micro × 0.25.
  5. Engine type: Racing +6 Acceleration and +100 Top Speed; High Performance +3 and +50 (half for a Cargo Hauler); Economy −3 and −50.
  6. Off Road Chassis: Top Speed − 10.
  7. Multi-Engine: Acceleration and Top Speed × 2 (× 1.5 for a Cargo Hauler, half the bonus).
  8. Underpowered: Top Speed ÷ 2.
  9. Below 0 is shown as 0, and flagged.
- Pilot and Sensor: the console's.
- Mod slots: the final Body in each of Chassis, Powertrain, and Electronics, plus 2 universal slots per level of Easily Modified (at most 4 levels or the chassis's Body; no reverse).
- Build Points: the chassis, powertrain, and console, each improvement's levels × its cost (negative in reverse), and the design modifications.
- Quality factors multiply together; a drone chassis always has the Drone quality (× 0.5). The Build Point Value is the Build Points × the multiplier. Price: Build Point Value × 1,000¥; parts to build it yourself, half that.
- Availability: 2, +1 for an aircraft (an air chassis or an air powertrain, so a rotor drone counts), +1 Luxury, +1(L) Security, +2(I) Military; I outranks L.
- Choices that exclude each other are flagged: one engine type; Heavy Duty or Lightweight; Mass Produced, One-off Custom, or Junkyard Frankenstein; Security or Military.
- Quality factors' own rules: Anthropomorphic needs the Walker powertrain; Superstructure needs Size Class 15; Environmental Adaptation needs an environment (a submersible is already underwater); Specialized Function asks what it's for. Security and Military note when the console is below Expert ("usually at least Expert").
- Design test: Engineering + Logic Extended, threshold Build Point Value ÷ 5 rounded up, with a 1-day interval; ÷ 2 rounded up for a week, ÷ 4 for a month. Build test: Engineering + Logic (20, 1 week), in a facility.

## Rules decisions

Where the book contradicts itself or leaves something open, the owner decided (2026-11-15):

- Top Speed: the speed modifier × 10 comes off the powertrain's base Top Speed. The rule's "base Acceleration" is a typo; every example uses Top Speed.
- The final Body sets the speed modifier, Improved Body (and Improved Size, Heavy Duty, Lightweight) included. Example 1 lowers Body to 9 but works its speed with Body 10 (Top Speed 160, Acceleration 11; its final stats say Body 10), so it's a slip: with Body 9 its Top Speed is 170, 270 with its racing engine.
- Rounding down applies to halving the speed modifier: Example 2's Acceleration is 40 − 9 = 31 (34 with its high performance engine, 68 with multi-engine), not the example's 30.
- Walker: the powertrain table's values (Acceleration 10, Speed Interval 15, Top Speed 30, 15 Build Points), not Example 3's 10, 40, and 10. Nothing else in Example 3 changes speed, and the secondary propulsion Walker (p. 137) agrees with the table.
- High Performance Engine: 15 Build Points, as its entry says; Example 2 charged 25 (the Racing Engine's), so its total is 163, not 173.
- Underpowered halves Top Speed only.
- The small-drone Acceleration multiplier comes right after the speed modifier, before the engine type and Multi-Engine.
- Quality factors apply to the Build Point Value everywhere, so the design test's threshold uses the multiplied value (Junkyard Frankenstein is labeled "× 0.5 BP").
- The console table's Special column (10, 12, --, --) is a layout error and left out, and so is the stray "0" at the end of the chassis table's Bike row.

Assumed, not yet confirmed with the owner (each is one place in `calc.rs`, with a test):

- The chassis table's Special column is kept for the Truck (+4 CF cargo) and the Submersible (built for underwater use).
- Heavy Duty and Lightweight apply after Improved Body and Improved Size.
- Improved Size's +50% seats apply after Improved Seating, and its cargo is the new Size Class's.
- Cargo Hauler halves the High Performance Engine's and Multi-Engine's bonuses, as written, and not the Racing Engine's.
- Off Road Chassis's −10 Top Speed comes with the engine type, before Multi-Engine.
- A drone chassis always has the Drone quality; it isn't a choice.
- An aircraft, for Availability, is an air chassis or an air powertrain.
- Easily Modified has no reverse, and its limit uses the chassis's Body.
- The takeoff and landing profiles (long, short, VTOL) and operation time aren't calculated: the book doesn't tie them to a chassis or a cost.

## Testing

- `cargo test -p bav-core`: a test per rule, named after it, the book's three examples (as decided above), every chassis × powertrain × console combination, and design files saved and reopened.
- `npm test`: the UI's text helpers.
- Test data is made up; the book's examples are restated as numbers only.

## Future features

| Feature | Notes |
|---|---|
| Making It Yours: mods | Mod slots by category (shifting 2:1), each mod's slots, rating, Availability, and price; installing it yourself (test, tools, half price). Mods built in as design options at 2 BP per slot |
| Stock vehicles | Type in a vehicle's stats to add mods to it |
| Weapon mounts | From Making It Yours: size, visibility, flexibility, control |
| Export to ShadowrunEdit | Today Copy gives the stat line; a `.md` with an `sr6-vehicle` block would carry the lines too |
| Installer and updates | A release build (`npm run tauri build`) and where to share it |
