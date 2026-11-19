// The builder window: a form of the design's choices on the left, and the
// vehicle Rust works out from them on the right, redone on every change.

import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask, message, open, save } from "@tauri-apps/plugin-dialog";
import type { Update } from "@tauri-apps/plugin-updater";
import { checkNow, installUpdate, startUpdateChecks } from "./updater.ts";
import { CANCEL, DONT_SAVE, mayClose, SAVE } from "./closing.ts";
import { isPicture, loadPicture } from "./picture.ts";
import { summary, usage, ZONES, type Zone, zoneAtPixel, zoneOf } from "./zones.ts";
import { chassisLabel, consoleLabel, factor, fileName, powertrainLabel, signed, thousands } from "./format.ts";
import {
  type Catalog,
  type Choice,
  type Design,
  type Improvements,
  type InstalledMod,
  type MilspecHardpoint,
  newDesign,
  type Report,
  type Weapon,
} from "./types.ts";

const formEl = document.getElementById("form") as HTMLFormElement;
const reportEl = document.getElementById("report") as HTMLElement;
const fileEl = document.getElementById("file") as HTMLElement;

let catalog: Catalog;
let design: Design = newDesign();
let path: string | null = null;
/** The design as last saved or opened, to tell whether it has changed. */
let saved = JSON.stringify(design);

const FILTERS = [{ name: "Vehicle design", extensions: ["bav"] }];
const KIND_NAMES: Record<string, string> = { ground: "Ground", water: "Water", air: "Air", drone: "Drones" };

/** Makes an element with attributes and children. */
function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attrs: Record<string, string> = {},
  ...children: (Node | string)[]
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) node.setAttribute(key, value);
  node.append(...children);
  return node;
}

function isDirty(): boolean {
  return JSON.stringify(design) !== saved;
}

function showFile(): void {
  const name = path ? fileName(path) : "Untitled";
  fileEl.textContent = isDirty() ? `${name} •` : name;
  document.title = `${design.name || name} — ShadowrunBAV`;
}

// The form.

function select(
  label: string,
  value: string,
  options: { value: string; label: string; group?: string }[],
  onChange: (v: string) => void,
) {
  const box = el("select");
  const groups = new Map<string, HTMLElement>();
  for (const option of options) {
    const item = el("option", { value: option.value }, option.label);
    if (option.group) {
      let group = groups.get(option.group);
      if (!group) {
        group = el("optgroup", { label: option.group });
        groups.set(option.group, group);
        box.append(group);
      }
      group.append(item);
    } else {
      box.append(item);
    }
  }
  box.value = value;
  box.addEventListener("change", () => onChange(box.value));
  return el("label", { class: "field" }, el("span", {}, label), box);
}

/** Radio buttons for a group of choices that exclude each other, with "none" first. */
function radios(name: string, none: string, choices: Choice[], chosen: string[], onChange: (id: string | null) => void) {
  const current = choices.find((c) => chosen.includes(c.id))?.id ?? "";
  const row = el("div", { class: "radios", role: "radiogroup" });
  for (const [id, label] of [["", none], ...choices.map((c) => [c.id, c.name])]) {
    const input = el("input", { type: "radio", name, value: id });
    input.checked = id === current;
    input.addEventListener("change", () => onChange(id || null));
    row.append(el("label", {}, input, label));
  }
  return row;
}

function checkbox(label: string, checked: boolean, onChange: (on: boolean) => void, title = "") {
  const input = el("input", { type: "checkbox" });
  input.checked = checked;
  input.addEventListener("change", () => onChange(input.checked));
  return el("label", { class: "check", title }, input, label);
}

/** Replaces the chosen member of a group in a list of ids. */
function setInGroup(list: string[], choices: Choice[], id: string | null): string[] {
  const group = new Set(choices.map((c) => c.id));
  const rest = list.filter((x) => !group.has(x));
  return id ? [...rest, id] : rest;
}

function toggle(list: string[], id: string, on: boolean): string[] {
  const rest = list.filter((x) => x !== id);
  return on ? [...rest, id] : rest;
}

