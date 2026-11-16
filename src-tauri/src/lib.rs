//! The desktop shell: exposes `bav-core` to the UI as Tauri commands.
//!
//! Each `#[tauri::command]` function can be called from TypeScript with
//! `invoke("command_name", { args })`. Arguments and return values cross the
//! boundary as JSON, converted by serde. Errors are `String`s, which the UI
//! shows as they are.

use std::path::PathBuf;

use bav_core::{Catalog, Design, Report, evaluate, files, pdf};

/// The book's tables, for the UI to build its menus from.
///
/// `&'static Catalog` can't cross to the UI by itself; Tauri serializes
/// what the function returns, and `Catalog` derives `Serialize`, so a
/// reference to it is enough.
#[tauri::command]
fn catalog() -> &'static Catalog {
    Catalog::builtin()
}

/// Works out a design's stats, Build Points, cost, and tests.
#[tauri::command]
fn evaluate_design(design: Design) -> Result<Report, String> {
    evaluate(&design, Catalog::builtin())
}

#[tauri::command]
fn open_design(path: PathBuf) -> Result<Design, String> {
    files::open(&path)
}

#[tauri::command]
fn save_design(path: PathBuf, design: Design) -> Result<(), String> {
    files::save(&path, &design)
}

/// Writes the design's vehicle sheet, laid out like the book's entries.
#[tauri::command]
fn export_pdf(path: PathBuf, design: Design) -> Result<(), String> {
    pdf::export(&path, &design, Catalog::builtin())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![catalog, evaluate_design, open_design, save_design, export_pdf])
        .run(tauri::generate_context!())
        .expect("error while running ShadowrunBAV");
}
