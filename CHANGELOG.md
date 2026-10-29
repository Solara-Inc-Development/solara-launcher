# Changelog

## Unreleased source expansion — 2026-10-29

- Added official Fabric and Quilt loader version selection, profile validation and verified Maven library installation alongside vanilla.
- Added verified Modrinth mod, resource pack, shader and `.mrpack` modpack installation with required dependency resolution, dual SHA-1 + SHA-512 checksum verification and safe override extraction.
- Added automatic Eclipse Temurin Java runtime acquisition with origin, size, SHA-256 and archive extraction safety checks.
- Rebuilt the home, library and per-instance workspaces with Overview, Mods, Files, Backups, Logs, recoverable Trash and Settings tabs, conflict-safe text editing, isolated world/config snapshots and local JAR mod metadata inspection/import/enable/disable.
- Testing build V-0.0.03 (`8326bb7`) remains withdrawn from public download promotion while a reported VirusTotal generic/ML detection is investigated. Newer source changes above have not been packaged into a new executable.

## Custom-core testing preview (archived V-0.0.03) — 2026-10-29

- Replaced the inherited engine with independently written vanilla metadata, verified downloads, isolated installs, local device sign-in and Java/process handling.
- Added real download cancellation/retry. This first preview exposes vanilla only; mod/content installation is coming later.
- Added the black, rounded desktop layout, smoother toolbar, coloured controls and compact account sidebar.
- Simplified the README and kept build/acceptance status clear.
- Actual Windows installation, licensed sign-in/play and signed upgrades still need testing. This is not a stable release.

## Earlier unreleased work — 2026-10-29

- Fixed first-modpack setup to create a new instance without an existing library entry, with explicit version selection and independent native validation.
- Added bounded local activity for inherited background content installs, including real completion/failure events and indeterminate pending progress.
- Bound search results/actions to their content type and added trusted provider icons, stale-request cancellation and retry states.
- Added account-owned server address listing, endpoint update and explicit release confirmation; kept backend ownership/proof/provider requirements enforced.

- Rebuilt desktop and website with a clean matte dark design, six original optimized landscape artworks and flat controls.
- Added resource/native startup loading, catalogue/provider skeletons, image fallback and retry/error handling, with reduced-motion support.
- Seasonal features are disabled during the current development phase.

## Earlier development — 2026-10-28

- Added Solara desktop hub, shared public-data integration and official website source.
- Added pinned Theseus engine adaptation, native command bridge and isolated local-server management.
- Added shared seasons/festivals, deterministic schedules, previews and outage caching.
- Added independent secure account flows, private API adapters and versioned release tooling.

This is development work, not a public V-0.0.01 release. Windows installation, live SSO, panel mappings and DNS remain subject to the integration and verification checklist. No distributable executable is claimed by this entry.