function renderForm(): void {
  const isDrone = catalog.chassis.find((c) => c.id === design.chassis)?.kind === "drone";

  const name = el("input", { type: "text", placeholder: "Name this vehicle" });
  name.value = design.name;
  name.addEventListener("input", () => {
    design.name = name.value;
    changed(false);
  });

  const base = el(
    "fieldset",
    {},
    el("legend", {}, "Base"),
    select(
      "Chassis",
      design.chassis,
      catalog.chassis.map((c) => ({ value: c.id, label: chassisLabel(c), group: KIND_NAMES[c.kind] })),
      (v) => {
        design.chassis = v;
        changed();
      },
    ),
    select(
      "Powertrain",
      design.powertrain,
      catalog.powertrain.map((p) => ({ value: p.id, label: powertrainLabel(p), group: KIND_NAMES[p.medium] })),
      (v) => {
        design.powertrain = v;
        changed();
      },
    ),
    select(
      "Console",
      design.console,
      catalog.console.map((c) => ({ value: c.id, label: consoleLabel(c) })),
      (v) => {
        design.console = v;
        changed();
      },
    ),
    select(
      "Stat block",
      design.layout,
      [
        { value: "gear", label: "Gear book: Avail and Cost, Standard Equipment" },
        { value: "gamemaster", label: "Gamemaster: DR, I/ID, AC, CM, Equipment" },
      ],
      (v) => {
        design.layout = v === "gamemaster" ? "gamemaster" : "gear";
        changed();
      },
    ),
  );

  // Functional improvements: a level each, negative for the reverse.
  const levels = el("div", { class: "levels" });
  for (const imp of catalog.improvement) {
    // The book's limit where it's fixed (Easily Modified has no reverse);
    // where it depends on the vehicle, the box allows ±20 and the report
    // flags a level past the real limit.
    const most = imp.max_level > 0 ? imp.max_level : 20;
    const least = imp.id === "easily_modified" ? 0 : -most;
    const input = el("input", { type: "number", step: "1", min: String(least), max: String(most) });
    input.value = String(design.improvements[imp.id]);
    input.addEventListener("change", () => {
      const level = Math.trunc(Number(input.value)) || 0;
      design.improvements = { ...design.improvements, [imp.id]: level } as Improvements;
      changed();
    });
    const limit = imp.max_level > 0 ? `up to ${imp.max_level}` : "limit varies";
    levels.append(
      el("label", { for: `imp-${imp.id}` }, imp.name),
      Object.assign(input, { id: `imp-${imp.id}` }),
      el("span", { class: "hint" }, `${imp.per_level} BP a level, ${limit}`),
    );
  }
  const options = el(
    "fieldset",
    {},
    el("legend", {}, "Design options"),
    el("p", { class: "hint" }, "Levels; a negative level is the option in reverse, for fewer Build Points."),
    levels,
  );

  // Design modifications.
  const mods = catalog.modification;
  const engines = mods.filter((m) => m.group === "engine");
  const builds = mods.filter((m) => m.group === "build");
  const modBoxes = mods
    .filter((m) => !m.group)
    .map((m) =>
      checkbox(`${m.name} (${signed(m.build_points)} BP)`, design.modifications.includes(m.id), (on) => {
        design.modifications = toggle(design.modifications, m.id, on);
        changed();
      }),
    );
  const modifications = el(
    "fieldset",
    {},
    el("legend", {}, "Design modifications"),
    el("div", { class: "row-label" }, "Engine"),
    radios(
      "engine",
      "Standard",
      engines.map((m) => ({ ...m, name: `${m.name.replace(" Engine", "")} (${signed(m.build_points)})` })),
      design.modifications,
      (id) => {
        design.modifications = setInGroup(design.modifications, engines, id);
        changed();
      },
    ),
    el("div", { class: "row-label" }, "Build"),
    radios(
      "build",
      "Standard",
      builds.map((m) => ({ ...m, name: `${m.name} (${signed(m.build_points)})` })),
      design.modifications,
      (id) => {
        design.modifications = setInGroup(design.modifications, builds, id);
        changed();
      },
    ),
    el("div", { class: "checks" }, ...modBoxes),
  );

  // Quality factors. The Drone quality comes with a drone chassis.
  const qualities = catalog.quality.filter((q) => q.id !== "drone");
  const production = qualities.filter((q) => q.group === "production");
  const service = qualities.filter((q) => q.group === "service");
  const qualityBoxes = qualities
    .filter((q) => !q.group)
    .map((q) => {
      const box = checkbox(`${q.name} (${factor(q.multiplier)})`, design.qualities.includes(q.id), (on) => {
        design.qualities = toggle(design.qualities, q.id, on);
        changed();
      });
      if (q.id === "environmental-adaptation" && design.qualities.includes(q.id)) {
        return el(
          "div",
          { class: "with-detail" },
          box,
          select(
            "Environment",
            design.environment,
            [{ value: "", label: "Choose…" }, ...catalog.environments.map((e) => ({ value: e, label: e }))],
            (v) => {
              design.environment = v;
              changed();
            },
          ),
        );
      }
      if (q.id === "specialized-function" && design.qualities.includes(q.id)) {
        const what = el("input", { type: "text", placeholder: "What it's for (ambulance, tow truck…)" });
        what.value = design.specialized_function;
        what.addEventListener("change", () => {
          design.specialized_function = what.value.trim();
          changed();
        });
        return el("div", { class: "with-detail" }, box, what);
      }
      return box;
    });
  const withFactor = (q: Choice & { multiplier: number }) => ({ ...q, name: `${q.name} (${factor(q.multiplier)})` });
  const quality = el(
    "fieldset",
    {},
    el("legend", {}, "Quality factors"),
    isDrone ? el("p", { class: "hint" }, "A drone chassis has the Drone quality (× 0.5).") : "",
    el("div", { class: "row-label" }, "Production"),
    radios("production", "Standard", production.map(withFactor), design.qualities, (id) => {
      design.qualities = setInGroup(design.qualities, production, id);
      changed();
    }),
    el("div", { class: "row-label" }, "Service"),
    radios("service", "Civilian", service.map(withFactor), design.qualities, (id) => {
      design.qualities = setInGroup(design.qualities, service, id);
      changed();
    }),
    el("div", { class: "checks" }, ...qualityBoxes),
  );

  const notes = el("textarea", { rows: "4", placeholder: "Notes" });
  notes.value = design.notes;
  notes.addEventListener("input", () => {
    design.notes = notes.value;
    changed(false);
  });

  formEl.replaceChildren(
    el("label", { class: "field name" }, el("span", {}, "Name"), name),
    booksFieldset(),
    base,
    options,
    modifications,
    quality,
    ...(design.books.includes(DEADLY_ARTS) ? [milspecFieldset()] : []),
    modsFieldset(),
    weaponsFieldset(),
    el("fieldset", {}, el("legend", {}, "Notes"), notes),
  );
}

// Sourcebooks beyond Double Clutch, turned on per design.

const DEADLY_ARTS = "deadly-arts";

function booksFieldset(): HTMLElement {
  return el(
    "fieldset",
    {},
    el("legend", {}, "Books"),
    el("p", { class: "hint" }, "Double Clutch, plus the rules of any book turned on here, saved with the design."),
    el(
      "div",
      { class: "checks" },
      ...catalog.book.map((book) =>
        checkbox(book.name, design.books.includes(book.id), (on) => {
          design.books = on ? [...design.books, book.id] : design.books.filter((b) => b !== book.id);
          changed();
        }),
      ),
    ),
  );
}

