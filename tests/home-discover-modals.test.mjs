import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const ROOT = path.resolve(import.meta.dirname, '..');

function readText(rel) {
  return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

test('SolaraModal supports dialog and frosted slide-over sheet variants', () => {
  const modal = readText('apps/desktop/ui/components/SolaraModal.vue');
  const css = readText('apps/desktop/launcher.css');

  assert.match(modal, /variant\?:\s*'dialog'\s*\|\s*'sheet'/);
  assert.match(modal, /sol-modal-backdrop--sheet/);
  assert.match(modal, /sol-modal--sheet/);
  for (const token of [
    '.sol-modal-backdrop--sheet',
    '.sol-modal--sheet',
    '@keyframes sol-sheet-in',
    '.sol-sheet-tabs',
    '.sol-sheet-tab.is-active',
    '.sol-sheet-stats',
    '.sol-sheet-action-grid',
  ]) {
    assert.ok(css.includes(token), `Missing sheet modal rule: ${token}`);
  }
});

test('HomeView provides welcoming time-of-day header and in-place Quick Profile Sheet', () => {
  const home = readText('apps/desktop/ui/views/HomeView.vue');
  assert.match(home, /greetingLabel/);
  assert.match(home, /Good morning/);
  assert.match(home, /Good afternoon/);
  assert.match(home, /Good evening/);
  assert.match(home, /Quick Info/);
  assert.match(home, /variant="sheet"/);
  assert.match(home, /sol-sheet-action-grid/);
  assert.match(home, /sol-launch-hero--\$\{heroInstance\.loader\}/);
});

test('DiscoverView uses in-place tabbed sheet for modpack overview, versions, and gallery', () => {
  const discover = readText('apps/desktop/ui/views/DiscoverView.vue');
  assert.match(discover, /variant="sheet"/);
  assert.match(discover, /activeSheetTab/);
  assert.match(discover, /role="tablist"/);
  assert.match(discover, /Versions \(\{\{ versions\.length \}\}\)/);
  assert.match(discover, /Gallery \(\{\{ packGallery\.length \}\}\)/);
  assert.match(discover, /sol-pack-version-row/);
});
