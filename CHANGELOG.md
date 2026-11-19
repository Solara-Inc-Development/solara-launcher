# Changelog

## V-0.0.73 — Pose-Only 3D Character Animations & Borderless Skin Studio Cards (2026-11-19)

- Pose-Only 3D Character Animations: Removed hand-held tool and item prop animations (Sword, Pickaxe, Torch, Golden Apple) and kept 6 clean, smooth pose animations across Home and Skin Studio: Wave, Sit & Stand, Victory Jump, Stretch, Look Around, and Salute.
- Borderless Skin Studio Cards: Removed excessive and nested borders across Skin Studio skin cards, pills, action buttons, header, Cape Vault, and pedestal for a cleaner wardrobe gallery in both OLED and White themes.
- Signed Windows testing preview. Real-device installation, gameplay and update/relaunch acceptance remain pending.

## V-0.0.72 — Animated Solara-Inc Hero Emblem, Library Page, 2-Theme Mode & Top Download Manager (2026-11-19)

- Animated Solara-Inc Hero Emblem: Added a balanced floating Solara logo with cyan-amber aura, dual orbital rings, orbiting spark dot, and `Solara-Inc` signature on the Home screen opposite the 3D animated character.
- Dedicated Library Page & Refreshed Sidebar Icons: Replaced Screenshots in the left sidebar with a dedicated Library page (`/library`) featuring search, loader filter pills, sorting, and quick instance launch/management, and updated sidebar icons for ModPackInstaller, Skin Studio, and Library.
- Two Clean Themes Only: Simplified Settings → Appearance to **OLED** (default) and **White**.
- Always-Visible Top Download Manager: Placed the Modrinth-style Download Manager permanently right next to Check for updates, showing live Java runtime, instance, and modpack downloads alongside completed history with Clear all.
- Signed Windows testing preview. Real-device installation, gameplay and update/relaunch acceptance remain pending.

## V-0.0.71 — Custom 3D Character Animations & Interactive Minecraft Props (2026-11-19)

- 7 Custom Procedural Character Actions: Replaced default idle sub-clips with original Solara procedural animations playing automatically on both Home and Skin Studio: Eating a Golden Apple (with 3D voxel apple & golden crumbs), Diamond Sword flourish, Diamond Pickaxe mining swings, Glowing Torch cave look-around (with rising embers), friendly Waving, Sitting & Standing, and Victory Jump.
- Skin Studio Action Bar: Added one-click Action pills (`Wave`, `Golden Apple`, `Sword`, `Pickaxe`, `Torch`, `Sit`, `Jump`) on the right-side 3D pedestal and click-to-animate character interactions.
- Signed Windows testing preview. Real-device installation, gameplay and update/relaunch acceptance remain pending.

## V-0.0.70 — Original Solara Skin Studio, Centered Home Hero & Sidebar Home Icon (2026-11-19)

- Original Solara Skin Studio: Completely redesigned Wardrobe & Cape Editor with a top command bar, segmented My Skins / Presets / Capes tabs, drag-and-drop `.PNG` upload banner, dedicated Cape Vault, and a sticky elevated right-side 3D animated character pedestal with Classic/Slim quick switcher and nametag toggle.
- Sidebar Home Icon: Replaced the left sidebar Home button and top breadcrumb icon with a clean Home icon, and renamed the Skins tooltip to Skin Studio.
- Premium Centered Home Hero: Centered the Welcome Back header, cyan gradient player name, "Good To See You Back :]", and Sign In / Account dropdown pill right in the center of the Home screen with the live 3D animated player skin cleanly docked to the far right edge.
- Signed Windows testing preview. Real-device installation, gameplay and update/relaunch acceptance remain pending.

## V-0.0.68 — Home Screen Redesign with 3D Animated Skin & Instant Update Checks (2026-11-19)

- Redesigned Home Screen Hero: Full-viewport Welcome Back screen with indented player name, "Good To See You Back :]", rounded Sign In / Account dropdown pill (switch accounts, add more accounts, logout), and live 3D animated player skin showcase matching the Skin Editor.
- All News from Solara: Added a 5-card portrait news strip below the fold with "... Read more" modals and a "See all news" viewer capped to the latest 30 updates.
- All Updates of Solara Launcher: Added a framed horizontal update strip with "... Read more" structured changelog modals and an "All version history" viewer capped to the latest 30 releases.
- Fast Check for Updates: Replaced the unpooled 60s updater client with pooled IPv4-first and dual-stack clients racing GitHub Releases CDN and Solara API in parallel (eliminating 20-30s Windows IPv6 SYN stalls) and removed duplicate startup checks.
- Signed Windows testing preview. Real-device installation, gameplay and update/relaunch acceptance remain pending.

## V-0.0.67 — Startup Black Screen Fix & Bootstrap Reliability (2026-11-19)

- Startup Fix: Resolved the blank black screen on launch caused by calling `useMinecraftAccounts()` inside root `App.vue` without passing `appEvents`.
- Update Check Ordering: Moved initial `checkUpdates()` to the post-mount lifecycle hook so update listeners, state variables, and modals are initialized before startup update checks run.
- Early Error Safety: Guarded `useError()` severe error reporting when an error occurs before `ErrorModal` mounts.
- GitHub Release Notes: Included direct download links (`solaralauncher.exe`, `.sig`, `latest.json`, `source.zip`, and GitHub source `.zip`/`.tar.gz` archives) and SHA-256 checksums in release descriptions.
- Signed Windows testing preview. Real-device installation, gameplay and update/relaunch acceptance remain pending.

## V-0.0.66 — Home Dashboard Redesign, ModPackInstaller & Clean Update Flow (2026-11-18)

- Redesigned Home Dashboard: Single scrollable screen with personalized welcome greeting ("Welcome Back, [Username]" / "Good To See You back :]"), integrated "Playing as" Minecraft account card with 3D skin head avatar, quick-play instances grid, Solara News, and structured Version History.
- ModPackInstaller: Renamed sidebar Discover content button to ModPackInstaller and restricted global browse mode strictly to community modpacks (hiding other tabs and redirecting non-modpack global routes).
- Floating Update Modal: Replaced top-bar "No instances running" with an active "Check for updates" button (auto-checks on start and manual click) triggering a clean floating Update Available modal with structured changelogs (Features Added, Issues Fixed, Future Updates) free of emojis.
- Full-Width Layout: Removed 300px right sidebar and title-bar sidebar toggle to provide an expansive, clean desktop workspace.
- Shared Minecraft Account State: Decoupled account management into a shared reactive composable across components.
- Signed Windows testing preview. Real-device installation, gameplay and update/relaunch acceptance remain pending.

## V-0.0.65 — Solara identity and cyan accents (2026-11-15)

- Rebrand built-in desktop copy, translations, welcome artwork, file dialogs and support links for Solara Launcher.
- Use cyan accents across light, dark, OLED and retro themes while retaining the existing layout and controls.
- Remove unavailable catalog sign-in and cloud-sync prompts. Public content browsing and local installs need no catalog account; Microsoft/Minecraft sign-in remains required for licensed play.
- Preserve existing theme choices, provider protocols, local-data compatibility, and required third-party licences and attribution. This update retains the inherited launcher engine.
- Signed Windows testing preview. Real-device installation, gameplay and update/relaunch acceptance remain pending.

## Testing Preview V-0.0.63 — 2026-11-15

