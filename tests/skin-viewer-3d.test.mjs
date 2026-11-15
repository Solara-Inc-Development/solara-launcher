import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const ROOT = path.resolve(import.meta.dirname, '..');

function readText(rel) {
  return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

test('SkinViewer3D implements directional normals lighting, Idle/Walk/Run/Cape poses, inertia, flip view, and GPU cleanup', () => {
  const viewer = readText('apps/desktop/ui/components/SkinViewer3D.vue');
  assert.match(viewer, /export type SkinViewerPose = 'idle' \| 'walk' \| 'run' \| 'cape'/);
  assert.match(viewer, /attribute vec3 aNorm/);
  assert.match(viewer, /uniform mat4 uModel/);
  assert.match(viewer, /vNormal/);
  assert.match(viewer, /pose === 'run'/);
  assert.match(viewer, /pose === 'idle'/);
  assert.match(viewer, /pose === 'cape'/);
  assert.match(viewer, /velYaw/);
  assert.match(viewer, /flipView/);
  assert.match(viewer, /aria-label="Flip front and back view"/);
  assert.match(viewer, /releaseGpu\?\.\(\)/);
});

test('SkinsView wires pose selector pills to SkinViewer3D and launcher.css styles .sol-pose-pills', () => {
  const skins = readText('apps/desktop/ui/views/SkinsView.vue');
  const css = readText('apps/desktop/launcher.css');
  assert.match(skins, /:pose="viewerPose"/);
  assert.match(skins, /\['idle', 'walk', 'run', 'cape'\]/);
  assert.ok(css.includes('.sol-pose-pills'));
  assert.ok(css.includes('.sol-pose-btn.is-active'));
});
