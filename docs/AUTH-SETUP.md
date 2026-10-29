# Authentication setup guide (Microsoft / Minecraft & Solara SSO)

Solara Launcher uses **two completely separate authentication flows**:

1. **Minecraft Java account (`SOLARA_MICROSOFT_CLIENT_ID`)** — talks directly from the user's Windows PC to official Microsoft (`login.microsoftonline.com`), Xbox Live (`*.auth.xboxlive.com`), and Minecraft (`api.minecraftservices.com`) endpoints. Refresh tokens are stored only in Windows Credential Manager under `in.solara.launcher.minecraft.custom`. No Microsoft email, password, or token is ever sent to Solara.
2. **Solara account (`SOLARA_CLIENT_ID`)** — authenticates optional Solara services, hosted server controls, and subdomain assignments through `https://auth.solara-inc.in` and `https://api.solara-inc.in` using OAuth 2.0 Authorization Code with PKCE (`S256`). Session tokens are stored only in Windows Credential Manager under `in.solara.launcher.solara`.

Neither integration is enabled with placeholder or third-party credentials. Until the external registrations and authorized backend steps below are completed, the desktop app fails closed with a clear configuration message.

---

## 1. Microsoft Entra public-client registration (Minecraft Java sign-in)

The native implementation in [`apps/desktop/src/game_core/auth.rs`](../apps/desktop/src/game_core/auth.rs) uses the OAuth 2.0 **Device Authorization Grant** against `https://login.microsoftonline.com/consumers/oauth2/v2.0` with scope `XboxLive.signin offline_access`.

### Step-by-step registration in Microsoft Entra

1. Sign in to the [Microsoft Entra admin center](https://entra.microsoft.com) with the Solara owner Microsoft account.
2. Navigate to **Identity → Applications → App registrations** and select **New registration**.
3. Configure the registration:
   - **Name:** `Solara Launcher`
   - **Supported account types:** **Personal Microsoft accounts only** (matches the `/consumers/` authority required for Xbox Live consumer accounts).
   - **Redirect URI (optional for device code, recommended for public client):** select **Public client/native (mobile & desktop)** and enter `https://login.microsoftonline.com/common/oauth2/nativeclient`.
4. Select **Register** and copy the **Application (client) ID** UUID from the **Overview** page.
5. In the app menu, open **Manage → Authentication → Advanced settings**:
   - Set **Allow public client flows** (*"Enable the following mobile and desktop flows"*) to **Yes**.
   - Select **Save**. Without this toggle, `POST /consumers/oauth2/v2.0/devicecode` will reject device-code requests.
6. **Do not create a client secret.** Desktop applications are public clients; secrets cannot be kept confidential in distributed binaries and are never used by `auth.rs`. Never reuse another launcher's client ID.

### Xbox Live & Minecraft Services approval

Microsoft restricts the `rp://api.minecraftservices.com/` relying party and `api.minecraftservices.com` authentication endpoints to approved application IDs:

1. Submit the official Minecraft Services / Xbox Live API access request for the newly created **Application (client) ID** UUID.
2. Wait for approval confirmation before expecting live `xsts.auth.xboxlive.com` and `api.minecraftservices.com/authentication/login_with_xbox` exchanges to succeed.

### Building and verifying on Windows

1. Pass the approved public client ID at build time:
   ```powershell
   $env:SOLARA_MICROSOFT_CLIENT_ID = "<your-entra-application-client-id-uuid>"
   ```
2. On a Windows 10/11 PC with a licensed Minecraft Java account:
   - Click **Add account** in the Minecraft Launcher view.
   - Confirm the one-time device code is displayed inside the main Solara window and opens `https://www.microsoft.com/link` in the system browser.
   - Complete sign-in and verify that `entitlements/mcstore` (`game_minecraft` or `product_minecraft`) and `minecraft/profile` succeed.
   - Confirm `custom-minecraft-accounts/accounts.json` contains only `{ "id", "name", "active" }` (no tokens) and that the refresh token is stored in Windows Credential Manager under `in.solara.launcher.minecraft.custom`.
   - Confirm signing out of Minecraft removes the Windows Credential Manager entry without affecting any Solara SSO session.

---

## 2. Solara Native PKCE SSO setup (`auth.solara-inc.in` & `api.solara-inc.in`)

The desktop implementation in [`apps/desktop/src/identity.rs`](../apps/desktop/src/identity.rs) uses a system-browser loopback PKCE flow (`http://127.0.0.1:{1024..65535}/solara/callback`, `code_challenge_method=S256`, scope `profile services`). It refuses to use the legacy confidential-client `/api/v1/oauth/token` flow.

The private backend module in `/var/www/solara-launcher-private` (maintained outside this public repository) implements the additive routes and schema, but **no production migration or deployment has been applied**. Do not run production migrations or reload Nginx without explicit owner authorization.

### Reviewed additive setup steps (requires owner authorization)

1. **Back up the central identity database** before making any schema change.
2. **Apply the additive native SSO migration** from the private backend directory:
   - Inspect `/var/www/solara-launcher-private/sql/` to confirm all statements are strictly additive (`CREATE TABLE` for native PKCE clients, short-lived authorization codes, and rotating hashed token families) and do not alter existing corporate tables.
   - Apply the migration against the configured identity database.
3. **Register and approve the public desktop client ID**:
   ```sh
   php /var/www/solara-launcher-private/bin/admin.php client:register --name="Solara Launcher Desktop"
   php /var/www/solara-launcher-private/bin/admin.php client:approve --client="<generated-client-id>"
   ```
   - Only loopback callbacks matching `http://127.0.0.1:{1024..65535}/solara/callback` with `S256` PKCE challenges are accepted. No client secret is generated or embedded.
4. **Enable the prepared reverse-proxy locations**:
   - Route `GET/POST /native/authorize`, `POST /native/token`, and `POST /native/revoke` on `https://auth.solara-inc.in`.
   - Route `/health` and `/v1/launcher/*` on `https://api.solara-inc.in`.
   - Test configuration with `nginx -t` before reloading.
5. **Run contract and end-to-end verification**:
   - Run `php /var/www/solara-launcher-private/tests/contracts.php` to verify callback validation, PKCE challenge binding, refresh-token rotation/replay revocation, and release/DNS contracts.
   - Build the desktop app with `SOLARA_CLIENT_ID="<approved-client-id>"`.
   - Verify browser consent, loopback callback handoff, `/v1/launcher/me` profile load, automatic token refresh, refresh-token replay family revocation, and `/native/revoke` sign-out on Windows while confirming Minecraft account state remains completely independent.
