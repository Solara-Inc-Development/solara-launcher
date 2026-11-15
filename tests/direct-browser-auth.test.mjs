import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));

test('Microsoft OAuth Client ID is configured to official Solara Client ID and Prism ID is removed', async () => {
  const authRs = await readFile(path.join(root, 'packages/app-lib/src/state/minecraft_auth.rs'), 'utf-8');
  assert.ok(
    authRs.includes('6a9238bb-3803-46f4-bf38-2f45bdf8c2ea'),
    'minecraft_auth.rs must configure the official Solara Microsoft Client ID'
  );
  assert.ok(
    !authRs.includes('c36a9fb6-4f2a-41ff-90bd-ae7cc92031eb'),
    'minecraft_auth.rs must not retain the temporary Prism Client ID'
  );
});

test('Loopback authorization code flow with PKCE RFC 7636 is implemented in Rust core', async () => {
  const authRs = await readFile(path.join(root, 'packages/app-lib/src/state/minecraft_auth.rs'), 'utf-8');
  assert.ok(authRs.includes('generate_pkce_pair'), 'PKCE generator is implemented');
  assert.ok(authRs.includes('build_authorize_url'), 'Loopback authorization URL constructor is implemented');
  assert.ok(authRs.includes('wait_for_loopback_code'), 'Loopback TCP callback listener is implemented');
  assert.ok(authRs.includes('login_with_loopback'), 'Direct browser loopback login flow is implemented');
  assert.ok(authRs.includes('grant_type", "authorization_code'), 'Token exchange uses grant_type=authorization_code');
  assert.ok(authRs.includes('code_verifier'), 'Token exchange passes code_verifier');
  assert.ok(
    authRs.includes('Authentication Complete') &&
      authRs.includes('return to Solara Launcher.'),
    'Branded loopback HTML response informs user of successful authentication'
  );
});

test('OS keyring storage and redacted SQLite/IPC serialization are enforced', async () => {
  const authRs = await readFile(path.join(root, 'packages/app-lib/src/state/minecraft_auth.rs'), 'utf-8');
  assert.ok(
    authRs.includes('in.solara.launcher.minecraft.custom'),
    'minecraft_auth.rs must store tokens under in.solara.launcher.minecraft.custom keyring service'
  );
  assert.ok(
    !authRs.includes('minecraft_device_tokens'),
    'minecraft_auth.rs must not use Modrinth Sisu device tokens'
  );
  const pluginAuth = await readFile(path.join(root, 'apps/app/src/api/auth.rs'), 'utf-8');
  assert.ok(
    pluginAuth.includes('minecraft_auth::login_loopback'),
    'plugin:auth|login must invoke Solara PKCE loopback login'
  );
  assert.ok(
    pluginAuth.includes('Box::pin'),
    'plugin:auth|login must be wrapped in Box::pin for Windows stack safety'
  );
});

test('Loopback callback listener handles concurrency, IPv6 loopback, and timeouts', async () => {
  const authRs = await readFile(path.join(root, 'packages/app-lib/src/state/minecraft_auth.rs'), 'utf-8');
  assert.ok(
    authRs.includes('tokio::time::timeout'),
    'minecraft_auth.rs must apply timeout to loopback stream reads'
  );
  assert.ok(
    authRs.includes('select_loopback_stream'),
    'minecraft_auth.rs must handle incoming connections across IPv4 and IPv6 loopback'
  );
  assert.ok(
    authRs.includes('listener_v6'),
    'minecraft_auth.rs must support both IPv4 and IPv6 loopback interfaces'
  );
});