// Deadly Arts' mil-spec design options (pp. 150–152).

/** Each mil-spec hardpoint's Grade, from the last report (Rust works them out). */
let lastGrades: number[] = [];

/** Replaces one mil-spec hardpoint with a changed copy, and recalculates. */
function updateHardpoint(index: number, change: Partial<MilspecHardpoint>): void {
  const hardpoints = design.milspec.hardpoints.map((h, i) => (i === index ? { ...h, ...change } : h));
  design.milspec = { ...design.milspec, hardpoints };
  changed(false);
}

/** A select of fixed choices that commits on change. */
function plainSelect(value: string, title: string, options: [string, string][], onChange: (v: string) => void) {
  const box = el("select", { title });
  for (const [v, label] of options) box.append(el("option", { value: v }, label));
  box.value = value;
  box.addEventListener("change", () => onChange(box.value));
  return box;
}

function hardpointRow(hp: MilspecHardpoint, index: number): HTMLElement {
  const remove = el("button", { type: "button", title: "Remove this hardpoint", class: "remove" }, "×");
  remove.addEventListener("click", () => {
    design.milspec = { ...design.milspec, hardpoints: design.milspec.hardpoints.filter((_, i) => i !== index) };
    // A weapon on it comes off; those on later hardpoints move up a number.
    design.weapons = design.weapons.map((w) => {
      if (!w.mount.startsWith("milspec-")) return w;
      const n = Number(w.mount.slice("milspec-".length));
      if (n <= index) return w;
      return { ...w, mount: n === index + 1 ? "" : `milspec-${n - 1}` };
    });
    changed();
  });
  const grade = lastGrades[index];
  return el(
    "div",
    { class: "mod-row" },
    el("span", { class: "mod-name" }, `Hardpoint ${index + 1}${grade == null ? "" : `: Grade ${grade}`}`),
    el(
      "label",
      { class: "small" },
      "Weapon Size ",
      numberBox(hp.size, 0, 20, "The weapon's Size", (size) => updateHardpoint(index, { size })),
    ),
    plainSelect(
      hp.mount,
      "How the weapon is mounted",
      [
        ["turreted", "Turret"],
        ["inline", "Inline (−2)"],
        ["open", "Open (−2)"],
      ],
      (v) => updateHardpoint(index, { mount: v as MilspecHardpoint["mount"] }),
    ),
    plainSelect(
      String(hp.linked),
      "Identical weapons linked in it",
      [
        ["1", "One weapon"],
        ["2", "Twin-linked (+1)"],
        ["4", "Quad-linked (+2)"],
      ],
      (v) => updateHardpoint(index, { linked: Number(v) }),
    ),
    checkbox("Pop-out (+1)", hp.pop_out, (pop_out) => updateHardpoint(index, { pop_out }), "Hidden until needed"),
    checkbox(
      "Primary",
      hp.primary,
      (primary) => updateHardpoint(index, { primary }),
      "A dedicated primary weapon, built in at construction: its minimum Body is a Grade smaller's",
    ),
    textBox(hp.note, "Note", (note) => updateHardpoint(index, { note }), "mod-note"),
    remove,
  );
}

function milspecFieldset(): HTMLElement {
  const ms = design.milspec;
  const set = (change: Partial<typeof ms>) => {
    design.milspec = { ...design.milspec, ...change };
    changed();
  };
  const add = el("button", { type: "button" }, "Add a mil-spec hardpoint");
  add.addEventListener("click", () =>
    set({
      hardpoints: [...ms.hardpoints, { size: 1, mount: "turreted", pop_out: false, linked: 1, primary: false, note: "" }],
    }),
  );
  return el(
    "fieldset",
    {},
    el("legend", {}, "Mil-spec (Deadly Arts)"),
    el(
      "p",
      { class: "hint" },
      "Design options from Deadly Arts, pp. 150–152, in Build Points. Any of them makes it a military vehicle: +8 to build it. A hardpoint's Grade is its weapon's Size, changed by its mounting.",
    ),
    el(
      "div",
      { class: "checks" },
      checkbox("Airframe ceramic plate (2 BP a point of Armor)", ms.airframe_plate, (airframe_plate) => set({ airframe_plate })),
    ),
    select(
      "Heavy plate",
      String(ms.heavy_plate),
      [{ value: "0", label: "None" }, ...[4, 5, 6].map((r) => ({ value: String(r), label: `Rating ${r} (+${5 * r} BP)` }))],
      (v) => set({ heavy_plate: Number(v) }),
    ),
    select(
      "Comms",
      String(ms.comms),
      [
        { value: "0", label: "None" },
        { value: "1", label: "Mil-spec comms I (+5 BP)" },
        { value: "2", label: "Mil-spec comms II (+20 BP)" },
        { value: "3", label: "Mil-spec comms III (+30 BP)" },
      ],
      (v) => set({ comms: Number(v) }),
    ),
    el(
      "label",
      { class: "field", title: "Body + its rating, up to half the Body; 10 BP + 5 a rating" },
      el("span", {}, "Frame"),
      numberBox(ms.frame, 0, 12, "Mil-spec frame rating (0 for none)", (frame) => set({ frame: Math.max(0, frame) })),
    ),
    ...ms.hardpoints.map(hardpointRow),
    add,
  );
}

// Mods ("Making It Yours").

/** Replaces one installed mod with a changed copy, and recalculates. */
function updateMod(index: number, change: Partial<InstalledMod>, redraw = false): void {
  design.mods = design.mods.map((m, i) => (i === index ? { ...m, ...change } : m));
  changed(redraw);
}

