import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const ROOT = path.resolve(import.meta.dirname, '..');

function readText(rel) {
  return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

test('LibraryView uses loader dots, frosted Quick Profile Sheet, and SolaraModal delete confirmation', () => {
  const lib = readText('apps/desktop/ui/views/LibraryView.vue');
  assert.match(lib, /sol-loader-dot--\$\{inst\.loader\}/);
  assert.doesNotMatch(lib, /'sol-loader-dot',\s*`is-\$\{inst\.loader\}`/);
  assert.match(lib, /variant="sheet"/);
  assert.match(lib, /openQuickSheet/);
  assert.match(lib, /<SolaraModal[\s\S]*Boolean\(deleteConfirm\)/);
  assert.doesNotMatch(lib, /sol-overlay/);
});

test('InstanceDetailView uses frosted loader-tinted header, loader badge, and SolaraModal lightbox', () => {
  const detail = readText('apps/desktop/ui/views/InstanceDetailView.vue');
  assert.match(detail, /sol-inst-header--\$\{instance\.loader\}/);
  assert.match(detail, /sol-badge--\$\{instance\.loader\}/);
  assert.match(detail, /<SolaraModal[\s\S]*Boolean\(lightboxShot\)/);
  assert.doesNotMatch(detail, /sol-overlay/);
});

test('launcher.css applies frosted glass and loader accents to Library menus, rows, and instance header', () => {
  const css = readText('apps/desktop/launcher.css');
  for (const token of [
    '.sol-inst-header--fabric',
    '.sol-inst-header--forge',
    '.sol-inst-header--neoforge',
    '.sol-inst-header--quilt',
    '.sol-inst-header--vanilla',
  ]) {
    assert.ok(css.includes(token), `Missing instance header loader rule: ${token}`);
  }
});
