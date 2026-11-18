//! Writes a sample design's vehicle sheet, to look at outside the app:
//! `cargo run -p bav-core --example sheet_pdf -- target/sample.pdf`.

use std::path::PathBuf;

use bav_core::design::{InstalledMod, Layout, Weapon};
use bav_core::{Catalog, Design, Improvements, pdf};

fn main() {
    // `nth(1)` skips the program's own name; `expect` stops with a message
    // when no path was given.
    let path = PathBuf::from(std::env::args().nth(1).expect("give the .pdf path to write"));
    let design = Design {
        name: "Example LAV".into(),
        chassis: "vtol".into(),
        powertrain: "vectored-thrust".into(),
        console: "expert".into(),
        improvements: Improvements { armor: 2, ..Improvements::default() },
        modifications: vec!["high-performance-engine".into(), "multi-engine".into()],
        qualities: vec!["military".into()],
        notes: "A low-altitude vehicle for moving a fire team in a hurry, armored against small arms.".into(),
        mods: vec![
            InstalledMod { id: "standard-weapon-mount".into(), count: 2, ..InstalledMod::default() },
            InstalledMod { id: "spoof-kit".into(), ..InstalledMod::default() },
            InstalledMod { id: "ram-plating".into(), rating: 2, ..InstalledMod::default() },
        ],
        weapons: vec![Weapon {
            name: "Ares Alpha".into(),
            weapon_type: "Assault Rifles".into(),
            dv: "4P".into(),
            mode: "SA/BF/FA".into(),
            ar: vec![3, 10, 9, 6, 2],
            mods: "smartgun system".into(),
            mount: "standard-weapon-mount".into(),
            ..Weapon::default()
        }],
        // A second argument, "gm", lays it out for a gamemaster.
        layout: if std::env::args().nth(2).as_deref() == Some("gm") { Layout::Gamemaster } else { Layout::Gear },
        ..Design::default()
    };
    match pdf::export(&path, &design, Catalog::builtin()) {
        Ok(()) => println!("Wrote {}", path.display()),
        Err(e) => eprintln!("{e}"),
    }
}