- **Full Modrinth App Integration + Solara Microsoft OAuth & GitHub Releases Auto-Updater**:
  - Integrated the complete Modrinth App desktop experience (`apps/app` + `apps/app-frontend` + `packages/*`, GPL-3.0-only) including Home, Library, Discover (Modpacks, Mods, Shaders, Resource Packs, Data Packs), 3D Skin & Cape Studio (`Skins.vue`), Instance Creation & Import Flow, Worlds, Mods/Content Manager, Logs, Screenshots, and App Settings.
  - Preserved Solara's Microsoft OAuth 2.0 PKCE loopback login (`DEFAULT_SOLARA_MICROSOFT_CLIENT_ID = "6a9238bb-3803-46f4-bf38-2f45bdf8c2ea"`, Windows Credential Manager `in.solara.launcher.minecraft.custom`, redacted SQLite/IPC credentials, and automatic migration of existing `V-0.0.6x` `accounts.json` and instances).
  - Preserved Solara's GitHub Releases CDN + Minisign auto-updater (`apps/app/src/app_updates.rs`, `update_policy.rs`, `updater.pub`, startup `"New Version Available"` modal with `"Update Now"` / `"I'll do it later"`, and topbar `"Restart to Update"` button).
  - Stripped all ads, telemetry, billing, server hosting, Modrinth account/friends/shared-instances modules, and collapsed the 300x250 sidebar ad placeholder so zero empty ad space remains.
  - Enforced Windows MSVC 8 MiB main-thread stack (`/STACK:8388608`), `Box::pin` around heavy async Tauri commands, and redacted `crash.log` panic hook.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.62 — 2026-11-15

- **Website Decoupling, Toolchain Upgrade & Upstream License Audit**:
  - Relocated Solara's website UI components and CSS tokens cleanly from `packages/ui/` into `apps/website/src/ui/` (`apps/website/main.ts`, `apps/website/App.vue`, `apps/desktop/ui/icons.ts`) so the website stays independently buildable ahead of desktop workspace integration.
  - Upgraded GitHub Actions workflows (`.github/workflows/windows-check.yml`, `windows-preview.yml`, `publish-release.yml`, `windows.yml`) and VPS build environment to Node `24.15.0`, `pnpm@12.11.2`, and Temurin JDK `17` cleanly.
  - Completed full `GPL-3.0-only` & transitive dependency license review (`docs/upstream-license-review.json`, `THIRD_PARTY_NOTICES.md`) and upgraded `scripts/third-party.mjs` to support both `package-lock.json` and `pnpm-lock.yaml`, local workspace crates, `license-file` crates (`cfg_block`), and strict blocking of proprietary/telemetry dependencies.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.61 — 2026-11-15

- **Mojang Skin & Cape Sync Hardening + Cache & Concurrency Fixes**:
  - Updated `apps/desktop/src/game_core/extras.rs` (`upload_minecraft_skin_with_base`, `set_minecraft_cape_with_base`) so partial Mojang profile responses missing either `skins` or `capes` automatically re-fetch `GET /minecraft/profile` and preserve both arrays without wiping state, with a dedicated Rust loopback regression test (`partial_mojang_skin_or_cape_responses_preserve_both_arrays_via_refetch`).
  - Updated `apps/desktop/ui/views/SkinsView.vue` to split local-only `reloadSavedSkins()` from Mojang profile sync, key head/cape thumbnail caches by source `pngDataUrl` (cleared on account switch), guard `applySkinToAccount()` and `equipCape()` against concurrent/stale account-switch races via `profileSyncSeq`, and disable `Equip` while uploading.
  - Expanded `scripts/verify-redesign-e2e.cjs` and `tests/release-hardening-v59.test.mjs` to assert zero redundant Mojang profile syncs on local skin saves and partial-response re-fetching.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.60 — 2026-11-15

- **Flagship OLED Glass & Live Skin Studio Final Polish**:
  - Added front-face (`10×16`) pixel-art cape thumbnail extraction (`extractCapeThumb`) for owned Mojang capes and defensive dimension fallbacks in `extractHeadThumbnail` inside `SkinsView.vue`, alongside crisp pixelated cape previews and snappier startup splash progress tracking (`140ms`) in `apps/desktop/launcher.css`.
  - Upgraded Chromium E2E skin & cape fixtures in `scripts/verify-redesign-e2e.cjs` with full 64×64 skin and 64×32 cape textures and audited all desktop and website captures (`e2e-splash.png`, `e2e-home.png`, `e2e-skins.png`, `e2e-capes.png`, `e2e-skin-editor.png`, `e2e-settings.png`, `e2e-website.png`, `public/screenshots/dashboard.png`, `public/screenshots/library.png`).
  - Added `tests/release-hardening-v60.test.mjs` completing the sequential `V-0.0.42` through `V-0.0.60` pure-launcher OLED glass, Skin/Cape Studio, website, and live Mojang sync release series.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.59 — 2026-11-15

- **Live Mojang Skin & Cape Sync + Pure Catalog Copy**:
  - Upgraded `apps/desktop/src/game_core/extras.rs`, `engine.rs`, `shell.rs`, and `SkinsView.vue` so signed-in Microsoft accounts fetch live skins and owned capes directly from Mojang (`GET /minecraft/profile`, including `http://` -> `https://textures.minecraft.net` normalization), display them cleanly as the live source of truth (`Live on Mojang`, `Sync from Mojang`), and upload edited/imported/selected skins live to Mojang (`POST /minecraft/profile/skins`) and equip/hide owned capes (`PUT`/`DELETE /minecraft/profile/capes/active`) with hydrated profile responses and honest error/signed-out states.
  - Added Rust HTTP loopback unit tests in `extras.rs` covering profile hydration, multipart skin upload, cape activation/removal, untrusted host blocking, and access-token redaction across errors.
  - Replaced remaining user-facing Modrinth brand strings across `HomeView.vue`, `DiscoverView.vue`, `OnboardingView.vue`, `InstanceDetailView.vue`, `GlobalSearchModal.vue`, `GlobalSettingsModal.vue`, `InstanceContentBrowserModal.vue`, and `ModUpdateModal.vue` with clean community catalog copy while preserving required provenance notices and API endpoints.
  - Added `tests/release-hardening-v59.test.mjs` and expanded `scripts/verify-redesign-e2e.cjs` live Mojang upload/cape assertions.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.58 — 2026-11-15

- **Website Frosted OLED Redesign & Refreshed Desktop Previews**:
  - Redesigned `apps/website/website.css` and `apps/website/App.vue` around true OLED `#000000`, a sticky frosted glass navigation header, `backdrop-filter: blur(...)` surfaces across cards/panels/download archive, Cobalt Blue primary CTAs, colorful per-feature accent badges (`cobalt`, `emerald`, `purple`, `amber`, `sky`, `rose`), framed `/logo.svg`, and `reduced-motion` guards.
  - Refreshed `public/screenshots/library.png` alongside `public/screenshots/dashboard.png` directly from the redesigned pure launcher via `scripts/verify-redesign-e2e.cjs`.
  - Added `tests/website-redesign.test.mjs` and expanded Chromium website verification (`e2e-website.png`, `e2e-website-features.png`).

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.57 — 2026-11-15

- **Frosted OLED Polish Across Secondary Views & Dialogs**:
  - Replaced remaining solid gray surface overrides in `apps/desktop/launcher.css` with true OLED `#000000` frosted glass (`--glass-surface`, `--glass-surface-strong`, `--glass-btn`, `--glass-blur`) across Settings (`GlobalSettingsModal.vue`, `SettingsView.vue`, `InstanceSettingsModal.vue`), Onboarding (`OnboardingView.vue`), Quick Jump (`GlobalSearchModal.vue`), Tasks (`TasksView.vue`, `TasksPopover.vue`), and Accounts (`AccountView.vue`).
  - Added colorful Cobalt, Emerald, and Purple highlight icons in `OnboardingView.vue`, per-status colored borders on `.sol-task-card`, a Cobalt focus ring + `Esc` keyboard hint in `GlobalSearchModal.vue`, and reduced-motion transform guards for all newly animated cards/chips.
  - Added `tests/settings-onboarding-glass.test.mjs` and expanded Chromium E2E captures (`e2e-account.png`, `e2e-search.png`, `e2e-tasks.png`, `e2e-settings.png`).

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.56 — 2026-11-15

