import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const ROOT = path.resolve(import.meta.dirname, '..');

test('SkinsView extracts crisp cape crops for live Mojang capes and launcher.css uses pixelated rendering', () => {
  const skinsView = fs.readFileSync(path.join(ROOT, 'apps/desktop/ui/views/SkinsView.vue'), 'utf8');
  const css = fs.readFileSync(path.join(ROOT, 'apps/desktop/launcher.css'), 'utf8');

  assert.match(skinsView, /function extractCapeThumb\(/);
  assert.match(skinsView, /capeThumbs\[cape\.id\] \|\| cape\.dataUrl/);
  assert.match(skinsView, /if \(img\.width >= 64 && img\.height >= 32\)/);
  assert.match(css, /\.sol-cape-preview\s*\{[\s\S]*image-rendering:\s*pixelated/);
  assert.match(css, /\.sol-splash-progress-fill\s*\{[\s\S]*transition:\s*width 140ms/);
});

test('pure Minecraft launcher redesign V-0.0.42 through V-0.0.60 preserves bordered logo, OLED tokens, and GitHub Releases updater', () => {
  const tokens = fs.readFileSync(path.join(ROOT, 'apps/desktop/tokens.css'), 'utf8');
  const releases = JSON.parse(fs.readFileSync(path.join(ROOT, 'builds/releases.json'), 'utf8'));

  assert.match(tokens, /--bg-main:\s*#000000/);
  assert.match(tokens, /--accent:\s*#3B82F6/);
  assert.ok(fs.existsSync(path.join(ROOT, 'public/logo.svg')));
  assert.ok(fs.existsSync(path.join(ROOT, 'apps/desktop/icons/icon.ico')));
  assert.ok(Array.isArray(releases.releases) && releases.releases.length >= 18);
});