/** A number box that commits a whole number on change. */
function numberBox(value: number, min: number, max: number, title: string, onChange: (n: number) => void) {
  const input = el("input", { type: "number", step: "1", min: String(min), max: String(max), title });
  input.value = String(value);
  input.addEventListener("change", () => onChange(Math.trunc(Number(input.value)) || 0));
  return input;
}

/**
 * One installed mod: its rating, count, built in, note, and remove. Under a
 * ticked mod in the zone's list, its name is already there, so it's left out.
 */
function modRow(installed: InstalledMod, index: number, withName = true): HTMLElement {
  const def = catalog.mods.find((m) => m.id === installed.id);
  const row = withName
    ? el("div", { class: "mod-row" }, el("span", { class: "mod-name" }, def?.name ?? installed.id))
    : el("div", { class: "mod-row mod-detail" });
  if (def?.rating_max != null) {
    const min = def.rating_min ?? 1;
    row.append(
      el(
        "label",
        { class: "small" },
        "Rating ",
        numberBox(installed.rating, min, def.rating_max, `Rating ${min} to ${def.rating_max}`, (rating) =>
          updateMod(index, { rating }),
        ),
      ),
    );
  }
  if (def?.many) {
    row.append(
      el(
        "label",
        { class: "small" },
        "× ",
        numberBox(installed.count, 1, 99, "How many", (count) => updateMod(index, { count: Math.max(1, count) })),
      ),
    );
  }
  row.append(
    checkbox(
      "Built in",
      installed.built_in,
      (built_in) => updateMod(index, { built_in }),
      "Built into the design: 2 Build Points a slot, its price added to the vehicle's",
    ),
  );
  const note = el("input", { type: "text", placeholder: "Note", class: "mod-note" });
  note.value = installed.note;
  note.addEventListener("change", () => updateMod(index, { note: note.value.trim() }));
  const remove = el("button", { type: "button", title: "Remove this mod", class: "remove" }, "×");
  remove.addEventListener("click", () => {
    design.mods = design.mods.filter((_, i) => i !== index);
    changed();
  });
  row.append(note, remove);
  return row;
}

// Weapons, typed in as ShadowrunEdit's weapon rows are, each in a mount.

/** Replaces one weapon with a changed copy, and recalculates. */
function updateWeapon(index: number, change: Partial<Weapon>, redraw = false): void {
  design.weapons = design.weapons.map((w, i) => (i === index ? { ...w, ...change } : w));
  changed(redraw);
}

/** A text box that commits on change. */
function textBox(value: string, placeholder: string, onChange: (v: string) => void, cls = "") {
  const input = el("input", { type: "text", placeholder, class: cls });
  input.value = value;
  input.addEventListener("change", () => onChange(input.value.trim()));
  return input;
}

function weaponRow(w: Weapon, index: number): HTMLElement {
  // The mounts to choose from: the vehicle's weapon mounts, and the one
  // this weapon is in even if it's since been removed.
  const mountIds = catalog.mods.filter((m) => m.id.endsWith("weapon-mount")).map((m) => m.id);
  const installed = new Set(design.mods.map((m) => m.id));
  const choices = mountIds
    .filter((id) => installed.has(id) || id === w.mount)
    .map((id) => ({ value: id, label: catalog.mods.find((m) => m.id === id)?.name ?? id }));
  // Deadly Arts' mil-spec hardpoints, numbered from 1.
  if (design.books.includes(DEADLY_ARTS)) {
    design.milspec.hardpoints.forEach((_, i) => {
      const grade = lastGrades[i];
      choices.push({
        value: `milspec-${i + 1}`,
        label: `Mil-spec hardpoint ${i + 1}${grade == null ? "" : ` (Grade ${grade})`}`,
      });
    });
  }
  // A mil-spec hardpoint keeps the weapon's own ammunition and Attack Ratings.
  const onMilspec = w.mount.startsWith("milspec-");
  const mount = select(
    "Mount",
    w.mount,
    [{ value: "", label: choices.length ? "Choose a mount…" : "Add a weapon mount in Mods first" }, ...choices],
    (v) => updateWeapon(index, { mount: v }, true),
  );

  const ratings = el("span", {
    class: "ratings",
    title: onMilspec
      ? "Attack Ratings as the weapon has them; 0 or blank for none"
      : "Attack Ratings before the mount's +2; 0 or blank for none",
  });
  for (const [i, label] of ["Close", "Near", "Medium", "Far", "Extreme"].entries()) {
    const box = numberBox(w.ar[i] ?? 0, 0, 99, `${label} Attack Rating (0 for none)`, (n) => {
      const ar = [0, 1, 2, 3, 4].map((j) => (j === i ? Math.max(0, n) : (w.ar[j] ?? 0)));
      updateWeapon(index, { ar });
    });
    ratings.append(box);
  }

  const remove = el("button", { type: "button", title: "Remove this weapon", class: "remove" }, "×");
  remove.addEventListener("click", () => {
    design.weapons = design.weapons.filter((_, i) => i !== index);
    changed();
  });

  return el(
    "div",
    { class: "weapon" },
    el(
      "div",
      { class: "weapon-top" },
      textBox(w.name, "Weapon name", (name) => updateWeapon(index, { name }), "weapon-name"),
      remove,
    ),
    el(
      "div",
      { class: "weapon-fields" },
      textBox(w.weapon_type, "Type (Assault Rifles)", (weapon_type) => updateWeapon(index, { weapon_type })),
      textBox(w.dv, "DV (4P)", (dv) => updateWeapon(index, { dv }), "short"),
      textBox(w.mode, "Modes (SA/BF/FA)", (mode) => updateWeapon(index, { mode }), "short"),
      ...(onMilspec ? [textBox(w.ammo, "Ammo (200(br))", (ammo) => updateWeapon(index, { ammo }), "short")] : []),
    ),
    el("label", { class: "field" }, el("span", {}, "Attack Ratings"), ratings),
    textBox(w.mods, "Its mods (smartgun system)", (mods) => updateWeapon(index, { mods }), "weapon-mods"),
    mount,
  );
}

