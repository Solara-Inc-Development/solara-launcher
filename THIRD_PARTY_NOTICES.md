# Solara Launcher — third-party notices

Solara Launcher public application source is licensed under **GNU General Public License v3.0 only (`GPL-3.0-only`)**, without warranty; see `LICENSE`. Distributions include the exact corresponding source and locked dependency versions.

## Current & Upstream Provenance

- **Active V-0.0.62 build:** Solara Launcher (`apps/desktop`) currently uses its independently authored native core (`apps/desktop/src/game_core/`). Modrinth content browsing uses Modrinth's public HTTP API; content authors retain their rights, and provider images load from validated provider CDNs without claiming authorship.
- **Prepared upstream adoption (`github.com/modrinth/code` @ `1b27f67b4`, scheduled for `V-0.0.63+`):** Solara is audited to adopt the open-source Modrinth App desktop monorepo (`apps/app`, `apps/app-frontend`, `packages/app-lib`, `packages/ui`, `packages/assets`, `packages/utils`, `packages/api-client`, `packages/blog`, `packages/moderation`, `packages/highlightjs-mcfunction`, and workspace crates `ariadne`, `async-minecraft-ping`, `minecraft-meta-manifest`, `minecraft-meta-model`, `modrinth-content-management`, `modrinth-log`, `modrinth-util`, `path-util`, `serde-binhum`, `toasty`) under `GPL-3.0-only` (and `LGPL-3.0-only` for `@modrinth/api-client`, `MIT` for `toasty`, `MIT OR Apache-2.0` for `async-minecraft-ping`).
- **Trademark & proprietary exclusions (`COPYING.md`):** Modrinth trademarks, logos, `.idea/icon.svg`, `.github/assets/`, `packages/ui/src/components/brand/*`, `packages/assets/branding/*`, `packages/assets/external/*`, `ModrinthServersIcon.vue`, `MedalBackgroundImage.vue`, and third-party commercial SaaS packages (`@intercom/messenger-js-sdk`, `@stripe/stripe-js`, `stripe`, `@sentry/vue`, `apexcharts`, `vue3-apexcharts`) are strictly excluded and replaced with original Solara Launcher branding (`Solara-Inc.in`, `"Solara Launcher"`, `in.solara.launcher`, `solaralauncher.exe`) and Solara's own Microsoft auth and GitHub Releases auto-updater.
- **Honest documentation of upstream packaging gaps (`modrinth/code@1b27f67b4`):**
  - `packages/app-lib/COPYING.md` states `GPL-3.0-only` and links `./LICENSE`, although `packages/app-lib/LICENSE` is omitted in upstream's repository; GPL-3.0-only terms are provided in root `LICENSE`.
  - Workspace crates `ariadne`, `minecraft-meta-manifest`, `modrinth-content-management`, `path-util`, and `serde-binhum` omit `license` in their `Cargo.toml` and per-crate `LICENSE` files; they originate verbatim from `https://github.com/modrinth/code` (`1b27f67b4`) as part of the `GPL-3.0-only` `theseus_gui` desktop distribution. `cfg_block 0.1.1` on crates.io specifies `license-file = "LICENSE"` instead of an SPDX expression; its bundled `LICENSE` file is preserved.

## Dependency Inventory Generation

Vue and Vite use MIT; Tauri uses MIT OR Apache-2.0. Other libraries retain their individual licences. `node scripts/third-party.mjs` generates an exact Windows (`x86_64-pc-windows-msvc`) dependency inventory from `Cargo.lock` and either `package-lock.json` or `pnpm-lock.yaml`, preserving available copyright/licence/notice texts under `dist/third-party/`. Its `manifest.json` records package versions, licences, and corresponding-source download addresses.

Where published crate archives omit licence text, `third-party/licence-supplements/sources.json` records verified upstream texts, exact revision URLs where applicable, and checksums. The `selectors` crate uses `MPL-2.0`; its unmodified source is referenced at the exact registry download in the inventory.

## Fonts, Branding & Runtime Terms

DM Sans and Space Grotesk fonts use SIL Open Font License 1.1; original licences and notices remain under `public/fonts/` and distributed font assets. Solara branding belongs to Solara-Inc.in; inclusion does not grant trademark rights. Original Solara landscape assets are included in this repository.

Minecraft, Microsoft, and Xbox marks belong to their respective owners. Solara Launcher is not affiliated with Mojang or Microsoft. Playing Minecraft requires accepting applicable Minecraft terms and owning a valid Java Edition licence. Windows WebView2 and Java runtimes retain their own licences.
