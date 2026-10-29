# Builds, installation and release archive

Public display mapping: internal `0.0.1` → `V-0.0.01` → `builds/Solara-Launcher-V-0.0.01.exe`. Increment numerically; `09` → `10`. Old executables are immutable. This workspace currently has no generated installer; do not rename another file to `.exe` or claim it is signed.

## Before a public build

1. Resolve all missing upstream dependency grants and record actual per-package evidence in `docs/upstream-license-review.json`. Preserve GPL corresponding source, original notices and Solara patch source. Review dependency notice report against lockfiles.
2. Supply a Windows x64 build machine/Actions repository with Rust stable, MSVC/Windows SDK, Node, Python 3.12 and Java 17. `.github/workflows/windows.yml` is explicit workflow_dispatch and serializes build allocation.
3. Register an approved Solara PKCE public client. Configure public native IDs `SOLARA_CLIENT_ID`, `SOLARA_MICROSOFT_CLIENT_ID`, optional `SOLARA_DISCORD_CLIENT_ID`; Microsoft's app must have the necessary Xbox/Minecraft permissions and supported redirect/flow. No upstream ID is reused and no secret is embedded.
4. Configure trusted `SOLARA_UPDATER_PUBLIC_KEY`; keep `TAURI_SIGNING_PRIVATE_KEY` and optional password only in private CI secret storage. Generate keys using Tauri CLI in a private environment; never put keys into source/artifact logs. Updater signatures are distinct from Authenticode: Windows publisher signing is a separate optional configuration and must not be claimed unless supplied/verified.
5. Run shared tests, Vue type checks, native checks, local-server tests and real Windows acceptance checks. Verify licensed launch, OS-vault persistence, independent logout, process shutdown, NSIS install/uninstall and signed update/install before public release.

## Workflow

```sh
node scripts/release.mjs reserve
# Or in the configured GitHub repository: reserve --github
node scripts/release.mjs package
node scripts/release.mjs stage <actual-NSIS-installer-path> <release-notes-path>
```

Reserve uses an exclusive filesystem lock, consumed-version reservation files and numeric counter. GitHub mode examines existing draft/published build tags and creates a new draft reservation, under serialized workflow concurrency. Failed builds consume their reserved number; they do not overwrite previous artifacts. No number has been consumed in this workspace merely by tests (tests use temporary directories).

Packaging fails outside Windows, while licence review is unresolved, or without signing configuration. `tauri build` produces a real NSIS current-user installer plus updater signature. Stage verifies real x86_64 PE headers, file size, SHA-256, Minisign/Tauri file signature and authenticated signature comment before copying with exclusive-create semantics into `builds/`. Repeated staging only accepts identical bytes. Metadata records public/internal versions, build ID, commit, architecture/platform, verified timestamp, size/hash, signature, URL, notes and staged state.

Upload immutable installer/signature to the prepared same-VPS artifact path (or deliberately configured GitHub release archive), then:

```sh
node scripts/release.mjs publish-manifest V-0.0.01
```

Publication verifies the trusted signature again and downloads the actual public URL to compare bytes before adding a published entry/date to `builds/releases.json`. Both website and desktop read main-API metadata. A missing file, altered checksum, invalid signature, staged or withdrawn release is not offered as an update.

```sh
node scripts/release.mjs withdraw V-0.0.01 "Specific withdrawal reason"
```

Withdrawal preserves the executable and records state/reason/time. For rollback, withdraw a bad release and issue a higher numeric version containing corrected code; do not change an existing version's bytes or silently downgrade updater validation.

## Local data

Solara-specific preferences/local-server files: platform local app-data directory under `in.solara.launcher`. Instance/game/world/cache SQLite paths follow the pinned engine's `in.solara.launcher` configuration, separate from upstream's own app identity. Windows tokens use Credential Manager service names `in.solara.launcher.solara` and `in.solara.launcher.minecraft`; SQLite stores Minecraft vault references. No plaintext credential fallback is allowed. Back up instance/local-server folders before manual relocation; don't overwrite existing worlds or upstream app directories.

Tauri updater public-key verification is mandatory. Missing keys produce a useful unconfigured state. No public update is available when the archive is empty. Discord is optional and does not block game launch if missing/not running; it only emits coarse app/game state and clears on exit.

## Native content activity verification

`SQLX_OFFLINE=true cargo test -p solara-launcher --lib --locked` covers exact API-resource allowlisting, selected modpack identity/archive validation, bounded activity state and the actual inherited headless event emitter/channel/ledger path. The pinned preparation invokes `scripts/patch-content-events.py` to reproduce that headless event bridge. Browser workflow fixtures run with `npm run test:workflows`; actual Windows compatible installation and game launch still require separate acceptance. Content activity records are in memory and do not imply persisted/resumable work after restart.
