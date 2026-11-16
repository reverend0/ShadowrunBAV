# Contributing to ShadowrunBAV

`DESIGN.md` is the source of truth for what the app does and why. `RULINGS.md` lists every place *Double Clutch* contradicts itself or its examples and what the app does instead. Read the parts that touch your change first, and record new decisions there: a ruling on the book goes in `RULINGS.md`, numbered, with a test.

## Setting up

You need Node.js 24 and npm, Rust (stable, from <https://rustup.rs>, with `rustfmt` and `clippy`), and on Windows the Visual Studio C++ Build Tools. Then:

```bash
npm install
```

```bash
npm run tauri dev
```

The first `tauri dev` compiles everything and takes a few minutes.

## The rules

1. **No change is done without tests.** Each rule gets a test named after it; a bug fix gets a test that fails before the fix.
2. **No rules text from the book.** The app is shared, so `data/design.toml` holds only names and numbers, and comments say what a rule does in our own words, briefly, with its page. Before committing text from or about the book, run `node scripts/book-text-check.mjs <book.txt>` on the book's text from your own copy (`pdftotext -layout`); it lists every run of five or more words the repo shares with it. Stat headers, rule names, and numbers are fine; reword any sentence it finds.
3. **Run the checks before each commit:** `npm run lint`, `npm run typecheck`, `npm test`, `cargo test -p bav-core`. `npm run format` fixes most formatting.
4. **Readable Rust.** The owner is learning Rust through this project: idiomatic code, with comments on the non-obvious parts (ownership, lifetimes, traits, error handling).

## Where things are

- `data/design.toml`: the book's tables (size classes, chassis, powertrains, consoles, design options, design modifications, quality factors).
- `data/mods.toml`: the mods ("Making It Yours"), with what each field means at its top.
- `crates/bav-core/src/`: `catalog.rs` (the tables), `design.rs` (a design), `calc.rs` (the calculations), `mods.rs` (the mods on a design), `block.rs` (the stat block, as in ShadowrunEdit), `files.rs` (`.bav` files), `pdf.rs` (the PDF vehicle sheet), `tests.rs` and `mod_tests.rs`.
- `src-tauri/`: the window and its Tauri commands.
- `src/`: the UI (`main.ts`), its text helpers (`format.ts`), update checks (`updater.ts`), and the JSON shapes shared with Rust (`types.ts`, kept in step with the Rust structs).

## Commands

- Run the app: `npm run tauri dev`.
- Rust tests: `cargo test -p bav-core`.
- UI tests: `npm test`. Type check: `npm run typecheck`. Lint: `npm run lint`.
- Build an installer: `npm run tauri build` (NSIS and MSI, in `target/release/bundle/`). A release is signed for updates: see `RELEASING.md`.
- Print the catalog and a sample report as JSON (to check the UI outside the app): `cargo run -p bav-core --example report_json`.
- Write a sample vehicle sheet: `cargo run -p bav-core --example sheet_pdf -- target/sample.pdf`.
