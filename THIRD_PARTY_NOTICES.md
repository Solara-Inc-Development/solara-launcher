# Solara Launcher — third-party notices

Solara Launcher is an independent application, with changes by Solara in 2026. The public desktop source is GPL-3.0-only, without warranty. See LICENSE. Provide corresponding source (including patches and exact dependency lockfiles) with every distributed executable.

The local Minecraft engine derives from Modrinth Theseus v0.22.0, upstream commit e7757b14ed04e824ac7e8b8b05652d4290df721a, © Rinth, Inc. and contributors. Its apps/app/LICENSE and packages/app-lib/COPYING.md specify GPL-3.0-only. Source: https://github.com/modrinth/code . Solara modifications are reproducibly applied by scripts/prepare-upstream.py and recorded in the fetched SOLARA_PATCHES.json.

Upstream COPYING branding restrictions apply. Solara does not copy Modrinth logos, proprietary landing artwork or frontend branding. Required references to Modrinth identify the content provider and attribution; they are not claims of affiliation.

Daedalus: MIT. async-minecraft-ping: MIT OR Apache-2.0. Preserve their original notices in corresponding source. Vue: MIT. Vite: MIT. Tauri: MIT OR Apache-2.0. Rust dependencies retain their own licences; generate a full locked dependency notice report before release.

**Distribution blocker:** upstream packages ariadne, modrinth-content-management, path-util and serde-binhum lack explicit package licence grants in the inspected pinned tree. Fetching locally does not resolve redistribution rights. Obtain upstream clarification or replace/remove the dependencies before publishing installers. No release workflow may bypass this unresolved review.

Solara branding assets originate from the owner's existing Solara corporate site. Their inclusion does not grant third parties trademark rights. Minecraft and Microsoft/Xbox marks belong to their respective owners. Solara is not affiliated with Mojang, Microsoft or Modrinth.

DM Sans and Space Grotesk fonts: SIL Open Font License 1.1. Original font licence texts are included in `public/fonts/`. Fonts are self-hosted to support offline UI and avoid runtime font-provider requests. Original designers/copyrights are recorded in those licence files.

The reproducible Theseus adaptation also adds a headless-only in-process instance event channel and typed exports for Solara's local content-install activity. Original notices/licences remain. Patch source: `scripts/patch-content-events.py`, invoked by `scripts/prepare-upstream.py`. This does not clear the outstanding dependency distribution-licence gate.
