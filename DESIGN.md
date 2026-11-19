# ShadowrunBAV — Design Overview

A Windows app for designing vehicles and drones from scratch by the rules of Shadowrun Sixth World's *Double Clutch*, "Build Your Dream" (pp. 147–157): pick a chassis, powertrain, and console, add design options, design modifications, and quality factors, and the app works out the vehicle's stats, Build Points, price, Availability, and the tests to design and build it. Then add mods ("Making It Yours", pp. 120–144), installed afterward or built into the design.

## Goals

- A tool to share with other players and gamemasters. So it holds only the names and numbers the calculations need, never the book's rules text or descriptions; the book stays the reference for what things do.
- Windows first (Tauri also builds for macOS and Linux).
- The rules live in Rust, which the owner is learning; TypeScript is only the window's UI.
- Every rule has a test, and the book's three build examples are tests, as decided below where the book contradicts itself.

## Scope

- Designing from scratch ("Build Your Dream"), and mods on the design ("Making It Yours"): accessories, chassis, skin, powertrain, secondary propulsion, electronics, hardpoints, and hardpoint devices.
- Other books, each turned on per design: Deadly Arts' mil-spec design options and its two mods, Smooth Operations' drone-mounted tag sprayer, Explosive Decompression's Spaceworthy, and Sixth World Companion's vehicle packs (see "Other books").
- Not covered: the book's stock vehicles (their stats are the book's data; one could be typed in later), trailers and sidecars (vehicles of their own), the rigger cyberware and autosofts (not vehicle mods), and redlining (a driving rule).
- Open source under the MIT License (decided, 2026-11-15).

## Stack

