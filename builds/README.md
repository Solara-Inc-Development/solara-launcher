# Windows build archive

All completed distributable Windows installers use the exact public name `solaralauncher.exe`. Preserve each release in its own directory, e.g. `builds/V-0.0.01/solaralauncher.exe`, then increment the internal/public version numerically. The installed application is also solaralauncher.exe. Never overwrite old files with different bytes. Keep associated release metadata, checksums and release notes.

**V-0.0.03 is temporarily withdrawn from download promotion while an antivirus report is investigated. Do not run it yet. Archived bytes remain preserved for analysis.**

The first genuine testing installer is `V-0.0.03/solaralauncher.exe` (7,144,205 bytes). See the [public README](../README.md) for its immutable testing download and exact source. Earlier reservations 1 and 2 were consumed by failed packaging and are not reused.

The actual Windows build, updater signature/version, SHA-256, exact source and embedded x64 payload were checked. Real Windows installation, licensed-account/game play and automatic upgrades still need acceptance. A Windows Authenticode publisher signature is not claimed.

Version directories remain local and on the separate solara-builds archive branch, not main. Each includes source.zip, notes and hashes. Raw Cargo output is under application/; the actual NSIS-installed app is under installed-application/. Their hashes differ only by Tauri's expected UNK-to-NSS bundle marker, verified byte-for-byte. Never overwrite archived files.
