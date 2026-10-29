# Builds, installation and release archive

Public display mapping: internal `0.0.1` → `V-0.0.01` → `builds/V-0.0.01/solaralauncher.exe`. Increment numerically; `09` → `10`. Old executables are immutable. This workspace currently has no generated installer; do not rename another file to `.exe` or claim it is signed.

## Before a public build

1. The inherited engine and its blocked dependencies have been removed. Generate notices with `node scripts/third-party.mjs` after the frontend build. Preserve exact corresponding source and both lockfiles alongside every executable. Do not remove remaining library/font licences.
2. Supply a Windows x64 build machine/Actions repository with Rust stable, MSVC/Windows SDK, Node 22. `.github/workflows/windows.yml` is explicit workflow_dispatch and serializes build allocation.
3. Register an approved Solara PKCE public client. Configure public native IDs `SOLARA_CLIENT_ID`, `SOLARA_MICROSOFT_CLIENT_ID`, optional `SOLARA_DISCORD_CLIENT_ID`; Microsoft's app must have the necessary Xbox/Minecraft permissions and supported redirect/flow. No upstream ID is reused and no secret is embedded.
4. Configure trusted `SOLARA_UPDATER_PUBLIC_KEY`; keep `TAURI_SIGNING_PRIVATE_KEY` and optional password only in private CI secret storage. Generate keys using Tauri CLI in a private environment; never put keys into source/artifact logs. Updater signatures are distinct from Authenticode: Windows publisher signing is a separate optional configuration and must not be claimed unless supplied/verified.
5. Run shared tests, Vue type checks, native checks, local-server tests and real Windows acceptance checks. Verify licensed launch, OS-vault persistence, independent logout, process shutdown, NSIS install/uninstall and signed update/install before public release.

## Windows source checks before packaging

`.github/workflows/windows-check.yml` runs on source pushes to `main` and can also be dispatched from GitHub Actions. It installs locked dependencies, checks the frontend and actual Windows desktop shell, then runs native unit tests. It has read-only repository permissions and does not use signing secrets, allocate build versions, package installers or upload binaries. The Windows runner is configured; check the actual run result before claiming compilation/tests passed.

The separate `Verify and package Windows` workflow remains manual and refuses unresolved licence/signing setup before version allocation. Source checks do not clear distribution licences or replace real installed-app, OS-vault, game-launch and update acceptance.

Both workflows use locked source dependencies. No upstream-engine fetch or Gradle step is required. The release script runs the installed Tauri JavaScript CLI through Node (avoiding Windows npm command-shim execution) and merges the configured updater public key into the bundler config. Private signing values remain environment-only.

## Workflow

```sh
node scripts/release.mjs reserve
# Or in the configured GitHub repository: reserve --github
node scripts/release.mjs package
node scripts/release.mjs stage <actual-NSIS-installer-path> <release-notes-path> <bundled-x64-application-path>
```

Reserve uses an exclusive filesystem lock, consumed-version reservation files and numeric counter. GitHub mode examines existing draft/published build tags and creates a new draft reservation, under serialized workflow concurrency. Failed builds consume their reserved number; they do not overwrite previous artifacts. No number has been consumed in this workspace merely by tests (tests use temporary directories).

Packaging fails outside Windows, while licence review is unresolved, or without signing configuration. `tauri build` produces a real NSIS current-user installer plus updater signature. Stage checks PE structure, file size, SHA-256, Minisign/Tauri file signature and authenticated version comment before copying with exclusive-create semantics into `builds/`. NSIS uses an x86 installer bootstrapper for the x64 application, so staging accepts x86/x64 installer headers while separately requiring the actual bundled application binary to have x64 headers. Its checksum/size are recorded too; header checks do not substitute for installed-app acceptance. Release notes and both binaries are validated before archive writes. Repeated staging only accepts identical bytes. Metadata records public/internal versions, build ID, commit, architecture/platform, verified timestamp, size/hash, signature, URL, notes and staged state.

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

Solara-specific preferences/local-server files: platform local app-data directory under `in.solara.launcher`. Custom instances live in `custom-instances/<uuid>/`, with worlds in `game/` and replaceable files in `runtime/`. Verified content is cached separately in `verified-game-files/`. The old engine namespace is never overwritten. Windows tokens use Credential Manager services `in.solara.launcher.solara` and `in.solara.launcher.minecraft.custom`; `custom-minecraft-accounts/accounts.json` holds local profile metadata only. No plaintext credential fallback is allowed. Back up instance/local-server folders before manual relocation; don't overwrite existing worlds or upstream app directories.