function weaponsFieldset(): HTMLElement {
  const add = el("button", { type: "button" }, "Add a weapon");
  add.addEventListener("click", () => {
    // A new weapon goes in the first weapon mount the vehicle has.
    const firstMount = design.mods.find((m) => m.id.endsWith("weapon-mount"))?.id ?? "";
    design.weapons = [
      ...design.weapons,
      { name: "", weapon_type: "", dv: "", mode: "", ar: [0, 0, 0, 0, 0], mods: "", mount: firstMount, ammo: "" },
    ];
    changed();
  });
  return el(
    "fieldset",
    {},
    el("legend", {}, "Weapons"),
    el(
      "p",
      { class: "hint" },
      design.books.includes(DEADLY_ARTS)
        ? "Each in a weapon mount (add mounts in Mods), which sets its ammunition and adds 2 to its Attack Ratings, or on a mil-spec hardpoint, which keeps the weapon's own."
        : "Each in a weapon mount (add mounts in Mods), which sets its ammunition and adds 2 to its Attack Ratings.",
    ),
    ...design.weapons.map(weaponRow),
    add,
  );
}

// The vehicle drawing: click a zone to list its mods. The zone map is a
// second picture in flat colors (see scripts/vehicle-art.html), read once.

/** The zone whose mods are listed. */
let selectedZone: Zone = "chassis";
/** The zone map's pixels, once loaded. */
let zoneMap: ImageData | null = null;
/** The last report, for each zone's usage. */
let lastReport: Report | null = null;

function loadZoneMap(): void {
  const img = new Image();
  img.addEventListener("load", () => {
    const canvas = el("canvas", { width: String(img.naturalWidth), height: String(img.naturalHeight) });
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(img, 0, 0);
    zoneMap = ctx.getImageData(0, 0, canvas.width, canvas.height);
    paintHighlight();
  });
  img.src = "/vehicles/suv-zones.png";
}

/** The zone under the pointer, read from the zone map. */
function zoneUnder(event: MouseEvent, target: HTMLElement): Zone | null {
  if (!zoneMap) return null;
  const box = target.getBoundingClientRect();
  const x = Math.floor(((event.clientX - box.left) / box.width) * zoneMap.width);
  const y = Math.floor(((event.clientY - box.top) / box.height) * zoneMap.height);
  if (x < 0 || y < 0 || x >= zoneMap.width || y >= zoneMap.height) return null;
  const i = (y * zoneMap.width + x) * 4;
  const d = zoneMap.data;
  return zoneAtPixel(d[i], d[i + 1], d[i + 2], d[i + 3]);
}

/** Tints the selected zone on the drawing's overlay. */
function paintHighlight(): void {
  const canvas = document.querySelector<HTMLCanvasElement>(".vehicle canvas");
  if (!canvas || !zoneMap) return;
  canvas.width = zoneMap.width;
  canvas.height = zoneMap.height;
  const ctx = canvas.getContext("2d");
  const color = ZONES.find((z) => z.id === selectedZone)?.color;
  if (!ctx || !color) return;
  const out = ctx.createImageData(zoneMap.width, zoneMap.height);
  const [r, g, b] = [1, 3, 5].map((k) => Number.parseInt(color.slice(k, k + 2), 16));
  const d = zoneMap.data;
  for (let i = 0; i < d.length; i += 4) {
    if (zoneAtPixel(d[i], d[i + 1], d[i + 2], d[i + 3]) === selectedZone) {
      out.data.set([r, g, b, 90], i);
    }
  }
  ctx.putImageData(out, 0, 0);
}

function selectZone(zone: Zone): void {
  selectedZone = zone;
  const scroll = formEl.scrollTop;
  renderForm();
  formEl.scrollTop = scroll;
}

/** Each zone's chip shows how full it is; redone after every report. */
function showZoneUsage(r: Report): void {
  lastReport = r;
  const accessories = design.mods.filter((m) => {
    const def = catalog.mods.find((d) => d.id === m.id);
    return def != null && zoneOf(def) === "fittings";
  }).length;
  for (const chip of document.querySelectorAll<HTMLElement>(".zone-chip")) {
    const { text, over } = usage(chip.dataset.zone as Zone, r, accessories);
    const count = chip.querySelector(".zone-usage");
    if (count) count.textContent = text;
    chip.classList.toggle("over", over);
  }
}

function vehicleView(): HTMLElement {
  const picture = el(
    "div",
    { class: "vehicle", title: "Click a part of the vehicle to list its mods" },
    el("img", { class: "light", src: "/vehicles/suv-light.png", alt: "The vehicle, its mod zones tinted" }),
    el("img", { class: "dark", src: "/vehicles/suv-dark.png", alt: "" }),
    el("canvas"),
  );
  picture.addEventListener("click", (e) => {
    const zone = zoneUnder(e, picture);
    if (zone) selectZone(zone);
  });
  picture.addEventListener("mousemove", (e) => {
    const zone = zoneUnder(e, picture);
    picture.style.cursor = zone ? "pointer" : "default";
    picture.title = zone ? (ZONES.find((z) => z.id === zone)?.name ?? "") : "Click a part of the vehicle to list its mods";
  });
  const chips = ZONES.map((z) => {
    const chip = el(
      "button",
      { type: "button", class: `zone-chip${z.id === selectedZone ? " selected" : ""}`, "data-zone": z.id },
      el("span", { class: "swatch", style: `background:${z.color}` }),
      z.name,
      el("span", { class: "zone-usage" }),
    );
    chip.addEventListener("click", () => selectZone(z.id));
    return chip;
  });
  // The image may already be loaded from an earlier drawing of the form.
  queueMicrotask(() => {
    paintHighlight();
    if (lastReport) showZoneUsage(lastReport);
  });
  return el("div", {}, picture, el("div", { class: "zone-chips" }, ...chips));
}

