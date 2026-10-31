# Solara visual design — current direction

Latest owner direction, 2026-10-31: a colourful desktop layout guided by the owner's launcher screenshots, with Solara's own branding, artwork and implementation. The reference structure is a slim icon rail, a single top toolbar, an OLED workspace (`#030405`), rounded dark panels (`#111215`, `#191b20`) and a persistent account/information column. Mint (`#45dc9b`) is the default desktop accent; existing saved accent choices are preserved. Artwork and complementary loader colours keep the interface colourful. Short press, page and dialog transitions respect reduced-motion preferences. Seasons and festivals remain deferred with `features.seasons=false`.

## Layout and imagery

The icon rail contains accessible navigation labels/tooltips, a create action and shortcuts to actual local instances. The single top toolbar provides guarded navigation history, working content search on larger windows, observed running-game state, separate account actions and downloads. Home starts with a compact welcome and a list of installed/running local instances, followed by the library. It does not copy the reference's server counts or play-time data. Instance tiles use original local artwork; their loaders, versions and running state come from the native snapshot. Quick starts install clean game/loader setups and make no claims about bundled mods.

`ContextRail.vue` keeps Playing as, Solara identity, a working preferences link and Solara services available beside Home, Library and instance pages. It collapses below 1100px so small desktop windows retain usable content width. Its geometric slider illustration is original CSS; the service image is existing Solara artwork. Discover places supported content categories to the right of readable project rows. Instance headers are compact and their tabs use pill navigation. Create-instance choices are stacked, with the existing two supported paths. Library, services, preferences, instance files/mods/backups/screenshots/logs and dialogs share the desktop tokens. Upstream skins, subscriptions, sync promotions and unsupported import actions are not represented as working Solara features.

Guest, empty, cached and unavailable states remain visible. Popularity counts are shown only when returned by the content provider. The snapshot dialog describes its actual world/config backup scope; it excludes mods, resource packs and shaders. Creating a similar instance creates a fresh setup with matching version/loader rather than claiming to copy user files.

`packages/ui/WorldArt.vue` renders original illustrative landscape WebPs with actual load/decode fade, pending indicator and failure fallback. Artwork is generated with built-in imagegen, visually reviewed. It is not a screenshot of an actual user world or installed pack. Full-size/thumbnail delivery assets total 1,765,840 bytes; original PNGs remain in `assets/artwork/source/` outside the shipped public directory. No external image service is needed.

Active desktop styling is `packages/ui/base.css`, `packages/ui/motion.css` and `apps/desktop/launcher.css`. The old desktop/reference/workspace stylesheets remain historical source and are no longer imported. Website styling remains separate in `apps/website/website.css` and its existing entrypoint imports. No upstream interface code or brand assets were copied for this redesign.

## Loading and motion

- Startup waits for actual interface images/fonts and native initialization. There is no invented progress percentage or minimum splash duration. Interface-resource waiting is bounded to eight seconds; failed images fall back instead of trapping the app. Native failure offers retry or browsing without a working engine.
- Public API loading does not gate local startup. Catalogue skeletons appear while a real public request is pending and there is no data; cached listings remain visible during refresh. Concurrent public refreshes are deduplicated.
- Discovery has actual provider-request pending, success, empty and failure states. It never invents popular content or installed instances.
- Distinct short navigation glyph, press, toggle, loader selection, page, image, dropdown, dialog and toast transitions provide movement when useful. Artwork does not constantly orbit or drift, and listing rows are manually scrollable.
- Page visibility pauses CSS motion. Offscreen pending artwork/skeletons pause. Reduced motion disables animations/transitions and image hover zoom. No WebGL/canvas, animation framework or runtime remote fonts are added.
- Keyboard labels, focus-contained dialogs, nested Escape and custom dropdown keyboard behavior remain required.

`npm run test:motion` checks images/matte controls, real resource-driven startup, controlled provider pending/error states, native initialization UI through an explicitly labelled IPC test fixture, reduced motion and image fallback. Fixture checks are browser component evidence, never a native Windows acceptance claim. `npm run test:ui` checks actual navigation, controls and desktop/mobile layout.

## Screenshots and recording

`npm run capture:preview` captures the actual app and records `checks/solara-motion.webm`. For actual published catalogue data through the existing read-only local adapter, run it with `SOLARA_PREVIEW_API=http://127.0.0.1:1431` while the private dev server is running. No game accounts, installed instances or process activity are fabricated.

`npm run test:redesign` verifies search, loader filters, sidebar shortcuts, initial mod loading, guarded history, the snapshot modal, quick-start arguments, reduced motion and layouts at 1440, 1100, 800 and 650 pixels. It captures a local review gallery at `checks/redesign/index.html`. Populated screens are explicitly labelled QA fixtures. Guest screenshots depict the actual browser preview with an unavailable API.

Publishable screenshots live in `public/screenshots/`; earlier designs are preserved locally under `checks/previous-design/`. Keep development previews labelled. Screenshots/recordings are browser evidence, not Windows game/SSO/provider acceptance, production deployment or an executable release.
