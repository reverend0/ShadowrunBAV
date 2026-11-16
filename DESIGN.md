# ShadowrunBAV — Design Overview

A Windows app for designing vehicles and drones from scratch by the rules of Shadowrun Sixth World's *Double Clutch*, "Build Your Dream" (pp. 147–157): pick a chassis, powertrain, and console, add design options, design modifications, and quality factors, and the app works out the vehicle's stats, Build Points, price, Availability, and the tests to design and build it. Then add mods ("Making It Yours", pp. 120–144), installed afterward or built into the design.

## Goals

- A tool to share with other players and gamemasters. So it holds only the names and numbers the calculations need, never the book's rules text or descriptions; the book stays the reference for what things do.
- Windows first (Tauri also builds for macOS and Linux).
- The rules live in Rust, which the owner is learning; TypeScript is only the window's UI.
- Every rule has a test, and the book's three build examples are tests, as decided below where the book contradicts itself.

## Scope

- Designing from scratch ("Build Your Dream"), and mods on the design ("Making It Yours"): accessories, chassis, skin, powertrain, secondary propulsion, electronics, hardpoints, and hardpoint devices.
- Not covered: the book's stock vehicles (their stats are the book's data; one could be typed in later), trailers and sidecars (vehicles of their own), the rigger cyberware and autosofts (not vehicle mods), and redlining (a driving rule).
- Open source under the MIT License (decided, 2026-11-15).

## Stack