/** Adds one of a mod at its lowest rating. */
function addMod(id: string): void {
  const def = catalog.mods.find((m) => m.id === id);
  if (!def) return;
  // A rated mod starts at its lowest positive rating.
  const rating = def.rating_max == null ? 0 : Math.max(1, def.rating_min ?? 1);
  design.mods = [...design.mods, { id, rating, count: 1, built_in: false, note: "" }];
  changed();
}

/** The selected zone's mods: tick one to add it, untick to remove it. */
function zoneList(): HTMLElement[] {
  const installedIds = new Set(design.mods.map((m) => m.id));
  const mods = catalog.mods.filter(
    (m) => zoneOf(m) === selectedZone && (m.book == null || design.books.includes(m.book) || installedIds.has(m.id)),
  );
  const rows: HTMLElement[] = [];
  for (const def of mods) {
    const entries = design.mods.map((m, i) => [m, i] as const).filter(([m]) => m.id === def.id);
    const box = el("input", { type: "checkbox" });
    box.checked = entries.length > 0;
    box.addEventListener("change", () => {
      if (box.checked) {
        addMod(def.id);
      } else {
        design.mods = design.mods.filter((m) => m.id !== def.id);
        // A weapon in a mount that's gone keeps its place, flagged.
        changed();
      }
    });
    const off = def.book != null && !design.books.includes(def.book);
    rows.push(
      el(
        "label",
        { class: `zone-mod${off ? " off" : ""}` },
        box,
        el("span", { class: "mod-name" }, def.name),
        el("span", { class: "hint" }, off ? "its book is off" : summary(def)),
      ),
    );
    for (const [m, i] of entries) rows.push(modRow(m, i, false));
  }
  // Mods the catalog doesn't know (a hand-edited file) stay removable.
  if (selectedZone === "chassis") {
    design.mods.forEach((m, i) => {
      if (!catalog.mods.some((d) => d.id === m.id)) rows.push(modRow(m, i));
    });
  }
  return rows;
}

function modsFieldset(): HTMLElement {
  return el(
    "fieldset",
    {},
    el("legend", {}, "Mods"),
    el(
      "p",
      { class: "hint" },
      "Installed afterward, or built into the design (2 Build Points a slot, its price added to the vehicle's).",
    ),
    checkbox(
      "Built-in mods use mod slots",
      design.built_in_uses_slots,
      (on) => {
        design.built_in_uses_slots = on;
        changed();
      },
      "The book doesn't say whether a mod built into the design uses the vehicle's mod slots (RULINGS.md R19). Unticked, they cost Build Points only.",
    ),
    vehicleView(),
    ...packPicker(),
    el("div", { class: "zone-list" }, ...zoneList()),
  );
}

/** A book's packs of mods (Sixth World Companion): picking one adds its mods. */
function packPicker(): HTMLElement[] {
  const packs = catalog.pack.filter((p) => design.books.includes(p.book));
  if (!packs.length) return [];
  return [
    select(
      "Add a pack",
      "",
      [
        { value: "", label: "Choose a pack…" },
        ...packs.map((p) => ({
          value: p.id,
          label: p.min_body == null ? p.name : `${p.name} (Body ${p.min_body} or more)`,
        })),
      ],
      (id) => {
        const pack = packs.find((p) => p.id === id);
        if (!pack) return;
        // Each mod as the pack lists it; a missing count is 1.
        const added = pack.mods.map((m) => ({ ...m, count: Math.max(1, m.count) }));
        design.mods = [...design.mods, ...added];
        changed();
      },
    ),
    el("p", { class: "hint" }, "A pack adds its mods, each priced as usual."),
  ];
}

// The report.

function table(rows: (string | Node)[][], head?: string[]): HTMLTableElement {
  const t = el("table");
  if (head) t.append(el("thead", {}, el("tr", {}, ...head.map((h) => el("th", {}, h)))));
  t.append(el("tbody", {}, ...rows.map((r) => el("tr", {}, ...r.map((c) => el("td", {}, c))))));
  return t;
}

/** The vehicle's picture, with buttons to add, change, or remove it. */
function pictureView(): HTMLElement {
  const input = el("input", { type: "file", accept: "image/png,image/jpeg,image/webp", hidden: "" });
  input.addEventListener("change", async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      design.image = await loadPicture(file);
      changed(false);
    } catch (error) {
      await message(`Couldn't load that picture: ${error}`, { title: "ShadowrunBAV", kind: "error" });
    }
  });
  const choose = (label: string) => {
    const button = el("button", { type: "button" }, label);
    button.addEventListener("click", () => input.click());
    return button;
  };
  if (!isPicture(design.image)) {
    return el("div", { class: "picture empty" }, input, choose("Add a picture…"));
  }
  const remove = el("button", { type: "button" }, "Remove picture");
  remove.addEventListener("click", () => {
    design.image = "";
    changed(false);
  });
  return el(
    "div",
    { class: "picture" },
    el("img", { src: design.image, alt: design.name ? `The ${design.name}` : "The vehicle" }),
    el("div", { class: "picture-buttons" }, input, choose("Change picture…"), remove),
  );
}

