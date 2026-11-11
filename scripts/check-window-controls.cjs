const { chromium } = require('playwright');
const assert = require('node:assert/strict');
const http = require('node:http');
const { spawn } = require('node:child_process');

function checkServer(url) {
  return new Promise(resolve => {
    const req = http.get(url, res => {
      resolve(res.statusCode === 200);
    });
    req.on('error', () => resolve(false));
    req.setTimeout(500, () => {
      req.destroy();
      resolve(false);
    });
  });
}

(async () => {
  const base = process.env.SOLARA_UI_URL || 'http://127.0.0.1:1420';
  let devServer = null;

  const isUp = await checkServer(`${base}/desktop.html`);
  if (!isUp) {
    devServer = spawn('npx', ['vite', '--port', '1420', '--host', '127.0.0.1'], {
      stdio: 'ignore',
      detached: false
    });
    const start = Date.now();
    let ready = false;
    while (Date.now() - start < 15000) {
      await new Promise(r => setTimeout(r, 300));
      if (await checkServer(`${base}/desktop.html`)) {
        ready = true;
        break;
      }
    }
    if (!ready) {
      if (devServer) devServer.kill();
      throw new Error('Timed out waiting for Vite dev server on ' + base);
    }
  }

  const browser = await chromium.launch({ headless: true, args: ['--no-sandbox'] });
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    await context.addInitScript(() => {
      window.__ACTION_LOG__ = [];
      window.__TAURI_INTERNALS__ = {
        transformCallback: () => 1,
        unregisterCallback: () => {},
        invoke: async (command, args = {}) => {
          if (command === 'plugin:event|listen') return 1;
          if (command === 'initialize_launcher') return null;
          if (command === 'launcher_settings') return { memory: 4096, javaPath: '', discord: true };
          if (command === 'launcher_snapshot') return { instances: [], localServers: [], downloads: [], minecraftAccounts: [] };
          if (command === 'launcher_capabilities') return { loaders: ['vanilla', 'fabric', 'quilt'], contentInstall: true };
          if (command === 'check_update') {
            window.__ACTION_LOG__.push('check_update');
            return { available: false };
          }
          if (command === 'app_update_status') {
            return { state: 'up-to-date', downloaded: 0, total: null };
          }
          if (command === 'window_action') {
            window.__ACTION_LOG__.push(args.action);
            return null;
          }
          if (command === 'start_dragging') {
            window.__ACTION_LOG__.push('start_dragging');
            return null;
          }
          return null;
        }
      };
    });

    const page = await context.newPage();
    await page.goto(`${base}/desktop.html`, { waitUntil: 'networkidle' });

    // 1. Verify CSS app-region and pointer-events
    const titlebarRegion = await page.locator('.titlebar').evaluate(e => getComputedStyle(e).getPropertyValue('-webkit-app-region') || getComputedStyle(e).getPropertyValue('app-region'));
    assert.equal(titlebarRegion, 'no-drag', 'Titlebar container must not be a drag region');

    const dragAreaRegion = await page.locator('.titlebar-drag-area').evaluate(e => getComputedStyle(e).getPropertyValue('-webkit-app-region') || getComputedStyle(e).getPropertyValue('app-region'));
    assert.equal(dragAreaRegion, 'drag', 'Drag area must be a drag region');

    const buttonsRegion = await page.locator('.window-buttons').evaluate(e => getComputedStyle(e).getPropertyValue('-webkit-app-region') || getComputedStyle(e).getPropertyValue('app-region'));
    assert.equal(buttonsRegion, 'no-drag', 'Window buttons must be no-drag');

    const closeBtnRegion = await page.locator('.window-buttons button[title="Close Solara"]').evaluate(e => getComputedStyle(e).getPropertyValue('-webkit-app-region') || getComputedStyle(e).getPropertyValue('app-region'));
    assert.equal(closeBtnRegion, 'no-drag', 'Close button must be no-drag');

    // 2. Simulate dragging on titlebar-drag-area
    const dragBox = await page.locator('.titlebar-drag-area').boundingBox();
    assert.ok(dragBox, 'Drag area exists');
    await page.mouse.move(dragBox.x + 50, dragBox.y + dragBox.height / 2);
    await page.mouse.down();
    await page.mouse.move(dragBox.x + 200, dragBox.y + 100);
    // Mouse up outside
    await page.mouse.up();

    // Verify start_dragging was called
    await page.waitForFunction(() => window.__ACTION_LOG__.includes('start_dragging'), { timeout: 5000 });
    let log = await page.evaluate(() => window.__ACTION_LOG__);
    assert.ok(log.includes('start_dragging'), 'start_dragging was invoked on titlebar mousedown');

    // 3. Click window control buttons after dragging
    const closeBtn = page.locator('.window-buttons button[title="Close Solara"]');
    await closeBtn.click();
    log = await page.evaluate(() => window.__ACTION_LOG__);
    assert.equal(log[log.length - 1], 'close', 'Close button triggered close window action after dragging');

    const minBtn = page.locator('.window-buttons button[title="Minimize"]');
    await minBtn.click();
    log = await page.evaluate(() => window.__ACTION_LOG__);
    assert.equal(log[log.length - 1], 'minimize', 'Minimize button triggered minimize window action');

    const maxBtn = page.locator('.window-buttons button[title="Maximize or restore"]');
    await maxBtn.click();
    log = await page.evaluate(() => window.__ACTION_LOG__);
    assert.equal(log[log.length - 1], 'maximize', 'Maximize button triggered maximize window action');

    // 4. Test rapid pointerup + click deduplication
    const initialCloseCount = (await page.evaluate(() => window.__ACTION_LOG__)).filter(a => a === 'close').length;
    const btnBox = await closeBtn.boundingBox();
    assert.ok(btnBox, 'Close button bounding box exists');
    await closeBtn.dispatchEvent('pointerup', { clientX: btnBox.x + btnBox.width / 2, clientY: btnBox.y + btnBox.height / 2 });
    await closeBtn.dispatchEvent('click');
    const newCloseCount = (await page.evaluate(() => window.__ACTION_LOG__)).filter(a => a === 'close').length;
    assert.equal(newCloseCount, initialCloseCount + 1, 'Dual pointerup + click within 300ms is deduplicated to exactly one action');

    // 5. Test clean minimal launcher greeting and check-for-updates action
    const greeting = page.locator('.greeting');
    await greeting.waitFor({ state: 'visible' });
    assert.equal(await greeting.textContent(), 'HI ;)', 'Greeting heading displays "HI ;)"');

    const updateBtn = page.locator('.update-btn');
    await updateBtn.waitFor({ state: 'visible' });
    assert.ok((await updateBtn.textContent()).includes('Check for updates'), 'Update button is labeled "Check for updates"');

    // Initially, before clicking, update status should not be shown
    const statusBefore = await page.locator('.update-status').count();
    assert.equal(statusBefore, 0, 'No status message is shown before clicking Check for updates');

    await updateBtn.click();
    await page.waitForFunction(() => window.__ACTION_LOG__.includes('check_update'), { timeout: 5000 });
    const updateStatus = page.locator('.update-status');
    await updateStatus.waitFor({ state: 'visible' });
    assert.ok((await updateStatus.textContent()).includes('You are up to date.'), 'Update status displays confirmation after checking');

    console.log('PASS: Window controls dragging, responsiveness after drag, dedup, and minimal launcher UI verified successfully!');
  } finally {
    await browser.close();
    if (devServer) {
      devServer.kill();
    }
  }
})().catch(err => {
  console.error(err);
  process.exit(1);
});
