const { chromium } = require('playwright');
const assert = require('node:assert/strict');
const http = require('node:http');
const path = require('node:path');
const { spawn } = require('node:child_process');

function checkServer(url) {
  return new Promise((resolve) => {
    const req = http.get(url, (res) => {
      res.resume();
      resolve(res.statusCode === 200);
    });
    req.on('error', () => resolve(false));
    req.setTimeout(500, () => {
      req.destroy();
      resolve(false);
    });
  });
}

async function main() {
  const base = process.env.SOLARA_UI_URL || 'http://127.0.0.1:1420';
  let devServer = null;

  const isUp = await checkServer(`${base}/desktop.html`);
  if (!isUp) {
    const viteBin = path.resolve(__dirname, '..', 'node_modules', 'vite', 'bin', 'vite.js');
    devServer = spawn(process.execPath, [viteBin, '--port', '1420', '--host', '127.0.0.1'], {
      stdio: 'ignore',
      detached: true,
    });
    const start = Date.now();
    let ready = false;
    while (Date.now() - start < 15000) {
      await new Promise((r) => setTimeout(r, 250));
      if (await checkServer(`${base}/desktop.html`)) {
        ready = true;
        break;
      }
    }
    if (!ready) {
      if (devServer && devServer.pid) {
        try {
          process.kill(-devServer.pid, 'SIGTERM');
        } catch {}
      }
      throw new Error(`Timed out waiting for Vite dev server on ${base}`);
    }
  }

  let browser;
  try {
    browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
    const context = await browser.newContext({
      viewport: { width: 1280, height: 720 },
    });

    await context.addInitScript(() => {
      window.__ACTION_LOG__ = [];
      const callbacks = new Map();
      const events = new Map();
      let callbackCounter = 1;

      window.__SOLARA_ACCOUNTS__ = [];
      window.__FINISH_SIGNIN__ = null;

      window.__EMIT_TAURI_EVENT__ = (eventName, payload) => {
        const handlerId = events.get(eventName);
        if (!handlerId) return false;
        const cb = callbacks.get(handlerId);
        if (typeof cb === 'function') {
          cb({ event: eventName, id: handlerId, payload });
          return true;
        }
        return false;
      };

      window.__TAURI_INTERNALS__ = {
        transformCallback: (cb) => {
          const id = callbackCounter++;
          callbacks.set(id, cb);
          return id;
        },
        unregisterCallback: (id) => callbacks.delete(id),
        metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
        invoke: async (command, args = {}) => {
          if (command === 'plugin:event|listen') {
            events.set(args.event, args.handler);
            return args.handler;
          }
          if (command === 'plugin:event|unlisten') return null;
          if (command === 'initialize_launcher') return null;
          if (command === 'launcher_settings') {
            return {
              memory: 4096,
              javaPath: 'auto',
              theme: 'obsidian-blue',
              uiScale: 100,
              reducedMotion: false,
              density: 'comfortable',
              onLaunch: 'keep',
              confirmLaunch: false,
              maxConcurrentDownloads: 4,
              defaultWidth: 854,
              defaultHeight: 480,
              fullscreen: false,
              jvmArgs: '',
              diagnosticsEnabled: true,
              onboardingCompleted: true,
              groups: [],
            };
          }
          if (command === 'launcher_snapshot') {
            return {
              instances: [],
              localServers: [],
              downloads: [],
              minecraftAccounts: window.__SOLARA_ACCOUNTS__,
            };
          }
          if (command === 'minecraft_versions') return [{ id: '1.21.4', type: 'release' }];
          if (command === 'loader_versions') return [];
          if (command === 'app_update_status') return { stage: 'idle', version: null, error: null };
          if (command === 'check_update') {
            window.__ACTION_LOG__.push('check_update');
            return { stage: 'current', version: null, error: null };
          }
          if (command === 'install_update') {
            window.__ACTION_LOG__.push('install_update');
            return { stage: 'installing', version: '0.0.99', error: null };
          }
          if (command === 'window_action') {
            window.__ACTION_LOG__.push(args.action);
            return null;
          }
          if (command === 'start_dragging') {
            window.__ACTION_LOG__.push('start_dragging');
            return null;
          }
          if (command === 'open_external') {
            window.__ACTION_LOG__.push(`open_external:${args.url}`);
            return null;
          }
          if (command === 'minecraft_sign_in') {
            window.__ACTION_LOG__.push('minecraft_sign_in');
            return new Promise((resolve, reject) => {
              window.__FINISH_SIGNIN__ = { resolve, reject };
            });
          }
          if (command === 'cancel_minecraft_sign_in') {
            window.__ACTION_LOG__.push('cancel_minecraft_sign_in');
            if (window.__FINISH_SIGNIN__) {
              window.__FINISH_SIGNIN__.reject(new Error('Sign-in cancelled'));
              window.__FINISH_SIGNIN__ = null;
            }
            return null;
          }
          if (command === 'select_minecraft_account') {
            window.__ACTION_LOG__.push(`select_minecraft_account:${args.id}`);
            window.__SOLARA_ACCOUNTS__ = window.__SOLARA_ACCOUNTS__.map((a) => ({
              ...a,
              selected: a.id === args.id,
              active: a.id === args.id,
            }));
            return null;
          }
          if (command === 'minecraft_sign_out') {
            window.__ACTION_LOG__.push(`minecraft_sign_out:${args.id}`);
            window.__SOLARA_ACCOUNTS__ = window.__SOLARA_ACCOUNTS__.filter((a) => a.id !== args.id);
            return null;
          }
          return null;
        },
      };
    });

    const page = await context.newPage();
    page.setDefaultTimeout(10000);

    const errors = [];
    page.on('pageerror', (err) => errors.push(`pageerror: ${err.message}`));
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(`console.error: ${msg.text()}`);
    });

    const png1x1 = Buffer.from(
      'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=',
      'base64',
    );
    await page.route('https://api.solara-inc.in/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ items: [] }),
      });
    });
    await page.route('https://mc-heads.net/**', async (route) => {
      await route.fulfill({ status: 200, contentType: 'image/png', body: png1x1 });
    });

    await page.goto(`${base}/desktop.html`, { waitUntil: 'domcontentloaded', timeout: 30000 });
    await page.waitForSelector('.sol-titlebar', { timeout: 25000 });
    await page.waitForSelector('.sol-rail', { timeout: 25000 });
    await page.waitForSelector('.sol-splash', { state: 'detached', timeout: 15000 });

    // 1. Verify titlebar drag region, no-drag buttons, and CSS app-region rules
    assert.equal(await page.locator('.sol-titlebar').getAttribute('data-tauri-drag-region'), '');
    const regions = await page.evaluate(() => {
      const tb = document.querySelector('.sol-titlebar');
      const winControls = document.querySelector('.sol-win-controls');
      const closeBtn = document.querySelector('.sol-win-btn--close');
      const getRegion = (el) => {
        if (!el) return null;
        const cs = getComputedStyle(el);
        return cs.getPropertyValue('-webkit-app-region') || cs.getPropertyValue('app-region');
      };
      return {
        titlebar: getRegion(tb),
        winControls: getRegion(winControls),
        closeBtn: getRegion(closeBtn),
      };
    });
    assert.equal(regions.winControls, 'no-drag', 'Window controls must enforce no-drag');
    assert.equal(regions.closeBtn, 'no-drag', 'Close button must enforce no-drag');

    // 2. Dragging empty titlebar area calls start_dragging; clicking controls does NOT call start_dragging
    const dragSpace = await page.locator('.sol-titlebar-center').boundingBox();
    assert.ok(dragSpace, 'Titlebar center exists');
    await page.mouse.move(dragSpace.x - 24, dragSpace.y + dragSpace.height / 2);
    await page.mouse.down();
    await page.mouse.move(dragSpace.x + 20, dragSpace.y + 40);
    await page.mouse.up();
    await page.waitForFunction(() => window.__ACTION_LOG__.includes('start_dragging'));

    // Double-click empty titlebar triggers maximize
    await page.mouse.dblclick(dragSpace.x - 24, dragSpace.y + dragSpace.height / 2);
    await page.waitForFunction(() => window.__ACTION_LOG__.includes('maximize'));

    const dragCountBeforeBtn = await page.evaluate(
      () => window.__ACTION_LOG__.filter((x) => x === 'start_dragging').length,
    );

    // 3. Window action buttons & rapid click deduplication (<180ms)
    await page.click('button[aria-label="Minimize window"]');
    await page.click('button[aria-label="Maximize or restore window"]');
    await page.click('button[aria-label="Close window"]');

    const dragCountAfterBtn = await page.evaluate(
      () => window.__ACTION_LOG__.filter((x) => x === 'start_dragging').length,
    );
    assert.equal(dragCountAfterBtn, dragCountBeforeBtn, 'Clicking window buttons must not trigger start_dragging');

    const closeBefore = await page.evaluate(
      () => window.__ACTION_LOG__.filter((x) => x === 'close').length,
    );
    await page.waitForTimeout(210);
    await page.evaluate(() => {
      const btn = document.querySelector('button[aria-label="Close window"]');
      btn?.click();
      btn?.click();
    });
    const closeAfter = await page.evaluate(
      () => window.__ACTION_LOG__.filter((x) => x === 'close').length,
    );
    assert.equal(closeAfter, closeBefore + 1, 'Rapid duplicate close clicks within 180ms deduplicate to 1 action');

    // 4. Permanently compact 84px icon-only left rail
    const railBox = await page.locator('.sol-rail').boundingBox();
    assert.ok(railBox && Math.round(railBox.width) === 84, `Expected 84px compact rail, got ${railBox?.width}`);
    assert.equal(await page.locator('button[title="Expand sidebar"], button[title="Collapse sidebar"]').count(), 0);

    // 5. Global browser contextmenu suppression across redesigned shell
    const ctxCheck = await page.evaluate(() => {
      const targets = [
        document.body,
        document.querySelector('.sol-shell'),
        document.querySelector('.sol-titlebar'),
        document.querySelector('.sol-rail'),
        document.querySelector('.sol-workspace'),
      ].filter(Boolean);
      let prevented = 0;
      for (const el of targets) {
        const ev = new MouseEvent('contextmenu', { bubbles: true, cancelable: true, buttons: 2 });
        el.dispatchEvent(ev);
        if (ev.defaultPrevented) prevented++;
      }
      return { total: targets.length, prevented };
    });
    assert.equal(ctxCheck.total, 5);
    assert.equal(ctxCheck.prevented, 5, 'All shell targets suppress default browser context menu');

    // 6. Direct-Browser Microsoft PKCE Sign-In Modal & Account Management
    await page.click('.sol-rail-item.sol-rail-account');
    await page.waitForSelector('h1:has-text("Minecraft Accounts")');
    await page.click('button:has-text("Sign in with Microsoft")');
    await page.waitForSelector('.direct-signin-box');
    await page.waitForFunction(() => window.__ACTION_LOG__.includes('minecraft_sign_in'));

    const reopenBtn = page.locator('.btn-reopen-browser');
    assert.equal(await reopenBtn.isDisabled(), true, 'Re-open button disabled until auth URL arrives');

    const authUrl = 'https://login.live.com/oauth20_authorize.srf?client_id=test&code_challenge=abc';
    const emitted = await page.evaluate((u) => window.__EMIT_TAURI_EVENT__('minecraft-login-code', { url: u }), authUrl);
    assert.equal(emitted, true, 'Tauri minecraft-login-code listener was registered');
    assert.equal(await reopenBtn.isEnabled(), true, 'Re-open button becomes enabled once auth URL arrives');

    await reopenBtn.click();
    await page.waitForFunction((u) => window.__ACTION_LOG__.includes(`open_external:${u}`), authUrl);

    await page.click('.sol-signin-actions button:has-text("Cancel")');
    await page.waitForFunction(() => window.__ACTION_LOG__.includes('cancel_minecraft_sign_in'));
    await page.waitForSelector('.direct-signin-box', { state: 'hidden' });

    // Complete PKCE sign-in and verify multi-account switch + sign-out
    await page.click('button:has-text("Sign in with Microsoft")');
    await page.waitForSelector('.direct-signin-box');
    await page.evaluate(() => {
      window.__SOLARA_ACCOUNTS__ = [
        { id: 'mc-1', username: 'SolaraBuilder', selected: true, name: 'SolaraBuilder', active: true, owned: true },
        { id: 'mc-2', username: 'AlexBuilder', selected: false, name: 'AlexBuilder', active: false, owned: true },
      ];
      window.__FINISH_SIGNIN__?.resolve(null);
      window.__FINISH_SIGNIN__ = null;
    });
    await page.waitForSelector('.sol-account-name:has-text("SolaraBuilder")');
    await page.click('button:has-text("Set Active")');
    await page.waitForFunction(() => window.__ACTION_LOG__.includes('select_minecraft_account:mc-2'));
    await page.locator('.sol-account-row:has-text("AlexBuilder") button:has-text("Sign Out")').click();
    await page.waitForFunction(() => window.__ACTION_LOG__.includes('minecraft_sign_out:mc-2'));

    assert.equal(errors.length, 0, `Browser errors occurred:\n${errors.join('\n')}`);
    console.log('PASS: Window controls, 84px compact rail, context menu guard, and Microsoft PKCE modal verified cleanly!');
  } finally {
    if (browser) await browser.close().catch(() => {});
    if (devServer && devServer.pid) {
      try {
        process.kill(-devServer.pid, 'SIGTERM');
      } catch {}
    }
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
