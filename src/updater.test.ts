// Background update checks and installing (src/updater.ts), with Tauri's
// updater and dialogs mocked.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Update } from "@tauri-apps/plugin-updater";

const mocks = vi.hoisted(() => ({
  check: vi.fn(),
  relaunch: vi.fn(async () => {}),
  ask: vi.fn(async (_text: string, _options?: unknown) => true),
  message: vi.fn(async (_text: string, _options?: unknown) => {}),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({ check: mocks.check }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch: mocks.relaunch }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ ask: mocks.ask, message: mocks.message }));

const { CHECK_EVERY_MS, FIRST_CHECK_MS, checkNow, findUpdate, installUpdate, startUpdateChecks } = await import("./updater");

function anUpdate(install: () => Promise<void> = async () => {}) {
  return { version: "0.9.0", currentVersion: "0.8.0", downloadAndInstall: vi.fn(install) } as unknown as Update & {
    downloadAndInstall: ReturnType<typeof vi.fn>;
  };
}

const saved = vi.fn(async () => true);

beforeEach(() => {
  vi.clearAllMocks();
  mocks.ask.mockResolvedValue(true);
  saved.mockResolvedValue(true);
});

describe("findUpdate", () => {
  it("returns the update, or null when there is none", async () => {
    const update = anUpdate();
    mocks.check.mockResolvedValueOnce(update).mockResolvedValueOnce(null);
    expect(await findUpdate()).toBe(update);
    expect(await findUpdate()).toBeNull();
  });

  it("returns null when offline (you may be), without any dialog", async () => {
    mocks.check.mockRejectedValue(new Error("network unreachable"));
    expect(await findUpdate()).toBeNull();
    expect(mocks.ask).not.toHaveBeenCalled();
    expect(mocks.message).not.toHaveBeenCalled();
  });
});

describe("startUpdateChecks", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("waits before the first check, so startup isn't slowed", async () => {
    mocks.check.mockResolvedValue(null);
    const stop = startUpdateChecks(() => {});
    await vi.advanceTimersByTimeAsync(FIRST_CHECK_MS - 1);
    expect(mocks.check).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(mocks.check).toHaveBeenCalledOnce();
    stop();
  });

  it("checks again in the background while the app stays open, even after failures", async () => {
    mocks.check.mockResolvedValueOnce(null).mockRejectedValueOnce(new Error("offline")).mockResolvedValue(null);
    const stop = startUpdateChecks(() => {});
    await vi.advanceTimersByTimeAsync(FIRST_CHECK_MS + 2 * CHECK_EVERY_MS);
    expect(mocks.check).toHaveBeenCalledTimes(3);
    stop();
  });

  it("reports a found update once, without a dialog, and stops checking", async () => {
    const update = anUpdate();
    mocks.check.mockResolvedValueOnce(null).mockResolvedValue(update);
    const found = vi.fn();
    startUpdateChecks(found);
    await vi.advanceTimersByTimeAsync(FIRST_CHECK_MS + 5 * CHECK_EVERY_MS);
    expect(found).toHaveBeenCalledExactlyOnceWith(update);
    expect(mocks.check).toHaveBeenCalledTimes(2);
    expect(mocks.ask).not.toHaveBeenCalled();
  });

  it("stop() ends the checks", async () => {
    mocks.check.mockResolvedValue(null);
    const stop = startUpdateChecks(() => {});
    stop();
    await vi.advanceTimersByTimeAsync(FIRST_CHECK_MS + CHECK_EVERY_MS);
    expect(mocks.check).not.toHaveBeenCalled();
  });
});

describe("checkNow (the Check for updates button)", () => {
  it("says so when this is the latest version", async () => {
    mocks.check.mockResolvedValue(null);
    expect(await checkNow("0.8.0", () => {}, saved)).toBe("latest");
    expect(mocks.message.mock.calls[0][0]).toBe("You have the latest version of ShadowrunBAV (0.8.0).");
  });

  it("says so when the check fails, unlike the silent background check", async () => {
    mocks.check.mockRejectedValue(new Error("network unreachable"));
    expect(await checkNow("0.8.0", () => {}, saved)).toBe("unreachable");
    expect(mocks.message.mock.calls[0][0]).toContain("couldn’t check for updates");
    expect(mocks.ask).not.toHaveBeenCalled();
  });

  it("shows a found update, then offers to install it", async () => {
    const update = anUpdate();
    mocks.check.mockResolvedValue(update);
    const found = vi.fn();
    expect(await checkNow("0.8.0", found, saved)).toBe("installed");
    expect(found).toHaveBeenCalledExactlyOnceWith(update);
    expect(found.mock.invocationCallOrder[0]).toBeLessThan(mocks.ask.mock.invocationCallOrder[0]);
    expect(update.downloadAndInstall).toHaveBeenCalledOnce();
  });

  it("leaves the update offered when you says Later", async () => {
    mocks.check.mockResolvedValue(anUpdate());
    mocks.ask.mockResolvedValue(false);
    const found = vi.fn();
    expect(await checkNow("0.8.0", found, saved)).toBe("declined");
    expect(found).toHaveBeenCalledOnce();
  });
});

describe("installUpdate", () => {
  it("asks, saves you's work, installs, and restarts", async () => {
    const update = anUpdate();
    expect(await installUpdate(update, saved)).toBe("installed");
    expect(mocks.ask.mock.calls[0][0]).toContain("ShadowrunBAV 0.9.0 is available (you have 0.8.0)");
    expect(saved).toHaveBeenCalledOnce();
    expect(update.downloadAndInstall).toHaveBeenCalledOnce();
    expect(mocks.relaunch).toHaveBeenCalledOnce();
    // Saving happens before the download replaces the app.
    expect(saved.mock.invocationCallOrder[0]).toBeLessThan(update.downloadAndInstall.mock.invocationCallOrder[0]);
  });

  it("does nothing when you says Later", async () => {
    const update = anUpdate();
    mocks.ask.mockResolvedValue(false);
    expect(await installUpdate(update, saved)).toBe("declined");
    expect(saved).not.toHaveBeenCalled();
    expect(update.downloadAndInstall).not.toHaveBeenCalled();
  });

  it("cancels when the work can't be saved", async () => {
    const update = anUpdate();
    saved.mockResolvedValue(false);
    expect(await installUpdate(update, saved)).toBe("declined");
    expect(update.downloadAndInstall).not.toHaveBeenCalled();
    expect(mocks.relaunch).not.toHaveBeenCalled();
  });

  it("reports a failed install and doesn't restart", async () => {
    expect(
      await installUpdate(
        anUpdate(async () => Promise.reject(new Error("bad signature"))),
        saved,
      ),
    ).toBe("failed");
    expect(mocks.message.mock.calls[0][0]).toContain("bad signature");
    expect(mocks.relaunch).not.toHaveBeenCalled();
  });
});