function renderReport(r: Report): void {
  const s = r.stats;
  const copy = el("button", { type: "button", title: "Copy the stat block, to paste into ShadowrunEdit or a document" }, "Copy");
  copy.addEventListener("click", async () => {
    await navigator.clipboard.writeText(r.block.text);
    copy.textContent = "Copied";
    setTimeout(() => {
      copy.textContent = "Copy";
    }, 1200);
  });

  // The stat block, as ShadowrunEdit lays it out, under the vehicle's picture.
  const b = r.block;
  const parts: (Node | string)[] = [
    pictureView(),
    el("h2", {}, b.title, " ", el("span", { class: "detail" }, `(${b.detail})`), " ", copy),
    el("div", { class: "statline" }, table([b.main.values], b.main.headers)),
  ];
  if (b.derived) parts.push(el("div", { class: "statline" }, table([b.derived.values], b.derived.headers)));
  for (const line of b.lines) {
    if (line.one_per_line) {
      parts.push(
        el(
          "div",
          { class: "block-line" },
          el("b", {}, `${line.label}:`),
          ...line.entries.map((e) => el("div", { class: "block-entry" }, e)),
        ),
      );
    } else {
      parts.push(el("p", { class: "block-line" }, el("b", {}, `${line.label}: `), line.entries.join(", ")));
    }
  }
  // As built, with its built-in mods: shown when mods are installed afterward.
  if (r.mods.lines.some((line) => !line.built_in)) {
    parts.push(
      el("p", { class: "hint" }, "As built, before the mods installed afterward:"),
      el("div", { class: "statline" }, table([r.factory_stat_line.values], r.factory_stat_line.headers)),
    );
  }
  if (r.problems.length) {
    parts.push(el("ul", { class: "problems" }, ...r.problems.map((p) => el("li", {}, p))));
  }
  parts.push(
    el("h3", {}, "Vehicle"),
    table([
      ["Size Class", `${s.size_class} (${s.length}${s.weight ? `, ${s.weight}` : ""})`],
      ["Cargo", s.cargo],
      ["Speed modifier", String(s.speed_modifier)],
      [
        "Mod slots",
        `Chassis ${s.mod_slots.chassis}, Powertrain ${s.mod_slots.powertrain}, Electronics ${s.mod_slots.electronics}` +
          (s.mod_slots.universal ? `, Universal ${s.mod_slots.universal}` : ""),
      ],
    ]),
  );
  if (r.notes.length) parts.push(el("ul", { class: "notes" }, ...r.notes.map((n) => el("li", {}, n))));
  parts.push(
    el(
      "details",
      {},
      el("summary", {}, "How the speeds were worked out"),
      el("ol", {}, ...r.speed_steps.map((x) => el("li", {}, x))),
    ),
  );

  const pointRows: (string | Node)[][] = r.build_points.map((line) => [line.label, signed(line.points)]);
  pointRows.push([el("b", {}, "Build Points"), el("b", {}, String(r.base_points))]);
  for (const m of r.multipliers) pointRows.push([m.label, factor(m.factor)]);
  if (r.multipliers.length) pointRows.push([el("b", {}, "Build Point Value"), el("b", {}, String(r.build_point_value))]);
  parts.push(el("h3", {}, "Build Points"), table(pointRows));

  parts.push(
    el("h3", {}, "Price and building it"),
    table([
      ["Price", `${thousands(r.cost)}¥ (Build Point Value × 1,000)`],
      ["Availability", r.availability],
      [
        "Design it",
        `Engineering + Logic (${r.design_test.day}, 1 day), (${r.design_test.week}, 1 week), or (${r.design_test.month}, 1 month) Extended test`,
      ],
      ["Parts", `${thousands(r.parts_cost)}¥ to build it yourself, Availability ${r.availability}`],
      [
        "Build it",
        `Engineering + Logic (${r.design_test.build_threshold}, 1 week) Extended test, in a facility` +
          (r.milspec.military ? " (+8, military)" : ""),
      ],
    ]),
  );
  if (r.mods.lines.length) parts.push(...modsReport(r));
  reportEl.replaceChildren(...parts);
}

/** The report's Mods section: each mod, then slots, hardpoints, and prices. */
function modsReport(r: Report): Node[] {
  const m = r.mods;
  const rows: (string | Node)[][] = m.lines.map((line) => [
    line.label,
    slotsText(line.slots) + (line.cf ? `, ${slotsText(line.cf)} CF` : ""),
    line.availability,
    `${thousands(line.price)}¥`,
    line.built_in ? `built in, ${line.build_points} BP` : `${thousands(line.diy_price)}¥ yourself; ${line.install_test}`,
  ]);
  const used = m.slots.map((s) => `${s.category} ${slotsText(s.used)} of ${s.available}`).join(", ");
  const summary: (string | Node)[][] = [
    ["Mod slots", used + (m.universal_slots ? `, plus ${m.universal_slots} universal` : "")],
    [
      "Hardpoints",
      `${slotsText(m.hardpoints.used)} of ${slotsText(m.hardpoints.free + m.hardpoints.added)} standard hardpoints used ` +
        `(${slotsText(m.hardpoints.free)} free, ${slotsText(m.hardpoints.added)} added)`,
    ],
    ["Skin", m.form],
  ];
  if (m.cargo_used) summary.push(["Cargo taken", `${slotsText(m.cargo_used)} CF`]);
  if (m.secondary) {
    const s = m.secondary;
    summary.push([
      "Secondary propulsion",
      `${s.name}: Handling ${s.handling}, Acceleration ${s.acceleration}, Speed Interval ${s.speed_interval}, Top Speed ${s.top_speed}`,
    ]);
  }
  if (m.built_in_cost) summary.push(["Built in", `${thousands(m.built_in_cost)}¥, part of the vehicle's price`]);
  if (m.aftermarket_cost) {
    summary.push([
      "Installed afterward",
      `${thousands(m.aftermarket_cost)}¥ at a shop, or ${thousands(m.aftermarket_diy_cost)}¥ doing it yourself`,
    ]);
  }
  summary.push([el("b", {}, "With its mods"), el("b", {}, `${thousands(r.total_cost)}¥`)]);
  return [
    el("h3", {}, "Mods"),
    table(rows, ["Mod", "Slots", "Avail", "Price", "Installing it"]),
    el("div", { class: "spacer-row" }),
    table(summary),
  ];
}

