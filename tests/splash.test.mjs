import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

test('SplashScreen renders bordered logo, real startup progressbar, stage chips, and retry error state', () => {
  const splash = fs.readFileSync(path.join(ROOT, 'apps/desktop/ui/components/SplashScreen.vue'), 'utf8');
  assert.match(splash, /<SolaraLogo\s+:size="88"/);
  assert.match(splash, /role="progressbar"/);
  assert.match(splash, /store\.initStages/);
  assert.match(splash, /store\.initPhase/);
  assert.match(splash, /Retry Startup/);

  const state = fs.readFileSync(path.join(ROOT, 'apps/desktop/ui/state.ts'), 'utf8');
  for (const id of ["'core'", "'prefs'", "'instances'", "'updates'"]) {
    assert.equal(state.includes(id), true, `initializeEngine must drive startup stage ${id}`);
  }
});

test('launcher.css splash rules use true OLED #000000 backdrop, frosted glass card, and reduced-motion guard', () => {
  const css = fs.readFileSync(path.join(ROOT, 'apps/desktop/launcher.css'), 'utf8');
  assert.match(css, /\.sol-splash\s*\{[\s\S]*background:\s*#000000/);
  assert.match(css, /\.sol-splash-card\s*\{[\s\S]*backdrop-filter:\s*blur/);
  assert.match(css, /data-reduced-motion='true'\]\s*\.sol-splash-aura/);
});
