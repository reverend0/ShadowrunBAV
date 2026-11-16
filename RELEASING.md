# Releasing ShadowrunBAV

Releases are built on the owner's Windows machine and published on this repository's GitHub releases page. Installed copies check its latest release for updates (see "Updates" in `DESIGN.md`).

## The signing key

Updates are signed, and the app only installs an update that matches the public key in `src-tauri/tauri.conf.json`. The private key is `%USERPROFILE%\.tauri\shadowrunbav.key` (no password). It never goes in the repository. Back it up somewhere safe: without it, no update can reach installed copies, and a new key would mean every user installing the next version by hand.

## Steps

1. Set the new version in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and `crates/bav-core/Cargo.toml`, and run `npm install` and `cargo check` so the lock files follow.
2. Run the checks in `CONTRIBUTING.md`, and commit.
3. Build the installers, signed (PowerShell):

   ```powershell
   $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "$HOME\.tauri\shadowrunbav.key" -Raw
   $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
   npm run tauri build
   ```

   Each installer in `target\release\bundle\nsis\` and `\msi\` gets a `.sig` beside it.
4. Write the release notes to a file, then the update manifest and checksums:

   ```powershell
   node scripts/release-files.mjs notes.md
   ```

   This writes `latest.json` and `SHA256SUMS.txt` in `target\release\bundle\`.
5. Tag and publish, with the installers, their signatures, the manifest, and the checksums:

   ```powershell
   $v = "0.2.0"; $b = "target/release/bundle"
   git tag "v$v"; git push origin "v$v"
   gh release create "v$v" --title "ShadowrunBAV $v" --notes-file notes.md `
     "$b/nsis/ShadowrunBAV_${v}_x64-setup.exe" "$b/nsis/ShadowrunBAV_${v}_x64-setup.exe.sig" `
     "$b/msi/ShadowrunBAV_${v}_x64_en-US.msi" "$b/msi/ShadowrunBAV_${v}_x64_en-US.msi.sig" `
     "$b/latest.json" "$b/SHA256SUMS.txt"
   ```

Installed copies find it the next time they check (10 seconds after starting, then every 6 hours, or with Check for updates). Version 0.1.1 has no updater, so its users install the first version with one by hand.
