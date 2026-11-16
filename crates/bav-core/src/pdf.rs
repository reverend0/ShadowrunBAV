//! A design as a PDF vehicle sheet: its entry laid out like the book's
//! vehicle entries (the name, its type in parentheses, the stat table, a
//! description), then how it was designed.
//!
//! The PDF is written by hand rather than with a library. It needs very
//! little: text in the three standard Helvetica fonts every PDF reader has
//! (so no font is embedded), filled rectangles, and lines. That keeps the
//! dependencies short and the output exact, which the tests check.

use std::fmt::Write as _;
use std::path::Path;

use crate::calc::{Report, evaluate, thousands};
use crate::catalog::{Catalog, Chassis, Kind};
use crate::design::Design;
use crate::files;

// US Letter, in points (1/72 inch), with 3/4-inch margins.
const PAGE_WIDTH: f32 = 612.0;
const PAGE_HEIGHT: f32 = 792.0;
const MARGIN: f32 = 54.0;
const TEXT_WIDTH: f32 = PAGE_WIDTH - 2.0 * MARGIN;

/// The book's stat table headers, in the order of `Report::stat_line`. Some
/// are printed on two lines (SPD over INT).
const HEADERS: [(&str, &str); 11] = [
    ("HAND", ""),
    ("ACC", ""),
    ("SPD", "INT"),
    ("TOP", "SPD"),
    ("BODY", ""),
    ("ARM", ""),
    ("PILOT", ""),
    ("SENS", ""),
    ("SEAT", ""),
    ("AVAIL", ""),
    ("COST", ""),
];

/// Writes a design's vehicle sheet to a `.pdf` file.
pub fn export(path: &Path, design: &Design, catalog: &Catalog) -> Result<(), String> {
    files::check_extension(path, "pdf", "a PDF")?;
    let bytes = vehicle_sheet(design, catalog)?;
    files::write_atomically(path, &bytes)
}

/// The vehicle sheet's PDF, as bytes.
pub fn vehicle_sheet(design: &Design, catalog: &Catalog) -> Result<Vec<u8>, String> {
    let report = evaluate(design, catalog)?;
    // `evaluate` succeeded, so the chassis exists; `ok_or` turns a `None`
    // into an error anyway rather than panicking.
    let chassis = catalog.chassis(&design.chassis).ok_or("Unknown chassis")?;
    let name = if design.name.trim().is_empty() { "Untitled vehicle" } else { design.name.trim() };

    let mut sheet = Sheet::new();

    // The entry, as the book prints one.
    sheet.paragraph(Font::Bold, 18.0, name);
    let mut kind = vehicle_type(chassis);
    if !design.specialized_function.trim().is_empty() {
        kind = format!("{kind}, {}", design.specialized_function.trim());
    }
    sheet.paragraph(Font::Italic, 11.0, &format!("({kind})"));
    sheet.gap(10.0);
    sheet.stat_table(&report);
    if !design.notes.trim().is_empty() {
        sheet.gap(10.0);
        sheet.paragraph(Font::Regular, 10.0, design.notes.trim());
    }

    // How it was designed.
    sheet.gap(18.0);
    sheet.rule();
    sheet.gap(4.0);
    sheet.paragraph(Font::Bold, 11.0, "Design");
    sheet.gap(2.0);
    for line in design_lines(&report) {
        sheet.paragraph(Font::Regular, 9.0, &line);
        sheet.gap(3.0);
    }
    if !report.problems.is_empty() {
        sheet.gap(4.0);
        sheet.paragraph(Font::Bold, 9.0, "Problems");
        for problem in &report.problems {
            sheet.paragraph(Font::Regular, 9.0, &format!("• {problem}"));
        }
    }

    Ok(document(name, &sheet.pages))
}

