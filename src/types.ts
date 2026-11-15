// The shapes Rust sends and receives (bav-core's Catalog, Design, Report),
// as JSON. Keep these in step with crates/bav-core/src/{catalog,design,calc}.rs.

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

export interface Catalog {
  environments: string[];
  chassis: Chassis[];
  powertrain: Powertrain[];
  console: Console[];
  improvement: Improvement[];
  modification: Modification[];
  quality: Quality[];
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
  design_test: { day: number; week: number; month: number };
  speed_steps: string[];
  problems: string[];
  notes: string[];
  stat_line: StatLine;
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
  };
}
