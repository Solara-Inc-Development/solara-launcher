# Solara Launcher — third-party notices

Solara Launcher and its native game core are independently written. Public application source is GPL-3.0-only, without warranty; see LICENSE. Distributions include the exact corresponding source and locked dependency versions.

The current dependency graph contains no inherited launcher engine or its workspace packages. Modrinth content browsing uses their public HTTP API; it does not include their launcher code. Content authors retain their rights. Provider images are loaded from the validated provider CDN, not claimed as Solara artwork.

Vue and Vite use MIT; Tauri uses MIT OR Apache-2.0. Other libraries retain their individual licences. `node scripts/third-party.mjs` generates an exact Windows dependency inventory and preserves available copyright/licence/notice texts under `dist/third-party/`, including frontend build tools. This directory is bundled with the app. Its `manifest.json` records package versions, licences and corresponding-source download addresses; exact Cargo.lock and package-lock.json accompany Solara's source.

Where published crate archives omit licence text, `third-party/licence-supplements/sources.json` records verified upstream texts, exact revision URLs where applicable and checksums. The selectors source uses MPL-2.0; its source is available at the exact registry download recorded in the inventory, with the MPL text included. No third-party Rust source has been modified by Solara.

DM Sans and Space Grotesk fonts use SIL Open Font License 1.1. Original licences and designer/copyright notices are included in `public/fonts/` and the distributed fonts directory. Solara branding comes from the owner's corporate site; inclusion does not grant trademark rights. Original Solara landscape assets are included in this repository.

Minecraft, Microsoft and Xbox marks belong to their respective owners. Solara is not affiliated with Mojang or Microsoft. Installing the game requires accepting the applicable Minecraft terms and a valid Java Edition licence to play. Windows WebView2 and Java runtimes retain their own terms; Solara does not claim authorship of these runtimes.
