<div align="center">

<img src="public/logo.svg" alt="Solara" width="72" />

# Solara Launcher

**Your Minecraft worlds. Your Solara services. One home.**

A Windows desktop launcher with a clean dark interface.

[Website](https://app.solara-inc.in) · [Support](https://www.solara-inc.in/contact) · [Build checks](https://github.com/Solara-Inc-Development/solara-launcher/actions)

![Development status](https://img.shields.io/badge/status-in_development-61c48c?style=flat-square)
[![Windows checks](https://github.com/Solara-Inc-Development/solara-launcher/actions/workflows/windows-check.yml/badge.svg)](https://github.com/Solara-Inc-Development/solara-launcher/actions/workflows/windows-check.yml)

</div>

![Solara Launcher dashboard — development preview](public/screenshots/dashboard.png)

## What is Solara?

Solara brings Minecraft Java and Solara services into one desktop app. Browse your instances, find content, manage local servers and keep track of downloads from a single place.

- **A clear home:** a matte dark interface, smooth motion and reduced-motion support.
- **Your Minecraft space:** separate instances, game settings and content browsing.
- **Your services:** Solara account access and supported server tools.
- **Install once:** the planned Windows download is `solaralauncher.exe`. Automatic signed updates are being tested.
- **Your data stays yours:** Minecraft account credentials and worlds stay on your device.

## Download status

**No public Windows installer is available yet.** Screenshots show the real development interface, not a finished release.

The local Minecraft engine is being replaced with an independently written implementation. Account setup, complete game flows and automatic updates still need real Windows acceptance testing. Passing source checks alone does not mean a release is ready.

When the first verified installer is published, it will be available through [app.solara-inc.in](https://app.solara-inc.in). A valid Minecraft Java licence is required to play.

## Development

The desktop app uses **Vue 3, Tauri 2 and Rust**. The website shares the interface components and public API client.

```sh
npm ci
npm run dev
```

Open `http://127.0.0.1:1420/` for the website or `/desktop.html` for the desktop preview. Native actions require the installed desktop runtime; the browser preview does not simulate game accounts or running games.

```sh
npm test
npm run check
npm run build
```

See [build instructions](docs/BUILD-RELEASE.md) for native prerequisites and release checks, and [API contracts](docs/API.md) for public integration details.

## Privacy and accounts

The Solara service backend is private and is not included in this repository. Public app data uses `https://api.solara-inc.in`.

Solara sign-in and Minecraft sign-in are separate. Microsoft passwords and Minecraft tokens must never be sent to the Solara backend. Native sign-in uses the official account services and secure local credential storage.

## Licence

Public source is licensed under [GPL-3.0-only](LICENSE). See [third-party notices](THIRD_PARTY_NOTICES.md) for dependencies and required attribution. Solara is not affiliated with Mojang or Microsoft.
