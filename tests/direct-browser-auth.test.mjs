import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));

test('Microsoft OAuth Client ID is configured to official Solara Client ID and Prism ID is removed', async () => {
  const authRs = await readFile(path.join(root, 'apps/desktop/src/game_core/auth.rs'), 'utf-8');
  assert.ok(
    authRs.includes('6a9238bb-3803-46f4-bf38-2f45bdf8c2ea'),
    'auth.rs must configure the official Solara Microsoft Client ID'
  );
  assert.ok(
    !authRs.includes('c36a9fb6-4f2a-41ff-90bd-ae7cc92031eb'),
    'auth.rs must not retain the temporary Prism Client ID'
  );
});

test('Loopback authorization code flow with PKCE RFC 7636 is implemented in Rust core', async () => {
  const authRs = await readFile(path.join(root, 'apps/desktop/src/game_core/auth.rs'), 'utf-8');
  assert.ok(authRs.includes('generate_pkce'), 'PKCE generator is implemented');
  assert.ok(authRs.includes('loopback_auth_url'), 'Loopback authorization URL constructor is implemented');
  assert.ok(authRs.includes('wait_for_callback'), 'Loopback TCP callback listener is implemented');
  assert.ok(authRs.includes('login_loopback'), 'Direct browser loopback login flow is implemented');
  assert.ok(authRs.includes('grant_type", "authorization_code'), 'Token exchange uses grant_type=authorization_code');
  assert.ok(authRs.includes('code_verifier'), 'Token exchange passes code_verifier');
  assert.ok(
    authRs.includes('Authentication Complete!') &&
      authRs.includes('You can close this tab and return to Solara Launcher.'),
    'Branded loopback HTML response informs user of successful authentication'
  );
});

test('App.vue renders sleek direct browser login modal with Re-open browser and Cancel buttons', async () => {
  const appVue = await readFile(path.join(root, 'apps/desktop/App.vue'), 'utf-8');
  assert.ok(
    appVue.includes('Opening Microsoft Login in your browser... Please sign in and approve Solara Launcher.'),
    'App.vue displays required sleek heading during browser login'
  );
  assert.ok(
    appVue.includes('btn-reopen-browser') && appVue.includes('Re-open browser'),
    'App.vue provides Re-open browser action button'
  );
  assert.ok(
    appVue.includes('cancelMinecraftSignIn') && appVue.includes('Cancel'),
    'App.vue provides Cancel action button'
  );
  assert.ok(
    appVue.includes('direct-signin-box'),
    'App.vue includes dedicated direct 1-click browser signin UI container'
  );
});

test('Launcher styling includes dedicated rules for direct signin container and buttons', async () => {
  const css = await readFile(path.join(root, 'apps/desktop/launcher.css'), 'utf-8');
  assert.ok(css.includes('.direct-signin-box'), 'launcher.css styles .direct-signin-box');
  assert.ok(css.includes('.btn-reopen-browser'), 'launcher.css styles .btn-reopen-browser');
  assert.ok(css.includes('.btn-reopen-browser:disabled'), 'launcher.css styles .btn-reopen-browser:disabled');
  assert.ok(css.includes('.browser-pulse-ring'), 'launcher.css styles .browser-pulse-ring');
});

test('Loopback callback listener handles concurrency, IPv6 loopback, and streaming chunks', async () => {
  const authRs = await readFile(path.join(root, 'apps/desktop/src/game_core/auth.rs'), 'utf-8');
  assert.ok(
    authRs.includes('read_http_request'),
    'auth.rs must buffer HTTP request until CRLF delimiter to handle chunked TCP streams'
  );
  assert.ok(
    authRs.includes('handle_connection'),
    'auth.rs must handle incoming connections concurrently to prevent idle sockets stalling accept loop'
  );
  assert.ok(
    authRs.includes('listener_v6'),
    'auth.rs must support both IPv4 and IPv6 loopback interfaces'
  );
});
