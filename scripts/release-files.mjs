// Writes a release's extra files from a signed build (`npm run tauri build`
// with TAURI_SIGNING_PRIVATE_KEY set; see RELEASING.md):
//
// - latest.json, the update manifest installed copies check: the version,
//   the notes, and each installer's download link and signature;
// - SHA256SUMS.txt, the installers' checksums.
//
// Usage: node scripts/release-files.mjs [notes.md]
// Both files land in target/release/bundle/, beside the installers.

import { createHash } from "node:crypto";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";

const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const version = conf.version;
const bundle = "target/release/bundle";
const download = `https://github.com/reverend0/ShadowrunBAV/releases/download/v${version}`;

const installers = {
  nsis: join(bundle, "nsis", `ShadowrunBAV_${version}_x64-setup.exe`),
  msi: join(bundle, "msi", `ShadowrunBAV_${version}_x64_en-US.msi`),
};

for (const file of Object.values(installers)) {
  if (!existsSync(file)) throw new Error(`${file} is missing: build ${version} first.`);
  if (!existsSync(`${file}.sig`)) {
    throw new Error(`${file}.sig is missing: build with TAURI_SIGNING_PRIVATE_KEY set (see RELEASING.md).`);
  }
}

/** An installer's entry in the manifest: its signature and where to download it. */
const entry = (file) => ({
  signature: readFileSync(`${file}.sig`, "utf8").trim(),
  url: `${download}/${basename(file)}`,
});

const notesFile = process.argv[2];
const manifest = {
  version,
  notes: notesFile ? readFileSync(notesFile, "utf8").trim() : `ShadowrunBAV ${version}`,
  pub_date: new Date().toISOString(),
  platforms: {
    // The NSIS installer is the default; the MSI is offered to copies
    // installed from the MSI.
    "windows-x86_64": entry(installers.nsis),
    "windows-x86_64-nsis": entry(installers.nsis),
    "windows-x86_64-msi": entry(installers.msi),
  },
};
writeFileSync(join(bundle, "latest.json"), `${JSON.stringify(manifest, null, 2)}\n`);

const sums = Object.values(installers)
  .map((file) => `${createHash("sha256").update(readFileSync(file)).digest("hex")}  ${basename(file)}`)
  .join("\n");
writeFileSync(join(bundle, "SHA256SUMS.txt"), `${sums}\n`);

console.log(`Wrote ${bundle}/latest.json and SHA256SUMS.txt for ${version}.`);
