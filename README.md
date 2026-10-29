<p align="center">
  <img src="assets/readme/hero.svg" alt="Solara Launcher — one home, every adventure" width="100%" />
</p>

<p align="center">
  <strong>A calmer home for Minecraft Java and your Solara services.</strong><br />
  Dark by design. Clear controls. More room for your next world.
</p>

<p align="center">
  <a href="https://github.com/Solara-Inc-Development/solara-launcher/actions/workflows/windows-check.yml"><img src="https://github.com/Solara-Inc-Development/solara-launcher/actions/workflows/windows-check.yml/badge.svg" alt="Windows source checks" /></a>
  <img src="https://img.shields.io/badge/platform-Windows-19232D?style=flat&amp;labelColor=11151A" alt="Windows first" />
  <img src="https://img.shields.io/badge/status-in%20development-315E43?style=flat&amp;labelColor=11151A" alt="In development" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/licence-GPL%203.0-19232D?style=flat&amp;labelColor=11151A" alt="GPL 3.0 licence" /></a>
</p>

<p align="center">
  <a href="#meet-your-launcher">Take a look</a> &nbsp; · &nbsp;
  <a href="#download">Download status</a> &nbsp; · &nbsp;
  <a href="#build-it-yourself">Build from source</a> &nbsp; · &nbsp;
  <a href="https://www.solara-inc.in/contact">Get help</a>
</p>

---

## Meet your launcher

A black desktop, a small sidebar and your account right where you need it. Open your worlds, check your services, or just look around. We wanted the everyday stuff to feel easy.

<img src="public/screenshots/dashboard.png" alt="Actual Solara development dashboard with compact navigation, original landscape artwork, real Solara listings and account sidebar" width="100%" />

<p align="center"><sub>Actual development interface. Listings come from a local development API using existing Solara sources. This is not a release or a live sign-in demo.</sub></p>

<table>
<tr>
<td width="33%" valign="top">

### Your worlds

Give each Minecraft setup its own space. Find instances by name, version or loader, and switch between grid and list views.

</td>
<td width="33%" valign="top">

### Your services

Explore Solara projects and supported server tools. Public browsing works as a guest; protected actions use your Solara account.

</td>
<td width="33%" valign="top">

### Your pace

Smooth little animations, clear loading screens and colours that don't shout at you. Less motion if you prefer it. And yep, no glow effects.

</td>
</tr>
</table>

## A place for your next adventure

<img src="public/screenshots/library.png" alt="Actual Solara Minecraft library development screen with search, grid and list controls, instance creation and separate local account controls" width="100%" />

**A clean library.** Keep setups separate, choose your version and make room for mods.<br />
**A clear view of downloads.** Follow real progress and useful error states.<br />
**Separate accounts.** Solara connects your services. Minecraft sign-in stays on your device.

> **Still being built:** the local Minecraft engine is now independently written. The first testing build supports vanilla installs; mod loaders and content installation are still being added. Real sign-in, game launch and automatic Windows updates still need acceptance testing. Screenshots show working interface code, not proof that every integration is ready.

## Download

**The public Windows installer is not available yet.**

The download and installed app will be named **`solaralauncher.exe`**. Install once; signed updates are designed to download and apply automatically when games, local servers and downloads are idle. Real update/relaunch testing is still required before release.

| | Status |
| --- | --- |
| First platform | Windows 10/11 x64 — final compatibility testing pending |
| Installer | Not published |
| Minecraft core | Custom vanilla preview; Windows game testing pending |
| App updates | Automatic-update code added; signed Windows upgrade testing pending |
| Website address | [app.solara-inc.in](https://app.solara-inc.in) — deployment pending |

There's still a few things to test before the first download goes up. Passing source checks alone doesn't make it a release. We'll put the real download here and on the official website once it's built and checked. You need a valid Minecraft Java licence to play.

Found something that feels off? Tell us what happened, even the small stuff helps.

## Build it yourself

**Vue 3** for the interface. **Tauri 2 + Rust** for desktop code. Shared components for the launcher and website.

```sh
npm ci
npm run dev
```

| Preview | Address |
| --- | --- |
| Website | `http://127.0.0.1:1420/` |
| Desktop interface | `http://127.0.0.1:1420/desktop.html` |

The browser preview does not simulate Minecraft accounts, installed games or native processes.

```sh
npm test
npm run check
npm run build
```

Native prerequisites, signing and packaging: [build guide](docs/BUILD-RELEASE.md). Public service contracts: [API guide](docs/API.md).

<details>
<summary><strong>More development checks</strong></summary>

With the UI development server running and Playwright Chromium installed:

```sh
npm run test:ui
npm run test:motion
npm run test:workflows
npm run test:csp
```

Browser workflow tests use clearly labelled provider/native fixtures. They do not install Minecraft, provision DNS or prove live authentication.

</details>

## Built around your device

Minecraft worlds, profiles and account credentials stay local. Microsoft passwords and Minecraft tokens must never go to the Solara backend. Solara and Minecraft have separate sign-in and sign-out controls.

The Solara service backend is private and **is not included in this repository**. Public application data uses `https://api.solara-inc.in`. This repository contains the public desktop and website source.

---

<p align="center">
  <img src="public/logo.svg" alt="Solara" width="28" /><br /><br />
  <strong>Make it your world.</strong><br />
  <sub>Made by Solara · Windows first · Minecraft Java</sub><br /><br />
  <a href="https://www.solara-inc.in">Solara</a> &nbsp; · &nbsp;
  <a href="https://www.solara-inc.in/contact">Support</a> &nbsp; · &nbsp;
  <a href="LICENSE">Licence</a> &nbsp; · &nbsp;
  <a href="THIRD_PARTY_NOTICES.md">Third-party notices</a>
</p>

<sub>Public source: GPL-3.0-only. Dependency notices and their licences are included. Solara is not affiliated with Mojang or Microsoft.</sub>
