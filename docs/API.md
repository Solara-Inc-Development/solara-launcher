# Main API and identity contracts

All application/data calls use `https://api.solara-inc.in`. Existing SSO remains at `https://auth.solara-inc.in`; this is an identity service on the same VPS, not another application API. The separately maintained private PHP module is `/var/www/solara-launcher-private`.

These launcher routes are implemented and deployed on this VPS via `/var/www/solara-launcher-private`. Key routes including `/v1/launcher/update/windows/x86_64`, `/native/authorize`, `/native/token`, and `/v1/launcher/me` are active, serving verified releases and central SSO tokens.

| Method/path | Authentication | Response / rules |
| --- | --- | --- |
| GET `/health` | Public | Module health; not proof every provider works |
| GET `/v1/launcher/bootstrap` | Public | `schemaVersion:1`, generatedAt, public projects/services, sources with freshness, network config, release metadata; earlier event source preserved but feature deferred |
| GET `/v1/launcher/releases` | Public | `{releases:[]}`; incomplete artifacts are never downloadable |
| GET `/v1/launcher/update/windows/x86_64` | Public | Tauri updater object or HTTP 204; signed installer metadata only |
| GET `/v1/launcher/me` | Native Solara Bearer | Minimal central ID, display name and username; no Microsoft fields |
| GET `/v1/launcher/servers` | Native Solara Bearer | Only servers permitted by the user's verified panel client key; bounded 50/page, max 10 pages, server states/actions reflect actual provider data |
| POST `/v1/launcher/servers/{id}/power` | Native Solara Bearer | `{action:start/stop/restart}`; refetch permission/ownership and let panel enforce permission too |
| GET `/v1/launcher/entitlements` | Native Solara Bearer | Per-user feature, expiry and active/revoked state |
| GET `/v1/launcher/assignments` | Native Solara Bearer | Owner-only assignment list (bounded 100): `{assignments:[{id,serverId,name,hostname,endpoint,port,status,lastError,updatedAt}]}`; hostname is null until a Solara-controlled namespace is configured; provider IDs/raw errors stay private |
| POST `/v1/launcher/assignments` | Native Solara Bearer + local-hosting entitlement | Strict name/endpoint/port/serverId/consent; endpoint control proof; quota; provider required |
| PATCH `/v1/launcher/assignments/{id}` | Owner + active entitlement | Strict `{endpoint,port,consent:true}` only; verified updated endpoint and provider reconciliation; assignment name/server linkage stay fixed |
| DELETE `/v1/launcher/assignments/{id}` | Owner | Release provider records; permitted even if entitlement expired |

Errors use `{error:{code,message}}`: 400 malformed/unsupported requests, 401 missing/expired/revoked session, 403 unapproved client/permission/entitlement, 404 inaccessible assignment, 409 quota/endpoint-proof/name conflict, 422 invalid endpoint/port/name/consent, 429 with Retry-After, 502 DNS failure, 503 unconfigured/unavailable provider. No upstream secrets or raw authentication responses enter error bodies.

Public fetches use no credentials and a timestamped bounded local cache. Main API outages don't block local Minecraft launch. Browser download links are ordinary public versioned artifact URLs and do not require an account. The website and app share the same release predicate; the backend also verifies local artifact existence, executable header, size, checksum and signature metadata before serving a published release.

## Native Solara identity extension

The inspected existing SSO code only provides a confidential-client code/JWT exchange. It does not have PKCE binding, rotating refresh grant or revocation. Native clients do not use that unsafe fallback and do not embed a secret. See [AUTH-SETUP.md](AUTH-SETUP.md) for the exact step-by-step Microsoft Entra device-code registration and reviewed additive Solara PKCE setup/test checklist.

Prepared additions at the **existing SSO host**:

- GET/POST `/native/authorize`: existing central browser identity, exact loopback callback policy, explicit consent + CSRF, S256 challenge and state. Callbacks only `http://127.0.0.1:{1024..65535}/solara/callback`, no userinfo/query/fragment.
- POST `/native/token`: form authorization_code or refresh_token grants. Codes expire after 2 minutes, single-use under DB transaction, bound to public client/redirect/challenge. Access tokens last 15 minutes, refresh family has an absolute 30-day expiry. Database stores hashes; used refresh tokens revoke their family on replay.
- POST `/native/revoke`: revoke the matching client's session. Account status, client approval and revocation are rechecked on protected main API calls.

The desktop opens the system browser and binds an ephemeral IPv4 loopback listener. It validates method/path/state, bounds input, handles cancellation/180-second expiry, and stores local access/refresh tokens only in Windows Credential Manager. Tokens are never returned to the Vue UI. Sign-out removes only Solara credentials; Minecraft sign-in/out is separate.

Registration/recovery remain on the existing central account website. Global central-account session-revocation hooks must be wired/tested before a claim of fully shared revocation. Private CLI supports explicit native user/client session revocation now.

## Hosting and DNS configuration

Panel is confirmed Pterodactyl by source inspection. Provider keys stay encrypted in private storage, with an explicit admin-verified central UUID → panel user ID mapping. Client API permissions govern resources/actions. Native console integration beyond the web panel is pending; existing web console is linked honestly.

DNS provider is not assumed from proxy/CDN use. A private `DnsProvider` implementation must be supplied for actual credentials/provider, supporting idempotent A/AAAA + SRV upsert/release/reconciliation. SRV targets a hostname with A/AAAA, never a raw IP. A private endpoint challenge adapter must prove control; merely pinging an IP is not ownership proof. No gameplay tunnel or VPS relay is introduced.

## Restricted developer access

Register a public client ID with `bin/admin.php client:register`, review ownership/scope, then explicitly approve it. No universal client secret or binary-attestation claim. Restrict the underlying operation using active user session, entitlement, ownership and quotas, not an official-build flag. Revoking a client immediately revokes native sessions. Developer and entitlement administration stays private, outside the app.

## Content provider and install selection

Public content browsing goes directly to Modrinth without Solara/Minecraft credentials. Search results retain their own project type; changing the search filter aborts obsolete work and clears stale results. Trusted provider icons are optional. API shapes were verified against official [search](https://docs.modrinth.com/api/operations/searchprojects/), [project-version list](https://docs.modrinth.com/api/operations/getprojectversions/) and [version lookup](https://docs.modrinth.com/api/operations/getversion/) documentation.

The archived `V-0.0.03` testing build (`8326bb7`, currently withdrawn on security hold) advertised `launcher_capabilities` with vanilla only and `contentInstall:false`. Current `main` source advertises `loaders: ["vanilla", "fabric", "quilt"]`, `contentInstall: true`, `automaticJava: true`, `instanceFiles: true`, `modManager: true` and `instanceBackups: true`, backed by `apps/desktop/src/game_core/content.rs`, `mods.rs` and `backups.rs` (verifying both SHA-1 and SHA-512 digests from `cdn.modrinth.com`, resolving required Modrinth dependencies, extracting `.mrpack` overrides safely, importing validated local `.jar` mods, and managing isolated per-instance world/config backups).

`npm run test:workflows` uses explicitly labelled browser/provider/IPC fixtures to check UI selection, retry, account separation and owner-address management.

Vanilla, Fabric and Quilt installs use the independently authored metadata/integrity/cache/staging code. Snapshot downloads show actual queued/running/completed/failed/cancelled jobs (`install`, `java`, `content`), with cancellation and kind-aware retry. History is bounded in memory; restarting does not claim resumable jobs. Microsoft device-code prompts go only to the main local window; credentials remain native/on-device. Approved Microsoft registration and Windows account/game tests are still required.