- Tauri 2 with a Rust core, the same layout as ShadowrunEdit:
  - `crates/bav-core/`: the rules, with no Tauri dependency, so `cargo test` is quick. `catalog.rs` reads the tables, `design.rs` is a design (the player's choices), `calc.rs` works it out, `mods.rs` works out the mods on it, `block.rs` lays out the stat block, `files.rs` reads and writes design files, `pdf.rs` writes the vehicle sheet.
  - `data/design.toml` and `data/mods.toml`: the book's tables, built into the program (`include_str!`).
  - `src-tauri/`: the window, exposing five commands (`catalog`, `evaluate_design`, `open_design`, `save_design`, `export_pdf`), and the updater.
  - `src/`: the UI in strict TypeScript with no framework.
- A design file (`.bav`) is TOML holding the choices only, never results, so a file always follows the current rules: `format = 1`, then the design's fields. A field left out takes its default, so older or hand-edited files open; a design option level past ±100 is worked out as ±100 (flagged as past its limit), so a wild number can't break the arithmetic. A file from a newer format is refused with a message. Saving writes a temporary file, flushes it, and renames it into place. Only `.bav` files are read or written.

## The window

- Left: the design. Name; Base (chassis grouped by ground, water, air, drones; powertrain grouped by medium; console, and the stat block's layout), each option's label carrying its numbers; Design options, a level per functional improvement (a negative level is the improvement in reverse); Design modifications (engine type and Heavy Duty/Lightweight as radio buttons, the rest checkboxes); Quality factors (production and Security/Military as radio buttons; Environmental Adaptation asks for its environment and Specialized Function for what it's for); Mods (Add picks one from a list grouped as the chapter is; each has its rating, a count for those installed more than once, Built in, a note, and ×); Weapons (each typed in: name, type, DV, firing modes, five Attack Ratings, its own mods, and the weapon mount it's in, from the vehicle's; Add a weapon puts a new one in the first mount); Notes.
- Right, recalculated on every change: the stat block (see "The stat block"), with Copy, and under it, when there are mods, the stat line as designed; problems in red; Size Class, cargo, speed modifier, and mod slots; notes on what comes with the choices; the speed calculation step by step; the Build Points line by line with the multipliers; and price, Availability, the design test, the parts' price, and the build test; then the mods: each with its slots (and CF), Availability, price, and its do-it-yourself price and install test (or its Build Points, built in), then the slots used of each category's, the hardpoints, the skin's form, cargo taken, a secondary propulsion's stats, and the prices.
- A design that breaks a rule still calculates; the problems say what to fix.
- New, Open, Save, Save As, Export PDF (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Shift+S, Ctrl+P). Unsaved changes show as a dot after the file name, and New or Open asks before discarding them.
- Export PDF writes a vehicle sheet (`pdf.rs`), US Letter, laid out like the book's vehicle entries: the name in bold, the type in italic parentheses (the chassis in lowercase, acronyms kept, `medium drone` for a drone chassis, a Specialized Function after a comma), then the stat block: its rows, their headers as the book prints them (`HAND ACC SPD/INT TOP/SPD BODY ARM PILOT SENS SEAT AVAIL COST`, SPD over INT; `DR I/ID AC CM` for a gamemaster) white on a dark band, and its lines with their labels bold, weapons one per line under "Weapons:"; then a secondary propulsion's stats, and the Notes as the description. A Design section follows: the Build Points line by line, the quality factors and Build Point Value, Size Class, cargo, and mod slots, the price, parts, and Availability, the design and build tests, the notes on what comes with the choices, and any problems. The PDF is written by hand in the standard Helvetica fonts, so no font is embedded and nothing is added to the dependencies. Only `.pdf` files are written, through a temporary file like a design.

## Updates

- The app checks this repository's latest GitHub release for a newer version in the background, 10 seconds after it starts and every 6 hours after (`src/updater.ts`). A found update never interrupts: an "Update to X" button appears on the toolbar. Clicking it asks once, saves the design (Save As for an untitled one, or asks to discard its changes), installs, and restarts. A failed check (offline) says nothing.
- The toolbar also shows the version and Check for updates, which checks right away and says what it found.
- Updates are signed with the project's own key (Tauri's updater, not code signing), and the app installs only an update that matches the public key it ships with. Each release carries `latest.json`, the installers, and their signatures (`RELEASING.md`). Drafts and pre-releases are never offered, since GitHub's "latest" release skips them.
- Version 0.1.1 has no updater, so its users install the next version by hand.

## The rules as calculated

All division rounds down unless noted.

- Chassis, powertrain, console: the tables' values (p. 150).
- Size Class: the chassis's, + Improved Size, up to 20, at most 2 levels down. A drone's size comes from its chassis (a different drone chassis), except that a large drone can be made bigger.
- Body: the chassis's + Improved Body (4 levels either way) + 2 per level of Improved Size, then Heavy Duty × 1.5 or Lightweight × 0.5. Below 0 is a problem.
- Armor: the chassis's + Improved Armor; more than twice the Body is a problem.
- Handling (lower is better): the chassis's, the powertrain's change (hover −2; tracked −2 off road only), Improved Handling overall (5 BP a level) and on or off road only (3 BP a level), at most 3 levels better or worse on each, then Off Road Chassis (+1 on road, −1 off road). Shown `on/off` when they differ. Below 1 is shown as 1, and flagged.
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
  9. Acceleration or Top Speed below 0 is shown as 0, and a Speed Interval below 1 as 1; either is flagged.
- Pilot and Sensor: the console's.
- Mod slots: the final Body in each of Chassis, Powertrain, and Electronics, plus 2 universal slots per level of Easily Modified (at most 4 levels or the chassis's Body; no reverse).
- Build Points: the chassis, powertrain, and console, each improvement's levels × its cost (negative in reverse), and the design modifications.
- Quality factors multiply together; a drone chassis always has the Drone quality (× 0.5). The Build Point Value is the Build Points × the multiplier. Price: Build Point Value × 1,000¥; parts to build it yourself, half that.
- Availability: 2, +1 for an aircraft (an air chassis or an air powertrain, so a rotor drone counts), +1 Luxury, +1(L) Security, +2(I) Military; I outranks L.
- Choices that exclude each other are flagged: one engine type; Heavy Duty or Lightweight; Mass Produced, One-off Custom, or Junkyard Frankenstein; Security or Military.
- Quality factors' own rules: Anthropomorphic needs the Walker powertrain; Superstructure needs Size Class 15; Environmental Adaptation needs an environment (a submersible is already underwater); Specialized Function asks what it's for. Security and Military note when the console is below Expert ("usually at least Expert").
- Design test: Engineering + Logic Extended, threshold Build Point Value ÷ 5 rounded up, with a 1-day interval; ÷ 2 rounded up for a week, ÷ 4 for a month. Build test: Engineering + Logic (20, 1 week), in a facility.

## The stat block

The vehicle as ShadowrunEdit's `sr6-vehicle` block lays it out (`block.rs`), so it reads the same in both apps, and Copy pastes it into ShadowrunEdit (or a document): the title with its type in parentheses, the rows tab-separated, then the lines.

- Two layouts, chosen per design, as ShadowrunEdit's modes are (decided): Gear book, the main row with Avail and Cost, and the mods as "Standard Equipment"; or Gamemaster, the main row without them, a second row (DR `+Armor (Armor + Pilot)`; a drone's own I/ID Pilot × 2 with 4D6 and AC A1, I5, a vehicle's an em dash since it uses its driver's; CM Body ÷ 2 rounded up + 8), and the mods as "Equipment". A file without a layout is Gear book.
- Every mod, built in or installed afterward, is on that line (decided): in name order, in lowercase but for the first entry and acronyms and names (RAM, ECM, GridLink, xGuide, Valkyrie), a rating after the name, and a count and note in parentheses: `RAM plating 2, spoof kit, standard weapon mount (2, roof)`.
- Weapons: one per line under "Weapons:", in name order, as ShadowrunEdit writes them: `Ares Alpha [Assault Rifles, DV 4P, SA/BF/FA, 250(c), 5/12/11/8/4, w/ smartgun system] (standard mount)`. They're typed in (decided; no weapon list ships with the app). The mount sets the ammunition (small 100(c), standard 250(c), large 500(b)) and adds 2 to each Attack Rating in range (p. 142); an Attack Rating of 0 is out of range, an em dash. Each weapon needs a weapon mount, one weapon to a mount, and more weapons in a kind of mount than the vehicle has of it is flagged. The mount's own hardpoint is checked with the mods.

## Mods

"Making It Yours" (pp. 120–144), in `mods.rs` with the table in `data/mods.toml`. Mods go on the vehicle as designed (its "factory" stats).

- Each mod: its slots (fixed, per rating, per Body rounded up, or per Armor), Cargo Factors, Availability (fixed or per rating), and price (fixed, per rating, per Body, per Armor, or per rating × Body); a mod the book tabulates by rating (milspec plate, anti-theft, gecko tips, winch, living space) has a row per rating. Tires have a second price for bikes and drones. A count for a mod installed more than once (gun ports, weapon mounts); a note for what it's for.
- Installed afterward: the shop price, or half doing it yourself, except accessories (no discount) and a portable facility (its 50,000¥ facility isn't halved). The install test is Engineering + Logic, threshold mod slots × 8, interval mod slots × hours (a mod without slots counts as 1), with a toolkit under 2 slots, a shop for 2 to 5, a facility for 6 or more.
- Built in (p. 151): 2 Build Points a slot, rounded up, a "Built in" line in the Build Points; its price is added to the vehicle's, and the parts are half of that total (Example 3).
- Mod slots: the designed Body in each of Chassis, Powertrain, and Electronics; Increased Structural Integrity's Body doesn't add slots. Slots past a category's are covered first by Easily Modified's universal slots, then by spare slots elsewhere at 2 for 1; anything left is flagged. A suncell takes whichever of powertrain or electronics has more room. Removed Manual Controls and stripping the form free chassis slots.
- Hardpoints, counted in standard hardpoints (small ½, large 2, huge 3): Body ÷ 3 free (a small drone over Body 3 gets one small one instead), plus those added; devices (weapon mounts, drone racks, arms, ram plate, winch) use them, and more than there are is flagged. A microdrone can't add hardpoints; small drones and minidrones add only small ones.
- Cargo: CF mods (seating, living space, portable facility, rigger cocoon, smuggling compartment, Valkyrie module) take the vehicle's cargo space; more than it has is flagged, and Increased Cargo Space adds 2 CF a rating (a slot each).
- What mods change, from the factory stats: Increased Structural Integrity +Body (rating up to half the factory Body); standard armor +Armor (up to twice the Body added, or all of it removed), −1 Acceleration per 2 points and −10 Top Speed per point (removed armor gives them back); milspec plate −5 Speed Interval a rating; Improved Handling and Pilot-By-Wire −Handling (they don't stack; Pilot-By-Wire needs Pilot twice its rating); multi-terrain suspension (static) moves off-road Handling points on road until they're level; Improved Acceleration + 30% and Top Speed Increase + 20% of the factory value a rank, rounded up; Improved Pilot and Enhanced Sensors replace Pilot and Sensor; Increased Seating + seats; squatter amenities double them. Handling stays at least 1 (p. 125), Speed Interval at least 1, Acceleration and Top Speed at least 0, and a drone has no seats.
- Forms (p. 131): a chassis is standard, open (bike, personal watercraft), or totally enclosed (drones). Changing it: a slot per level more (6,000¥ a level, Availability 3) or freed per level less (4,000¥, Availability 2); more skin needs Armor at least the Body (standard) or 1.5 × Body (enclosed). Life safety and chameleon coating need standard or enclosed; grey mana warding needs enclosed.
- Secondary propulsion: its own Handling, Acceleration, Speed Interval, and Top Speed, shown beside the stat line; one at a time.
- Only one of each group: tires, amenities, form, life safety level, multi-terrain suspension, grey mana, secondary propulsion. Racing suspension doesn't work with improved stability or tuned suspension. Mods for passenger vehicles, wheeled vehicles, or drone sizes are flagged on others.
- Availability: the highest of the vehicle's and its mods' (p. 155), I outranking L. The stat line's Cost is the vehicle's price plus the mods installed afterward.
- Tires, active multi-terrain suspension, and a few others note what they do rather than change the stat line (their bonuses depend on the road).

## Rules decisions

Where the book contradicts itself or leaves something open, the owner decided (2026-11-15):

- Top Speed: the speed modifier × 10 comes off the powertrain's base Top Speed. The rule's "base Acceleration" is a typo; every example uses Top Speed.
- The final Body sets the speed modifier, Improved Body (and Improved Size, Heavy Duty, Lightweight) included. Example 1 lowers Body to 9 but works its speed with Body 10 (Top Speed 160, Acceleration 11; its final stats say Body 10), so it's a slip: with Body 9 its Top Speed is 170, 270 with its racing engine.
- Rounding down applies to halving the speed modifier: Example 2's Acceleration is 40 − 9 = 31 (34 with its high performance engine, 68 with multi-engine), not the example's 30.
- Walker: the powertrain table's values (Acceleration 10, Speed Interval 15, Top Speed 30, 15 Build Points), not Example 3's 10, 40, and 10. Nothing else in Example 3 changes speed, and the secondary propulsion Walker (p. 137) agrees with the table.
- High Performance Engine: 15 Build Points, as its entry says; Example 2 charged 25 (the Racing Engine's), so its total is 163, not 173.
- The design test's threshold, Build Point Value ÷ 5 (and ÷ 2 or ÷ 4 for a week or a month), rounds up, as Examples 1 and 2 do (53 gives 11; 173 gives 35, 18 by the week). Example 3 rounds its 67 down to 13.
- Handling and Speed Interval are at least 1: the book gives no floor, and Handling is a test's threshold. Reverse options (p. 151) can otherwise take them to 0 or below (a car on hover with Improved Handling 3; a sail with Improved Speed Interval −3).
- Underpowered halves Top Speed only.
- The small-drone Acceleration multiplier comes right after the speed modifier, before the engine type and Multi-Engine.
- Quality factors apply to the Build Point Value everywhere, so the design test's threshold uses the multiplied value (Junkyard Frankenstein is labeled "× 0.5 BP").
- The console table's Special column (10, 12, --, --) is a layout error and left out, and so is the stray "0" at the end of the chassis table's Bike row.

Mods (decided 2026-11-15):

- Increased Structural Integrity's rating is up to half the factory Body, as its example and its box say (a Body 4 bike gets 2), not 1.5 × as its sentence reads.
- Improved Pilot's rating replaces the Pilot, 1 to 9, as its text says (its box says "–", and the 1–9 sits in xGuide Override's box, a mod with no rating); its slots and price are for the full rating. Enhanced Sensors works the same way for Sensor.
- Medevac Modifications cost a flat 3,500¥: it has no rating, so its "× Rating" is a slip.
- Improved Economy/Capacity takes a slot per rating ("1–3").
- Gun Ports take 0.5 slots each: the box's "Rating 0.5" is the slots in the wrong column.
- Weapon mounts are priced by their table. Example 1's 7,300¥ of mods and Example 3's 33,000¥ leave them out.
- Run-flat on racing or off-road tires is +50% of their price, as the tires' entry says, not Example 1's 1,800¥ (racing tires plus run-flat tires).
- A built-in mod uses the vehicle's mod slots too, as well as its 2 Build Points a slot.
- Built-in mods add their Build Points: Example 3's 67 leaves out the 12 it gives for Realistic Features 4 and Chameleon Coating 2, so it's 84 (79 with its walker at 10).

Assumed, not yet confirmed with the owner (each is one place in `calc.rs` or `mods.rs`, with a test):

- The chassis table's Special column is kept for the Truck (+4 CF cargo) and the Submersible (built for underwater use).
- Heavy Duty and Lightweight apply after Improved Body and Improved Size.
- Improved Size's +50% seats apply after Improved Seating, and its cargo is the new Size Class's.
- Cargo Hauler halves the High Performance Engine's and Multi-Engine's bonuses, as written, and not the Racing Engine's.
- Off Road Chassis's −10 Top Speed comes with the engine type, before Multi-Engine.
- A drone chassis always has the Drone quality; it isn't a choice.
- An aircraft, for Availability, is an air chassis or an air powertrain.
- Easily Modified has no reverse, and its limit uses the chassis's Body.
- The takeoff and landing profiles (long, short, VTOL) and operation time aren't calculated: the book doesn't tie them to a chassis or a cost.
- Forms: a bike and a personal watercraft start open, drones totally enclosed, everything else standard.
- Standard armor costs a walker Top Speed too: the armor rule (p. 133) has no exception, unlike the design's speed modifier.
- Improved Acceleration and Top Speed Increase add a share of the designed vehicle's own Acceleration and Top Speed, each rank rounded up (Improved Acceleration 2 on Acceleration 11 is +8, not 30% × 2 = 6.6 rounded to 7).
- Easy Assembly's ⅓ Body and Powered Breakdown's ½ Body slots round up (Easy Assembly says so; Powered Breakdown doesn't).
- Anti-theft rating 1 comes with any Pilot, so it's free.
- Living space costs a month of its lifestyle per occupant at the core rulebook's prices (Squatter 500¥, Low 2,000¥, Middle 5,000¥, High 10,000¥, Luxury 100,000¥).
- Increased Seating takes cargo space (1 CF a seat); the "or 0.5 slots" in its box is the cargo space Increased Cargo Space would make, so it's added that way.
- Elemental hardening has no Availability (its box says "--").

## Testing

- `cargo test -p bav-core`: a test per rule, named after it, the book's three examples (as decided above, Examples 1 and 3 again with their mods), every chassis × powertrain × console combination, every mod on a car and a drone, and design files saved and reopened.
- `npm test`: the UI's text helpers and the update checks.
- Test data is made up; the book's examples are restated as numbers only.

## Future features

| Feature | Notes |
|---|---|
| Stock vehicles | Type in a vehicle's stats to add mods to it |
| Trailers and sidecars | Their own small profiles (p. 127) and the towing rules |
| Operation time | The base 6 hours, Improved Economy/Capacity's +33% a rank, and what drains it |
| Export to ShadowrunEdit | Copy gives the stat block to paste; a `.md` with an `sr6-vehicle` block would open as one, ready to edit |
| Large mounts with rockets or missiles | A large mount holds as many as the vehicle's Body (p. 142); today every large mount shows 500(b) |