- **Skins Studio — Clean Cape Studio Overhaul**:
  - Upgraded `SkinsView.vue` Capes tab and `apps/desktop/launcher.css` with a frosted `.sol-cape-status-strip`, expanded **6 built-in Local 3D Preview Capes**, custom **64×32 PNG cape preview upload** with strict dimension validation (`Preview only`), automatic 3D character back-turn (`viewerPose = 'cape'`) when equipping or previewing a cape, and busy-locked `Equip` / `3D View` / `Clear Preview` actions.
  - Added `tests/cape-studio.test.mjs` and expanded `scripts/verify-redesign-e2e.cjs` to assert all 6 preview capes, status strip, automatic `data-pose="cape"` turn, and `e2e-capes.png` capture.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.55 — 2026-11-15

- **Skins Studio — Painting Tools & Shortcuts (Part 2)**:
  - Extracted pure pixel editing helpers into `apps/desktop/ui/skinTools.mjs` (`floodFillRegion`, `mirrorPixelX`, `normalizeHexColor`, `parseHexRgb`, `pushRecentColor`).
  - Upgraded `SkinEditor.vue` and `apps/desktop/launcher.css` with a **Pixel Grid** toggle, **Mirror symmetry axis guide**, **Recent Colors** strip, active color highlight ring, keyboard shortcuts (`B` Draw, `E` Erase, `G` Fill, `I` Pick, `M` Mirror, `Ctrl+Z`/`Ctrl+Y` Undo/Redo), and `64×64` / `64×32` PNG dimension guard on editor import without losing undo history.
  - Added `tests/skin-editor-tools.test.mjs` and expanded `scripts/verify-redesign-e2e.cjs` to exercise Undo/Redo and Pixel Grid toggling.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.54 — 2026-11-15

- **Skins Studio — 6-Face UV Body-Part Editor (Part 1)**:
  - Extracted shared `apps/desktop/ui/skinUv.mjs` mapping every Minecraft 64×64 body part (`Head`, `Torso`, `R. Arm`, `L. Arm`, `R. Leg`, `L. Leg`, `Full UV (64×64)`) across all 6 cube faces (`Front`, `Back`, `Right`, `Left`, `Top`, `Bottom`), Base vs Outer layers, and Classic (4px) / Slim (3px) arm geometry.
  - Upgraded `SkinEditor.vue` and `apps/desktop/launcher.css` with interactive 6-face UV cube selection, live UV pixel coordinate readout (`UV (x, y) · w×h`), and a toggleable translucent Base Layer guide while painting Outer Layer pixels.
  - Added `tests/skin-editor-uv.test.mjs` and expanded `scripts/verify-redesign-e2e.cjs` to test multi-part 6-face UV switching and capture `e2e-skin-editor.png`.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.53 — 2026-11-15

- **Skins Studio — 3D WebGL Viewport Rebuild**:
  - Rebuilt `SkinViewer3D.vue` with directional 3-point + cool ambient studio lighting (`aNorm` + `uModel` vertex/fragment GLSL shading while preserving GPU mesh buffer reuse), four character poses (`Idle`, `Walk`, `Run`, `Cape`), smooth damped drag inertia (`velYaw` gated cleanly by pause/reduced-motion), and a one-click **Flip Front/Back** view button.
  - Added `Idle / Walk / Run / Cape` pose selector pills (`.sol-pose-pills`) to `SkinsView.vue` and `apps/desktop/launcher.css`.
  - Added `tests/skin-viewer-3d.test.mjs` and expanded `scripts/verify-redesign-e2e.cjs` to exercise Run/Walk pose transitions and Front/Back flip alongside zero-buffer-leak Pause & Zoom checks.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.52 — 2026-11-15

- **Skins Studio — Skin Selector Overhaul**:
  - Upgraded `SkinsView.vue` Skin Selector with search + `All Models` / `Classic (4px)` / `Slim (3px)` filter chips, **New Blank Skin** creation, **6 built-in starter presets**, 64×64 / 64×32 PNG dimension validation on upload, one-click **Equip**, **Duplicate**, active account skin sync (`Save Active to Library`), head-crop preview badges (`skinHeadThumbs`), and focus-trapped `<SolaraModal>` delete confirmation.
  - Upgraded `.sol-skin-stage`, `.sol-skin-card`, `.sol-cape-card`, `.sol-skin-add-tile`, `.sol-skin-toolbar`, and `.sol-skin-account-strip` in `apps/desktop/launcher.css` with frosted OLED glass surfaces and reduced-motion-safe hover lift.
  - Added `tests/skins-selector.test.mjs` and expanded `scripts/verify-redesign-e2e.cjs` to test skin search, model filtering, and selector layout.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.51 — 2026-11-15

- **In-Place Modal/Sheet Ergonomics (Part 2) & Frosted Library / Instance Detail UI**:
  - Added an in-place **Instance Quick Profile Sheet** (`variant="sheet"`) to `LibraryView.vue` (via card hover button, list row button, and right-click context menu), fixed `.sol-loader-dot--${inst.loader}` indicators across Grid and List views, and replaced the legacy delete overlay with a focus-trapped `<SolaraModal>` dialog.
  - Upgraded `InstanceDetailView.vue` with a frosted OLED glass `.sol-inst-header` with per-loader left accent strips (`fabric`, `forge`, `neoforge`, `quilt`, `vanilla`), color-matched loader badges, and a focus-trapped `<SolaraModal>` screenshot lightbox with Escape dismissal.
  - Upgraded `.sol-menu-dropdown`, `.sol-context-fixed`, `.sol-row-item`, and `.sol-content-row` in `apps/desktop/launcher.css` to frosted glass surfaces.
  - Added `tests/library-instance-ui.test.mjs` and expanded `scripts/verify-redesign-e2e.cjs` to exercise the Library quick-info sheet.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.50 — 2026-11-15

- **Welcoming Home Redesign & In-Place Modal/Sheet UIs (Part 1)**:
  - Upgraded `SolaraModal.vue` and `apps/desktop/launcher.css` with a frosted slide-over `variant="sheet"` drawer (`@keyframes sol-sheet-in`, reduced-motion gated), `.sol-sheet-tabs`, `.sol-sheet-stats`, and `.sol-sheet-action-grid`.
  - Redesigned `HomeView.vue` with dynamic time-of-day welcoming header, active profile/session pills, loader-tinted Launch Hero accents, `Jump back in` quick-info triggers, and an in-place **Instance Quick Profile Sheet** so users can inspect stats, errors, folders, logs, content, or launch directly without full-page redirects.
  - Upgraded `DiscoverView.vue` Modpack Detail dialog into a crisp in-place tabbed frosted sheet (`Overview`, `Versions`, `Gallery`) with one-click release row selection and instant instance creation.
  - Added `tests/home-discover-modals.test.mjs` and expanded `scripts/verify-redesign-e2e.cjs` to test and capture the Home Quick Profile Sheet.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.49 — 2026-11-15

- **Keyframe Modal Entrance & Instant Escape Unmount**:
  - Replaced `SolaraModal.vue` leave transition delay with GPU-friendly `@keyframes sol-backdrop-in` and `@keyframes sol-modal-in` entrance animations so modals animate smoothly on open while dismissing immediately on `Escape` or backdrop click.
  - Verified full `npm run check`, `npm test` (`43/43`), `npm run test:csp`, `npm run test:window-controls`, and Chromium E2E (`verify-redesign-e2e.cjs`) on commit `e9707f7`.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.48 — 2026-11-15

