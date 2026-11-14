import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const ROOT = path.resolve(import.meta.dirname, '..');

function readText(rel) {
  return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

test('tokens.css and launcher.css implement true OLED #000000, frosted glass tokens, and loader color accents', () => {
  const tokens = readText('apps/desktop/tokens.css');
  const css = readText('apps/desktop/launcher.css');

  assert.match(tokens, /--bg-main:\s*#000000;/);
  assert.match(tokens, /--accent:\s*#3B82F6;/);
  assert.match(tokens, /--glass-surface:/);
  assert.match(tokens, /--glass-btn:/);
  assert.match(tokens, /--glass-blur:\s*blur\(/);
  for (const loader of ['fabric', 'forge', 'neoforge', 'quilt', 'vanilla']) {
    assert.match(tokens, new RegExp(`--loader-${loader}:`));
    assert.match(css, new RegExp(`\\.sol-badge--${loader}\\b`));
    assert.match(css, new RegExp(`\\.sol-loader-dot\\.is-${loader}\\b`));
  }

  assert.match(css, /\.sol-btn--primary\s*\{[\s\S]*linear-gradient\(/);
  assert.match(css, /\.sol-btn--secondary\s*\{[\s\S]*backdrop-filter:\s*var\(--glass-blur-sm\)/);
  assert.match(css, /\.sol-card\s*\{[\s\S]*backdrop-filter:\s*var\(--glass-blur\)/);
  assert.match(css, /\.sol-modal\s*\{[\s\S]*backdrop-filter:\s*blur\(/);
});

test('InstanceIcon, HomeView, and LibraryView expose per-loader color identities', () => {
  const icon = readText('apps/desktop/ui/components/InstanceIcon.vue');
  const home = readText('apps/desktop/ui/views/HomeView.vue');
  const lib = readText('apps/desktop/ui/views/LibraryView.vue');

  assert.match(icon, /#A78BFA/);
  assert.match(icon, /#F59E0B/);
  assert.match(icon, /#FB923C/);
  assert.match(home, /sol-badge--\$\{heroInstance\.loader\}/);
  assert.match(lib, /sol-loader-dot/);
});
