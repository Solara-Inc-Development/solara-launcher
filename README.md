# Solara Launcher — Windows testing builds

## V-0.0.03

[Download solaralauncher.exe](builds/V-0.0.03/solaralauncher.exe?raw=true) · [Exact source](builds/V-0.0.03/source.zip?raw=true) · [Checksums and signature](builds/V-0.0.03/metadata.json)

Genuine x64 Windows testing preview. The installer is 7,144,205 bytes (about 7.1 MB). Native Windows compilation/tests and static installer/source/signature checks passed. This build has **not** completed real Windows install, account login, game-launch or automatic-upgrade acceptance. Vanilla only; mod loaders and content installation are coming later. Account/hosted features need approved application IDs and service configuration.

The installer and installed app are named **solaralauncher.exe**. The included signature is for Solara's updater verification; a Windows Authenticode publisher signature is not claimed. Keep the installer together with its exact corresponding source and notices when redistributing.

Installer SHA-256: `c4e07da610ea9e1f108323b1413f2d41a6a0f2274f12419b6f4064e49f931513`.

Built from [8326bb7a32da40ecb5db7725416a545de72eee1c](https://github.com/Solara-Inc-Development/solara-launcher/tree/8326bb7a32da40ecb5db7725416a545de72eee1c), with exact generated version/lock changes in source.zip. The raw Cargo application and actual installed application are preserved separately. Their only difference is Tauri's documented UNK-to-NSS bundle marker; metadata records both hashes. First two build reservations were consumed by unsuccessful packaging attempts; their numbers are not reused.

Stable auto-update metadata and production website/API deployment remain separate. Never overwrite an archived version or force-push this branch. Public application source: GPL-3.0-only; dependencies retain their original licences/notices.