- **Smooth Animation & Micro-Interaction System (`reducedMotion` Gated)**:
  - Added `<Transition name="sol-view-rise" mode="out-in">` across workspace route views in `apps/desktop/App.vue` and `<Transition name="sol-modal-fade">` spring/opacity transitions in `SolaraModal.vue`.
  - Added staggered entrance animations (`@keyframes sol-card-rise`) across jump-back-in, library, and discover card grids, tactile button lift/press springs, icon micro-interactions, smooth tab/pill transitions, and shimmering progress indicators in `apps/desktop/launcher.css`.
  - Enforced strict `reducedMotion` gating across both `:root[data-reduced-motion='true']` and `@media (prefers-reduced-motion: reduce)` so all animations, transitions, and hover transforms disable cleanly when requested.
  - Added `tests/animations.test.mjs` contract tests and live `reducedMotion` Chromium E2E assertions in `scripts/verify-redesign-e2e.cjs`.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.47 — 2026-11-15

- **True Oiled OLED `#000000` + Frosted Glass & Colorful Loader Accents**:
  - Added frosted/blurred OLED glass design tokens (`--glass-surface`, `--glass-surface-strong`, `--glass-btn`, `--glass-blur`) and tasteful loader/category highlight tokens (`--loader-fabric`, `--loader-forge`, `--loader-neoforge`, `--loader-quilt`, `--loader-vanilla`) in `apps/desktop/tokens.css`.
  - Applied `backdrop-filter: blur(...)` translucency, hairline highlights, and hover elevation across `.sol-card`, `.sol-modal`, `.sol-launch-hero`, `.sol-jump-item`, `.sol-inst-card`, `.sol-strip-box`, `.sol-btn--secondary`, `.sol-btn--ghost`, `.sol-seg-group`, and `.sol-chip`, while concentrating Cobalt Blue gradients strictly on primary action buttons and active controls.
  - Wired per-loader color badges & dots across `InstanceIcon.vue`, `HomeView.vue`, and `LibraryView.vue` and added `tests/theme-glass.test.mjs` + frosted-glass runtime checks in `scripts/verify-redesign-e2e.cjs`.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.46 — 2026-11-15

- **Rebuilt Sidebar Rail, Balanced 62px Title Bar & Update Prompt Polish**:
  - Rebuilt `SidebarRail.vue` cleanly from scratch (`84px` icon rail) with row-aligned active indicators (`left: 0` with zero height-breakpoint drift), scrollable pinned instances, clean bottom Settings/Accounts controls, numeric active-job badge, update-ready dot, and viewport-clamped frosted `<Teleport to="body">` tooltips dismissed reliably on click, scroll, blur, resize, or `Escape`.
  - Upgraded `TitleBar.vue` (`62px` height via `--titlebar-height`) to a balanced 3-column grid so the center search trigger remains truly centered alongside the top-left `30px` bordered Solara logo, navigation history, breadcrumbs, `"Restart to Update"` pill, and window controls.
  - Polished `UpdatePromptModal.vue` to surface `"New Version Available V-x.x.x"` clearly with `Update Now` / `I'll do it later` controls and consolidated duplicate `.sol-rail*` CSS rules in `apps/desktop/launcher.css`.
  - Added `tests/shell-nav.test.mjs` unit/contract tests and centered-header/update-title assertions in `scripts/verify-redesign-e2e.cjs`.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.45 — 2026-11-15

- **Restored Animated OLED Startup Loading Screen (`SplashScreen.vue`)**:
  - Restored a frosted-glass OLED `#000000` splash screen featuring the official bordered Solara logo (`88px`), pulsing Cobalt Blue aura ring, real percentage progress bar (`role="progressbar"`), and four live startup stage chips (`Core`, `Themes`, `Library`, `Updates`).
  - Wired `initializeEngine()` (`apps/desktop/ui/state.ts`) to advance each startup stage sequentially through native engine initialization, preferences loading, instance/account sync, and update checks, with `reducedMotion` gating and error retry fallback.
  - Added `tests/splash.test.mjs` unit contract tests and E2E splash assertions + screenshot capture (`e2e-splash.png`) in `scripts/verify-redesign-e2e.cjs`.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.44 — 2026-11-15

- **Pure Minecraft Launcher Clean Removals (Part 2 — Frontend, Home, Accounts, Settings & Website)**:
  - Removed all Solara Hub, `play.solaranetwork.in`, Server Hosting, Solara ID SSO, `.sol-home-side`, and promotional news/bootstrap hooks across `HomeView.vue`, `AccountView.vue`, `SidebarRail.vue`, `GlobalSearchModal.vue`, `GlobalSettingsModal.vue`, `App.vue`, `launcher.css`, `state.ts`, `types.ts`, `engine.rs`, `apps/website/App.vue`, and `packages/shared/client.ts`.
  - Replaced Home footer strip cards with direct Minecraft actions (Discover Modpacks & Mods + 3D Skins, Capes & Screenshots) and kept pure on-device Microsoft Minecraft accounts + GitHub Releases CDN auto-update intact.
  - Added `tests/pure-launcher.test.mjs` contract tests and E2E negative assertions in `scripts/verify-redesign-e2e.cjs` preventing regression of removed promo/hub/server-hosting references.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.43 — 2026-11-15

- **Pure Minecraft Launcher Clean Removals (Part 1 — Backend, Local Server & Hub Module Removal)**:
  - Removed `apps/desktop/src/identity.rs`, `apps/desktop/src/resource_policy.rs`, `packages/local-server/`, and `apps/desktop/ui/views/SolaraHubView.vue`.
  - Removed `solara-local-server`, `create_local_server`, `local_server_*`, and `solara_{profile,sign_in,sign_out,request}` commands and workspace dependencies from `Cargo.toml`, `apps/desktop/Cargo.toml`, `lib.rs`, `engine.rs`, `shell.rs`, and `update_policy.rs`.
  - Updated `.github/workflows/{windows-check.yml,windows-preview.yml,windows.yml}` and `tests/autoupdate.test.mjs` to reflect a pure on-device Minecraft launcher core without local server hosting or Solara Master Account hooks.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.42 — 2026-11-15

- **Official Bordered Solara Logo & Multi-Size Desktop Icons**:
  - Copied the official bordered Solara logo assets (`logo.svg`, `logo-192.png`, `logo-512.png`) into `public/` and wired `SolaraLogo.vue` across `TitleBar.vue`, `SplashScreen.vue`, `UpdatePromptModal.vue`, `OnboardingView.vue`, and `GlobalSettingsModal.vue` About.
  - Regenerated multi-size `apps/desktop/icons/icon.ico` (16x16 through 256x256) and `apps/desktop/icons/icon.png` (256x256) from the official bordered logo.
- **Azul Zulu Rounded Metadata Size Fix & Improved Crash Diagnostics**:
  - Handled Azul Zulu's rounded `detail.size` estimates separately from exact SHA-256 verification and `Content-Length` bounds (`apps/desktop/src/game_core/java.rs`), plus cancellation checks and regular non-symlink `bin/javaw.exe` cache verification.
  - Added `logs/latest.log` fallback alongside `hs_err_pid*.log`, launch-timestamp cutoff filtering (`inspect_crash_diagnosis_since`), clean-exit skipping (`launch.rs`, `engine.rs`), and Netty `aarch_64` native filtering (`plan.rs`, `launch.rs`).

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.41 — 2026-11-15

- **Automatic Missing/Incompatible Java Installation & Azul Zulu + Adoptium Support**:
  - Updated `resolve_java` (`apps/desktop/src/engine.rs`) to validate configured custom and PATH Java runtimes against the Minecraft version's required major version (`java_check`) and automatically download/install managed Java whenever missing, corrupted, or incompatible, while streaming phase and percentage progress into `launch-progress`.
  - Added verified Azul Zulu Windows x64 JRE (`api.azul.com` / `cdn.azul.com`) support with SHA-256 verification and automatic Eclipse Adoptium Temurin fallback inside `apps/desktop/src/game_core/java.rs`.
  - Updated managed runtime caching and Windows runtime detection (`java.rs`, `extras.rs`) to use `bin/javaw.exe` (verifying `java_sha256` on `bin/java.exe` for backward compatibility) and resolve sibling `bin/java.exe` during `-version` probing (`launch.rs`, `extras.rs`).