/// The type in an entry's parentheses: the chassis in lowercase, keeping
/// acronyms ("van/SUV", "LTA (lighter than air)"), and "medium drone" for
/// "Drone, Medium".
pub(crate) fn vehicle_type(chassis: &Chassis) -> String {
    if chassis.kind == Kind::Drone
        && let Some((_, size)) = chassis.name.split_once(", ")
    {
        return format!("{} drone", size.to_lowercase());
    }
    // `split_inclusive` keeps each separator with the word before it, so
    // joining the pieces back gives the original spacing.
    chassis
        .name
        .split_inclusive([' ', '/'])
        .map(|word| {
            let letters: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
            let acronym = letters.len() > 1 && letters.iter().all(|c| c.is_uppercase());
            if acronym { word.to_string() } else { word.to_lowercase() }
        })
        .collect()
}

/// The Design section's lines: what it's built from, its Build Points, and
/// its price and tests.
fn design_lines(r: &Report) -> Vec<String> {
    let s = &r.stats;
    let mut lines = Vec::new();

    let points: Vec<String> = r.build_points.iter().map(|l| format!("{} ({})", l.label, signed(l.points))).collect();
    lines.push(format!("Build Points: {}; total {}.", points.join(", "), r.base_points));
    if !r.multipliers.is_empty() {
        let factors: Vec<String> = r.multipliers.iter().map(|f| format!("{} (× {})", f.label, f.factor)).collect();
        lines.push(format!(
            "Quality factors: {}; Build Point Value {} × {} = {}.",
            factors.join(", "),
            r.base_points,
            r.multiplier,
            r.build_point_value
        ));
    } else {
        lines.push(format!("Build Point Value {}.", r.build_point_value));
    }

    let weight = if s.weight.is_empty() { String::new() } else { format!(", {}", s.weight) };
    let mut slots = format!(
        "chassis {}, powertrain {}, electronics {}",
        s.mod_slots.chassis, s.mod_slots.powertrain, s.mod_slots.electronics
    );
    if s.mod_slots.universal > 0 {
        let _ = write!(slots, ", universal {}", s.mod_slots.universal);
    }
    lines.push(format!("Size Class {} ({}{weight}). Cargo {}. Mod slots: {slots}.", s.size_class, s.length, s.cargo));

    lines.push(format!(
        "Price {}¥; parts to build it yourself {}¥; Availability {}.",
        thousands(r.cost),
        thousands(r.parts_cost),
        r.availability
    ));
    lines.push(format!(
        "Design it: Engineering + Logic ({}, 1 day), ({}, 1 week), or ({}, 1 month) Extended test. \
         Build it: Engineering + Logic (20, 1 week) Extended test, in a facility.",
        r.design_test.day, r.design_test.week, r.design_test.month
    ));
    lines.extend(r.notes.iter().cloned());
    lines
}

/// A number with its sign, an en dash for minus as the book prints it.
fn signed(n: i32) -> String {
    if n < 0 { format!("–{}", -n) } else { format!("+{n}") }
}

// Laying out text.

#[derive(Clone, Copy)]
pub(crate) enum Font {
    Regular,
    Bold,
    Italic,
}

impl Font {
    /// The font's name in each page's resources.
    fn resource(self) -> &'static str {
        match self {
            Font::Regular => "F1",
            Font::Bold => "F2",
            Font::Italic => "F3",
        }
    }
}

/// Character widths of Helvetica and Helvetica-Bold, in thousandths of the
/// font size, for the printable ASCII characters (space to `~`). Oblique has
/// Helvetica's widths. These are the fonts' published metrics, needed to wrap
/// lines and size the table's columns.
const HELVETICA: [u16; 95] = [
    278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, // space to /
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, // 0 to 9
    278, 278, 584, 584, 584, 556, 1015, // : to @
    667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944,
    667, 667, 611, // A to Z
    278, 278, 278, 469, 556, 333, // [ to `
    556, 556, 500, 556, 556, 278, 556, 556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722,
    500, 500, 500, // a to z
    334, 260, 334, 584, // { to ~
];
const HELVETICA_BOLD: [u16; 95] = [
    278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, // space to /
    556, 556, 556, 556, 556, 556, 556, 556, 556, 556, // 0 to 9
    333, 333, 584, 584, 584, 611, 975, // : to @
    722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944,
    667, 667, 611, // A to Z
    333, 278, 333, 584, 556, 333, // [ to `
    556, 611, 556, 611, 556, 333, 611, 611, 278, 278, 556, 278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778,
    556, 556, 500, // a to z
    389, 280, 389, 584, // { to ~
];

