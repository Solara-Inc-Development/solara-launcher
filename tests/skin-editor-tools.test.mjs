import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import {
  floodFillRegion,
  mirrorPixelX,
  normalizeHexColor,
  parseHexRgb,
  pushRecentColor,
} from '../apps/desktop/ui/skinTools.mjs';

const ROOT = path.resolve(import.meta.dirname, '..');

test('skinTools validates hex colors and maintains deduplicated bounded recent history', () => {
  assert.equal(normalizeHexColor('3b82f6'), '#3B82F6');
  assert.equal(normalizeHexColor(' #a1b2c3 '), '#A1B2C3');
  assert.equal(normalizeHexColor('#GGGGGG'), null);
  assert.equal(normalizeHexColor('#123'), null);
  assert.deepEqual(parseHexRgb('#3b82f6'), [59, 130, 246]);
  assert.equal(parseHexRgb('bad'), null);

  let recent = ['#111111', '#222222'];
  recent = pushRecentColor(recent, '#111111', 3);
  assert.deepEqual(recent, ['#111111', '#222222']);
  recent = pushRecentColor(recent, '#333333', 3);
  recent = pushRecentColor(recent, '#444444', 3);
  assert.deepEqual(recent, ['#444444', '#333333', '#111111']);
  assert.deepEqual(pushRecentColor(recent, 'invalid', 3), recent);
});

test('skinTools mirrors across even 8px faces and odd 3px slim arms accurately', () => {
  const headBox = { x: 8, y: 8, w: 8, h: 8 };
  assert.equal(mirrorPixelX(8, headBox), 15);
  assert.equal(mirrorPixelX(15, headBox), 8);
  assert.equal(mirrorPixelX(11, headBox), 12);

  const slimArm = { x: 36, y: 52, w: 3, h: 12 };
  assert.equal(mirrorPixelX(36, slimArm), 38);
  assert.equal(mirrorPixelX(37, slimArm), 37);
  assert.equal(mirrorPixelX(38, slimArm), 36);
});

test('skinTools floodFillRegion stays strictly inside UV box bounds and ignores already-matching pixels', () => {
  const buf = new Uint8ClampedArray(64 * 64 * 4);
  // Paint a 1px vertical barrier splitting the box (8,8..16,16) in half at x=12
  for (let y = 8; y < 16; y++) {
    const i = (y * 64 + 12) * 4;
    buf[i] = 255;
    buf[i + 1] = 255;
    buf[i + 2] = 255;
    buf[i + 3] = 255;
  }
  const box = { x: 8, y: 8, w: 8, h: 8 };
  const changed = floodFillRegion(buf, 64, 9, 9, box, '#3B82F6');
  assert.equal(changed, 4 * 8, 'Left 4x8 half of head front (32 pixels) should fill without crossing barrier or box');
  // Out-of-box neighbor at x=7, y=9 must remain untouched transparent
  assert.equal(buf[(9 * 64 + 7) * 4 + 3], 0);
  // Re-running on already filled pixel changes 0 pixels
  assert.equal(floodFillRegion(buf, 64, 9, 9, box, '#3B82F6'), 0);
  // Out-of-bounds start returns 0
  assert.equal(floodFillRegion(buf, 64, 2, 2, box, '#3B82F6'), 0);
});

test('SkinEditor wires Grid toggle, Recent Colors strip, keyboard shortcuts, and 64x64/64x32 PNG import validation', () => {
  const src = fs.readFileSync(path.join(ROOT, 'apps/desktop/ui/components/SkinEditor.vue'), 'utf8');
  assert.match(src, /onEditorKeydown/);
  assert.match(src, /showGrid/);
  assert.match(src, /Recent Colors/);
  assert.match(src, /probe\.width !== 64 \|\| \(probe\.height !== 64 && probe\.height !== 32\)/);
});
