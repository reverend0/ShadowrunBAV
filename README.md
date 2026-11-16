# ShadowrunBAV

**A vehicle and drone builder for *Shadowrun, Sixth World*, by the rules of *Double Clutch*.**

Design a vehicle from scratch ("Build Your Dream", pp. 147–157), mod it ("Making It Yours", pp. 120–144), arm it, and get its stat block, ready for the table or for ShadowrunEdit.

[![Latest release](https://img.shields.io/github/v/release/reverend0/ShadowrunBAV)](https://github.com/reverend0/ShadowrunBAV/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**[Download the latest version for Windows](https://github.com/reverend0/ShadowrunBAV/releases/latest)** · [Rulings](RULINGS.md) · [What's new](https://github.com/reverend0/ShadowrunBAV/releases)

## What it does

**Design.** Pick a chassis, powertrain, and console, then add design options (forward or in reverse), design modifications, and quality factors. The app works out the stats, step by step where it matters (the speed calculation), the Build Points line by line, the price, the Availability, and the Engineering + Logic tests to design and build it.

**Mod.** Add any of the chapter's mods: accessories, chassis, skin, powertrain, secondary propulsion, electronics, hardpoints, and the devices on them. Install each afterward (shop price or do-it-yourself, with its install test) or build it into the design (2 Build Points a slot). The app keeps the mod slots, hardpoints, and cargo space in check, and the stat line follows the mods that change it.

**Arm.** Type in the vehicle's weapons and fit each in a weapon mount, which sets its ammunition and adds 2 to its Attack Ratings.

**Share.** The stat block is laid out as ShadowrunEdit lays out a vehicle, as a gear book's (Avail, Cost, Standard Equipment) or a gamemaster's (DR, I/ID, AC, CM), and Copy pastes it there. Export PDF prints a vehicle sheet in the book's style. Designs save as `.bav` files.

A design that breaks a rule still calculates, and the app says what to fix.

## The book, and its rulings

You need *Double Clutch*: the app holds only the names and numbers its calculations use, never the book's rules text or descriptions.

The book sometimes disagrees with itself: a rule and its example give different numbers, or a value sits in the wrong column. Every such case, what the app does instead, the three build examples corrected, and the assumptions where the book is silent are in **[RULINGS.md](RULINGS.md)**. Found another? [Open an issue](https://github.com/reverend0/ShadowrunBAV/issues) with the page and what disagrees.

## Install

1. Download `ShadowrunBAV_x.y.z_x64-setup.exe` from the [latest release](https://github.com/reverend0/ShadowrunBAV/releases/latest) (or the `.msi`). `SHA256SUMS.txt` there lists each file's checksum.
2. Run it. The installer isn't code-signed, so Windows SmartScreen says "Windows protected your PC" the first time: choose **More info**, then **Run anyway**.

After that the app keeps itself up to date: it checks for a new version in the background and shows an **Update to …** button on the toolbar when there is one; **Check for updates** checks right away. Updates are signed, and the app installs only those that match its key. (Version 0.1.1 has no updater: install a newer version by hand once.)

Windows is the target; Tauri can build for macOS and Linux too, untested.

## Building from source

ShadowrunBAV is Rust (the rules) and TypeScript (the window), on Tauri 2. See [CONTRIBUTING.md](CONTRIBUTING.md) for setting up, the checks, and where things are; [DESIGN.md](DESIGN.md) for what the app does and why; and [RELEASING.md](RELEASING.md) for making a release.

## License

Open source under the [MIT License](LICENSE): use, change, and share it freely, keeping the copyright notice. Contributions are welcome; see CONTRIBUTING.md.

Shadowrun is a trademark of The Topps Company, Inc. This is an unofficial fan tool, not affiliated with Topps or Catalyst Game Labs.
