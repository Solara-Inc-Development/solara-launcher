import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

const ROOT = path.resolve(import.meta.dirname, '..');

test('Settings, Onboarding, Search, Tasks, and Accounts wire frosted glass tokens and colorful accents', () => {
  const onboarding = fs.readFileSync(path.join(ROOT, 'apps/desktop/ui/views/OnboardingView.vue'), 'utf8');
  const search = fs.readFileSync(path.join(ROOT, 'apps/desktop/ui/components/GlobalSearchModal.vue'), 'utf8');
  const tasks = fs.readFileSync(path.join(ROOT, 'apps/desktop/ui/views/TasksView.vue'), 'utf8');
  const settingsModal = fs.readFileSync(path.join(ROOT, 'apps/desktop/ui/components/GlobalSettingsModal.vue'), 'utf8');

  assert.match(onboarding, /sol-mini-feature-icon--cobalt/);
  assert.match(onboarding, /sol-mini-feature-icon--emerald/);
  assert.match(onboarding, /sol-mini-feature-icon--purple/);
  assert.match(search, /class="sol-kbd">Esc<\/kbd>/);
  assert.match(tasks, /`sol-task-card--\$\{job\.status\}`/);
  assert.match(settingsModal, /OLED Obsidian/);
});

test('launcher.css applies frosted OLED glass and reduced-motion guards across settings, onboarding, popover, search, tasks, and accounts', () => {
  const css = fs.readFileSync(path.join(ROOT, 'apps/desktop/launcher.css'), 'utf8');

  for (const selector of [
    '.sol-popover',
    '.sol-onboarding-card',
    '.sol-mini-feature',
    '.sol-signin-box',
    '.sol-account-row',
    '.sol-search-input-wrap',
    '.sol-seg-group',
    '.sol-chip',
    '.sol-settings-nav-btn',
    '.sol-task-card--running',
    '.sol-task-card--completed',
  ]) {
    assert.ok(css.includes(selector), `Expected ${selector} in launcher.css`);
  }

  assert.match(css, /\.sol-popover\s*\{[^}]*background:\s*var\(--glass-surface-strong\)/s);
  assert.match(css, /\.sol-onboarding-card\s*\{[^}]*backdrop-filter:\s*var\(--glass-blur\)/s);
  assert.match(css, /\.sol-seg-group\s*\{[^}]*background:\s*var\(--glass-surface\)/s);
  assert.match(css, /\.sol-chip\s*\{[^}]*background:\s*var\(--glass-btn\)/s);
  assert.match(css, /:root\[data-reduced-motion='true'\][\s\S]*\.sol-task-card:hover/s);
});
