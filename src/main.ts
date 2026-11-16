// The builder window: a form of the design's choices on the left, and the
// vehicle Rust works out from them on the right, redone on every change.

import { invoke } from "@tauri-apps/api/core";
import { ask, message, open, save } from "@tauri-apps/plugin-dialog";
import { chassisLabel, consoleLabel, factor, fileName, powertrainLabel, signed, statLineText, thousands } from "./format.ts";
import { type Catalog, type Choice, type Design, type Improvements, newDesign, type Report } from "./types.ts";

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
    base,
    options,
    modifications,
    quality,
    el("fieldset", {}, el("legend", {}, "Notes"), notes),
  );
}

// The report.

function table(rows: (string | Node)[][], head?: string[]): HTMLTableElement {
  const t = el("table");
  if (head) t.append(el("thead", {}, el("tr", {}, ...head.map((h) => el("th", {}, h)))));
  t.append(el("tbody", {}, ...rows.map((r) => el("tr", {}, ...r.map((c) => el("td", {}, c))))));
  return t;
}

function renderReport(r: Report): void {
  const s = r.stats;
  const copy = el("button", { type: "button", title: "Copy the stat line as tab-separated text" }, "Copy");
  copy.addEventListener("click", async () => {
    await navigator.clipboard.writeText(statLineText(r.stat_line));
    copy.textContent = "Copied";
    setTimeout(() => {
      copy.textContent = "Copy";
    }, 1200);
  });

  const parts: (Node | string)[] = [
    el("h2", {}, design.name || "Untitled vehicle", " ", copy),
    el("div", { class: "statline" }, table([r.stat_line.values], r.stat_line.headers)),
  ];
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
      ["Build it", "Engineering + Logic (20, 1 week) Extended test, in a facility"],
    ]),
  );
  reportEl.replaceChildren(...parts);
}

async function recalculate(): Promise<void> {
  try {
    renderReport(await invoke<Report>("evaluate_design", { design }));
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

async function saveTo(target: string | null): Promise<void> {
  const where = target ?? (await save({ filters: FILTERS, defaultPath: `${design.name || "Vehicle"}.bav` }));
  if (!where) return;
  try {
    await invoke("save_design", { path: where, design });
    path = where;
    saved = JSON.stringify(design);
    showFile();
  } catch (error) {
    await message(String(error), { title: "ShadowrunBAV", kind: "error" });
  }
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
  save: () => saveTo(path),
  saveAs: () => saveTo(null),
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

async function start(): Promise<void> {
  catalog = await invoke<Catalog>("catalog");
  load(newDesign(), null);
}

void start();
