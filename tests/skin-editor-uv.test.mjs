import assert from 'node:assert/strict';
import test from 'node:test';
import { getUvRegionBox, SKIN_BODY_PARTS, SKIN_FACES } from '../apps/desktop/ui/skinUv.mjs';

test('skinUv maps all 6 body parts x 6 faces x 2 layers within 64x64 texture bounds', () => {
  const parts = SKIN_BODY_PARTS.filter((p) => p.id !== 'full').map((p) => p.id);
  assert.deepEqual(parts, ['head', 'torso', 'arm-r', 'arm-l', 'leg-r', 'leg-l']);
  assert.equal(SKIN_FACES.length, 6);

  for (const variant of ['classic', 'slim']) {
    for (const layer of ['base', 'overlay']) {
      for (const part of parts) {
        for (const face of SKIN_FACES.map((f) => f.id)) {
          const box = getUvRegionBox({ part, face, layer, variant });
          assert.ok(box.w > 0 && box.h > 0, `Empty box for ${part}/${face}`);
          assert.ok(box.x >= 0 && box.y >= 0, `Negative coords for ${part}/${face}`);
          assert.ok(
            box.x + box.w <= 64 && box.y + box.h <= 64,
            `Out-of-bounds UV box for ${part}/${face}/${layer}/${variant}: ${JSON.stringify(box)}`,
          );
        }
      }
    }
  }
});

test('skinUv applies official 3px slim arm width and exact Minecraft head/arm/leg coordinates', () => {
  assert.deepEqual(getUvRegionBox({ part: 'head', face: 'front', layer: 'base', variant: 'classic' }), { x: 8, y: 8, w: 8, h: 8 });
  assert.deepEqual(getUvRegionBox({ part: 'head', face: 'top', layer: 'overlay', variant: 'classic' }), { x: 40, y: 0, w: 8, h: 8 });
  assert.deepEqual(getUvRegionBox({ part: 'arm-l', face: 'front', layer: 'base', variant: 'slim' }), { x: 36, y: 52, w: 3, h: 12 });
  assert.deepEqual(getUvRegionBox({ part: 'arm-r', face: 'back', layer: 'overlay', variant: 'slim' }), { x: 51, y: 36, w: 3, h: 12 });
  assert.deepEqual(getUvRegionBox({ part: 'leg-l', face: 'bottom', layer: 'overlay', variant: 'classic' }), { x: 8, y: 48, w: 4, h: 4 });
  assert.deepEqual(getUvRegionBox({ part: 'full', face: 'front', layer: 'base', variant: 'classic' }), { x: 0, y: 0, w: 64, h: 64 });
});

test('skinUv throws RangeError on invalid part or cube face', () => {
  assert.throws(() => getUvRegionBox({ part: 'wings', face: 'front', layer: 'base', variant: 'classic' }), RangeError);
  assert.throws(() => getUvRegionBox({ part: 'head', face: 'diagonal', layer: 'base', variant: 'classic' }), RangeError);
});
