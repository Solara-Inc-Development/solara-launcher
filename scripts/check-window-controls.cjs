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
    const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
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
    page.on('pageerror', err => console.error('PAGE ERROR:', err));
    page.on('console', msg => {
      if (msg.type() === 'error') console.error('PAGE CONSOLE ERROR:', msg.text());
    });
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

    // 6. Test minimalist loading screen overlay, logo, clean status line, progress bar, percentage, and natural transition
    const previewBtn = page.locator('.preview-anim-btn');
    await previewBtn.waitFor({ state: 'visible' });
    assert.ok((await previewBtn.textContent()).includes('Preview loading animation'), 'Preview button is labeled "Preview loading animation"');

    await previewBtn.click();
    const loadingScreen = page.locator('.loading-screen');
    await loadingScreen.waitFor({ state: 'visible' });

    assert.equal(await page.locator('.loading-logo').count(), 1, 'Loading screen shows glowing Solara brand logo');
    const logoSrc = await page.locator('.loading-logo').getAttribute('src');
    assert.equal(logoSrc, '/logo-512.png', 'Loading logo uses /logo-512.png');

    // Verify zero cut-off background logo watermark artifacts
    assert.equal(await page.locator('.solara-loader-watermark').count(), 0, 'Zero cut-off background logo watermark artifacts');
    assert.equal(await page.locator('.loading-track').count(), 1, 'Loading screen shows sleek progress track');
    assert.equal(await page.locator('.loading-progress-bar').count(), 1, 'Loading screen shows fill progress bar');

    // Verify clean status line below loading bar
    const statusEl = page.locator('.loading-status-text');
    assert.equal(await statusEl.count(), 1, 'Clean status line is displayed below the loading bar');
    const validStatuses = [
      'Reading Files…',
      'Checking Integrity…',
      'Preparing Environment…',
      'Launching Solara-Launcher…'
    ];
    const statusText = (await statusEl.textContent())?.trim();
    assert.ok(validStatuses.includes(statusText), `Status text "${statusText}" is one of the valid progress stages`);

    // Verify skip button, title, and subtitles remain removed
    assert.equal(await page.locator('.loading-skip-btn').count(), 0, 'Skip button is completely removed');
    assert.equal(await page.locator('.solara-loader-title').count(), 0, 'Title heading is removed');
    assert.equal(await page.locator('.solara-loader-subtitle').count(), 0, 'Subtitle tagline is removed');

    const pctEl = page.locator('.loading-percentage');
    const pctEval = await pctEl.evaluate(el => {
      const rect = el.getBoundingClientRect();
      const style = window.getComputedStyle(el);
      return {
        rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
        display: style.display,
        visibility: style.visibility,
        opacity: style.opacity
      };
    });
    assert.ok(pctEval.rect.width > 0 && pctEval.rect.height > 0, 'Percentage element has non-zero dimensions');
    assert.notEqual(pctEval.display, 'none', 'Percentage element is not display: none');
    assert.notEqual(pctEval.visibility, 'hidden', 'Percentage element is not visibility: hidden');
    const pctText = (await pctEl.textContent()) || '';
    assert.ok(/^[0-9]{1,3}%$/.test(pctText.trim()), 'Displays accurate numeric percentage');

    // Wait for natural completion (animation runs 5-10s)
    await loadingScreen.waitFor({ state: 'hidden', timeout: 15000 });
    assert.equal(await greeting.isVisible(), true, 'Loading screen completes and reveals main container');

    // 7. Verify rapid replay does not cause premature sequence termination due to race conditions
    await previewBtn.click();
    await loadingScreen.waitFor({ state: 'visible' });
    await page.waitForTimeout(200);
    await page.evaluate(() => {
      const el = document.querySelector('.preview-anim-btn');
      if (el) el.click();
    });
    await page.waitForTimeout(1000);
    assert.equal(await loadingScreen.isVisible(), true, 'Loading screen remains active across re-entrant calls');
    await loadingScreen.waitFor({ state: 'hidden', timeout: 15000 });
    assert.equal(await greeting.isVisible(), true, 'Greeting is visible after final sequence');

    // 8. Verify Minimal Modrinth-inspired Top Bar & Sleek Horizon-inspired Icon Dock
    // Modrinth Top Bar checks:
    const headerBrand = page.locator('.header-brand');
    assert.equal(await headerBrand.count(), 1, 'Header brand exists');
    assert.ok((await headerBrand.textContent()).includes('solara'), 'Brand text contains "solara"');

    // Confirm search bar, breadcrumbs, history nav, and status pill are completely removed
    assert.equal(await page.locator('.header-search-box').count(), 0, 'Search bar is completely removed from header');
    assert.equal(await page.locator('.header-breadcrumb').count(), 0, 'Breadcrumbs are completely removed from header');
    assert.equal(await page.locator('.header-nav-history').count(), 0, 'Nav history buttons are completely removed from header');
    assert.equal(await page.locator('.instance-status-pill').count(), 0, 'Instance status pill is completely removed from header');

    // Horizon Sidebar Icon Dock checks:
    const sidebar = page.locator('.horizon-sidebar');
    assert.equal(await sidebar.count(), 1, 'Horizon sidebar icon dock exists');

    const discordBtn = page.locator('.horizon-discord-btn');
    assert.equal(await discordBtn.count(), 1, 'Discord button exists in sidebar dock');
    assert.equal(await discordBtn.getAttribute('href'), 'https://discord.solara-inc.in/', 'Discord button links to https://discord.solara-inc.in/');
    assert.equal(await discordBtn.getAttribute('title'), 'Join the Discord', 'Discord button has title "Join the Discord"');

    // Confirm all bloat (instances card, nav tabs, edition badges, play bars, views, settings) are completely removed
    assert.equal(await page.locator('.horizon-instance-card').count(), 0, 'Premade instance cards are completely removed');
    assert.equal(await page.locator('.nav-tab-btn').count(), 0, 'Sidebar nav tabs are completely removed');
    assert.equal(await page.locator('.hero-play-btn').count(), 0, 'Play bar is completely removed');
    assert.equal(await page.locator('.badge-edition').count(), 0, 'Edition badges are completely removed');
    assert.equal(await page.locator('.view-instances').count(), 0, 'Instances view is completely removed');
    assert.equal(await page.locator('.view-mods').count(), 0, 'Mods view is completely removed');
    assert.equal(await page.locator('.view-servers').count(), 0, 'Servers view is completely removed');
    assert.equal(await page.locator('.view-settings').count(), 0, 'Settings view is completely removed');
    assert.equal(await page.locator('.settings-panel').count(), 0, 'Settings panel is completely removed');

    console.log('PASS: Window controls, minimal Modrinth header, Horizon icon dock, and loading screen verified successfully with all bloat removed!');

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
