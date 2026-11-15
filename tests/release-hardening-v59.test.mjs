import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { getPartBoxSpec, getUvRegionBox } from '../apps/desktop/ui/skinUv.mjs';
import { floodFillRegion, mirrorPixelX, normalizeHexColor, pushRecentColor } from '../apps/desktop/ui/skinTools.mjs';

const ROOT = path.resolve(import.meta.dirname, '..');

function walkVueFiles(dir) {
  const out = [];
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) out.push(...walkVueFiles(p));
    else if (ent.name.endsWith('.vue')) out.push(p);
  }
  return out;
}

test('user-facing desktop and website Vue copy never exposes Modrinth brand strings', () => {
  const files = [
    ...walkVueFiles(path.join(ROOT, 'apps/desktop/ui')),
    path.join(ROOT, 'apps/website/App.vue'),
  ];
  for (const file of files) {
    const lines = fs.readFileSync(file, 'utf8').split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (/api\.modrinth\.com|cdn\.modrinth\.com|https:\/\/modrinth\.com|ModrinthHit|ModrinthProject|ModrinthVersion|modrinth_/.test(line)) {
        continue;
      }
      assert.ok(
        !/modrinth/i.test(line),
        `Found user-facing Modrinth copy in ${path.relative(ROOT, file)}:${i + 1}: ${line.trim()}`,
      );
    }
  }
});

test('SkinsView and extras.rs implement live Mojang sync, direct live upload, and honest signed-out/error states', () => {
  const skinsView = fs.readFileSync(path.join(ROOT, 'apps/desktop/ui/views/SkinsView.vue'), 'utf8');
  const extrasRs = fs.readFileSync(path.join(ROOT, 'apps/desktop/src/game_core/extras.rs'), 'utf8');

  assert.match(skinsView, /Apply Live to Mojang/);
  assert.match(skinsView, /Sync from Mojang/);
  assert.match(skinsView, /Live on Mojang/);
  assert.match(skinsView, /Optional Starter Templates/);
  assert.match(skinsView, /Sign in to sync &amp; update live on Mojang/);
  assert.match(skinsView, /watch\(\s*\(\) => selectedAccount\.value\?\.id/);
  assert.match(skinsView, /async function applySkinToAccount\(\): Promise<boolean>/);
  assert.match(skinsView, /const ok = await applySkinToAccount\(\);/);
  assert.doesNotMatch(skinsView, /notify\('info', `Selected \$\{label\}`/);

  assert.match(extrasRs, /validate_and_normalize_texture_url/);
  assert.match(extrasRs, /sanitize_mojang_error_message/);
  assert.match(extrasRs, /set_minecraft_cape_with_base/);
  assert.match(extrasRs, /upload_minecraft_skin_with_base/);
});

test('skinUv and skinTools enforce boundary guards, invalid hex normalization, and slim arm symmetry', () => {
  assert.throws(() => getUvRegionBox({ part: 'unknown', face: 'front', layer: 'base', variant: 'classic' }));
  assert.throws(() => getUvRegionBox({ part: 'head', face: 'sideways', layer: 'base', variant: 'classic' }));
  assert.deepEqual(getPartBoxSpec('arm-l', 'overlay', 'slim'), { u: 48, v: 48, w: 3, h: 12, d: 4 });
  assert.deepEqual(getUvRegionBox({ part: 'full', face: 'front', layer: 'base', variant: 'classic' }), { x: 0, y: 0, w: 64, h: 64 });

  assert.equal(normalizeHexColor('not-a-color'), null);
  assert.equal(normalizeHexColor(' #60a5fa '), '#60A5FA');

  const slimFront = getUvRegionBox({ part: 'arm-r', face: 'front', layer: 'base', variant: 'slim' });
  assert.equal(slimFront.w, 3);
  assert.equal(mirrorPixelX(slimFront.x, slimFront), slimFront.x + 2);
  assert.equal(mirrorPixelX(slimFront.x + 1, slimFront), slimFront.x + 1);

  const recent = pushRecentColor(['#111111', '#222222'], '#111111', 2);
  assert.deepEqual(recent, ['#111111', '#222222']);

  // Flood fill stays strictly contained within region bounds even on out-of-bounds start
  const data = new Uint8ClampedArray(64 * 64 * 4);
  assert.equal(floodFillRegion(data, 64, 0, 0, { x: 8, y: 8, w: 8, h: 8 }, '#FF0000'), 0);
});
