import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const ROOT = path.resolve(import.meta.dirname, '..');

function readText(rel) {
  return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

test('SkinsView Cape Studio has 6 local preview capes, status strip, auto cape pose, and 64x32 PNG validation', () => {
  const skins = readText('apps/desktop/ui/views/SkinsView.vue');
  const capeCount = (skins.match(/id:\s*'preview-/g) || []).length;
  assert.equal(capeCount, 6, `Expected 6 built-in local preview capes, found ${capeCount}`);
  assert.match(skins, /sol-cape-status-strip/);
  assert.match(skins, /sol-cape-preview-badge/);
  assert.match(skins, /Preview only — never claims to equip on Minecraft/);
  assert.match(skins, /onUploadPreviewCape/);
  assert.match(skins, /probe\.width !== 64 \|\| probe\.height !== 32/);
  assert.match(skins, /equippingCapeId/);
  assert.match(skins, /viewerPose\.value = 'cape'/);
});

test('launcher.css defines frosted OLED Cape Studio status strip and preview container', () => {
  const css = readText('apps/desktop/launcher.css');
  assert.ok(css.includes('.sol-cape-status-strip'));
  assert.ok(css.includes('.sol-cape-preview'));
});
