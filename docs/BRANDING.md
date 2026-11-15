# Solara product identity

Desktop product names, support destinations and the HTTP user-agent are defined in
`packages/shared/brand.ts`. The desktop keeps its existing navigation, dimensions,
controls and launcher flows. Light, dark, OLED and retro themes use cyan accents;
the semantic colour-token names remain compatible with existing components.

Built-in messages and translations use Solara for the application and neutral
catalog labels for external content. Translation IDs and ICU arguments remain
unchanged. Creator names, project descriptions, URLs and other provider/user data
are not renamed or presented as Solara-authored content.

Public catalog browsing and local modpack installs require no catalog account.
Unsupported catalog-account, cloud-settings and social entry points are hidden.
Shared online instance links explain that the feature is unavailable and never
grant authorization. Microsoft/Minecraft sign-in and ownership checks remain in
place for licensed gameplay.

Existing theme choices are read from the legacy preference keys and copied to
the Solara keys when storage is writable. Existing instance schemas, import
formats, provider endpoints and local data paths are preserved.

The current desktop retains the Modrinth-derived frontend and Theseus engine.
Product rebranding and refactoring do not remove the original copyright or
licence requirements. Keep package licences, copying notices, upstream source
references and corresponding-source obligations; see `THIRD_PARTY_NOTICES.md`.
This change does not claim a majority independent rewrite or legal clearance.

The rebrand is an unreleased source change. Existing versioned installers are
immutable; a new Windows candidate and real-device acceptance are separate work.

For browser verification, start `pnpm --filter @modrinth/app-frontend dev --host
127.0.0.1`, then run `npm run test:branding`. Install Playwright's Chromium or set
`SOLARA_CHROMIUM_PATH` to an existing Chromium executable. `SOLARA_DESKTOP_URL`
can override the local frontend URL. The script mocks native IPC and all external
requests. It checks four colour themes, settings, the unavailable-account panel,
the instance wizard and a second locale. Screenshots stay under ignored
`checks/rebrand/`; these fixtures do not prove Windows or live-service behavior.
