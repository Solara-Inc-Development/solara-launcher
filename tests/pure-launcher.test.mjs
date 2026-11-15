import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function walkFiles(dir, exts, out = []) {
  if (!fs.existsSync(dir)) return out;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walkFiles(full, exts, out);
    } else if (exts.some((ext) => entry.name.endsWith(ext))) {
      out.push(full);
    }
  }
  return out;
}

test('pure Minecraft launcher removals: legacy hub, server hosting, ads, mr_auth, and SSO identity modules are deleted', () => {
  for (const rel of [
    'apps/desktop',
    'apps/app/src/api/ads.rs',
    'apps/app/src/api/mr_auth.rs',
    'apps/app/src/api/friends.rs',
    'apps/app/capabilities/ads.json',
    'apps/app-frontend/src/app/shell/sidebar/promotion.vue',
    'apps/app-frontend/src/features/hosting',
    'apps/app-frontend/src/features/campaigns',
    'apps/app-frontend/src/pages/instance/share',
    'packages/local-server',
    'packages/local-server/Cargo.toml',
  ]) {
    assert.equal(fs.existsSync(path.join(ROOT, rel)), false, `${rel} must remain deleted`);
  }

  const rootCargo = fs.readFileSync(path.join(ROOT, 'Cargo.toml'), 'utf8');
  assert.equal(rootCargo.includes('local-server'), false, 'Cargo.toml must not reference local-server');
  assert.equal(rootCargo.includes('sentry'), false, 'Cargo.toml must not reference sentry');

  const appCargo = fs.readFileSync(path.join(ROOT, 'apps/app/Cargo.toml'), 'utf8');
  assert.equal(appCargo.includes('solara-local-server'), false, 'app Cargo.toml must not depend on solara-local-server');
  assert.equal(appCargo.includes('tauri-plugin-updater'), false, 'app Cargo.toml must not depend on tauri-plugin-updater');
});

test('pure Minecraft launcher UI & client contain zero server hosting, Solara ID SSO, or play.solaranetwork.in promos', () => {
  const files = [
    ...walkFiles(path.join(ROOT, 'apps/app/src'), ['.rs']),
    ...walkFiles(path.join(ROOT, 'apps/app-frontend/src'), ['.vue', '.ts', '.js', '.css']),
    ...walkFiles(path.join(ROOT, 'apps/website'), ['.vue', '.ts', '.css']),
    path.join(ROOT, 'packages/shared/client.ts'),
  ];

  const banned = [
    'play.solaranetwork.in',
    'auth.solara-inc.in',
    'solara_sign_in',
    'solara_sign_out',
    'solara_profile',
    'solara_request',
    'create_local_server',
    'local_server_action',
    'SolaraHubView',
    'solaraHubEnabled',
    'solaraBootstrap',
    'solaraProfile',
    'LocalServerRecord',
    'Sign in with Solara ID',
    'hasPride26Badge',
  ];

  for (const file of files) {
    const src = fs.readFileSync(file, 'utf8');
    const rel = path.relative(ROOT, file);
    for (const token of banned) {
      assert.equal(src.includes(token), false, `${rel} must not contain removed reference "${token}"`);
    }
  }
});
