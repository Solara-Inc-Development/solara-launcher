import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function readText(rel) {
  return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

test('V-0.0.63 integrates full Modrinth App frontend and Rust core with all launcher features', () => {
  for (const rel of [
    'apps/app/src/main.rs',
    'apps/app/tauri.conf.json',
    'apps/app-frontend/src/App.vue',
    'apps/app-frontend/src/pages/Skins.vue',
    'apps/app-frontend/src/pages/Index.vue',
    'apps/app-frontend/src/pages/Screenshots.vue',
    'apps/app-frontend/src/pages/project/Index.vue',
    'apps/app-frontend/src/platform/adapters/browse/browse-page.vue',
    'apps/app-frontend/src/pages/instance/layout.vue',
    'apps/app-frontend/src/pages/instance/worlds/index.vue',
    'apps/app-frontend/src/pages/instance/content/index.vue',
    'packages/app-lib/src/lib.rs',
    'packages/ui/src/components/skin/SkinPreviewRenderer.vue',
    'packages/ui/src/components/flows/creation-flow-modal/index.vue',
  ]) {
    assert.equal(fs.existsSync(path.join(ROOT, rel)), true, `Required Modrinth App module must exist: ${rel}`);
  }
});

test('V-0.0.63 preserves Solara GitHub Releases CDN + Minisign auto-updater and UI controls', () => {
  const appUpdatesRs = readText('apps/app/src/app_updates.rs');
  const updatePolicyRs = readText('apps/app/src/update_policy.rs');
  const mainRs = readText('apps/app/src/main.rs');
  const newUpdateModal = readText('apps/app-frontend/src/features/updates/new-update-modal/index.vue');
  const updateButton = readText('apps/app-frontend/src/components/ui/app-update-button/index.vue');

  assert.match(updatePolicyRs, /https:\/\/github\.com\/Solara-Inc-Development\/solara-launcher\/releases\/latest\/download\/latest\.json/);
  assert.match(appUpdatesRs, /PublicKey::decode/);
  assert.match(appUpdatesRs, /Signature::decode/);
  assert.match(appUpdatesRs, /app-update-status/);
  assert.doesNotMatch(mainRs, /tauri_plugin_updater/);

  assert.match(newUpdateModal, /New Version Available/);
  assert.match(newUpdateModal, /Update Now/);
  assert.match(newUpdateModal, /I'll do it later/);
  assert.match(updateButton, /Restart to Update/);
});

test('V-0.0.63 enforces Windows 8 MiB stack, Box::pin async command wrappers, and redacted crash.log panic hook', () => {
  const buildRs = readText('apps/app/build.rs');
  const mainRs = readText('apps/app/src/main.rs');
  const authApiRs = readText('apps/app/src/api/auth.rs');
  const installApiRs = readText('apps/app/src/api/install.rs');
  const instanceApiRs = readText('apps/app/src/api/instance.rs');

  assert.match(buildRs, /\/STACK:8388608/);
  assert.match(mainRs, /install_panic_hook/);
  assert.match(mainRs, /redact_panic_text/);
  assert.match(authApiRs, /Box::pin/);
  assert.match(installApiRs, /Box::pin/);
  assert.match(instanceApiRs, /Box::pin/);
});

test('V-0.0.63 removes all ads and collapses 300x250 ad placeholder space in sidebar and settings', () => {
  const sidebar = readText('apps/app-frontend/src/app/shell/sidebar/index.vue');
  const troubleshooting = readText('apps/app-frontend/src/features/settings/system/troubleshooting-settings.vue');
  const interScss = readText('packages/assets/styles/inter.scss');

  assert.doesNotMatch(sidebar, /Promotion/);
  assert.doesNotMatch(sidebar, /promotionPaddingStyle/);
  assert.doesNotMatch(sidebar, /250px/);
  assert.doesNotMatch(sidebar, /ad-parent/);
  assert.doesNotMatch(troubleshooting, /copy-ad-details/);
  assert.doesNotMatch(interScss, /cdn\.modrinth\.com\/fonts/);
});
