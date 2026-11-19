import { describe, expect, it } from "vitest";
import type { Report, VehicleMod } from "./types.ts";
import { summary, usage, zoneAtPixel, zoneOf } from "./zones.ts";

/** A report with just what `usage` reads. */
function report(overrides: { slots?: [string, number, number][]; cargoUsed?: number; cargoLeft?: number; problems?: string[] }) {
  return {
    stats: { cargo_cf: overrides.cargoLeft ?? 4 },
    problems: overrides.problems ?? [],
    mods: {
      slots: (overrides.slots ?? []).map(([category, used, available]) => ({ category, used, available })),
      hardpoints: { free: 3, added: 1, used: 2 },
      cargo_used: overrides.cargoUsed ?? 0,
    },
  } as unknown as Report;
}

function mod(fields: Partial<VehicleMod>): VehicleMod {
  return {
    id: "x",
    name: "X",
    group: "chassis",
    rating_min: null,
    rating_max: null,
    many: false,
    slots: 0,
    slots_per_rating: 0,
    cf: 0,
    cf_per_rating: 0,
    cost: 0,
    cost_per_rating: 0,
    book: null,
    ...fields,
  };
}

describe("zones", () => {
  it("puts each of the chapter's groups in a zone of the vehicle", () => {
    expect(zoneOf({ group: "accessory" })).toBe("fittings");
    expect(zoneOf({ group: "chassis" })).toBe("chassis");
    expect(zoneOf({ group: "skin" })).toBe("chassis");
    expect(zoneOf({ group: "powertrain" })).toBe("powertrain");
    expect(zoneOf({ group: "propulsion" })).toBe("powertrain");
    expect(zoneOf({ group: "electronics" })).toBe("electronics");
    expect(zoneOf({ group: "seating" })).toBe("cargo");
    expect(zoneOf({ group: "hardpoint" })).toBe("hardpoints");
    expect(zoneOf({ group: "device" })).toBe("hardpoints");
  });

  it("reads the zone map's colors, and its background as none", () => {
    expect(zoneAtPixel(255, 0, 0, 255)).toBe("powertrain");
    expect(zoneAtPixel(0, 255, 0, 255)).toBe("electronics");
    expect(zoneAtPixel(0, 0, 255, 255)).toBe("chassis");
    expect(zoneAtPixel(255, 255, 0, 255)).toBe("cargo");
    expect(zoneAtPixel(255, 0, 255, 255)).toBe("hardpoints");
    expect(zoneAtPixel(0, 0, 0, 0)).toBeNull();
    // An edge pixel the browser blended still counts.
    expect(zoneAtPixel(240, 20, 10, 255)).toBe("powertrain");
    expect(zoneAtPixel(128, 128, 128, 255)).toBeNull();
  });

  it("shows each zone's slots used of the vehicle's, and flags one that's over", () => {
    const r = report({
      slots: [
        ["Chassis", 3, 10],
        ["Powertrain", 11.5, 10],
        ["Electronics", 0, 10],
      ],
    });
    expect(usage("chassis", r, 0)).toEqual({ text: "3 of 10 slots", over: false });
    expect(usage("powertrain", r, 0)).toEqual({ text: "11.5 of 10 slots", over: true });
    expect(usage("hardpoints", r, 0)).toEqual({ text: "2 of 4 hardpoints", over: false });
    expect(usage("fittings", r, 1).text).toBe("1 fitted");
  });

  it("shows cargo taken of the vehicle's, or that it's too much", () => {
    expect(usage("cargo", report({ cargoUsed: 1, cargoLeft: 7 }), 0)).toEqual({ text: "1 of 8 CF", over: false });
    const over = report({ cargoUsed: 5, cargoLeft: 0, problems: ["The CF mods take 5 CF; the vehicle has 0.5 CF."] });
    expect(usage("cargo", over, 0)).toEqual({ text: "5 CF, more than it has", over: true });
  });

  it("sums up a mod's slots and price before it's added", () => {
    expect(summary(mod({ slots: 2, cost: 5000 }))).toBe("2 slots · 5,000¥");
    expect(summary(mod({ slots_per_rating: 1, cost_per_rating: 15000 }))).toBe("1 slot a rating · 15,000¥ a rating");
    expect(summary(mod({ cf: 50, cost: 80000 }))).toBe("50 CF · 80,000¥");
  });
});
