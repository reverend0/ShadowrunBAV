// The shapes Rust sends and receives (bav-core's Catalog, Design, Report),
// as JSON. Keep these in step with crates/bav-core/src/{catalog,design,calc,mods,block}.rs.

export type Kind = "ground" | "water" | "air" | "drone";

export interface Chassis {
  id: string;
  name: string;
  kind: Kind;
  drone_size: string | null;
  body: number;
  armor: number;
  handling: number;
  seat: number;
  build_points: number;
  size_class: number;
}

export interface Powertrain {
  id: string;
  name: string;
  medium: Kind;
  acceleration: number;
  speed_interval: number;
  top_speed: number;
  build_points: number;
}

export interface Console {
  id: string;
  name: string;
  pilot: number;
  sensor: number;
  build_points: number;
}

export interface Improvement {
  id: keyof Improvements;
  name: string;
  per_level: number;
  max_level: number;
}

export interface Choice {
  id: string;
  name: string;
  group: string | null;
}

export interface Modification extends Choice {
  build_points: number;
}

export interface Quality extends Choice {
  multiplier: number;
}

/** A vehicle mod ("Making It Yours"); data/mods.toml explains the fields. */
export interface VehicleMod {
  id: string;
  name: string;
  group: string;
  rating_min: number | null;
  rating_max: number | null;
  many: boolean;
  /** The sourcebook it's from, when it isn't Double Clutch. */
  book: string | null;
}

/** A sourcebook beyond Double Clutch that a design can turn on. */
export interface Book {
  id: string;
  name: string;
}

export interface Catalog {
  environments: string[];
  chassis: Chassis[];
  powertrain: Powertrain[];
  console: Console[];
  improvement: Improvement[];
  modification: Modification[];
  quality: Quality[];
  mods: VehicleMod[];
  book: Book[];
}

export interface Improvements {
  body: number;
  armor: number;
  handling: number;
  handling_on_road: number;
  handling_off_road: number;
  seating: number;
  size: number;
  acceleration: number;
  speed_interval: number;
  top_speed: number;
  cargo: number;
  easily_modified: number;
}

export interface Design {
  name: string;
  chassis: string;
  powertrain: string;
  console: string;
  improvements: Improvements;
  modifications: string[];
  qualities: string[];
  environment: string;
  specialized_function: string;
  notes: string;
  mods: InstalledMod[];
  weapons: Weapon[];
  layout: "gear" | "gamemaster";
  /** Whether built-in mods use the vehicle's mod slots (RULINGS.md R19). */
  built_in_uses_slots: boolean;
  /** Sourcebooks turned on for this design ("deadly-arts"). */
  books: string[];
  milspec: MilSpec;
}

/** Deadly Arts' mil-spec design options (pp. 150–152). */
export interface MilSpec {
  airframe_plate: boolean;
  /** 4 to 6, or 0 for none. */
  heavy_plate: number;
  /** I, II, III as 1 to 3, or 0 for none. */
  comms: number;
  /** 1 to 12, or 0 for none. */
  frame: number;
  hardpoints: MilspecHardpoint[];
}

export interface MilspecHardpoint {
  /** The weapon's Size. */
  size: number;
  mount: "turreted" | "inline" | "open";
  pop_out: boolean;
  /** 1, 2 (twin-linked), or 4 (quad-linked). */
  linked: number;
  primary: boolean;
  note: string;
}

/** A weapon, typed in; its mount sets its ammunition and adds 2 to its Attack Ratings. */
export interface Weapon {
  name: string;
  weapon_type: string;
  dv: string;
  mode: string;
  /** Five Attack Ratings; 0 is an em dash. */
  ar: number[];
  mods: string;
  /** The weapon mount's mod id, or a mil-spec hardpoint's ("milspec-1"). */
  mount: string;
  /** Its ammunition, for a mil-spec hardpoint, which doesn't set it. */
  ammo: string;
}

export interface InstalledMod {
  id: string;
  rating: number;
  count: number;
  built_in: boolean;
  note: string;
}

export interface StatLine {
  headers: string[];
  values: string[];
}

export interface Report {
  stats: {
    body: number;
    armor: number;
    handling: string;
    seat: number;
    size_class: number;
    length: string;
    weight: string;
    cargo: string;
    acceleration: number;
    speed_interval: number;
    top_speed: number;
    speed_modifier: number;
    pilot: number;
    sensor: number;
    mod_slots: { chassis: number; powertrain: number; electronics: number; universal: number };
  };
  build_points: { label: string; points: number }[];
  base_points: number;
  multipliers: { label: string; factor: number }[];
  multiplier: number;
  build_point_value: number;
  cost: number;
  parts_cost: number;
  availability: string;
  design_test: { day: number; week: number; month: number; build_threshold: number };
  speed_steps: string[];
  problems: string[];
  notes: string[];
  stat_line: StatLine;
  factory_stat_line: StatLine;
  total_cost: number;
  mods: ModsReport;
  /** Deadly Arts' mil-spec options, when it's on. */
  milspec: { hardpoints: number[]; military: boolean };
  block: StatBlock;
}

/** The stat block, laid out as ShadowrunEdit lays out a vehicle. */
export interface StatBlock {
  title: string;
  detail: string;
  main: StatLine;
  derived: StatLine | null;
  lines: { label: string; entries: string[]; one_per_line: boolean }[];
  text: string;
}

export interface ModsReport {
  lines: {
    id: string;
    label: string;
    slots: number;
    cf: number;
    availability: string;
    price: number;
    diy_price: number;
    built_in: boolean;
    build_points: number;
    install_test: string;
  }[];
  slots: { category: string; used: number; available: number }[];
  universal_slots: number;
  hardpoints: { free: number; added: number; used: number };
  cargo_used: number;
  built_in_points: number;
  built_in_cost: number;
  aftermarket_cost: number;
  aftermarket_diy_cost: number;
  secondary: { name: string; handling: string; acceleration: number; speed_interval: number; top_speed: number } | null;
  form: string;
}

export function newDesign(): Design {
  return {
    name: "",
    chassis: "car",
    powertrain: "wheeled",
    console: "basic",
    improvements: {
      body: 0,
      armor: 0,
      handling: 0,
      handling_on_road: 0,
      handling_off_road: 0,
      seating: 0,
      size: 0,
      acceleration: 0,
      speed_interval: 0,
      top_speed: 0,
      cargo: 0,
      easily_modified: 0,
    },
    modifications: [],
    qualities: [],
    environment: "",
    specialized_function: "",
    notes: "",
    mods: [],
    weapons: [],
    layout: "gear",
    built_in_uses_slots: true,
    books: [],
    milspec: { airframe_plate: false, heavy_plate: 0, comms: 0, frame: 0, hardpoints: [] },
  };
}
