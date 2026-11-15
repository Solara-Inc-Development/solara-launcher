import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '..');

test('website App.vue renders pure OLED launcher with bordered logo, colorful feature tones, and clean copy', () => {
  const app = fs.readFileSync(path.join(ROOT, 'apps/website/App.vue'), 'utf8');

  assert.match(app, /src="\/logo\.svg"/);
  assert.match(app, /src="\/screenshots\/dashboard\.png"/);
  assert.match(app, /download="solaralauncher\.exe"/);
  assert.match(app, /--accent':\s*'#3b82f6'/);
  for (const tone of ['cobalt', 'emerald', 'purple', 'amber', 'sky', 'rose']) {
    assert.ok(app.includes(`tone: '${tone}'`), `Expected feature tone ${tone}`);
  }
  for (const forbidden of ['play.solaranetwork.in', 'Server Hosting', 'Solara ID', 'Solara Master', 'Modrinth']) {
    assert.ok(!app.includes(forbidden), `Website must not reference ${forbidden}`);
  }
});

test('website.css implements true OLED #000000, frosted backdrop-filter glass, colorful feature accents, and reduced-motion guards', () => {
  const css = fs.readFileSync(path.join(ROOT, 'apps/website/website.css'), 'utf8');

  assert.match(css, /background:\s*#000000/);
  assert.match(css, /backdrop-filter:\s*var\(--web-blur\)/);
  assert.match(css, /\.web-header\s*\{[^}]*position:\s*sticky/s);
  assert.match(css, /\.feature\s*\{[^}]*backdrop-filter:\s*var\(--web-blur-sm\)/s);
  assert.match(css, /\.download-panel\s*\{[^}]*backdrop-filter:\s*var\(--web-blur\)/s);
  assert.match(css, /\.feature--emerald/);
  assert.match(css, /\.feature--purple/);
  assert.match(css, /\.website\[data-reduced-motion="true"\]/);
  assert.match(css, /@media \(prefers-reduced-motion: reduce\)/);
});
