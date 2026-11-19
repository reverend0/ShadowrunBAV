// The mod picker's zones: where each mod sits on the vehicle drawing, how
// full each zone is, and which zone a click on the drawing hit.
//
// A zone is one of the chapter's groups of mods and the budget they draw on
// (p. 120): chassis, powertrain, and electronics slots, hardpoints, and cargo
// space. Accessories take no slots, so they aren't on the drawing.

import type { Report, VehicleMod } from "./types.ts";

export type Zone = "chassis" | "powertrain" | "electronics" | "cargo" | "hardpoints" | "fittings";

export const ZONES: { id: Zone; name: string; color: string }[] = [
  { id: "chassis", name: "Chassis", color: "#7f77dd" },
  { id: "powertrain", name: "Powertrain", color: "#d85a30" },
  { id: "electronics", name: "Electronics", color: "#1d9e75" },
  { id: "cargo", name: "Seating and cargo", color: "#888780" },
  { id: "hardpoints", name: "Hardpoints", color: "#ba7517" },
  { id: "fittings", name: "Accessories", color: "#9aa0a6" },
];

/** The zone of a mod, by the group the chapter puts it in. */
export function zoneOf(mod: Pick<VehicleMod, "group">): Zone {
  switch (mod.group) {
    case "accessory":
      return "fittings";
    case "powertrain":
    case "propulsion":
      return "powertrain";
    case "electronics":
      return "electronics";
    case "seating":
      return "cargo";
    case "hardpoint":
    case "device":
      return "hardpoints";
    default:
      return "chassis"; // chassis and skin
  }
}

/**
 * The flat colors of public/vehicles/suv-zones.png, drawn by
 * scripts/vehicle-art.html: keep the two in step.
 */
const ZONE_COLORS: [Zone, number, number, number][] = [
  ["powertrain", 255, 0, 0],
  ["electronics", 0, 255, 0],
  ["chassis", 0, 0, 255],
  ["cargo", 255, 255, 0],
  ["hardpoints", 255, 0, 255],
];

/** The zone a pixel of the zone map shows, or null for the background. */
export function zoneAtPixel(r: number, g: number, b: number, a: number): Zone | null {
  if (a < 128) return null;
  // The nearest color, so an edge pixel blended by the browser still counts.
  let best: Zone | null = null;
  let bestDistance = 3 * 60 * 60;
  for (const [zone, zr, zg, zb] of ZONE_COLORS) {
    const d = (r - zr) ** 2 + (g - zg) ** 2 + (b - zb) ** 2;
    if (d < bestDistance) {
      best = zone;
      bestDistance = d;
    }
  }
  return best;
}

/** A count as the book writes it: whole, or with its fraction. */
function num(n: number): string {
  return Number.isInteger(n) ? String(n) : String(Math.round(n * 100) / 100);
}

/** How full a zone is, from the report: "3 of 10 slots", and whether it's over. */
export function usage(zone: Zone, r: Report, accessories: number): { text: string; over: boolean } {
  const m = r.mods;
  const slots = (category: string) => {
    const s = m.slots.find((x) => x.category === category);
    if (!s) return { text: "", over: false };
    return { text: `${num(s.used)} of ${s.available} slots`, over: s.used > s.available };
  };
  switch (zone) {
    case "chassis":
      return slots("Chassis");
    case "powertrain":
      return slots("Powertrain");
    case "electronics":
      return slots("Electronics");
    case "hardpoints": {
      const total = m.hardpoints.free + m.hardpoints.added;
      return { text: `${num(m.hardpoints.used)} of ${num(total)} hardpoints`, over: m.hardpoints.used > total };
    }
    case "cargo": {
      // The report's cargo is what's left after the mods, held at 0, so a
      // zone that's over says so rather than giving a total.
      const over = r.problems.some((p) => p.startsWith("The CF mods take"));
      if (over) return { text: `${num(m.cargo_used)} CF, more than it has`, over };
      return { text: `${num(m.cargo_used)} of ${num(m.cargo_used + r.stats.cargo_cf)} CF`, over };
    }
    case "fittings":
      return { text: accessories === 1 ? "1 fitted" : `${accessories} fitted`, over: false };
  }
}

/** A mod's slots and price in a few words, for its row before it's added. */
export function summary(mod: VehicleMod): string {
  const parts: string[] = [];
  if (mod.slots > 0) parts.push(`${num(mod.slots)} slot${mod.slots === 1 ? "" : "s"}`);
  if (mod.slots_per_rating > 0) parts.push(`${num(mod.slots_per_rating)} slot${mod.slots_per_rating === 1 ? "" : "s"} a rating`);
  if (mod.cf > 0) parts.push(`${num(mod.cf)} CF`);
  if (mod.cf_per_rating > 0) parts.push(`${num(mod.cf_per_rating)} CF a rating`);
  if (mod.cost > 0) parts.push(`${mod.cost.toLocaleString("en-US")}¥`);
  if (mod.cost_per_rating > 0) parts.push(`${mod.cost_per_rating.toLocaleString("en-US")}¥ a rating`);
  return parts.join(" · ");
}