- **Windows Minecraft `26.3` Launch Hygiene & GPU Crash Diagnostics (`atio6axx.dll`)**:
  - Filtered non-x86_64 native classifier artifacts (`:natives-windows-arm64`, `:natives-windows-x86`, `:natives-linux-*`, `:natives-macos*`) even when `os.arch` is omitted in Mojang metadata (`plan.rs` + `-cp` guard in `launch.rs`).
  - Fixed Windows launch behavior in `launch.rs`: omit `CREATE_NO_WINDOW` when spawning GUI `javaw.exe`, strip `_JAVA_OPTIONS` from child environments, emit `arguments.jvm` before `-Xmx` and extra JVM arguments, and ensure `natives/{java,jna,lwjgl,netty}` directories exist on both install and launch.
  - Added automatic `hs_err_pid*.log` and `logs/latest.log` crash diagnosis (`inspect_crash_diagnosis` in `launch.rs`, wired into `game-exited` and `InstanceDetailView.vue`) to surface AMD (`atio6axx.dll`), NVIDIA (`nvoglv64.dll`), Intel OpenGL, Java mismatch, OOM, and `0xc0000005` recovery steps.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.40 — 2026-11-14

- **GitHub Releases CDN Auto-Updater Migration**:
  - Migrated desktop auto-update checks and installer downloads off the Solara VPS (`api.solara-inc.in` / `app.solara-inc.in`) directly to GitHub Releases CDN (`https://github.com/Solara-Inc-Development/solara-launcher/releases/latest/download/latest.json`).
  - Added signed Tauri updater manifest (`latest.json`) generation and verification in `scripts/prepare-github-release.mjs`, `scripts/release.mjs`, and `.github/workflows/publish-release.yml`, uploading `latest.json` alongside `solaralauncher.exe`, `solaralauncher.exe.sig`, and `source.zip`.
  - Preserved strict Minisign signature verification (`apps/desktop/updater.pub`), startup manual-consent `"New Version Available"` prompt (`Update Now` / `I'll do it later`), topbar `"Restart to Update"`, and immutable local `builds/V-0.0.40/` copies while keeping `api.solara-inc.in` bootstrap metadata pointing to the `V-0.0.40` GitHub Release URL so `V-0.0.39` installs transition cleanly onto GitHub CDN.
  - Added trusted HTTPS redirect validation (`github.com` -> `objects.githubusercontent.com` / `release-assets.githubusercontent.com`) and blocked untrusted host or protocol downgrade redirects in release verification and live update checks.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.39 — 2026-11-14

- **Fixed Windows Play-Closes-Launcher Stack Overflow (`STATUS_STACK_OVERFLOW`)**:
  - Identified that release Tauri IPC builds pass async command futures by value on the Windows WebView2 UI thread, causing `launch_instance` (and nested 64 KB stack arrays inside async `java.rs`/`download.rs`/`content.rs`/`extras.rs` functions) to exceed the default 1 MB GUI stack reserve (`0xC00000FD`) before `launch_instance` ever started.
  - Wrapped heavy Tauri IPC commands (`launch_instance`, `apply_instance_content_updates`, `install_content`, `create_instance`, `repair_instance`, `check_update`, `install_update`) and `engine::launch_with_events` / `engine::apply_content_updates` / `extras::apply_content_updates` in `Box::pin(async move { ... }).await`, heap-allocated 64 KB async file/hashing buffers (`vec![0u8; 65536]`), configured Windows MSVC `/STACK:8388608` (8 MB stack reserve) in `build.rs`, and added a sanitized local `crash.log` panic hook.
- **Spacious Modrinth-Proportioned Header, Sidebar & Controls**:
  - Enlarged the custom TitleBar (`56px`), SidebarRail (`84px`, `50x50px` tiles, glowing Cobalt active left bar, click-dismissed floating tooltips), baseline typography (`15px`), buttons, cards, modals, Skin Studio single-line cape labels, and centered Settings dialog navigation across all views while keeping zero document overflow down to `940x560`.
- **Manual-Consent Update Flow & Topbar "Restart to Update"**:
  - Removed silent automatic background update downloads and installations; Solara now checks for available updates on startup (respecting `autoUpdateEnabled`) without applying changes unexpectedly, preserves pending update metadata across checks/errors, shows the **"New Version Available"** (`V-x.x.x`) modal with **"Update Now"** and **"I'll do it later"**, and keeps **"Restart to Update"** in the title bar when deferred.
- **Complete Solara Obsidian Blue Redesign (Launcher & Website)**:
  - Replaced Dawn Gold and old slate/green website styles across the desktop launcher, Solara Hub website (`index.html`, `website.css`, `base.css`, `motion.css`), 3D skin studio, instance icons, splash screen, installer branding, and app/SVG icons with Cobalt / Electric Blue (`#3B82F6` / `#60A5FA` / `#2563EB`) on True OLED Black (`#000000`), restored vertical scrolling on `index.html`, and migrated legacy saved `dawn-gold` preferences to `obsidian-blue`.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.38 — 2026-11-14

- **Fixed Launcher Closing When Clicking Play**:
  - Removed the `onLaunch` close and hide-without-tray actions that closed or hid the launcher window when starting an instance.
  - Added an early-exit safety guard in `launchOrStopInstance` so the launcher only minimizes (`minimize`) after verifying the launched Minecraft process is actively running, and automatically normalizes legacy `"close"`/`"hide"` preferences to `"keep"`.
- **Solara V3 Obsidian Dawn Gold Polish & Brand Identity**:
  - Updated the surface palette toward true OLED black (`#000000` / `#0D0D10` / `#151518`) paired with warm Dawn Gold accents, original geometric `SolaraLogo`, and animated OLED startup `SplashScreen`.
  - Refined modal workflows (`GlobalSettingsModal`, `InstanceSettingsModal`, `InstanceContentBrowserModal`), `SolaraHubView`, offline bundled `DM Sans` font assets, and keyboard focus styling.
- **Engine Safety & Lifecycle Hardening**:
  - Added RAII `LaunchReservation` tracking, real-time `launch-progress` and `game-exited` Tauri events, non-destructive `migrate_clean_slate_v23` user data preservation, `backups.rs` failure retention, and redacted `export_diagnostics`.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.37 — 2026-11-14

- **Solara Dawn Gold — Obsidian Edition Redesign**:
  - Rebuilt the desktop shell around the Dawn Gold (`#E9B86D`) and Obsidian Dark (`#090B0F` / `#11141A`) design system with a native custom title bar, fixed 68px permanent icon-only sidebar rail with floating tooltips, and independent content scrolling without an outer document scrollbar.
- **First-Time Onboarding & Returning Home Dashboard**:
  - Added a streamlined 4-step initial onboarding flow alongside a focused returning-user Home dashboard with Quick Play, active account card, recent instances, recent tasks/activity, and compact announcements.
- **Library, Groups & Create Instance Experience**:
  - Added grid/list Library view with search, custom groups, pinning, sorting, filtering, context menus, and a full Create Instance modal supporting built-in/custom icons, official Minecraft version filtering, and Vanilla, Fabric, Quilt, Forge, and NeoForge loaders.
