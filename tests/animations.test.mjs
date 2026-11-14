import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const ROOT = path.resolve(import.meta.dirname, '..');

function readText(rel) {
  return fs.readFileSync(path.join(ROOT, rel), 'utf8');
}

test('App.vue, SolaraModal.vue, and SolaraToasts.vue wire Vue Transitions for views, modals, and toasts', () => {
  const app = readText('apps/desktop/App.vue');
  const modal = readText('apps/desktop/ui/components/SolaraModal.vue');
  const toasts = readText('apps/desktop/ui/components/SolaraToasts.vue');

  assert.match(app, /<Transition v-else name="sol-view-rise" mode="out-in">/);
  assert.match(modal, /<Transition name="sol-modal-fade">/);
  assert.match(toasts, /<TransitionGroup name="sol-toast">/);
});

test('launcher.css defines view, modal, toast, staggered card, progress, and reducedMotion rules', () => {
  const css = readText('apps/desktop/launcher.css');

  for (const token of [
    'sol-card-rise',
    '.sol-view-rise-enter-active',
    '.sol-modal-fade-enter-active',
    '.sol-toast-enter-active',
    '.sol-progress-bar',
    ':root[data-reduced-motion=\'true\']',
    '@media (prefers-reduced-motion: reduce)',
  ]) {
    assert.ok(css.includes(token), `Missing animation/reducedMotion rule: ${token}`);
  }
});
