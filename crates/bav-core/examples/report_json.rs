//! Prints the catalog and one design's report as JSON, for checking the UI
//! outside the app: `cargo run -p bav-core --example report_json`.

use bav_core::design::InstalledMod;
use bav_core::{Catalog, Design, Improvements, evaluate};

fn main() {
    let design = Design {
        name: "Example LAV".into(),
        chassis: "vtol".into(),
        powertrain: "vectored-thrust".into(),
        console: "expert".into(),
        improvements: Improvements { armor: 2, ..Improvements::default() },
        modifications: vec!["high-performance-engine".into(), "multi-engine".into()],
        qualities: vec!["military".into()],
        mods: vec![
            InstalledMod { id: "chameleon-coating".into(), rating: 2, built_in: true, ..InstalledMod::default() },
            InstalledMod {
                id: "standard-weapon-mount".into(),
                count: 2,
                note: "nose".into(),
                ..InstalledMod::default()
            },
            InstalledMod { id: "spoof-kit".into(), ..InstalledMod::default() },
        ],
        ..Design::default()
    };
    let catalog = Catalog::builtin();
    let report = evaluate(&design, catalog).expect("valid ids");
    let out = serde_json::json!({ "catalog": catalog, "design": design, "report": report });
    println!("{out}");
}
