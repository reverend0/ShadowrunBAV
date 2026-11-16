// Updates (DESIGN.md: "Updates"). The app checks the GitHub releases in
// the background: shortly after it starts, then every few hours while it
// stays open. Tauri checks each download's signature against the public key
// in tauri.conf.json before installing it.
//
// A found update never interrupts: a button appears on the toolbar, and only
// clicking it asks to install. A failed check (offline, say) says nothing;
// the next one tries again.

import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import type { Update } from "@tauri-apps/plugin-updater";

import { ask, message } from "@tauri-apps/plugin-dialog";

/** The first check waits a little, so it never slows startup. */
export const FIRST_CHECK_MS = 10_000;
/** Then the app checks again this often while it stays open. */
export const CHECK_EVERY_MS = 6 * 60 * 60 * 1000;

/** A newer version, or null if there is none or the check failed (e.g. offline). */
export async function findUpdate(): Promise<Update | null> {
  try {
    return await check();
  } catch {
    return null;
  }
}

/**
 * Checks after `FIRST_CHECK_MS`, then every `CHECK_EVERY_MS`, until an
 * update turns up; then calls `found` once and stops. Returns a function
 * that stops the checks.
 */
export function startUpdateChecks(found: (update: Update) => void): () => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let stopped = false;
  const run = async () => {
    const update = await findUpdate();
    if (stopped) return;
    if (update) found(update);
    else timer = setTimeout(run, CHECK_EVERY_MS);
  };
  timer = setTimeout(run, FIRST_CHECK_MS);
  return () => {
    stopped = true;
    clearTimeout(timer);
  };
}

export type CheckNowResult = "latest" | "unreachable" | InstallResult;

/**
 * "Check for updates": checks right away and, unlike the background
 * check, always says what it found. `found` runs when there is an update
 * (to show it in the status bar), before you is asked to install it.
 */
export async function checkNow(
  currentVersion: string,
  found: (update: Update) => void,
  beforeRestart: () => Promise<boolean>,
): Promise<CheckNowResult> {
  let update: Update | null;
  try {
    update = await check();
  } catch {
    await message(
      "ShadowrunBAV couldn’t check for updates. You may be offline, or no release is available yet. Try again later.",
      { title: "Check for updates", kind: "warning" },
    );
    return "unreachable";
  }
  if (!update) {
    await message(`You have the latest version of ShadowrunBAV (${currentVersion}).`, {
      title: "Check for updates",
      kind: "info",
    });
    return "latest";
  }
  found(update);
  return installUpdate(update, beforeRestart);
}

export type InstallResult = "declined" | "installed" | "failed";

/**
 * Asks, then installs `update` and restarts. `beforeRestart` runs once the
 * user says yes, to save their design; returning false cancels.
 */
export async function installUpdate(update: Update, beforeRestart: () => Promise<boolean>): Promise<InstallResult> {
  const yes = await ask(
    `ShadowrunBAV ${update.version} is available (you have ${update.currentVersion}).\n\nInstall it and restart now?`,
    { title: "Update available", kind: "info", okLabel: "Install and restart", cancelLabel: "Later" },
  );
  if (!yes || !(await beforeRestart())) return "declined";

  try {
    await update.downloadAndInstall();
  } catch (error) {
    await message(`The update could not be installed. Your design is saved.\n\n${error}`, {
      title: "Update failed",
      kind: "error",
    });
    return "failed";
  }
  await relaunch();
  return "installed";
}
