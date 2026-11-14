import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const ROOT = path.resolve(import.meta.dirname, '..');

function readText(rel) {
  return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

test('SidebarRail uses Teleported clamped tooltips, row-aligned indicators, and clean bottom controls', () => {
  const rail = readText('apps/desktop/ui/components/SidebarRail.vue');
  assert.match(rail, /<Teleport to="body">/);
  assert.match(rail, /clampTooltipY/);
  assert.match(rail, /window\.addEventListener\('blur', hideTip\)/);
  assert.match(rail, /window\.addEventListener\('keydown', onGlobalDismiss\)/);
  assert.match(rail, /sol-rail-row/);
  assert.match(rail, /sol-rail-indicator/);
  assert.match(rail, /sol-rail-divider/);
  assert.match(rail, /sol-rail-badge/);
  assert.match(rail, /sol-rail-account/);
  assert.ok(!rail.includes('sol-rail-avatar"'), 'Rebuilt rail must not reintroduce conflicting .sol-rail-avatar width override');
});

test('TitleBar and UpdatePromptModal render balanced 62px header, bordered logo, Restart to Update, and versioned prompt', () => {
  const tokens = readText('apps/desktop/tokens.css');
  const titlebar = readText('apps/desktop/ui/components/TitleBar.vue');
  const updateModal = readText('apps/desktop/ui/components/UpdatePromptModal.vue');
  const css = readText('apps/desktop/launcher.css');

  assert.match(tokens, /--titlebar-height:\s*62px;/);
  assert.match(titlebar, /<SolaraLogo :size="30" \/>/);
  assert.match(titlebar, /Restart to Update/);
  assert.match(titlebar, /sol-status-pill--update/);

  assert.match(updateModal, /New Version Available \$\{newPublicVersion\}/);
  assert.match(updateModal, /Update Now/);
  assert.match(updateModal, /I'll do it later/);

  assert.match(css, /\.sol-titlebar\s*\{[\s\S]*grid-template-columns:/);
  assert.match(css, /\.sol-rail-tooltip\s*\{[\s\S]*backdrop-filter:\s*blur\(/);
  assert.ok(!css.includes('left: -17px'), 'Stale negative indicator offsets must not remain in launcher.css');
  assert.ok(!css.includes('left: -23px'), 'Stale breakpoint indicator offsets must not remain in launcher.css');
});
