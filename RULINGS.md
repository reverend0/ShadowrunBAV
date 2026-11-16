# Rulings

*Double Clutch* sometimes disagrees with itself: a rule says one thing and its example does another, a table and an entry give different numbers, or a value sits in the wrong column. Each time, the owner decided what ShadowrunBAV does, and it's recorded here, with a test in the app for each.

The page numbers are *Double Clutch*'s. The book's wording isn't quoted; each ruling says in a few words where the book disagrees, so you can find it in your copy.

- [Build Your Dream](#build-your-dream-pp-147157): designing a vehicle (R1–R11)
- [Making It Yours](#making-it-yours-pp-120144): mods (R12–R20)
- [The examples, corrected](#the-examples-corrected): the three build examples' numbers next to the app's
- [Not yet confirmed](#not-yet-confirmed): what the app assumes where the book is silent

Found another? [Open an issue](../../issues) with the page and what disagrees.

## Build Your Dream (pp. 147–157)

**R1. Top Speed loss comes off Top Speed** (p. 155). The final-speed step says to take the speed modifier × 10 off the base Acceleration; every example takes it off Top Speed. *The app:* off the powertrain's base Top Speed.

**R2. The final Body sets the speed modifier** (pp. 154–155). Example 1 lowers its car's Body to 9 but works its speed with Body 10, and lists Body 10 in its final stats. *The app:* the final Body, after Improved Body, Improved Size, Heavy Duty, and Lightweight. Example 1's Top Speed is 270 with its racing engine (the book: 260).

**R3. Halving the speed modifier rounds down** (p. 155). Example 2 halves a speed modifier of 19 to 10. *The app:* rounds down, as the book's general rule does: 9. Example 2's Acceleration is 31 before its engine mods (the book: 30).

**R4. A walker uses the powertrain table** (p. 150; Example 3, p. 156). The table gives the walker Acceleration 10, Speed Interval 15, Top Speed 30, and 15 Build Points; Example 3 uses 10, 40, and 10, and charges 10 Build Points. *The app:* the table, which the secondary propulsion Walker (p. 137) agrees with.

**R5. The High Performance Engine costs 15 Build Points** (pp. 152–153). Its entry says 15; Example 2 charges 25, the Racing Engine's. *The app:* 15. Example 2 totals 163 (the book: 173).

**R6. The design test's threshold rounds up** (p. 155). Examples 1 and 2 round the Build Point Value ÷ 5 up (and ÷ 2 or ÷ 4 for a week or a month); Example 3 rounds its down. *The app:* up, everywhere.

**R7. Handling and Speed Interval are at least 1** (p. 151). Design options in reverse can take them to 0 or below, and the book gives no floor; Handling is a test's threshold. *The app:* shows them as 1 and flags the design. (For mods the book does say Handling stops at 1, p. 125.)

**R8. Underpowered halves Top Speed only** (pp. 152–153). *The app:* Acceleration is left as it is.

**R9. The small-drone Acceleration multiplier comes first** (p. 155). The book doesn't say where it falls among the engine steps. *The app:* right after the speed modifier, before the engine type and Multi-Engine.

**R10. Quality factors multiply the Build Point Value everywhere** (pp. 153–155), so the design test's threshold uses the multiplied value (Junkyard Frankenstein is labeled as a Build Point multiplier).

**R11. Two stray table columns are layout errors** (p. 150). The console table's Special column (10, 12, --, --) and the "0" at the end of the chassis table's Bike row are left out. The chassis table's Special column is kept for the Truck and the Submersible (see "Not yet confirmed").

## Making It Yours (pp. 120–144)

**R12. Increased Structural Integrity goes up to half the Body** (p. 124). Its sentence allows a rating of 1.5 × the factory Body; its example (a Body 4 bike gets 2) and its stat box allow half the Body. *The app:* half, so the Body ends up at most 1.5 × the factory Body.

**R13. Improved Pilot's rating is the new Pilot, 1 to 9** (p. 141). Its text says Pilot runs 1 to 9, but its box shows no rating, and a "1–9" sits in the box of xGuide Override, which has no rating. *The app:* the rating replaces the console's Pilot; slots and price are for the full rating. Enhanced Sensors (p. 142) works the same way for Sensor.

**R14. Medevac Modifications cost a flat 3,500¥** (p. 124). It has no rating, but its price is "× Rating". *The app:* 3,500¥.

**R15. Improved Economy/Capacity takes a slot per rating** (p. 135). Its box gives slots as "1–3" for ratings 1–3.

**R16. A gun port takes half a slot** (p. 123). Its box has no mod slots, and "0.5" in the Rating column. *The app:* 0.5 slots each, no rating.

**R17. Weapon mounts have a price** (pp. 143–144). Example 1's 7,300¥ of mods and Example 3's 33,000¥ leave out their weapon mounts. *The app:* prices them by the weapon mounts table.

**R18. Run-flat racing or off-road tires cost +50%** (p. 122). The tires' entry says run-flat adds 50% to their price; Example 1 pays 1,800¥ (racing tires plus a set of run-flat tires). *The app:* +50%: 1,500¥ for a car.

**R19. A mod built into the design uses mod slots** (p. 151). Built in, a mod costs 2 Build Points a slot and its price; the book doesn't say whether it also uses the vehicle's slots. *The app:* it does.

**R20. Built-in mods add their Build Points** (p. 151; Example 3, p. 156). Example 3 gives Realistic Features 4 "8 Build Points" and Chameleon Coating 2 "4", but its Build Point Value of 67 leaves those 12 out. *The app:* adds them.

## The examples, corrected

What each example says, and what the app works out with the rulings above. The tests check the app's column.

| Example | The book | The app | Ruling |
|---|---|---|---|
| 1, street racer: Top Speed | 260 | 270 | R2 |
| 1: price with mods | 60,300¥ | 65,000¥ (two standard weapon mounts, run-flat racing tires at 1,500¥) | R17, R18 |
| 2, LAV: Acceleration | 30, before its engines | 31 before, 68 after | R3 |
| 2: Build Points | 173 | 163 | R5 |
| 2: design threshold | 35 | 33 (17 by the week, 9 by the month) | R5, R6 |
| 3, walker drone: Speed Interval, Top Speed | 40, 10 | 15, 30 | R4 |
| 3: Build Point Value | 67 | 84 (72 for the design, 12 for its built-in mods) | R4, R20 |
| 3: design threshold | 13 | 17 | R6 |

## Not yet confirmed

Where the book is silent, the app assumes the following. Each is one place in the code, with a test, and changes if the owner rules otherwise.

- The chassis table's Special column is kept for the Truck (+4 CF of cargo) and the Submersible (built for underwater use).
- Heavy Duty and Lightweight apply after Improved Body and Improved Size.
- Improved Size's +50% seats apply after Improved Seating, and its cargo is the new Size Class's.
- Cargo Hauler halves the High Performance Engine's and Multi-Engine's bonuses, as written, and not the Racing Engine's.
- Off Road Chassis's −10 Top Speed comes with the engine type, before Multi-Engine.
- A drone chassis always has the Drone quality; it isn't a choice.
- An aircraft, for Availability, is one with an air chassis or an air powertrain.
- Easily Modified has no reverse, and its limit uses the chassis's Body.
- The takeoff and landing profiles and the operation time aren't calculated: the book doesn't tie them to a chassis or a price.
- A bike and a personal watercraft start with open construction, drones totally enclosed, everything else standard.
- Added standard armor costs a walker Top Speed too: the armor rule (p. 133) has no exception, unlike the design's speed modifier.
- Improved Acceleration and Top Speed Increase add a share of the designed vehicle's own value, each rank rounded up (Improved Acceleration 2 on Acceleration 11 is +8).
- Easy Assembly's ⅓ Body and Powered Breakdown's ½ Body slots round up.
- An anti-theft system's rating 1 comes with any Pilot, so it's free.
- Living space costs a month of its lifestyle per occupant, at the core rulebook's prices.
- Increased Seating takes 1 CF of cargo a seat; its box's "or 0.5 slots" is the cargo space Increased Cargo Space would make.
- Elemental hardening has no Availability (its box has none).
- A large weapon mount shows 500(b) rounds even when it holds rockets or missiles, which the book counts by the vehicle's Body.
