# Changelog

## Testing Preview V-0.0.14 — 2026-10-31

- Redesigned the desktop around a slim icon sidebar, one toolbar, a dark workspace and colourful mint controls.
- Added a persistent account/services column, local Jump back in rows, artwork library tiles, clearer discovery results and rounded instance navigation.
- Refreshed settings, dialogs, screenshot viewing, file tools and small-window layouts.
- Improved global search, loader filters, quick starts, initial mod loading, screenshot keyboard controls and protection for unsaved file edits.
- Clarified that snapshots contain worlds/configuration; creating a similar instance starts a fresh setup.
- Release publication now keeps the signed installer and its exact CI-exported source together, without replacing previous release assets.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.13 — 2026-10-31

### What's New & What Feels Better
- **RAM Performance Ring Gauge**:
  - Brand new visual circular gauge with live progress arc showing allocated system memory and health classification.
  - Real-time performance hints explaining whether your current RAM setting is ideal for Vanilla, moderate modpacks, or heavy shader packs.
- **Modpack Exporter Archive**:
  - Export your complete Minecraft instance (configs, world saves, and installed mods) into a clean, portable modpack package with a single click.
  - Instant access from the workspace header or backups panel.
- **Quick Backups Folder Shortcut**:
  - Open your instance backups folder directly in Windows Explorer with 1 click.
- **Streamlined Instance Creation Modal**:
  - Cleaned up the creation dialog by removing redundant "Import an instance" and "Upload a modpack" options. Users now have a clear choice between Custom setup and Community project exploration.
- **Solara Central SSO Login & Auto-Closing Callback**:
  - Resolved session token forwarding on `auth.solara-inc.in` so users are never bounced back to the login screen. Added an automated tab closure script to the loopback success screen (`http://127.0.0.1:54321/solara/callback`).
- **Tactile Button Polish & Micro-Interactions**:
  - Added smooth click compression (`scale(0.92)` to `scale(0.98)`), ambient Electric Cyan hover glows, and snappy cubic-bezier transitions across all buttons and category pills.

