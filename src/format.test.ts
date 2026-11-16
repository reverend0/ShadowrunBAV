import { describe, expect, it } from "vitest";
import { chassisLabel, factor, fileName, signed, thousands } from "./format.ts";

describe("format", () => {
  it("writes thousands with commas", () => {
    expect(thousands(173000)).toBe("173,000");
    expect(thousands(999)).toBe("999");
  });

  it("signs Build Points with an en dash for minus", () => {
    expect(signed(25)).toBe("+25");
    expect(signed(-10)).toBe("–10");
    expect(signed(0)).toBe("0");
  });

  it("writes multipliers as the book does", () => {
    expect(factor(0.5)).toBe("× 0.5");
  });

  it("takes the file name from Windows and other paths", () => {
    expect(fileName("C:\\Designs\\Raptor.bav")).toBe("Raptor.bav");
    expect(fileName("/home/me/drone.bav")).toBe("drone.bav");
  });

  it("leaves Seat out of a drone chassis's label", () => {
    const drone = {
      id: "drone-small",
      name: "Drone, Small",
      kind: "drone" as const,
      drone_size: "small",
      body: 2,
      armor: 1,
      handling: 3,
      seat: 0,
      build_points: 8,
      size_class: 2,
    };
    expect(chassisLabel(drone)).toBe("Drone, Small — Body 2, Armor 1, Hand 3, Size 2 (8 BP)");
  });
});
