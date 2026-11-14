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

test('pure Minecraft launcher removals: legacy hub, server hosting, and SSO identity modules are deleted', () => {
  for (const rel of [
    'apps/desktop/src/identity.rs',
    'apps/desktop/src/resource_policy.rs',
    'apps/desktop/ui/views/SolaraHubView.vue',
    'packages/local-server',
    'packages/local-server/Cargo.toml',
  ]) {
    assert.equal(fs.existsSync(path.join(ROOT, rel)), false, `${rel} must remain deleted`);
  }

  const rootCargo = fs.readFileSync(path.join(ROOT, 'Cargo.toml'), 'utf8');
  assert.equal(rootCargo.includes('local-server'), false, 'Cargo.toml must not reference local-server');

  const desktopCargo = fs.readFileSync(path.join(ROOT, 'apps/desktop/Cargo.toml'), 'utf8');
  assert.equal(desktopCargo.includes('solara-local-server'), false, 'desktop Cargo.toml must not depend on solara-local-server');
});

test('pure Minecraft launcher UI & client contain zero server hosting, Solara ID SSO, or play.solaranetwork.in promos', () => {
  const files = [
    ...walkFiles(path.join(ROOT, 'apps/desktop'), ['.vue', '.ts', '.css', '.rs']),
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
    'sol-home-side',
    'solaraHubEnabled',
    'solaraBootstrap',
    'solaraProfile',
    'LocalServerRecord',
    'Sign in with Solara ID',
  ];

  for (const file of files) {
    const src = fs.readFileSync(file, 'utf8');
    const rel = path.relative(ROOT, file);
    for (const token of banned) {
      assert.equal(src.includes(token), false, `${rel} must not contain removed reference "${token}"`);
    }
  }
});