- **Instance Workspace, Content Manager, Worlds, Logs & Memory-Safe Screenshots**:
  - Added dense per-instance management tabs for Content (Mods, Datapacks, Resource Packs, Shaders, Modrinth browse, and bulk updates modal), Worlds (NBT metadata, backup, restore, rename, delete), Files, Launch Hooks/Concurrency controls, windowed Live Logs (`INFO`/`WARN`/`ERROR`/`FATAL`), and lazy `IntersectionObserver` screenshot browsing with on-demand full viewer and Copy Path.
- **3D Skin Selector, Skin Editor & Capes**:
  - Added interactive 3D skin and cape preview, pixel-level 2D Skin Editor with Wide/Slim arm styles, undo/redo, layers, local saved-skins library, and official Mojang account skin/cape status support.
- **Global & Per-Instance Settings with Sticky Save Bar**:
  - Added categorized Settings and Instance Settings with the sticky Unsaved Changes bar (`Reset` / `Save Changes`), navigation guard, Java runtime scanner, verified cache tools, diagnostic export, and preserved 1-click Microsoft PKCE loopback sign-in.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.36 — 2026-11-12

- **Official Solara Microsoft Identity & Branding**:
  - Replaced third-party client registration with official Solara Launcher Microsoft Client ID (`6a9238bb-3803-46f4-bf38-2f45bdf8c2ea`).
  - Microsoft's authentication and consent screens now officially display "Solara Launcher", providing branded and transparent verification.
- **1-Click Direct Browser Sign-In (RFC 7636 / RFC 8252 PKCE)**:
  - Replaced manual 8-character device code copy-pasting with a seamless 1-click browser login flow.
  - Opens the official Microsoft login prompt directly in the user's default browser while binding a secure local ephemeral loopback listener (`http://localhost:<port>/callback`).
  - Full cryptographic PKCE protection (S256 code challenge and in-memory verifier) preventing code interception.
  - Branded browser completion page automatically confirms successful authentication and closes the browser tab.
  - Instant background Xbox Live / XSTS authentication, Mojang Java purchase entitlement verification, player profile loading, and DPAPI-encrypted token vaulting.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.35 — 2026-11-12

- **Zero-Setup Microsoft & Minecraft Sign-In**:
  - Replaced manual Azure Portal / Entra ID configuration with the canonical public Minecraft OAuth client ID fallback (`c36a9fb6-4f2a-41ff-90bd-ae7cc92031eb`).
  - Completely fixes the `"Selected user account does not exist in tenant 'Microsoft Services'"` error that occurred when personal Microsoft accounts attempted to sign in.
  - Users can now click "Login with your Minecraft Account", visit `https://www.microsoft.com/link`, enter the 8-character device code, and instantly authenticate their genuine Java edition purchase without registering any Azure apps.
  - Credentials and access tokens remain 100% private and encrypted inside the user's local Windows vault (DPAPI/Keyring).
- **Refined First-Impression Layout & Account Switcher**:
  - Strictly contained the "Welcome User TO Solara-Launcher" hero above the fold (`1280x720` viewport), eliminating lower-section bleed.
  - Added global `Escape` key shortcut to dismiss the Account Switcher, Announcements Reader, and Device Login modals.
  - Aligned account action button dimensions (`30px` matching height) and added fallback avatar icons when loading player textures.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.


## Testing Preview V-0.0.34 — 2026-11-12

- **Centered Hero & Stylish Welcome ("Welcome User TO Solara-Launcher")**:
  - Implemented bold, centered above-the-fold hero layout with luminescent typography designed for maximum first impression impact.
  - Centered heading displays "Welcome User TO Solara-Launcher" in large stylish font with cyan glow drop-shadow.
- **Active Account Card & Multi-Account Switcher**:
  - Centered active account card directly underneath the hero heading.
  - When signed in, displays active player head avatar (`mc-heads.net`), username, and "Active Account · Java Edition Entitled" status.
  - When unsigned, displays "No Account Connected" with login button and private Windows vault assurance.
  - Interactive Account Switcher Modal:
    - Lists all connected Minecraft accounts with 1-click account switching (`select_minecraft_account`).
    - Dedicated "Add Minecraft Account" button initiating native Microsoft OAuth device flow (`minecraft_sign_in`).
    - Dedicated "Sign Out / Logout" action for each account (`minecraft_sign_out`).
    - Strictly local storage: credentials and tokens stay exclusively on user's PC in Windows vault (DPAPI/Keyring).
- **Below-the-Fold Scrolling Experience**:
  - Subtle scroll down indicator inviting users to scroll down smoothly to access announcements and update information.
  - Launcher Announcements Section: Last 3 announcement cards + "See all announcements" button opening 50-announcement reader.
  - Latest Update Info Section: Highlights, update checks, and loading animation preview.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.33 — 2026-11-12

- **Home Page & Welcome Header**:
  - Implemented top greeting displaying "Welcome user" when unsigned and "Welcome, <username>" with player avatar/head when signed in.
  - Dedicated avatar render using official Minecraft player head texture with pixelated rendering.
- **Minecraft Java Account Authentication**:
  - Wired to native Rust Microsoft OAuth authentication bridge (`minecraft_sign_in`, `cancel_minecraft_sign_in`, `minecraft_sign_out`, `select_minecraft_account`).
  - Entitlement verification checks for genuine Minecraft Java purchase.
  - Strict Security & Privacy: Credentials and tokens remain stored exclusively on the user's local PC in the Windows vault, never sent to Solara servers.
  - One-click device code modal with copy-to-clipboard button and direct link to official Microsoft login.
- **Launcher Announcements Section**:
  - Clean card section displaying the latest 3 announcements with date and category badges.
  - "See all announcements" button opening a clean modal overlay displaying up to 50 announcements with category filters.
  - Built-in reader displaying full announcement articles, author, date, and formatted content.
- **Latest Update Info Section**:
  - Positioned below announcements, displaying highlights and changes in the current launcher version.
  - Integrated "Check for updates" button, auto-updater progress, and loading animation preview.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.32 — 2026-11-12

- **Disabled Default Browser Context Menu**:
  - Eliminated the default Chromium/WebView2 browser context menu ("Back", "Forward", "Reload", "Save as", "Print") globally across the application.
  - Intercepts and suppresses right-click context menu events application-wide in capture phase on both module and component roots.
- **Wider Sidebar with Icon & Text Labels**:
  - Expanded sidebar navigation to ~210px (in the 200px - 220px range) for comfortable navigation.
  - Added dedicated Icon + Text labels for both "Home" and "Discord" items.
  - Preserved canonical link to `https://discord.solara-inc.in/` using native `open_external` integration.
- **Main Clean View ("HI ;)") & Modrinth Header**:
  - Preserved minimal Modrinth-inspired titlebar with brand logo, typography, and frameless window controls.
  - Preserved centered greeting `HI ;)`, update checks, and Minisign signature verification.
  - Preserved pure oiled black (`#000000`) theme and canonical 5-10s startup sequence.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.31 — 2026-11-12

- **Complete Bloat Removal & Clean Slate Experience**:
  - Removed all settings panels, modals, and settings views completely.
  - Removed mods management, mock mods, and lists completely.
  - Removed premade instances, instance cards/carousels, edition badges (no "rust edition", no "java edition"), and play bars completely.
  - Removed the search bar, navigation history controls, breadcrumbs, and instance status pills from the header.
- **Clean Minimal Modrinth-Inspired Top Bar**:
  - Minimalistic frameless top bar featuring Solara brand logo and typography on the left.
  - Smooth window drag regions (`data-tauri-drag-region`) across the titlebar with proper drag handling.
  - Dedicated window controls (minimize, maximize/restore, close) with 300ms deduplication.
- **Sleek Minimal Horizon-Inspired Icon Dock**:
  - Compact vertical sidebar icon dock in pure solid oiled black (`#000000`) with subtle border.
  - Active home indicator dock item and primary Discord button opening `https://discord.solara-inc.in/`.
