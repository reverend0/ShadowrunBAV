import { describe, expect, it } from "vitest";
import { CANCEL, type Closing, DONT_SAVE, mayClose, SAVE } from "./closing.ts";

function app(answer: string, { saved = true, dirty = true, path = "C:/Designs/Raptor.bav" as string | null } = {}) {
  const calls: string[] = [];
  const closing: Closing = {
    dirty,
    path,
    choose: async (text) => {
      calls.push(`ask: ${text}`);
      return answer;
    },
    save: async () => {
      calls.push("save");
      return saved;
    },
  };
  return { closing, calls };
}

describe("closing the window", () => {
  it("closes without asking when everything is saved", async () => {
    const { closing, calls } = app(SAVE, { dirty: false });
    expect(await mayClose(closing)).toBe(true);
    expect(calls).toEqual([]);
  });

  it("asks first with unsaved changes, and Save saves then closes", async () => {
    const { closing, calls } = app(SAVE);
    expect(await mayClose(closing)).toBe(true);
    expect(calls).toEqual(["ask: Save your changes to “Raptor.bav” before closing?", "save"]);
  });

  it("names an untitled design as this design", async () => {
    const { closing, calls } = app(CANCEL, { path: null });
    await mayClose(closing);
    expect(calls).toEqual(["ask: Save your changes to this design before closing?"]);
  });

  it("stays open if the save doesn't happen (Save As cancelled, or an error)", async () => {
    expect(await mayClose(app(SAVE, { saved: false }).closing)).toBe(false);
  });

  it("closes without saving on Don't Save", async () => {
    const { closing, calls } = app(DONT_SAVE);
    expect(await mayClose(closing)).toBe(true);
    expect(calls).not.toContain("save");
  });

  it("stays open on Cancel, or when the dialog is closed", async () => {
    expect(await mayClose(app(CANCEL).closing)).toBe(false);
    expect(await mayClose(app("").closing)).toBe(false);
  });
});