/// A character's byte in WinAnsiEncoding, the fonts' encoding: ASCII and
/// Latin-1 as they are, the typographic marks Windows adds, and `?` for
/// anything else.
pub(crate) fn win_ansi(c: char) -> u8 {
    match c {
        ' '..='~' => c as u8,
        // `as u8` keeps the low byte, which for U+00A0..U+00FF is the char.
        '\u{A0}'..='\u{FF}' => c as u32 as u8,
        '\t' | '\n' | '\r' => b' ',
        '€' => 0x80,
        '…' => 0x85,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '•' => 0x95,
        '–' | '−' => 0x96,
        '—' => 0x97,
        _ => b'?',
    }
}

/// How wide `text` is in points.
fn text_width(text: &str, font: Font, size: f32) -> f32 {
    let table = match font {
        Font::Bold => &HELVETICA_BOLD,
        Font::Regular | Font::Italic => &HELVETICA,
    };
    let units: u32 = text
        .chars()
        .map(|c| match win_ansi(c) {
            b @ 32..=126 => u32::from(table[usize::from(b - 32)]),
            0x85 | 0x97 => 1000,
            _ => 556,
        })
        .sum();
    units as f32 * size / 1000.0
}

/// Breaks text into lines no wider than `max`, at spaces. Each line break
/// in the text starts a new line; a word wider than `max` gets a line of
/// its own.
pub(crate) fn wrap(text: &str, font: Font, size: f32, max: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if !line.is_empty() && text_width(&candidate, font, size) > max {
                lines.push(std::mem::take(&mut line));
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        lines.push(line);
    }
    lines
}

/// Text as a PDF string literal's contents: `(`, `)`, and `\` escaped, and
/// bytes past ASCII as octal escapes, so the content stream stays ASCII.
fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match win_ansi(c) {
            b @ (b'(' | b')' | b'\\') => {
                out.push('\\');
                out.push(char::from(b));
            }
            b @ 32..=126 => out.push(char::from(b)),
            b => {
                let _ = write!(out, "\\{b:03o}");
            }
        }
    }
    out
}

/// The pages being laid out, top to bottom, with a cursor (`y`) at the top
/// of the space left on the last page.
struct Sheet {
    pages: Vec<String>,
    y: f32,
}

impl Sheet {
    fn new() -> Self {
        Sheet { pages: vec![String::new()], y: PAGE_HEIGHT - MARGIN }
    }

    /// The last page's drawing commands. `expect` can't fail: `new` starts
    /// with a page and none is ever removed.
    fn page(&mut self) -> &mut String {
        self.pages.last_mut().expect("a sheet always has a page")
    }

    /// Starts a new page unless `height` points fit on this one.
    fn need(&mut self, height: f32) {
        if self.y - height < MARGIN {
            self.pages.push(String::new());
            self.y = PAGE_HEIGHT - MARGIN;
        }
    }

    fn gap(&mut self, height: f32) {
        self.y -= height;
    }

    fn text(&mut self, x: f32, baseline: f32, font: Font, size: f32, text: &str) {
        let command = format!("BT /{} {size} Tf {x:.2} {baseline:.2} Td ({}) Tj ET\n", font.resource(), escape(text));
        self.page().push_str(&command);
    }

    /// Wrapped text across the page, a line at a time.
    fn paragraph(&mut self, font: Font, size: f32, text: &str) {
        let leading = size * 1.3;
        for line in wrap(text, font, size, TEXT_WIDTH) {
            self.need(leading);
            self.y -= leading;
            let baseline = self.y + (leading - size) + size * 0.2;
            self.text(MARGIN, baseline, font, size, &line);
        }
    }