- **Main Clean View ("HI ;)") & Core Auto-Updater**:
  - Centered clean greeting `HI ;)` and primary "Check for updates" button.
  - Preserved auto-updater core with Minisign cryptographic verification.
  - Preserved canonical startup loading screen (5 to 10 seconds random duration, glowing Solara logo, status stages, sleek progress bar, and numeric percentage).
  - Maintained pure solid oiled black (`#000000`) theme and Discord window proportions (`1280x720`, `center: true`).

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.30 — 2026-11-12

- **Modrinth-Inspired Top Bar / Header**:
  - Frameless obsidian titlebar with window drag regions (`data-tauri-drag-region`) and click deduplication.
  - Solara brand mark, history navigation controls (`<` and `>`), and active breadcrumb indicator (`▷ Home`, `▷ Instances`, `▷ Mods`, `▷ Servers`, `▷ Settings`).
  - Integrated quick search bar with `Ctrl K` badge and downloads/tasks tray button.
  - Dynamic instance status indicator pill (`● No instances running` / `● 1 instance running`).
- **Horizon-Inspired Sleek Sidebar**:
  - Floating obsidian panel (`#09090b`) with Electric Cyan (`#3dd6f5`) accents and strictly zero purple.
  - "Instances" section header with quick add button and Horizon instance card (`Solara Main`, `● Ready`, and options menu).
  - Modern navigation rail tabs (Home, Instances, Mods, Servers, Settings) with active cyan indicators.
  - Horizon bottom widgets: update status card, "Join the Discord" pill button, Sponsor/Website links, and version footer.
- **Horizon Hero Section & Interactive Workspace**:
  - Hero play section featuring `HI ;)`, edition badges (⚡ Rust Edition, ☕ Java Edition), and instance launch stats.
  - Horizon Play bar with primary Play/Stop toggle, version selector (`Fabric 1.21.4`), and directory explorer button.
  - Direct server invite code box and direct join action.
  - Preserved 5-10s startup loading animation, glowing logo, status progression, numeric percentage, and auto-updater core.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.29 — 2026-11-11

- **Discord-Proportioned Window Dimensions**:
  - Expanded launcher default window dimensions to match Discord desktop proportions: `width: 1280`, `height: 720`, with `minWidth: 940`, `minHeight: 560`, and `center: true`.
  - Provides a wide, expansive desktop canvas while maintaining comfortable margins on all standard displays.
- **Pure Oiled Black Theme & Startup Experience**:
  - Solid pure oiled black (`#000000`) theme across all surfaces with zero cut-off logo watermark artifacts.
  - 5 to 10 seconds random startup duration (`5000 + Math.random() * 5000`).
  - Natural progress status stages ("Reading Files…", "Checking Integrity…", "Preparing Environment…", "Launching Solara-Launcher…").
  - Prominent glowing Solara logo, sleek progress track, and accurate numeric percentage.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.28 — 2026-11-11

- **Spacious & Comfortable Window Dimensions**:
  - Increased default window dimensions from 1000x600 to `width: 1160`, `height: 700`, with `minWidth: 900`, `minHeight: 580`, and `center: true`.
  - Balanced rectangular proportions providing ample breathing room for UI elements without overflowing vertically or touching top/bottom margins on standard laptop (768p) and desktop (1080p) screens.
- **Preserved Loading Screen & Pure Oiled Black Theme**:
  - Maintained 5 to 10 seconds random loading duration (`5000 + Math.random() * 5000`).
  - Preserved natural progress stages ("Reading Files…", "Checking Integrity…", "Preparing Environment…", "Launching Solara-Launcher…").
  - Solid pure oiled black (`#000000`) across all surfaces with zero logo cut-off artifacts, glowing brand mark, progress bar, and numeric percentage.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.27 — 2026-11-11

- **Fast 5–10s Loading Screen Duration**:
  - Reduced loading screen startup duration strictly to 5 to 10 seconds (random: `5000 + Math.random() * 5000`).
- **Clean Progressive Status Line**:
  - Added clean, simple status lines below the loading bar that progress naturally across the 5–10s duration:
    - "Reading Files…"
    - "Checking Integrity…"
    - "Preparing Environment…"
    - "Launching Solara-Launcher…"
- **Centered Window Dimensions**:
  - Set default window dimensions to `width: 1000`, `height: 600`, with `minWidth: 800`, `minHeight: 540`, and `center: true` for a balanced, centered desktop interface.
- **Pure Oiled Black Theme**:
  - Background is now pure, solid oiled black (`#000000`) across all container, titlebar, and curtain surfaces.
  - Eliminated background logo watermark to ensure zero cut-off background logo artifacts while keeping the central prominent glowing brand mark.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.26 — 2026-11-11

- **Minimalist Oiled Black Loading Screen**:
  - Removed all fake process and status messages completely ("Starting launcher services…", "Connecting to Solara network…", etc.).
  - Removed the `Skip` animation button completely per user directive.
  - Removed extra subtitles and taglines ("Your next world awaits").
  - Minimalist layout focused exclusively on brand identity and loading progress:
    - Deep oiled black backdrop (`#020305` / `#000000`) with subtle ambient glowing Solara logo watermark in the background.
    - Central prominent glowing Solara brand logo (`/logo-512.png`) with soft ambient aura.
    - Sleek, minimalist loading progress bar below the logo.
    - Accurate numeric percentage below the bar (`X%`).
- **Smooth 10–15s Transition**:
  - Maintained smooth startup duration (10–15 seconds) progressing cleanly from 0% to 100%, followed by the split-curtain exit revealing the clean "HI ;)" launcher view.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.25 — 2026-11-11

- **Canonical Solara Website Loading Screen**:
  - Replicated authentic Solara loading screen directly from the official website source (`WebsiteLoader.tsx` and `solara-loader.css`).
  - Canonical SVG brand mark featuring rounded dark tile (`rx="14"`), concentric circle (`r="17"`), and the Solara lightning emblem.
  - Authentic gradient from Electric Cyan (`#3dd6f5`) to Warm Gold (`#ffb55a`).
  - Subtle breathing animations (`solara-logo-breathe`), background drift (`solara-mark-drift`), and animated glow track line (`solara-loading-pass`).
  - Split curtain exit transition (`solara-curtain-left`, `solara-curtain-right`) parting outward to reveal the desktop interface.
- **Extended 10–15s Immersive Duration**:
  - Replaced short timeouts with a random 10 to 15 seconds startup duration (`Math.floor(10000 + Math.random() * 5000)`).
  - Smooth, progressive stage messages: "Starting launcher services…", "Connecting to Solara network…", "Synchronizing client runtime…", "Verifying workspace readiness…", "Opening your Solara world…", and "Ready".
  - Live percentage indicator and instant `Skip` button for power users.
- **Deep Oiled Black Minimal Theme**:
  - Completely purged all purple hues (`#8b5cf6`, `#7c3aed`, `#c084fc`) across all desktop components.
  - Deep luxurious oiled black background (`#030406`) across window shell, frameless titlebar, loading curtains, and buttons.
  - High-contrast Electric Cyan accents and gold progress gradients.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.24 — 2026-11-11

- **Animated Loading Screen & Smooth Startup Transition**:
  - Implemented an animated Solara loading screen displaying on application startup with an obsidian backdrop and glowing ambient radial halo.
  - Floating, breathing Solara brand emblem with luminous purple-tinted glass tile and pulsing drop shadow.
  - Iridescent metallic shimmer sweep animation across brand typography (`solara Launcher`).
  - Sleek rounded progress track with live animated gradient bar and traveling shimmer highlight reflecting real engine initialization and network connectivity stages.
  - Dynamic status messages: "Starting launcher services…", "Connecting to Solara network…", "Verifying workspace readiness…", and "Ready".
  - Graceful fade-out transition smoothly revealing the clean minimal launcher view.
  - Interactive "Preview loading animation" control on main view and quick `Skip` action for immediate access.
  - Full accessibility compliance with system `prefers-reduced-motion` settings.
