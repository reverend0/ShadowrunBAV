//! Reading and writing design files (`.bav`): a design's choices as TOML.

use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::design::Design;

/// The file's format version, so a later app can upgrade older files.
pub const FORMAT: u32 = 1;

/// What's in a file: the format version, then the design.
///
/// `#[serde(flatten)]` writes the design's fields at the top level of the
/// file, beside `format`, rather than under a `[design]` table.
#[derive(Serialize, Deserialize)]
struct DesignFile {
    format: u32,
    #[serde(flatten)]
    design: Design,
}

/// The design as the text of a `.bav` file.
pub fn to_text(design: &Design) -> String {
    // `toml::to_string` only fails for values TOML can't hold (a map with
    // non-text keys, say); a Design has none, so the `expect` can't fire.
    toml::to_string(&DesignFile { format: FORMAT, design: design.clone() }).expect("a Design is valid TOML")
}

/// Reads a design from a `.bav` file's text.
pub fn from_text(text: &str) -> Result<Design, String> {
    let file: DesignFile = toml::from_str(text).map_err(|e| format!("This isn't a design file: {}", e.message()))?;
    if file.format > FORMAT {
        return Err(format!(
            "This design was saved by a newer ShadowrunBAV (format {}); update the app to open it.",
            file.format
        ));
    }
    Ok(file.design)
}

/// Only `.bav` files are read or written, so a slip can't overwrite
/// something else.
fn check_extension(path: &Path) -> Result<(), String> {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("bav") => Ok(()),
        _ => Err(format!("{} isn't a design file (.bav).", path.display())),
    }
}

pub fn open(path: &Path) -> Result<Design, String> {
    check_extension(path)?;
    let text = fs::read_to_string(path).map_err(|e| format!("Couldn't read {}: {e}", path.display()))?;
    from_text(&text)
}

/// Saves a design: writes a temporary file beside it, flushes it to the
/// disk, and renames it into place, so a crash mid-save leaves the old file
/// whole.
pub fn save(path: &Path, design: &Design) -> Result<(), String> {
    check_extension(path)?;
    let temp = path.with_extension("bav.tmp");
    // The block's `?`s return early on any error; the closure gathers them
    // into one `io::Result` so they share one error message below.
    let write = || -> std::io::Result<()> {
        let mut file = fs::File::create(&temp)?;
        file.write_all(to_text(design).as_bytes())?;
        file.sync_all()?;
        fs::rename(&temp, path)
    };
    write().map_err(|e| {
        let _ = fs::remove_file(&temp);
        format!("Couldn't save {}: {e}", path.display())
    })
}