- Tauri 2 with a Rust core, the same layout as ShadowrunEdit:
  - `crates/bav-core/`: the rules, with no Tauri dependency, so `cargo test` is quick. `catalog.rs` reads the tables, `design.rs` is a design (the player's choices), `calc.rs` works it out, `mods.rs` works out the mods on it, `milspec.rs` Deadly Arts' mil-spec options, `block.rs` lays out the stat block, `files.rs` reads and writes design files, `pdf.rs` writes the vehicle sheet.
  - `data/design.toml` and `data/mods.toml`: the book's tables, built into the program (`include_str!`).
  - `src-tauri/`: the window, exposing five commands (`catalog`, `evaluate_design`, `open_design`, `save_design`, `export_pdf`), and the updater.
  - `src/`: the UI in strict TypeScript with no framework.
- A design file (`.bav`) is TOML holding the choices only, never results, so a file always follows the current rules: `format = 1`, then the design's fields. A field left out takes its default, so older or hand-edited files open; a design option level past ±100 is worked out as ±100 (flagged as past its limit), so a wild number can't break the arithmetic. A file from a newer format is refused with a message. Saving writes a temporary file, flushes it, and renames it into place. Only `.bav` files are read or written.

## The window

- Left: the design. Name; Books (a box per book beyond Double Clutch); Base (chassis grouped by ground, water, air, drones; powertrain grouped by medium; console, and the stat block's layout), each option's label carrying its numbers; Design options, a level per functional improvement (a negative level is the improvement in reverse); Design modifications (engine type and Heavy Duty/Lightweight as radio buttons, the rest checkboxes); Quality factors (production and Security/Military as radio buttons; Environmental Adaptation asks for its environment and Specialized Function for what it's for); Mil-spec, when Deadly Arts is on (see "Other books"); Mods (whether built-in mods use mod slots, then the mod picker: see "Picking mods"); Weapons (each typed in: name, type, DV, firing modes, five Attack Ratings, its own mods, and the weapon mount it's in, from the vehicle's; Add a weapon puts a new one in the first mount); Notes.
- Right, recalculated on every change: the vehicle's picture, if it has one (Add a picture…, Change picture…, Remove picture); the stat block (see "The stat block"), with Copy, and under it, when mods are installed afterward, the stat line as built (the design with its built-in mods, at its price and their Availability); problems in red; Size Class, cargo, speed modifier, and mod slots; notes on what comes with the choices; the speed calculation step by step; the Build Points line by line with the multipliers; and price, Availability, the design test, the parts' price, and the build test; then the mods: each with its slots (and CF), Availability, price, and its do-it-yourself price and install test (or its Build Points, built in), then the slots used of each category's, the hardpoints, the skin's form, cargo taken, a secondary propulsion's stats, and the prices.
- A design that breaks a rule still calculates; the problems say what to fix.
- New, Open, Save, Save As, Export PDF (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Shift+S, Ctrl+P). Unsaved changes show as a dot after the file name, and New or Open asks before discarding them.
- Closing the window with unsaved changes asks first, “Save your changes to “Raptor.bav” before closing?” (“this design” when it has no file yet), with Save (Save As for a design without a file; the window stays open if that's cancelled or fails), Don’t Save, and Cancel (`src/closing.ts`). New and Open already ask before discarding changes.
- Export PDF writes a vehicle sheet (`pdf.rs`), US Letter, laid out like the book's vehicle entries: the name in bold, the type in italic parentheses (the chassis in lowercase, acronyms kept, `medium drone` for a drone chassis, a Specialized Function after a comma), then the stat block: its rows, their headers as the book prints them (`HAND ACC SPD/INT TOP/SPD BODY ARM PILOT SENS SEAT AVAIL COST`, SPD over INT; `DR I/ID AC CM` for a gamemaster) white on a dark band, and its lines with their labels bold, weapons one per line under "Weapons:"; then a secondary propulsion's stats, and the Notes as the description. A Design section follows: the Build Points line by line, the quality factors and Build Point Value, Size Class, cargo, and mod slots, the price, parts, and Availability, the design and build tests, the notes on what comes with the choices, and any problems. The PDF is written by hand in the standard Helvetica fonts, so no font is embedded and nothing is added to the dependencies. Only `.pdf` files are written, through a temporary file like a design.

## Updates

- The app checks this repository's latest GitHub release for a newer version in the background, 10 seconds after it starts and every 6 hours after (`src/updater.ts`). A found update never interrupts: an "Update to X" button appears on the toolbar. Clicking it asks once, saves the design (Save As for an untitled one, or asks to discard its changes), installs, and restarts. A failed check (offline) says nothing.
- The toolbar also shows the version and Check for updates, which checks right away and says what it found.
- Updates are signed with the project's own key (Tauri's updater, not code signing), and the app installs only an update that matches the public key it ships with. Each release carries `latest.json`, the installers, and their signatures (`RELEASING.md`). Drafts and pre-releases are never offered, since GitHub's "latest" release skips them.
- Version 0.1.1 has no updater, so its users install the next version by hand.

## The vehicle's picture

A picture the player loads (PNG, JPEG, or WebP) shows above the stat block. It's kept in the design file as a data URL (decided, 2026-11-19), so it travels with the design: shrunk to 1,000 pixels on its longest side, a PNG kept as a PNG (for its transparency) and anything else saved as a JPEG (`src/picture.ts`). A file's picture that isn't a PNG or JPEG data URL, or is over 3 MB, is left out and flagged. The PDF sheet doesn't print it.

## Picking mods

The Mods section shows a wireframe of a vehicle, isometric, with its mod zones tinted (decided, 2026-11-19). Each zone is one of the chapter's groups and the budget it draws on (p. 120):

- Chassis (chassis and skin mods): the body.
- Powertrain (powertrain and secondary propulsion): the hood and wheels.
- Electronics: the cabin.
- Seating and cargo: the rear.
- Hardpoints (hardpoints and the devices on them): the roof mount.
- Accessories use no slots, so they're a chip under the drawing, not a part of it.

Clicking a part of the drawing, or its chip, lists that zone's mods, each with its slots and price; ticking one adds it (at its lowest rating) and shows its rating, count, Built in, note, and ×, and unticking removes it. Each chip shows how full its zone is ("3 of 10 slots", hardpoints, CF of cargo), red when it's over. A book's mods are listed when the book is on, or when the design already has them, marked so.

- One drawing for every vehicle for now, an original SUV-type body built in code (`scripts/vehicle-art.html`, three.js in a browser), saved as pictures in `public/vehicles/`: one for the light theme, one for the dark, and a zone map in flat colors that the window reads to tell which zone was clicked (`src/zones.ts`). The app itself needs no 3D library. Other kinds of vehicle (bike, boat, aircraft, drone) can get their own drawings later.

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

- Two layouts, chosen per design, as ShadowrunEdit's modes are (decided): Gear book, the main row with Avail and Cost, and the built-in mods as "Standard Equipment"; or Gamemaster, the main row without them, a second row (DR `+Armor (Armor + Pilot)`; a drone's own I/ID Pilot × 2 with 4D6 and AC A1, I5, a vehicle's an em dash since it uses its driver's; CM Body ÷ 2 rounded up + 8), and the built-in mods as "Equipment". A file without a layout is Gear book.
- Mods built into the design (and Deadly Arts' mil-spec options) are on that line, and mods installed afterward on a "Modifications" line after it, as ShadowrunEdit's vehicle block has both (decided, 2026-11-19; until then every mod was standard equipment). Each in name order, in lowercase but for the first entry and acronyms and names (RAM, ECM, GridLink, xGuide, Valkyrie), a rating after the name, and a count and note in parentheses: `RAM plating 2, spoof kit, standard weapon mount (2, roof)`.
- Weapons: one per line under "Weapons:", in name order, as ShadowrunEdit writes them: `Ares Alpha [Assault Rifles, DV 4P, SA/BF/FA, 250(c), 5/12/11/8/4, w/ smartgun system] (standard mount)`. They're typed in (decided; no weapon list ships with the app). The mount sets the ammunition (small 100(c), standard 250(c), large 500(b)) and adds 2 to each Attack Rating in range (p. 142); an Attack Rating of 0 is out of range, an em dash. Each weapon needs a weapon mount, one weapon to a mount, and more weapons in a kind of mount than the vehicle has of it is flagged. The mount's own hardpoint is checked with the mods.

## Mods

"Making It Yours" (pp. 120–144), in `mods.rs` with the table in `data/mods.toml`. Mods go on the vehicle as designed (its "factory" stats).

- Each mod: its slots (fixed, per rating, per Body rounded up, or per Armor), Cargo Factors, Availability (fixed or per rating), and price (fixed, per rating, per Body, per Armor, or per rating × Body); a mod the book tabulates by rating (milspec plate, anti-theft, gecko tips, winch, living space) has a row per rating. Tires have a second price for bikes and drones. A count for a mod installed more than once (gun ports, weapon mounts); a note for what it's for.
- Installed afterward: the shop price, or half doing it yourself, except accessories (no discount) and a portable facility (its 50,000¥ facility isn't halved). The install test is Engineering + Logic, threshold mod slots × 8, interval mod slots × hours (a mod without slots counts as 1), with a toolkit under 2 slots, a shop for 2 to 5, a facility for 6 or more.
- Built in (p. 151): 2 Build Points a slot, rounded up, a "Built in" line in the Build Points; its price is added to the vehicle's, and the parts are half of that total (Example 3). Whether built-in mods also use the vehicle's mod slots is a choice per design, on by default (RULINGS.md R19).
- Mod slots: the designed Body in each of Chassis, Powertrain, and Electronics; Increased Structural Integrity's Body doesn't add slots. Slots past a category's are covered first by Easily Modified's universal slots, then by spare slots elsewhere at 2 for 1; anything left is flagged. A suncell takes whichever of powertrain or electronics has more room. Removed Manual Controls and stripping the form free chassis slots.
- Hardpoints, counted in standard hardpoints (small ½, large 2, huge 3): Body ÷ 3 free (a small drone over Body 3 gets one small one instead), plus those added; devices (weapon mounts, drone racks, arms, ram plate, winch) use them, and more than there are is flagged. A microdrone can't add hardpoints; small drones and minidrones add only small ones.
- Cargo: CF mods (seating, living space, portable facility, rigger cocoon, smuggling compartment, Valkyrie module) take the vehicle's cargo space; more than it has is flagged, and Increased Cargo Space adds 2 CF a rating (a slot each).
- What mods change, from the factory stats: Increased Structural Integrity +Body (rating up to half the factory Body); standard armor +Armor (up to twice the Body added, or all of it removed), −1 Acceleration per 2 points and −10 Top Speed per point (removed armor gives them back); milspec plate −5 Speed Interval a rating; Improved Handling and Pilot-By-Wire −Handling (they don't stack; Pilot-By-Wire needs Pilot twice its rating); multi-terrain suspension (static) moves off-road Handling points on road until they're level; Improved Acceleration + 30% and Top Speed Increase + 20% of the factory value a rank, rounded up; Improved Pilot and Enhanced Sensors replace Pilot and Sensor; Increased Seating + seats; squatter amenities double them. Handling stays at least 1 (p. 125), Speed Interval at least 1, Acceleration and Top Speed at least 0, and a drone has no seats.
- Forms (p. 131): a chassis is standard, open (bike, personal watercraft), or totally enclosed (drones). Changing it: a slot per level more (6,000¥ a level, Availability 3) or freed per level less (4,000¥, Availability 2); more skin needs Armor at least the Body (standard) or 1.5 × Body (enclosed). Life safety and chameleon coating need standard or enclosed; grey mana warding needs enclosed.
- Secondary propulsion: its own Handling, Acceleration, Speed Interval, and Top Speed, shown beside the stat line; one at a time.
- Only one of each group: tires, amenities, form, life safety level, multi-terrain suspension, grey mana, secondary propulsion. Racing suspension doesn't work with improved stability or tuned suspension. Mods for passenger vehicles, wheeled vehicles, or drone sizes are flagged on others.
- Availability: the highest of the vehicle's and its mods' (p. 155), I outranking L. The stat line's Cost is the vehicle's price plus the mods installed afterward.
- Tires, active multi-terrain suspension, and a few others note what they do rather than change the stat line (their bonuses depend on the road).

## Other books

Rules from books beyond *Double Clutch* are turned on per design, under Books, and saved with it (`books = ["deadly-arts"]`; a file without it uses Double Clutch alone). Off, a book's options are hidden; any already in the design stay in the file but are left out of the calculations and flagged, so turning it back on brings them back.

Deadly Arts, "Mil-Spec Mayhem" (pp. 150–152), in `milspec.rs`, its Design Points counted as Build Points (decided, RULINGS.md R21):

- Airframe ceramic plate: 2 Build Points a point of the designed Armor; noted as half the Armor in bonus dice.
- Heavy mil-spec plate, rating 4 to 6: 5 a rating, and Speed Interval −5 a rating.
- Mil-spec comms I, II, III: 5, 20, 30.
- Mil-spec frame, 1 to 12, at most half the designed Body: 10 + 5 a rating. It adds its rating to Body, as Increased Structural Integrity does (no mod slots), and every 2 points offset a level of mil-spec plate's Speed Interval loss, heavy or Double Clutch's. It doesn't go with Increased Structural Integrity.
- Only one kind of specialty armor: airframe ceramic plate, heavy mil-spec plate, and Double Clutch's ceramic and milspec plate.
- Mil-spec hardpoints: each sized for its weapon. The Grade is the weapon's Size, −2 inline or open, +1 pop-out, +1 twin-linked or +2 quad-linked (never below 0); it costs 5 a Grade, another 5 a Grade for pop-out, and +3 twin-linked or +5 quad-linked; and it needs Body 5 + 5 × Grade (a primary weapon's, a Grade less). Pop-out isn't for inline mounts. A weapon on one keeps its own ammunition and Attack Ratings, and a hardpoint holds one weapon, or its 2 or 4 linked. The stat block lists them as equipment: `mil-spec hardpoint (Grade 3, pop-out turret)`.
- Any of these makes it a military vehicle: the build test's threshold is 28 (decided, R23).
- Its mods: MDS (Missile Defense System), 2 electronics slots, and Mobile Magic Lodge, 50 CF (decided, R22).

Smooth Operations (p. 124): the drone-mounted tag sprayer installed permanently, one chassis slot, no Availability, 1,000¥ (decided, R24), for microdrones (1,000 tags) and minidrones (10,000). The clip-on version is gear, not a mod.

Explosive Decompression (p. 97): Spaceworthy, for drones. It changes no stats, takes no slots, and costs the drone's own price, so the drone ends up costing twice as much (decided, R25); built in, it doubles the drone's price.

Sixth World Companion (p. 71): its four vehicle packs (Dirty Tricks, Gun Ports, Rigger's Baby, Combat Biker). Picking one under Mods adds each of its mods, priced as usual, not at the pack's price (decided, R26); the list shows a pack's Body minimum. Its autosoft packs are software, not vehicle mods. Packs are `[[pack]]` entries in `data/mods.toml`.

## Rules decisions

Every place the book disagrees with itself or with its examples, what the app does instead, the examples corrected, and the assumptions not yet confirmed are in [RULINGS.md](RULINGS.md), numbered R1, R2, and so on. A new ruling goes there, with a test, and "(decided)" in this file points to one.

## Testing

- `cargo test -p bav-core`: a test per rule, named after it, the book's three examples (as decided above, Examples 1 and 3 again with their mods), every chassis × powertrain × console combination, every mod on a car and a drone, Deadly Arts' options and turning it on and off (`milspec_tests.rs`), the smaller books' mods and packs (`book_tests.rs`), and design files saved and reopened.
- `npm test`: the UI's text helpers, the mod picker's zones, and the update checks.
- Test data is made up; the book's examples are restated as numbers only.

## Future features

| Feature | Notes |
|---|---|
| Stock vehicles | Type in a vehicle's stats to add mods to it |
| Trailers and sidecars | Their own small profiles (p. 127) and the towing rules |
| Operation time | The base 6 hours, Improved Economy/Capacity's +33% a rank, and what drains it |
| Export to ShadowrunEdit | Copy gives the stat block to paste; a `.md` with an `sr6-vehicle` block would open as one, ready to edit |
| Beyond Measure | Holostreets' space mods, design options, powertrains, and secondary propulsion (pp. 101–103), written to work with Double Clutch; its tables need settling |
| Large mounts with rockets or missiles | A large mount holds as many as the vehicle's Body (p. 142); today every large mount shows 500(b) |