- **Window Dimensions Preserved**:
  - Window dimensions (`1280x880`, `minWidth: 800`, `minHeight: 620`) strictly preserved as-is per user directive.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.23 — 2026-11-11

- **One-Time Clean-Slate Reset & Sentinel Protection**:
  - Implemented an idempotent, one-time instance reset for V-0.0.23 (`migrate_clean_slate_v23`) in launcher initialization.
  - Quarantines and wipes stale instances once into `.wiping-instances-*` staging, recreating a clean `custom-instances` directory.
  - Persists atomic `.clean_slate_v23_reset` sentinel marker file immediately upon reset; subsequent launches and future updates strictly preserve all user instances, saves, mods, and servers.
  - Added recursive read-only permissions unsetting across Windows and Unix filesystem targets, ensuring non-blocking cleanup of locked files and directories.
- **Discord Activity & RPC Purge**:
  - Completely purged Discord RPC client loops, IPC connection handlers, and background presence polling from the launcher backend.
  - Eliminates unnecessary local socket scanning, reduces idle background CPU/memory footprint, and enhances user privacy.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.22 — 2026-11-11

- **Clean Slate Minimal Launcher Interface**:
  - Rebuilt desktop interface from a clean slate with a focused "HI ;)" minimal view, centered layout, and Electric Cyan accents.
  - Dedicated "Check for updates" button with animated loading spinner, real-time download progress bar, and byte counter (`X.X MB of Y.Y MB (Z%)`).
- **Frameless Window Controls & Titlebar Movement**:
  - Native window dragging region with smooth `-webkit-app-region` isolation.
  - Responsive Minimize, Maximize/Restore, and Close action buttons with 300ms debounce protection against rapid clicks.
- **Live Background Auto-Update Pipeline**:
  - Connected live auto-update pipeline on `https://api.solara-inc.in/v1/launcher/update/windows/x86_64`.
  - Cryptographically signed with production Minisign key; background update verification, download caching, and silent restart upon idle.
- **Synchronized GitHub Releases & CI Delivery**:
  - Fully automated GitHub release publication with signed installer (`solaralauncher.exe`), signature (`solaralauncher.exe.sig`), and verified source archive (`source.zip`).

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.21 — 2026-11-07

- **Comprehensive Global Keyboard Shortcuts & Cheatsheet**:
  - Global navigation shortcuts: `Ctrl+1` (Home), `Ctrl+2` (Library), `Ctrl+3` (Services), `Ctrl+4` (Explore), and `Ctrl+,` (Settings).
  - Quick downloads modal toggle with `Ctrl+D` and `Ctrl+J`.
  - Redesigned two-column Obsidian glass cheatsheet modal (`?` or `Ctrl+/`) with high-contrast `<kbd>` badges.
- **Installed Mod Inspector Details Drawer**:
  - Interactive details drawer when clicking any mod row or clicking the dedicated inspect action button.
  - Comprehensive metadata: mod ID, version, filename, file size, status badges, loader chips, authors/contributors extracted natively from mod manifests, and full descriptions.
  - 1-click actions: Toggle enabled/disabled state, reveal in system Explorer, or delete to instance Trash.
- **Downloads Queue Management & Header Summary Pills**:
  - Native background job cleanup (`clear_finished_downloads`) removing finished/failed downloads while preserving active tasks.
  - 1-click "Clear completed" button in downloads header.
  - Real-time transfer rate (`X.X MiB/s`) and queue summary pills.
- **Explore / Discover Search & Sorting Filters**:
  - Sleek SelectControls for sort index (Relevance, Downloads, Updated, Followers, Newest), mod loader, and Minecraft game versions.
- **Instance Console Log Viewer Ergonomics**:
  - Live search match count badge ("X matches"), error and warning count filter pills, and 1-click "Export logs" to `.txt` file.
- **Instance Notebook Starter Templates & Autosave Glow**:
  - Monospace notebook with animated cyan pulse confirming local persistence, and quick starter buttons (`+ Server IP & notes`, `+ To-do list`, `+ Coordinates`).
- **Non-Blocking Instance Duplication & Preserved Metadata**:
  - Background asynchronous instance cloning preserving custom avatars, notebook notes, and display settings.
- **Screenshot Lightbox Viewer & Smart Archive Routing**:
  - Fullscreen lightbox photo viewer with mouse-wheel zoom, copy to clipboard, and trash deletion.
  - Smart archive drop routing automatically intercepting `.zip` and `.mrpack` archives to resource packs or shader packs.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.18 — 2026-10-31

- **Compact Desktop Topbar Layout**:
  - Compacted the desktop topbar height from 70px down to a sleek 48px, freeing ~30px of vertical workspace for game cards and navigation.
  - Slimmed search bar, running status indicator, and window controls to match desktop ergonomics.
- **Dedicated Decoupled Dark UI**:
  - Fully decoupled Dark and Light theme styling, eliminating residual white borders and overlay artifacts.
  - Native dark palette with true deep background, glass borders, and consistent contrast across all panels.
- **In-Launcher Instance Deletion & Safety**:
  - Safe instance deletion workflow with confirmation dialog and runtime safety guards (preventing deletion while an instance is running).
  - 1-click folder shortcuts in Files tab (Open Instance Folder, Saves Folder, Mods Folder).
  - Global `Escape` shortcut to dismiss open modals immediately.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.17 — 2026-10-31

- **Responsive Window Controls**:
  - Close, minimize, and maximize buttons now remain responsive and reliable during and after dragging the application window.
  - Separated window titlebar drag region from control action buttons and enforced `-webkit-app-region: no-drag !important` to eliminate drag event interference.
  - Window control actions run independently and are never blocked by background task busy states.
- **Clean Theme Simplification (Default Dark)**:
  - Defaults to a clean, focused **Dark** theme out of the box with consistent deep slate styling.
  - Removed colorful custom accent swatches in favor of a clean, dedicated Light/Dark toggle in Settings.
- **SSO Consent Redirect Hardening**:
  - Resolved cross-scheme redirect hanging on the authentication consent page by updating loopback CSP `form-action` rules and providing a resilient auto-redirect fallback upon user approval.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

## Testing Preview V-0.0.15 — 2026-10-31

- **Bright & Cheerful Visual Design**:
  - Welcoming lilac and white surfaces with vibrant violet, sky blue, peach, and pink accents.
  - Large illustrated welcome banner on the dashboard for a friendly first impression.
- **Larger Controls & Accessible Typography**:
  - Enlarged interactive elements: 94px labelled navigation rail with 28px glyphs, ~44px primary action buttons, and 14–16px readable text.
  - Generous card padding, enhanced hierarchy, and spacious dialogs and account sidebars.
- **Personal Instance Avatars & Vector Defaults**:
  - Upload custom PNG, JPG, or WebP avatar images for any instance with interactive 256px crop and preview.
  - Crisp vector loader badges (Vanilla, Fabric, Quilt, Forge, NeoForge) replace AI landscape defaults.
  - Avatars persist securely in local device storage without sending files to remote servers.
- **Real-Time Task & Download Progress**:
  - Live progress bars reporting byte-accurate download and update progress percentage from native engine events.
  - Indeterminate progress animations for pending requests and clear waiting labels.
- **Responsive & Motion Polish**:
  - Responsive layout scaling comfortably across small viewports down to 800x620.
  - Full adherence to reduced-motion preferences with instantaneous animation zeroing.

This is a Windows testing preview. Updater signature verification is required. Physical Windows installation, licensed account/gameplay and automatic upgrade/relaunch acceptance remain pending.

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