/** Slots and CF as the book writes them: whole, or with their fraction. */
function slotsText(n: number): string {
  return Number.isInteger(n) ? String(n) : String(Math.round(n * 100) / 100);
}

async function recalculate(): Promise<void> {
  try {
    const report = await invoke<Report>("evaluate_design", { design });
    renderReport(report);
    showZoneUsage(report);
    // The form shows each mil-spec hardpoint's Grade, which Rust works out:
    // redraw it when they change.
    if (JSON.stringify(report.milspec.hardpoints) !== JSON.stringify(lastGrades)) {
      lastGrades = report.milspec.hardpoints;
      const scroll = formEl.scrollTop;
      renderForm();
      formEl.scrollTop = scroll;
    }
  } catch (error) {
    reportEl.replaceChildren(el("ul", { class: "problems" }, el("li", {}, String(error))));
  }
}

/** After any change: redraw the form when its shape may change, then the report. */
function changed(redrawForm = true): void {
  if (redrawForm) {
    const scroll = formEl.scrollTop;
    renderForm();
    formEl.scrollTop = scroll;
  }
  showFile();
  void recalculate();
}

function load(next: Design, nextPath: string | null): void {
  design = next;
  path = nextPath;
  saved = JSON.stringify(design);
  changed();
}

// Files.

/** Asks before throwing away unsaved changes. True to go ahead. */
async function discardOk(): Promise<boolean> {
  if (!isDirty()) return true;
  return ask("Discard the changes to this design?", { title: "ShadowrunBAV", kind: "warning" });
}

/** Saves to `target`, or asks where (Save As). True if it was saved. */
async function saveTo(target: string | null): Promise<boolean> {
  const where = target ?? (await save({ filters: FILTERS, defaultPath: `${design.name || "Vehicle"}.bav` }));
  if (!where) return false;
  try {
    await invoke("save_design", { path: where, design });
    path = where;
    saved = JSON.stringify(design);
    showFile();
    return true;
  } catch (error) {
    await message(String(error), { title: "ShadowrunBAV", kind: "error" });
    return false;
  }
}

/**
 * Closing the window with unsaved changes asks to save them first
 * (src/closing.ts). Tauri closes the window once the handler finishes,
 * unless it calls `preventDefault`.
 */
function askBeforeClosing(): void {
  void getCurrentWindow()
    .onCloseRequested(async (event) => {
      const close = await mayClose({
        dirty: isDirty(),
        path,
        choose: (text) =>
          message(text, {
            title: "ShadowrunBAV",
            kind: "warning",
            buttons: { yes: SAVE, no: DONT_SAVE, cancel: CANCEL },
          }),
        save: () => saveTo(path),
      });
      if (!close) event.preventDefault();
    })
    .catch(() => {}); // no window to watch (a plain browser)
}

const actions: Record<string, () => Promise<void>> = {
  async new() {
    if (await discardOk()) load(newDesign(), null);
  },
  async open() {
    if (!(await discardOk())) return;
    const chosen = await open({ filters: FILTERS, multiple: false });
    if (typeof chosen !== "string") return;
    try {
      load(await invoke<Design>("open_design", { path: chosen }), chosen);
    } catch (error) {
      await message(String(error), { title: "ShadowrunBAV", kind: "error" });
    }
  },
  async save() {
    await saveTo(path);
  },
  async saveAs() {
    await saveTo(null);
  },
  async exportPdf() {
    const where = await save({
      filters: [{ name: "PDF", extensions: ["pdf"] }],
      defaultPath: `${design.name || "Vehicle"}.pdf`,
    });
    if (!where) return;
    try {
      await invoke("export_pdf", { path: where, design });
    } catch (error) {
      await message(String(error), { title: "ShadowrunBAV", kind: "error" });
    }
  },
};

for (const button of document.querySelectorAll<HTMLButtonElement>("[data-action]")) {
  button.addEventListener("click", () => void actions[button.dataset.action ?? ""]?.());
}

document.addEventListener("keydown", (event) => {
  if (!event.ctrlKey && !event.metaKey) return;
  const key = event.key.toLowerCase();
  const shortcuts: Record<string, string> = { n: "new", o: "open", s: event.shiftKey ? "saveAs" : "save", p: "exportPdf" };
  const action = shortcuts[key] ?? null;
  if (action) {
    event.preventDefault();
    void actions[action]();
  }
});

// Updates (src/updater.ts): a found update shows "Update to X" on the
// toolbar; clicking it asks, saves the design if needed, installs, and
// restarts.

const updateEl = document.getElementById("update") as HTMLButtonElement;
let appVersion = "";

/**
 * Before an update restarts the app: saves unsaved changes (Save As for an
 * untitled design), or, if that's cancelled, asks to discard them. False
 * cancels the update.
 */
async function readyToRestart(): Promise<boolean> {
  if (!isDirty()) return true;
  await saveTo(path);
  return !isDirty() || discardOk();
}

function showUpdate(update: Update): void {
  updateEl.textContent = `Update to ${update.version}`;
  updateEl.title = `ShadowrunBAV ${update.version} is available; you have ${update.currentVersion}`;
  updateEl.hidden = false;
  updateEl.onclick = () => void installUpdate(update, readyToRestart);
}

actions.checkUpdates = async () => {
  await checkNow(appVersion, showUpdate, readyToRestart);
};

async function start(): Promise<void> {
  catalog = await invoke<Catalog>("catalog");
  loadZoneMap();
  load(newDesign(), null);
  appVersion = await getVersion();
  (document.getElementById("version") as HTMLElement).textContent = `v${appVersion}`;
  startUpdateChecks(showUpdate);
  askBeforeClosing();
}

void start();