    /// A thin line across the page.
    fn rule(&mut self) {
        let (left, right, y) = (MARGIN, PAGE_WIDTH - MARGIN, self.y);
        self.page().push_str(&format!("0.5 w 0.6 G {left} {y:.2} m {right} {y:.2} l S 0 G\n"));
    }

    /// The stat table: a dark band of white headers, then the values, each
    /// column as wide as its widest text.
    fn stat_table(&mut self, report: &Report) {
        const HEAD: f32 = 7.5;
        const VALUE: f32 = 10.0;
        const PAD: f32 = 5.0;
        let values = &report.stat_line.values;
        let widths: Vec<f32> = HEADERS
            .iter()
            .zip(values)
            .map(|((top, bottom), value)| {
                let head = text_width(top, Font::Bold, HEAD).max(text_width(bottom, Font::Bold, HEAD));
                head.max(text_width(value, Font::Regular, VALUE)) + 2.0 * PAD
            })
            .collect();
        let width: f32 = widths.iter().sum();
        let band = 2.0 * HEAD + 8.0;
        self.need(band + VALUE + 12.0);

        let top = self.y;
        let bottom = top - band;
        self.page().push_str(&format!("0.2 g {MARGIN} {bottom:.2} {width:.2} {band:.2} re f 1 g\n"));
        let mut x = MARGIN;
        for ((first, second), w) in HEADERS.iter().zip(&widths) {
            self.text(x + PAD, top - 3.0 - HEAD, Font::Bold, HEAD, first);
            if !second.is_empty() {
                self.text(x + PAD, top - 4.0 - 2.0 * HEAD, Font::Bold, HEAD, second);
            }
            x += w;
        }
        self.page().push_str("0 g\n");

        let baseline = bottom - VALUE - 3.0;
        let mut x = MARGIN;
        for (value, w) in values.iter().zip(&widths) {
            self.text(x + PAD, baseline, Font::Regular, VALUE, value);
            x += w;
        }
        let line = baseline - 5.0;
        let right = MARGIN + width;
        self.page().push_str(&format!("0.5 w 0.2 G {MARGIN} {line:.2} m {right:.2} {line:.2} l S 0 G\n"));
        self.y = line;
    }
}

/// Puts the pages' drawing commands into a PDF file: the catalog, the page
/// tree, the three fonts, the document's title, then each page and its
/// content, followed by the cross-reference table that says where each
/// object starts.
fn document(title: &str, pages: &[String]) -> Vec<u8> {
    let mut objects: Vec<String> = vec!["<< /Type /Catalog /Pages 2 0 R >>".into(), {
        let kids: Vec<String> = (0..pages.len()).map(|i| format!("{} 0 R", 7 + 2 * i)).collect();
        format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), pages.len())
    }];
    for base in ["Helvetica", "Helvetica-Bold", "Helvetica-Oblique"] {
        objects.push(format!("<< /Type /Font /Subtype /Type1 /BaseFont /{base} /Encoding /WinAnsiEncoding >>"));
    }
    objects.push(format!("<< /Title ({}) /Producer (ShadowrunBAV) >>", escape(title)));
    for (i, content) in pages.iter().enumerate() {
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH} {PAGE_HEIGHT}] \
             /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R >> >> /Contents {} 0 R >>",
            8 + 2 * i
        ));
        objects.push(format!("<< /Length {} >>\nstream\n{content}endstream", content.len()));
    }

    // Everything above is ASCII (`escape` saw to that), so the String's
    // length is the byte count the offsets need.
    let mut out = String::from("%PDF-1.4\n");
    let mut offsets = Vec::with_capacity(objects.len());
    for (i, object) in objects.iter().enumerate() {
        offsets.push(out.len());
        let _ = write!(out, "{} 0 obj\n{object}\nendobj\n", i + 1);
    }
    let xref = out.len();
    let _ = write!(out, "xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for offset in offsets {
        let _ = writeln!(out, "{offset:010} 00000 n ");
    }
    let _ =
        write!(out, "trailer\n<< /Size {} /Root 1 0 R /Info 6 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1);
    out.into_bytes()
}
