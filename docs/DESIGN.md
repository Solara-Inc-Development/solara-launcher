# Solara visual design — current direction

Latest owner direction, 2026-10-29: clean, simple and dark, with Modrinth-like clarity, original high-quality images, smooth animation and useful loading screens. No glow, neon, glossy buttons or floating portal artwork. The result should feel natural and carefully designed rather than generic AI fantasy artwork. This supersedes the earlier futuristic/glossy direction. Seasons and festivals remain deferred with `features.seasons=false`.

## Layout and imagery

Pure black surfaces, rounded controls, blue/purple/amber/teal accents, a compact icon sidebar with rounded hover labels, fewer boxed cards and easy English. A separate account/information rail collapses on smaller windows. The top toolbar has Home/current location and real Downloads access. The actual Solara logo and original local SVG glyphs remain. The dashboard uses forest artwork with readable text over a dark overlay, real local library actions and actual project/service listings. Other scenes have specific homes: alpine lake in the library, workshop in local servers, canyon in discovery, coast on Network/website and a naturally dark cave at startup.

`packages/ui/WorldArt.vue` renders original illustrative landscape WebPs with actual load/decode fade, pending indicator and failure fallback. Artwork is generated with built-in imagegen, visually reviewed. It is not a screenshot of an actual user world or installed pack. Full-size/thumbnail delivery assets total 1,765,840 bytes; original PNGs remain in `assets/artwork/source/` outside the shipped public directory. No external image service is needed.

Active styling is `packages/ui/base.css`, `packages/ui/motion.css`, `apps/desktop/desktop.css`, `apps/desktop/reference-layout.css` and `apps/website/website.css`. Older `future*.css` and `PortalArt.vue` are inactive historical source; do not re-enable the rejected design.

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

Latest source screenshots live in `public/screenshots/`; rejected portal screenshots are preserved locally in `checks/rejected-portal/`, earlier plain screenshots in `checks/previous-design/`. Keep development previews labelled. Screenshots/recordings are browser evidence, not Windows game/SSO/provider acceptance, production deployment or an executable release.
