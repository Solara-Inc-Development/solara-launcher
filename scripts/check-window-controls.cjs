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
    const context = await browser.newContext({
      viewport: { width: 1280, height: 720 },
      permissions: ['clipboard-read', 'clipboard-write']
    });
    await context.addInitScript(() => {
      window.__ACTION_LOG__ = [];
      const callbacks = new Map();
      const events = new Map();
      let callbackCounter = 1;
      window.__MINECRAFT_ACCOUNTS__ = [];

      window.__TAURI_INTERNALS__ = {
        transformCallback: (cb) => {
          const id = callbackCounter++;
          callbacks.set(id, cb);
          return id;
        },
        unregisterCallback: (id) => callbacks.delete(id),
        invoke: async (command, args = {}) => {
          if (command === 'plugin:event|listen') {
            events.set(args.event, args.handler);
            return events.size;
          }
          if (command === 'initialize_launcher') return null;
          if (command === 'launcher_settings') return { memory: 4096, javaPath: '', discord: true };
          if (command === 'launcher_snapshot') {
            return {
              instances: [],
              localServers: [],
              downloads: [],
              minecraftAccounts: window.__MINECRAFT_ACCOUNTS__
            };
          }
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
          if (command === 'open_external') {
            window.__ACTION_LOG__.push(`open_external:${args.url}`);
            return null;
          }
          if (command === 'start_dragging') {
            window.__ACTION_LOG__.push('start_dragging');
            return null;
          }
          if (command === 'minecraft_sign_in') {
            window.__ACTION_LOG__.push('minecraft_sign_in');
            const handlerId = events.get('minecraft-login-code');
            if (handlerId && callbacks.has(handlerId)) {
              setTimeout(() => {
                const cb = callbacks.get(handlerId);
                if (cb) {
                  cb({
                    event: 'minecraft-login-code',
                    id: 1,
                    payload: { code: 'QA-CODE', url: 'https://www.microsoft.com/link', expiresIn: 900 }
                  });
                }
              }, 20);
            }
            return new Promise((resolve, reject) => {
              window.__RESOLVE_LOGIN__ = () => {
                if (!window.__MINECRAFT_ACCOUNTS__ || window.__MINECRAFT_ACCOUNTS__.length === 0) {
                  window.__MINECRAFT_ACCOUNTS__ = [{ id: 'qa-user-1', name: 'SolaraHero', active: true, owned: true }];
                } else {
                  window.__MINECRAFT_ACCOUNTS__ = [
                    { id: 'qa-user-1', name: 'SolaraHero', active: false, owned: true },
                    { id: 'qa-user-2', name: 'SolaraSecond', active: true, owned: true }
                  ];
                }
                resolve();
              };
              window.__REJECT_LOGIN__ = (msg) => {
                reject(new Error(msg || 'Authentication cancelled'));
              };
            });
          }
          if (command === 'cancel_minecraft_sign_in') {
            window.__ACTION_LOG__.push('cancel_minecraft_sign_in');
            if (window.__REJECT_LOGIN__) {
              window.__REJECT_LOGIN__('Sign-in cancelled by user');
            }
            return null;
          }
          if (command === 'minecraft_sign_out') {
            window.__ACTION_LOG__.push(`minecraft_sign_out:${args.id}`);
            window.__MINECRAFT_ACCOUNTS__ = window.__MINECRAFT_ACCOUNTS__.filter(a => a.id !== args.id);
            if (window.__MINECRAFT_ACCOUNTS__.length > 0 && !window.__MINECRAFT_ACCOUNTS__.some(a => a.active)) {
              window.__MINECRAFT_ACCOUNTS__[0].active = true;
            }
            return null;
          }
          if (command === 'select_minecraft_account') {
            window.__ACTION_LOG__.push(`select_minecraft_account:${args.id}`);
            window.__MINECRAFT_ACCOUNTS__ = window.__MINECRAFT_ACCOUNTS__.map(a => ({
              ...a,
              active: a.id === args.id
            }));
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

    // 5. Test Main Home Page: Centered Welcome Hero, Minecraft Authentication & Account Switcher
    const greeting = page.locator('.greeting');
    await greeting.waitFor({ state: 'visible' });
    assert.equal((await greeting.textContent()).trim(), 'Welcome User TO Solara-Launcher', 'Greeting heading displays "Welcome User TO Solara-Launcher"');

    // 5.1 Verify Unsigned Hero First Impression & Active Account Card
    const heroCard = page.locator('.active-account-card');
    await heroCard.waitFor({ state: 'visible' });
    assert.ok((await heroCard.textContent()).includes('No Account Connected'), 'Hero displays "No Account Connected" when unsigned');
    assert.ok((await heroCard.textContent()).includes('Login with your Minecraft account'), 'Hero displays login guidance text');

    // Clicking the unsigned active account card opens Account Switcher Modal
    await heroCard.click();
    const accountsModal = page.locator('.accounts-modal');
    await accountsModal.waitFor({ state: 'visible' });
    assert.equal(await accountsModal.locator('#modal-accounts-title').textContent(), 'Minecraft Accounts', 'Modal title is "Minecraft Accounts"');
    assert.ok((await accountsModal.textContent()).includes('No Accounts Connected'), 'Modal shows empty state when no accounts');
    assert.ok((await accountsModal.textContent()).includes('Windows vault'), 'Modal displays Windows vault privacy reassurance');

    // Close accounts modal
    const closeAccountsBtn = accountsModal.locator('.modal-close-btn');
    await closeAccountsBtn.click();
    await accountsModal.waitFor({ state: 'hidden' });

    // 5.2 Verify Minecraft Sign-In Device Flow via Login Button
    const loginBtn = page.locator('.btn-minecraft-login');
    await loginBtn.waitFor({ state: 'visible' });
    await loginBtn.click();
    await page.waitForFunction(() => window.__ACTION_LOG__.includes('minecraft_sign_in'), { timeout: 5000 });

    // Verify device code prompt modal appears with QA-CODE
    const deviceCodeEl = page.locator('.minecraft-device-code');
    await deviceCodeEl.waitFor({ state: 'visible' });
    assert.equal((await deviceCodeEl.textContent()).trim(), 'QA-CODE', 'Device code QA-CODE is displayed');

    // Test Copy code button
    const copyBtn = page.locator('.btn-copy-code');
    await copyBtn.click();
    assert.ok((await copyBtn.textContent()).includes('Copied'), 'Copy button reflects copied state');

    // Test Cancel sign-in
    const cancelBtn = page.locator('.btn-cancel');
    await cancelBtn.click();
    await deviceCodeEl.waitFor({ state: 'hidden' });
    assert.ok((await page.evaluate(() => window.__ACTION_LOG__)).includes('cancel_minecraft_sign_in'), 'cancel_minecraft_sign_in was called');

    // Now test successful Minecraft Sign-In and Active Account Display
    await loginBtn.click();
    await deviceCodeEl.waitFor({ state: 'visible' });
    await page.evaluate(() => window.__RESOLVE_LOGIN__());
    await deviceCodeEl.waitFor({ state: 'hidden' });

    // Verify hero active account card updates with username and avatar
    await page.waitForFunction(() => document.querySelector('.active-account-card')?.textContent?.includes('SolaraHero'), { timeout: 5000 });
    assert.ok((await heroCard.textContent()).includes('SolaraHero'), 'Active account card displays "SolaraHero" after sign-in');
    assert.ok((await heroCard.textContent()).includes('Active Account'), 'Active account card indicates "Active Account" status');
    assert.equal((await greeting.textContent()).trim(), 'Welcome User TO Solara-Launcher', 'Greeting heading remains "Welcome User TO Solara-Launcher"');

    const playerHead = page.locator('.player-head');
    await playerHead.waitFor({ state: 'visible' });
    assert.equal(await playerHead.count(), 1, 'Player head avatar is rendered');

    // 5.3 Test Account Switcher Modal with 1-Click Switch and Logout
    await heroCard.click();
    await accountsModal.waitFor({ state: 'visible' });
    assert.ok((await accountsModal.textContent()).includes('SolaraHero'), 'Account switcher lists connected user SolaraHero');
    assert.ok((await accountsModal.locator('.account-active-badge').textContent()).includes('Active'), 'Active badge is displayed for SolaraHero');

    // Test adding second account via "Add Minecraft Account" in modal
    const addAccountBtn = accountsModal.locator('.btn-add-minecraft-account');
    await addAccountBtn.click();
    await accountsModal.waitFor({ state: 'hidden' });
    await page.waitForFunction(() => window.__ACTION_LOG__.filter(a => a === 'minecraft_sign_in').length >= 2, { timeout: 5000 });
    await deviceCodeEl.waitFor({ state: 'visible' });
    await page.evaluate(() => window.__RESOLVE_LOGIN__());
    await deviceCodeEl.waitFor({ state: 'hidden' });

    // Active card now displays second account SolaraSecond
    await page.waitForFunction(() => document.querySelector('.active-account-card')?.textContent?.includes('SolaraSecond'), { timeout: 5000 });
    assert.ok((await heroCard.textContent()).includes('SolaraSecond'), 'Active account card displays second account SolaraSecond');

    // Open switcher modal and test 1-click switch back to SolaraHero
    await heroCard.click();
    await accountsModal.waitFor({ state: 'visible' });
    const switchBtn = accountsModal.locator('.btn-account-switch').first();
    await switchBtn.waitFor({ state: 'visible' });
    await switchBtn.click();
    await page.waitForFunction(() => window.__ACTION_LOG__.some(a => a.startsWith('select_minecraft_account:qa-user-1')), { timeout: 5000 });

    // Verify SolaraHero is active again
    await page.waitForFunction(() => document.querySelector('.active-account-card')?.textContent?.includes('SolaraHero'), { timeout: 5000 });
    assert.ok((await heroCard.textContent()).includes('SolaraHero'), 'Hero active account switched back to SolaraHero');

    // Test Sign Out action on remaining accounts
    const signOutBtn = accountsModal.locator('.btn-account-signout').first();
    await signOutBtn.waitFor({ state: 'visible' });
    await signOutBtn.click();
    await page.waitForFunction(() => window.__ACTION_LOG__.some(a => a.startsWith('minecraft_sign_out:')), { timeout: 5000 });

    // Sign out remaining account if any
    await page.waitForTimeout(200);
    const remainingSignOutBtn = accountsModal.locator('.btn-account-signout').first();
    if (await remainingSignOutBtn.count() > 0) {
      await remainingSignOutBtn.click();
      await page.waitForTimeout(200);
    }

    // Close accounts modal
    await accountsModal.locator('.modal-close-btn').click();
    await accountsModal.waitFor({ state: 'hidden' });

    // Hero resets to "No Account Connected"
    await page.waitForFunction(() => document.querySelector('.active-account-card')?.textContent?.includes('No Account Connected'), { timeout: 5000 });
    assert.ok((await heroCard.textContent()).includes('No Account Connected'), 'Hero resets to "No Account Connected" after signing out all accounts');

    // 5.2 Verify Launcher Announcements Section (Latest 3 Cards & 50-Item Archive)
    const announcementsSection = page.locator('.announcements-section');
    await announcementsSection.waitFor({ state: 'visible' });
    assert.equal(await announcementsSection.locator('.section-title').textContent(), 'Launcher Announcements', 'Section title is "Launcher Announcements"');

    // Verify top 3 announcement cards
    const announcementCards = page.locator('.announcements-grid .announcement-card');
    assert.equal(await announcementCards.count(), 3, 'Displays exactly 3 latest announcements on the home page');

    // Click first card to read announcement
    await announcementCards.first().click();
    const readerModal = page.locator('.announcement-reader');
    await readerModal.waitFor({ state: 'visible' });
    const readerTitle = page.locator('.reader-title');
    assert.ok((await readerTitle.textContent()).length > 0, 'Announcement title is rendered in reader');
    assert.ok(await page.locator('.reader-para').count() > 0, 'Announcement content paragraphs are rendered');

    // Click Back to announcements list
    const backBtn = page.locator('.btn-back-to-list');
    await backBtn.click();

    // Verify 50 announcements archive view
    const archiveItems = page.locator('.announcements-archive-list .archive-item');
    await archiveItems.first().waitFor({ state: 'visible' });
    assert.equal(await archiveItems.count(), 50, 'Archive displays up to 50 announcements');

    // Test category filter pill
    const releaseFilter = page.locator('.category-filters .filter-pill:has-text("Release")');
    await releaseFilter.click();
    assert.ok(await archiveItems.count() > 0 && await archiveItems.count() <= 50, 'Filtered announcements list is displayed');

    // Close announcements modal
    const closeAnnouncementsBtn = page.locator('.announcements-modal .modal-close-btn');
    await closeAnnouncementsBtn.click();
    await page.locator('.announcements-modal').waitFor({ state: 'hidden' });

    // 5.3 Verify Latest Update Info Section & Check for Updates
    const updateInfoSection = page.locator('.update-info-section');
    await updateInfoSection.waitFor({ state: 'visible' });
    assert.equal(await updateInfoSection.locator('.section-title').textContent(), 'Latest Update Info', 'Section title is "Latest Update Info"');

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

    // Verify update highlights list includes key features
    const highlightsText = await updateInfoSection.locator('.update-highlights-list').textContent();
    assert.ok(highlightsText.includes('Minecraft Java Account Sign-In'), 'Highlights include Minecraft Auth');
    assert.ok(highlightsText.includes('Launcher Announcements Section'), 'Highlights include Announcements');
    assert.ok(highlightsText.includes('Disabled Default Browser Context Menu'), 'Highlights include Context Menu suppression');

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

    // Horizon Sidebar checks (wider sidebar ~200px - 220px with Icon + Text labels):
    const sidebar = page.locator('.horizon-sidebar');
    assert.equal(await sidebar.count(), 1, 'Horizon sidebar exists');

    const sidebarBox = await sidebar.boundingBox();
    assert.ok(sidebarBox, 'Sidebar bounding box exists');
    assert.ok(sidebarBox.width >= 200 && sidebarBox.width <= 220, `Sidebar width (${sidebarBox.width}px) is in the ~200px - 220px range`);

    // Verify Home item has icon and text label "Home"
    const homeBtn = page.locator('.horizon-sidebar .dock-top .dock-item');
    assert.equal(await homeBtn.count(), 1, 'Home button exists in dock');
    assert.ok((await homeBtn.textContent()).includes('Home'), 'Home button has "Home" text label');

    // Verify Discord item has icon and text label "Discord"
    const discordBtn = page.locator('.horizon-discord-btn');
    assert.equal(await discordBtn.count(), 1, 'Discord button exists in sidebar dock');
    assert.equal(await discordBtn.getAttribute('href'), 'https://discord.solara-inc.in/', 'Discord button links to https://discord.solara-inc.in/');
    assert.equal(await discordBtn.getAttribute('title'), 'Join the Discord', 'Discord button has title "Join the Discord"');
    assert.ok((await discordBtn.textContent()).includes('Discord'), 'Discord button has "Discord" text label');

    // Click Discord button and verify native open_external is invoked with canonical discord URL
    await discordBtn.click();
    await page.waitForFunction(() => window.__ACTION_LOG__.some(a => a.startsWith('open_external:')), { timeout: 5000 });
    log = await page.evaluate(() => window.__ACTION_LOG__);
    assert.ok(log.includes('open_external:https://discord.solara-inc.in/'), 'Clicking Discord button calls open_external with https://discord.solara-inc.in/');

    // 9. Verify Default Browser Context Menu is Disabled Globally
    const contextMenuResults = await page.evaluate(() => {
      let preventedCount = 0;
      const targets = [
        document.body,
        document.querySelector('.shell'),
        document.querySelector('.titlebar'),
        document.querySelector('.horizon-sidebar'),
        document.querySelector('.launcher-main'),
        document.querySelector('.greeting')
      ].filter(Boolean);

      for (const target of targets) {
        const ev = new MouseEvent('contextmenu', {
          bubbles: true,
          cancelable: true,
          view: window,
          buttons: 2
        });
        target.dispatchEvent(ev);
        if (ev.defaultPrevented) preventedCount++;
      }
      return { total: targets.length, preventedCount };
    });
    assert.ok(contextMenuResults.total >= 5, 'Found application targets for contextmenu test');
    assert.equal(contextMenuResults.preventedCount, contextMenuResults.total, 'All contextmenu events are prevented across all application targets');

    // Real right click via Playwright mouse
    await page.evaluate(() => {
      window.__RIGHT_CLICK_PREVENTED__ = null;
      window.addEventListener('contextmenu', (e) => {
        window.__RIGHT_CLICK_PREVENTED__ = e.defaultPrevented;
      }, { capture: true, once: true });
    });
    await page.mouse.click(500, 300, { button: 'right' });
    const rightClickPrevented = await page.evaluate(() => window.__RIGHT_CLICK_PREVENTED__);
    assert.equal(rightClickPrevented, true, 'Real mouse right-click contextmenu event default was prevented');

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
