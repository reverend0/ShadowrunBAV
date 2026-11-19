import { describe, expect, it } from "vitest";
import { fitSize, isPicture } from "./picture.ts";

describe("picture", () => {
  it("shrinks a big picture to fit its longest side, keeping its shape", () => {
    expect(fitSize(4000, 3000)).toEqual({ width: 1000, height: 750 });
    expect(fitSize(1200, 2400)).toEqual({ width: 500, height: 1000 });
  });

  it("leaves a small picture its size", () => {
    expect(fitSize(640, 480)).toEqual({ width: 640, height: 480 });
  });

  it("shows only PNG and JPEG pictures", () => {
    expect(isPicture("data:image/png;base64,iVBOR")).toBe(true);
    expect(isPicture("data:image/jpeg;base64,/9j/")).toBe(true);
    expect(isPicture("data:image/svg+xml;base64,PHN2")).toBe(false);
    expect(isPicture("")).toBe(false);
  });
});