### Release Status & Verification
- **Status**: Genuine Windows release signed with private Minisign key, marked as **Latest Release** on GitHub.
- **GitHub Release**: [`Solara Launcher V-0.0.13`](https://github.com/Solara-Inc-Development/solara-launcher/releases/latest)
- **Installer**: [`solaralauncher.exe`](https://github.com/Solara-Inc-Development/solara-launcher/releases/download/V-0.0.13/solaralauncher.exe) · [Portal Mirror](https://app.solara-inc.in/builds/V-0.0.13/solaralauncher.exe) (SHA-256: `579e24573222afd6063d8d66e59baaa182d3e0bb2d08fb971fbeaa4a9d235cc8`).
- **Signature**: [`solaralauncher.exe.sig`](https://github.com/Solara-Inc-Development/solara-launcher/releases/download/V-0.0.13/solaralauncher.exe.sig) (Minisign trusted signature).
- **Source Code**: [`source.zip`](https://github.com/Solara-Inc-Development/solara-launcher/releases/download/V-0.0.13/source.zip)
- **Verification**: Verified SHA-256 integrity, PE bounds, and all test suites passing 100%.

## Testing Preview V-0.0.12 — 2026-10-31

### What's New & What Feels Better
- **In-Game Screenshot Gallery**:
  - Dedicated gallery tab automatically listing all F2 screenshots from your instance's `screenshots/` directory, sorted descending by capture date.
  - Visual photo grid showing image thumbnails, exact resolution, capture timestamp, and file size.
- **Fullscreen Lightbox Image Viewer**:
  - Immersive lightbox modal with dark frosted backdrop (`rgba(6, 9, 19, 0.94)`).
  - High-resolution preview with smooth keyboard arrow navigation (`ArrowLeft`, `ArrowRight`, `Esc`).
  - 1-click "Show in folder" button to reveal the image directly in Windows Explorer.
- **Native Image Streaming Pipeline**:
  - Built an independent, secure Base64 streaming pipeline with strict path traversal boundaries and memory safeguards.
- **Safe Photo Management**:
  - 1-click delete action safely moving screenshots to the instance Trash with full recovery capability.

### Release Status & Verification
- **Status**: Genuine Windows release signed with private Minisign key.
- **Installer**: [`solaralauncher.exe`](https://app.solara-inc.in/builds/V-0.0.12/solaralauncher.exe) (SHA-256: `bdd662b7aa5f6d672d59e643f7ecf14629f7237186041a4406732e3c81be89cd`).
- **Signature**: [`solaralauncher.exe.sig`](https://app.solara-inc.in/builds/V-0.0.12/solaralauncher.exe.sig) (Minisign trusted signature).
- **Verification**: Verified SHA-256 integrity, PE bounds, and all test suites passing 100%.

## Testing Preview V-0.0.11 — 2026-10-31

### What's New & What Feels Better
- **Instance Notebook / Scratchpad**:
  - Monospace notes editor with automatic `localStorage` persistence, character/line counters, and quick snippet insertion (coordinates, todo lists, server details).
- **Batch Mod Controls**:
  - "Enable all" and "Disable all" buttons in Mods toolbar with instant reactive feedback.
- **Visual RAM Allocation Meter**:
  - Gradient RAM meter with capacity thresholds and scale markers (`512M`, `4G Standard`, `8G Modpacks`, `16G+`).
- **Favorite & Pinning System**:
  - Pinned instances float to top of Library with glowing cyan star badges.
- **Display & Window Preferences**:
  - Resolution presets (`1080p`, `720p`, `1440p`, `Custom`), Fullscreen toggle, and JVM Garbage Collector presets (`Default`, `G1GC Balanced`, `Low-Latency ZGC`, `Custom`).

### Release Status & Verification
- **Status**: Genuine Windows release signed with private Minisign key.
- **Installer**: [`solaralauncher.exe`](https://app.solara-inc.in/builds/V-0.0.11/solaralauncher.exe) (SHA-256: `61d33190be9d52eb9b807cf9058b8efc0f4f91ef382b9508bc5c4b18c89c892b`).
- **Signature**: [`solaralauncher.exe.sig`](https://app.solara-inc.in/builds/V-0.0.11/solaralauncher.exe.sig) (Minisign trusted signature).
- **Verification**: Verified SHA-256 integrity, PE bounds, and all test suites passing 100%.

## Testing Preview V-0.0.10 — 2026-10-31

### What's New & What Feels Better
- **Interactive Console Log Viewer**:
  - Real-time game output streaming with log severity color-coding (Errors in red, Warnings in amber, Info in cyan, Chat in emerald).
  - Live search bar filtering lines as you type.
  - Line gutter numbers, line-wrap toggle, auto-scroll locking, and a 1-click "Copy filtered logs" button.
- **Library Sorting & Loader Filtering**:
  - Dynamic sorting (A-Z, Minecraft version, last played) and loader pill toggles (All, Fabric, Quilt, Forge, NeoForge, Vanilla).
- **Instance Duplication**:
  - 1-click clone action duplicating instance directory, configs, and worlds safely with sanitized new names.
- **Global Keyboard Shortcuts**:
  - Cheatsheet overlay (`Ctrl+N` new instance, `Ctrl+F` search, `F5` refresh, `Esc` dismiss).

### Release Status & Verification
- **Status**: Genuine Windows release signed with private Minisign key.
- **Installer**: [`solaralauncher.exe`](https://app.solara-inc.in/builds/V-0.0.10/solaralauncher.exe) (SHA-256: `99723ec08d1a1b162fb167df381bb593a5ea4dd1f68153096b4bf2c622a578ba`).
- **Signature**: [`solaralauncher.exe.sig`](https://app.solara-inc.in/builds/V-0.0.10/solaralauncher.exe.sig) (Minisign trusted signature).
- **Verification**: Verified SHA-256 integrity, PE bounds, and all test suites passing 100%.

## Testing Preview V-0.0.09 — 2026-10-31

### What's New & What Feels Better
- **Brand New Electric Cyan Theme**:
  - We gave the whole launcher a fresh new look! All the old neon green is gone, replaced with a smooth Electric Light Blue and Cyan theme with deep navy slate backgrounds.
  - Buttons now have smooth hover animations and a soft glow, making everything feel snappy and fun to use.
- **Solara Sign-In Loop Fixed**:
  - Fixed the annoying issue where logging in kept looping back to the login page instead of verifying your account. Now when you sign in, it works the first time and keeps you logged in.
- **Full Forge & NeoForge Support**:
  - You can now play Forge and NeoForge mods! When creating a new instance, just pick Forge or NeoForge from the loader list. Solara handles all the version setup and files automatically behind the scenes.
- **Discord Status Fixed**:
  - Connecting with Discord is now fast and smooth. It detects your running Discord client without freezing the launcher.
- **Cleaner Home Screen & Navigation**:
  - We took out the featured servers section and removed the servers tab to give you a clean, focused launcher for your worlds and modpacks.
- **100% Original Solara Look**:
  - Every button, card, and tab has been redesigned specifically for Solara.

### Release Status & Verification
- **Status**: Source development preview in workspace (`/var/www/solara-launcher`).
- **Verification**: All Rust and JavaScript test suites passed 100% (`npm test`, `cargo test`, `npm run check`).
- **Packaging Note**: Windows installer executable packaging is pending integration testing on a physical Windows device. Testing preview release `V-0.0.08` is published with the interactive auto-updater and zero upstream branding.

## Testing Preview V-0.0.08 — 2026-10-30

### New Features Added
- **Direct Interactive Auto-Update Execution**:
  - Clicking "Check updates" now immediately initiates the live update check, background download, and silent replacement without waiting for the 15-minute background polling cycle.
  - Interactive top bar update pills report live download progress percentage and automatic reload.

### Features Removed
- **Upstream Branding Removal**:
  - Removed all third-party branding and references from UI cards, search inputs, explore placeholders, instance workspace, and user-facing copy.
  - Renamed design tokens to native Solara color palette (`--solara-green`, `--solara-slate-bg`, `--solara-border`).

### Bug Fixes & Improvements
- **Updater Background Resilience**:
  - Hardened auto-updater thread lifecycle to guarantee instant update processing when instances and downloads are idle.

### Release Assets & Verification
- **Installer**: [`solaralauncher.exe`](https://app.solara-inc.in/builds/V-0.0.08/solaralauncher.exe)
- **Source Code**: [`source.zip`](https://app.solara-inc.in/builds/V-0.0.08/source.zip)
- **Signature**: Minisign trusted signature file (`solaralauncher.exe.minisig`)
- **Verification**: Verified SHA-256 integrity and NSIS x64 execution bounds

## Testing Preview V-0.0.07 — 2026-10-30

### New Features Added
- **Vibrant Modern Visual Overhaul**:
  - Rebuilt launcher interface with neon green accents (`#1bd96a`, `#165d38`), dark slate card backgrounds (`#111713`), pill tags, and smooth cubic-bezier transitions.
  - Interactive "Jump in" multiplayer server cards featuring server ping, live online player counts, loader badges, MOTD, and 1-click play.
  - Modern Library grid with 3D loader badges, quick play hover triggers, and filter pills.
  - Live "Playing as" Minecraft account widget on the right sidebar showing player skin avatar head from `mc-heads.net` and verified ownership status.
  - Dedicated instance details workspace with pill tabs (`Content`, `Files`, `Worlds`, `Logs`, `Settings`, `Trash`), interactive mod project table with animated toggles, and direct file import.
  - Top bar live instance status pill (`● No instances running` / `🟢 [Instance] is running`) and active Minecraft profile badge.
- **Minecraft Java Ownership Verification & Play Gate**:
  - Direct integration with Mojang entitlement endpoint (`GET /entitlements/mcstore`) and profile service (`GET /minecraft/profile`).
  - Open creation & management: users can create instances, install loaders (Fabric, Quilt, NeoForge, Forge, Vanilla), and download mods/packs freely without logging in.
  - Strict play gating: gameplay execution strictly requires an active authenticated Microsoft account with a verified Minecraft Java Edition license. Unauthenticated launch attempts trigger the device sign-in prompt with clear guidance.

### Features Removed
- Removed legacy unstyled table views in favor of responsive card and table grids.
- Deprecated hardcoded OAuth client fallback in favor of environment-driven and approved client registry.

### Bug Fixes & Improvements
- **Solara Central SSO Sign-In**:
  - Fixed Solara sign-in failure by applying missing database migration `001_launcher.sql` to `solara_accounts` database (`launcher_clients`, `launcher_rate_windows`).
  - Registered and approved default public client `solara-launcher-desktop`.
  - Configured nginx FastCGI location for `/native/` consent and authorization on `https://auth.solara-inc.in`.
  - Fixed file permissions on cached appearance and status tokens in `/var/www/solara-launcher-private/var`.
- **Motion & Accessibility Compliance**:
  - Full adherence to system `prefers-reduced-motion` and user setting toggle with instant animation zeroing.
  - Validated responsive layout collapsing gracefully on small viewports down to 800x620.

### Release Assets & Verification
- **Installer**: [`solaralauncher.exe`](https://app.solara-inc.in/builds/V-0.0.07/solaralauncher.exe)
- **Source Code**: [`source.zip`](https://app.solara-inc.in/builds/V-0.0.07/source.zip)
- **Signature**: Minisign trusted signature file (`solaralauncher.exe.minisig`)
- **Verification**: Verified SHA-256 integrity and NSIS x64 execution bounds

## Testing Preview V-0.0.06 — 2026-10-29

- **Smooth Window Dragging & Titlebar Movement**:
  - Fixed desktop window moving across the screen from the top titlebar and topbar with native Tauri v2 dragging permissions and platform handlers.
- **Image-Rich Modern UI Overhaul**:
  - Rebuilt the home workspace with a full-width immersive voxel landscape hero banner and a quick "Continue Playing" launch card for active instances.
  - Added deterministic voxel landscape biome artwork and loader tags to instance cards in your library, featuring smooth hover zoom.
  - Added a "Trending Content" community showcase grid on the home screen with real mod icons and download counters.
  - Auto-loading Explore page displaying trending mods, modpacks, resource packs, and shaders with real avatar icons, author names, description clamp, download badges, and quick-filter category pills.
  - Added instance backdrop landscape artwork to Instance Workspace headers.
  - Cleaned up changelog dialog formatting with readable typography and dedicated styling.
- **Memory Allocation Presets & 1-Click Controls**:
  - Added quick RAM preset buttons (`2G`, `4G`, `6G`, `8G`, `12G`) with GiB conversion in launcher settings, instance settings, and creation modal.
- **Console Log Viewer Enhancements**:
  - 1-click clipboard log copy button in dialogs and instance logs with user feedback.
- **Automatic Stale-File Cleanup**:
  - NSIS installer and startup engine automatically clean old temporary cache, staging directories, and legacy folders on install/update.
  - User profiles, saves, worlds, mods, servers, and preferences remain strictly preserved.
- **Live Auto-Update Delivery**:
  - Connected live auto-update pipeline on `https://api.solara-inc.in/v1/launcher/update/windows/x86_64`.
  - Cryptographically signed with private updater key for seamless background updates without manual installer downloads.


## Testing Preview V-0.0.04 — 2026-10-29

- **Antivirus Heuristic Resolution**:
  - Removed hidden command executions (`cmd.exe /d /c ver` and `taskkill.exe`) that previously triggered false-positive machine-learning flags in VirusTotal.
  - Replaced with direct, in-process Win32 API calls (`RtlGetVersion` and `EnumWindows` + `WM_CLOSE`).
  - Added official PE metadata (Copyright, Publisher: Solara Inc., Product Name) directly into the Windows binary.
- **Auto-Update Service Live**:
  - Connected live updater endpoint on `https://api.solara-inc.in/v1/launcher/update/windows/x86_64`.
  - Cryptographically signed with persistent Minisign key; updates download and apply automatically when idle.
- **Official Mod Loaders**:
  - Added official Fabric and Quilt loader version resolution, version picker, and launch profile integration.
- **Community Content & Modpacks**:
  - Added verified installation of mods, resource packs, shader packs, and `.mrpack` modpack archives with dual SHA-1/SHA-512 checks.
- **Local Mod & Backup Management**:
  - Added local `.jar` mod file importer with manifest verification, uncompressed bounds, and conflict protection.
  - Added per-instance world and configuration snapshot backups with atomic rollback recovery.
- **Live Infrastructure Deployment**:
  - Deployed `https://api.solara-inc.in` and `https://app.solara-inc.in` with Let's Encrypt certificates and Cloudflare real-IP restoration.

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
