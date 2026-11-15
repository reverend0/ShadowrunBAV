// Small text helpers for the UI, kept apart from the DOM so they can be tested.

import type { Chassis, Console, Powertrain, StatLine } from "./types.ts";

/** 173000 → "173,000". */
export function thousands(n: number): string {
  return Math.trunc(n).toLocaleString("en-US");
}

/** A Build Point count with its sign: "+25", "–10", "0". */
export function signed(n: number): string {
  if (n > 0) return `+${n}`;
  if (n < 0) return `–${-n}`;
  return "0";
}

/** A multiplier as the book writes it: "× 0.5". */
export function factor(n: number): string {
  return `× ${n}`;
}

/** The stat line as tab-separated rows, to paste into a document. */
export function statLineText(line: StatLine): string {
  return `${line.headers.join("\t")}\n${line.values.join("\t")}`;
}

/** The last part of a path, for the title: "C:\\Designs\\Raptor.bav" → "Raptor.bav". */
export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function chassisLabel(c: Chassis): string {
  const seat = c.kind === "drone" ? "" : `, Seat ${c.seat}`;
  return `${c.name} — Body ${c.body}, Armor ${c.armor}, Hand ${c.handling}${seat}, Size ${c.size_class} (${c.build_points} BP)`;
}

export function powertrainLabel(p: Powertrain): string {
  return `${p.name} — Accel ${p.acceleration}, Spd Int ${p.speed_interval}, Top Spd ${thousands(p.top_speed)} (${thousands(p.build_points)} BP)`;
}

export function consoleLabel(c: Console): string {
  return `${c.name} — Pilot ${c.pilot}, Sensor ${c.sensor} (${c.build_points} BP)`;
}
