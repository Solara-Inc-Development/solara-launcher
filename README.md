# Solara Launcher

Windows-first desktop hub, Minecraft Java launcher and official website frontend. Vue 3 + Tauri 2 + Rust; local launcher engine derived from pinned Modrinth Theseus. Public source is GPL-3.0-only, without warranty; see `LICENSE` and `THIRD_PARTY_NOTICES.md`.

**Development state:** no public Windows installer has been produced. The first public release is planned as `V-0.0.01`; real executables will live in `builds/`. Upstream dependency licensing, public client approvals, provider credentials and real Windows verification remain release blockers. No domain deployment is claimed.

Read `AGENTS.md` and `PROGRESS.md` before continuing. The owner requires incremental progress updates so the next AI/developer can continue. Latest direction: bespoke controls/design/animations; **seasons and festivals deferred**. Existing prior source is preserved and disabled.

## Local UI development

```sh
npm ci
npm run dev
```

Website: `http://127.0.0.1:1420/`. Desktop browser preview: `http://127.0.0.1:1420/desktop.html`. The preview does not simulate native accounts, installed instances or running processes. The configured main API may show unavailable states until deployed.

```sh
npm run check
npm test
npm run build
```

Both HTML entrypoints build into `dist/`. Actual development screenshots are in `public/screenshots/`. Design system and motion rationale: `docs/DESIGN.md`.

## Native development

Install Rust stable (tested compiler 1.99.1), Java 17 for the inherited agent build, and Tauri platform prerequisites. Windows requires MSVC C++ tools, WebView2 and Windows SDK. Linux native build requires GTK 3/WebKitGTK 4.1 development packages. Secure account persistence is implemented for Windows Credential Manager; other platforms fail closed for account storage.

```sh
python3 scripts/prepare-upstream.py
cargo test -p solara-local-server --locked
cargo check -p solara-launcher --locked
npm run desktop
```

The preparation script fetches exact upstream commit `e7757b14ed04e824ac7e8b8b05652d4290df721a` into ignored `.upstream/modrinth` and applies recorded Solara changes. It does not copy private Solara backend source. Upstream branding assets/frontend are not used. SQLx offline metadata is inherited; use `SQLX_OFFLINE=true` if needed. Gradle downloads require network; `SOLARA_GRADLE_BIN` may point to a separately checksum-verified official Gradle 9.1.0 executable.

Configure public native identifiers at compile time: `SOLARA_CLIENT_ID`, `SOLARA_MICROSOFT_CLIENT_ID`, optional `SOLARA_DISCORD_CLIENT_ID`, and trusted `SOLARA_UPDATER_PUBLIC_KEY`. No client secrets belong in the app or website. See `docs/BUILD-RELEASE.md`.

## Private backend

Independently implemented backend source is maintained **outside this repository** at `/var/www/solara-launcher-private`. PHP 8.4 modules reuse the existing central account database and expose launcher routes on only `https://api.solara-inc.in`; native identity routes live at the existing SSO host. Nothing has been migrated/deployed in production. Read `docs/INSPECTION.md`, `docs/API.md` and `docs/DEPLOYMENT.md`.

Microsoft/Xbox/Minecraft authentication and local game data never go to Solara's backend. Solara and Minecraft accounts have independent controls. A valid Minecraft Java entitlement is required for play.

## Delivery references

- `PROGRESS.md`: exact work/checks, owner direction and next-AI handoff.
- `docs/REQUIREMENTS.md`: implementation/test/integration-blocked/deferred checklist.
- `docs/BUILD-RELEASE.md`: distinct executable names, signing, archive and withdrawals.
- `deployment/`: prepared same-VPS configuration, not enabled production files.
- `.github/workflows/windows.yml`: explicit Windows packaging workflow with licence/signing gates.

Reproducible browser checks (with `npm run dev` running in another terminal):

```sh
npx playwright install chromium
npm run test:ui
```

Local API development preview may use `/desktop.html?api=http://127.0.0.1:1431` while the private PHP development server is running; this loopback override is disabled in production builds. Keep screenshots labelled as development UI. Public domains are not proven live by local checks.

The current visual direction is clean matte dark, with simple English, flat controls, six optimized original landscape/interior artworks and real resource/native loading screens. No glow/neon effects are active. Artwork provenance and generation prompts are in `docs/ARTWORK.md`; originals stay outside the shipped public directory. Seasons/festivals remain deferred. Check motion behavior with `npm run test:motion`; capture real screenshots and a short browser animation recording with `npm run capture:preview`. The optional `SOLARA_PREVIEW_API` must point to a running local development API. Recordings stay in ignored `checks/` and are not bundled into the application.

Content and account-owned address browser workflows: `npm run test:workflows` (explicit provider/native fixtures, not a live game/DNS acceptance test). Native route-policy tests can run independently with `rustc --test apps/desktop/src/resource_policy.rs -o /tmp/solara-resource-policy-tests && /tmp/solara-resource-policy-tests`.
