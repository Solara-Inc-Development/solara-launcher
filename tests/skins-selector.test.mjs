import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const ROOT = path.resolve(import.meta.dirname, '..');

function readText(rel) {
  return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

test('SkinsView skin selector provides 6 starter skins, search, variant filters, and PNG dimension guard', () => {
  const skins = readText('apps/desktop/ui/views/SkinsView.vue');
  const starterCount = (skins.match(/id:\s*'starter-/g) || []).length;
  assert.equal(starterCount, 6, `Expected 6 built-in starter skins, found ${starterCount}`);
  assert.match(skins, /skinSearch/);
  assert.match(skins, /skinVariantFilter/);
  assert.match(skins, /filteredSavedSkins/);
  assert.match(skins, /filteredStarterSkins/);
  assert.match(skins, /probe\.width === 64 && \(probe\.height === 64 \|\| probe\.height === 32\)/);
  assert.match(skins, /duplicateSkin/);
  assert.match(skins, /createBlankSkin/);
  assert.match(skins, /quickEquipSkin/);
  assert.match(skins, /syncAccountSkinToLibrary/);
  assert.match(skins, /<SolaraModal[\s\S]*Boolean\(deleteConfirmSkin\)/);
  assert.match(skins, /confirmDeleteSkin/);
});

test('launcher.css includes frosted OLED Skin Selector toolbar, account strip, and head badge rules', () => {
  const css = readText('apps/desktop/launcher.css');
  for (const token of [
    '.sol-skin-account-strip',
    '.sol-skin-toolbar',
    '.sol-skin-search',
    '.sol-skin-filter-btn',
    '.sol-skin-thumb-wrap',
    '.sol-skin-head-chip',
    '.sol-skin-actions',
  ]) {
    assert.ok(css.includes(token), `Missing skin selector CSS rule: ${token}`);
  }
});