Tauri updater public-key verification is mandatory. Missing keys produce a useful unconfigured state. No public update is available when the archive is empty. Discord is optional and does not block game launch if missing/not running; it only emits coarse app/game state and clears on exit.

## Custom-core verification and first-preview limits

`cargo test -p solara-launcher -p solara-local-server --lib --locked` covers official metadata/path checks, verified cache integrity, world-preserving instance records, native ZIP traversal rejection, platform rules, real queue states, credential-free account schema, Java version parsing and split-stream log redaction. `cargo check -p solara-launcher --all-targets --locked` also compiles the actual shell.

The first custom-core testing build exposes vanilla installation only. Mod loaders/content installation, automatic Java acquisition, legacy resource layouts and recovery of previously running games after launcher restart are not ready. A compatible local Java executable must be selected. Microsoft sign-in requires Solara's approved public-client ID; an unconfigured build explains this instead of using another launcher's identity. Solara SSO/hosted operations also require the documented private deployment/client setup.

Local tests and browser IPC fixtures do not prove actual Windows installation, credentials or game launch. `docs/engine-migration.json` keeps Windows acceptance false until these are tested. Stable packaging/public-manifest publication remains gated; a clearly labelled testing candidate may be built for acceptance without pretending these checks have already happened.

NSIS architecture evidence: the [pinned Tauri CLI 2.12.4 bundler](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.4/crates/tauri-bundler/src/bundle/windows/nsis/mod.rs) lists x86 Unicode stubs/plugins. The official NSIS 3.13 archive was checked against that source's checksum, and its LZMA stub reports machine `0x14c` (x86). This toolchain inspection does not constitute a built Solara executable.

`npm run test:csp` exercises the desktop image policy in Chromium with labelled fixtures. It permits local images and the exact validated Modrinth CDN, rejects untrusted origins before requests, and checks no-referrer behavior. Installed WebView2 acceptance remains separate.

## Install once and automatic updates (latest owner instruction)

The website always downloads `solaralauncher.exe`; the installed binary uses that name too. Internal versions still increase, with immutable server artifacts in `builds/<publicVersion>/solaralauncher.exe`. Old versioned-filename archives remain readable. Signature, version, checksum and path validation still apply. Public downloads remain unavailable until real verified bytes are published.

Packaged Windows builds check automatically after local initialization and every 15 minutes. The current implementation downloads the full signed Tauri/NSIS update package; it does not implement delta/changed-files-only transfers. Verified bytes may be cached and reverified against fresh update metadata before installation. No new installer must be manually downloaded by the user. Quiet current-user installation replaces the app and relaunches it.

Updates wait while Minecraft, local servers, downloads (including paused/unknown operations), or a mutating native command are active. A native read/write gate protects the final idle check/installation against concurrent launch/setup commands. Missing signing configuration and offline/failed checks leave the current app usable and retry later. Development and non-Windows builds do not replace themselves. Game/world/configuration/credential data are outside the install directory.

Automatic apply/relaunch, cache recovery, upgrade/uninstall and preservation of real user data still need acceptance with two genuinely signed Windows releases. This source change does not clear the existing licence/signing/real-installer blockers or claim a shipped updater.

## Windows testing candidate workflow

`Build Windows testing candidate` is separate from stable release publication. It runs on a manual dispatch or a change to its own workflow file on main. The runner tests source, reserves a numeric draft build tag, and builds the actual NSIS installer with the committed public verification key. No private updater key is sent to this runner. Candidates are deliberately labelled unsigned until final signing on the private build host; no Authenticode publisher signature is claimed.

The pipeline exports a verified x64 application separately from the NSIS installer, SHA-256/size records, and `source.zip` containing the exact tracked build source and version/lock edits. Ignored files are excluded. It preserves these on a unique `solara-candidate-N` Git branch with a testing-only README. This also allows the build host's repository deploy key to retrieve actual bytes without GitHub artifact API credentials. Never force-push/replace a candidate branch or reuse a reserved build number.

On the private build host, verify the expected source commit, source archive and every artifact checksum, sign the genuine installer with Tauri CLI `signer sign --app-version <internal-version>` using the persistent private key, then use `release.mjs stage` with the corresponding reservation, release notes and x64 payload. Keep exact corresponding source with each staged build. The final name remains `builds/<publicVersion>/solaralauncher.exe`. Staging is not stable publication: actual installed-app/account/game/update acceptance must be recorded before `publish-manifest` can succeed.
