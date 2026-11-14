const { chromium } = require('playwright');
const { spawn } = require('node:child_process');
const path = require('node:path');
const assert = require('node:assert/strict');

async function waitForServer(url, timeoutMs = 15000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const r = await fetch(url);
      if (r.ok) return;
    } catch {}
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error(`Timed out waiting for ${url}`);
}

async function main() {
  const viteBin = path.resolve(__dirname, '..', 'node_modules', 'vite', 'bin', 'vite.js');
  const server = spawn(process.execPath, [viteBin, '--port', '4173', '--host', '127.0.0.1'], {
    stdio: 'ignore',
    detached: true,
  });

  let browser;
  try {
    const baseUrl = 'http://127.0.0.1:4173/desktop.html';
    await waitForServer(baseUrl);

    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    const page = await context.newPage();
    page.setDefaultTimeout(10000);

    const errors = [];
    page.on('pageerror', (err) => errors.push(`pageerror: ${err.message}`));
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(`console.error: ${msg.text()}`);
    });

    // Inject realistic Tauri IPC mock covering all redesigned desktop commands
    const installTauriMock = async (targetPage, opts = { freshOnboarding: false }) => {
      await targetPage.addInitScript((mockOpts) => {
        const state = {
          preferences: {
            memory: 4096,
            javaPath: 'auto',
            theme: 'dawn-gold',
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
            onboardingCompleted: !mockOpts.freshOnboarding,
            groups: ['Survival'],
          },
          instances: mockOpts.freshOnboarding
            ? []
            : [
                {
                  schemaVersion: 1,
                  id: 'inst-1',
                  name: 'Dawn SMP',
                  gameVersion: '1.21.4',
                  loader: 'fabric',
                  loaderVersion: '0.16.10',
                  memory: 4096,
                  javaPath: 'auto',
                  state: 'installed',
                  running: false,
                  icon: 'preset:dawn-crest',
                  group: 'Survival',
                  pinned: true,
                  favourite: true,
                  createdAt: new Date(Date.now() - 86400_000 * 5).toISOString(),
                  lastPlayed: new Date(Date.now() - 3600_000).toISOString(),
                  playtimeSeconds: 5400,
                },
                {
                  schemaVersion: 1,
                  id: 'inst-2',
                  name: 'Vanilla Testing',
                  gameVersion: '1.21.4',
                  loader: 'vanilla',
                  loaderVersion: null,
                  memory: 2048,
                  javaPath: 'auto',
                  state: 'installed',
                  running: false,
                  icon: 'preset:obsidian-cube',
                  group: null,
                  pinned: false,
                  favourite: false,
                  createdAt: new Date(Date.now() - 86400_000 * 2).toISOString(),
                  lastPlayed: null,
                  playtimeSeconds: 0,
                },
              ],
          accounts: [
            {
              id: 'acc-1',
              username: 'SolaraTester',
              selected: true,
              name: 'SolaraTester',
              active: true,
              owned: true,
            },
          ],
          downloads: [
            {
              id: 'job-1',
              instance: 'inst-1',
              name: 'Fabric 0.16.10',
              kind: 'install',
              status: 'completed',
              phase: 'Verified',
              done: 102400,
              total: 102400,
            },
          ],
          winActions: [],
          savedSkins: [],
          mods: [
            { path: 'mods/sodium-0.6.jar', fileName: 'sodium-0.6.jar', name: 'Sodium', version: '0.6.0', modId: 'sodium', enabled: true, size: 950000 },
            { path: 'mods/iris-1.8.jar.disabled', fileName: 'iris-1.8.jar.disabled', name: 'Iris Shaders', version: '1.8.0', modId: null, enabled: false, size: 1450000 },
          ],
        };
        window.__SOLARA_TEST_STATE__ = state;

        const callbacks = new Map();
        let cbSeq = 1;
        window.__TAURI_INTERNALS__ = {
          transformCallback(fn, once) {
            const id = cbSeq++;
            callbacks.set(id, { fn, once });
            return id;
          },
          unregisterCallback(id) {
            callbacks.delete(id);
          },
          convertFileSrc(path) {
            return path;
          },
          metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
          invoke: async (cmd, args = {}) => {
            if (cmd === 'plugin:event|listen') return 1;
            if (cmd === 'plugin:event|unlisten') return null;
            if (cmd === 'initialize_launcher') return null;
            if (cmd === 'launcher_snapshot') {
              return {
                instances: state.instances,
                minecraftAccounts: state.accounts,
                localServers: [],
                downloads: state.downloads,
              };
            }
            if (cmd === 'launcher_settings') return state.preferences;
            if (cmd === 'save_launcher_settings_full') {
              state.preferences = { ...state.preferences, ...args.preferences };
              return state.preferences;
            }
            if (cmd === 'app_update_status') return { stage: 'idle', version: null, error: null };
            if (cmd === 'check_update') return { stage: 'current', version: null, error: null };
            if (cmd === 'minecraft_versions') {
              return [
                { id: '1.21.4', type: 'release' },
                { id: '1.21.1', type: 'release' },
                { id: '25w09a', type: 'snapshot' },
                { id: 'b1.7.3', type: 'old_beta' },
              ];
            }
            if (cmd === 'loader_versions') {
              return [{ version: '0.16.10', stable: true }, { version: '0.16.9', stable: false }];
            }
            if (cmd === 'create_instance') {
              const id = `inst-${state.instances.length + 1}`;
              state.instances.push({
                schemaVersion: 1,
                id,
                name: args.name,
                gameVersion: args.gameVersion,
                loader: args.loader,
                loaderVersion: args.loaderVersion || null,
                memory: args.options?.memory ?? 4096,
                javaPath: args.options?.javaPath ?? 'auto',
                state: 'installed',
                running: false,
                icon: args.options?.icon ?? 'preset:dawn-crest',
                group: args.options?.group ?? null,
                pinned: false,
                favourite: false,
                createdAt: new Date().toISOString(),
                lastPlayed: null,
                playtimeSeconds: 0,
              });
              return { jobId: 'job-new', instanceId: id };
            }
            if (cmd === 'update_instance_meta') {
              const inst = state.instances.find((i) => i.id === args.id);
              if (!inst) throw new Error('Missing instance');
              Object.assign(inst, args.patch || {});
              return inst;
            }
            if (cmd === 'detect_java_runtimes') {
              return [{ path: 'C:\\Java\\21\\bin\\java.exe', version: '21.0.5', major: 21, vendor: 'Eclipse Adoptium', managed: true }];
            }
            if (cmd === 'launcher_storage_summary') {
              return { totalBytes: 104857600, instancesBytes: 83886080, cacheBytes: 10485760, javaBytes: 10485760, skinsBytes: 4096 };
            }
            if (cmd === 'instance_mods') {
              return state.mods.map((m) => ({ ...m }));
            }
            if (cmd === 'toggle_instance_mod') {
              const mod = state.mods.find((m) => m.path === args.path);
              if (!mod) throw new Error('Mod not found');
              if (mod.enabled === args.enabled) {
                throw new Error('This mod is already in the requested state');
              }
              mod.enabled = Boolean(args.enabled);
              if (mod.enabled && mod.path.endsWith('.jar.disabled')) {
                mod.path = mod.path.slice(0, -'.disabled'.length);
                mod.fileName = mod.fileName.slice(0, -'.disabled'.length);
              } else if (!mod.enabled && mod.path.endsWith('.jar')) {
                mod.path = `${mod.path}.disabled`;
                mod.fileName = `${mod.fileName}.disabled`;
              }
              return null;
            }
            if (cmd === 'instance_packs') return [];
            if (cmd === 'instance_worlds') {
              return [{ folder: 'New World', name: 'Dawn Survival', gameMode: 'Survival', difficulty: 'Normal', versionName: '1.21.4', lastPlayedMs: Date.now() - 10000, size: 12582912, iconDataUrl: null, locked: false, datapackCount: 1 }];
            }
            if (cmd === 'instance_files') {
              return [{ name: 'mods', path: 'mods', directory: true, size: 0, editable: false, image: false }, { name: 'options.txt', path: 'options.txt', directory: false, size: 1200, editable: true, image: false }];
            }
            if (cmd === 'instance_trash') return [];
            if (cmd === 'instance_disk_logs') return [{ name: 'latest.log', size: 16384, modifiedMs: Date.now(), gzipped: false }];
            if (cmd === 'instance_disk_log') {
              const lines = [];
              for (let i = 1; i <= 358; i++) {
                lines.push(`[10:00:${String(i % 60).padStart(2, '0')}] [main/INFO]: Boot step ${i}`);
              }
              lines.push('[10:05:59] [main/WARN]: Example warn line');
              lines.push('[14Nov2026 10:06:00.123] [main/ERROR]: Example error trace');
              lines.push('[14Nov2026 10:06:01.000] [main/FATAL]: Example fatal trace');
              return lines.join('\n');
            }
            if (cmd === 'instance_logs') return '[10:00:00] [main/INFO]: Live process';
            if (cmd === 'launcher_screenshots') {
              const items = [];
              for (let i = 0; i < 80; i++) {
                const name = `dawn-view-${String(i).padStart(3, '0')}.png`;
                items.push({
                  instanceId: 'inst-1',
                  instanceName: 'Dawn SMP',
                  fileName: name,
                  path: `screenshots/${name}`,
                  fullPath: `C:\\Users\\Solara\\AppData\\Local\\in.solara.launcher\\custom-instances\\inst-1\\game\\screenshots\\${name}`,
                  size: 2048 + i,
                  modifiedMs: Date.now() - i * 5000,
                });
              }
              return items;
            }
            if (cmd === 'launcher_screenshot_image') {
              state.screenshotImageCalls = (state.screenshotImageCalls || 0) + 1;
              state.screenshotImagePaths = state.screenshotImagePaths || [];
              state.screenshotImagePaths.push(args.path);
              return 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=';
            }
            if (cmd === 'launch_instance') {
              const inst = state.instances.find((i) => i.id === args.id);
              if (inst) inst.running = true;
              return null;
            }
            if (cmd === 'stop_instance') {
              const inst = state.instances.find((i) => i.id === args.id);
              if (inst) inst.running = false;
              return null;
            }
            if (cmd === 'instance_backups') return [];
            if (cmd === 'check_instance_content_updates') return [];
            if (cmd === 'saved_skins_list') return state.savedSkins;
            if (cmd === 'saved_skins_save') {
              const item = { id: 'skin-1', name: args.name, variant: args.variant, pngDataUrl: args.pngDataUrl, updatedAt: new Date().toISOString() };
              state.savedSkins = [item];
              return item;
            }
            if (cmd === 'profile_skins_and_capes') return { skins: [], capes: [] };
            if (cmd === 'window_action') {
              state.winActions.push(args.action);
              return null;
            }
            if (cmd === 'start_dragging') return null;
            return null;
          },
        };
      }, opts);
    };

    const installNetworkMocks = async (targetPage) => {
      const png1x1 = Buffer.from(
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+ip1sAAAAASUVORK5CYII=',
        'base64',
      );
      await targetPage.route('https://api.solara-inc.in/**', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            items: [
              {
                id: 'ann-1',
                title: 'Solara Dawn Gold Update',
                summary: 'Clean Obsidian workspace and faster setup.',
                content: 'Clean Obsidian workspace and faster setup.',
                category: 'Release',
                date: new Date().toISOString(),
              },
            ],
          }),
        });
      });
      await targetPage.route('https://mc-heads.net/**', async (route) => {
        await route.fulfill({ status: 200, contentType: 'image/png', body: png1x1 });
      });
      await targetPage.route('https://api.modrinth.com/**', async (route) => {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            hits: [
              {
                project_id: 'AANobbMI',
                project_type: 'mod',
                slug: 'sodium',
                title: 'Sodium',
                description: 'Modern rendering engine for Minecraft.',
                author: 'jellysquid3',
                icon_url: null,
                downloads: 45000000,
                categories: ['fabric', 'optimization'],
                versions: ['1.21.4'],
              },
            ],
          }),
        });
      });
    };

    await installTauriMock(page, { freshOnboarding: false });
    await installNetworkMocks(page);

    await page.goto(baseUrl, { waitUntil: 'domcontentloaded', timeout: 25000 });
    await page.waitForSelector('.sol-launch-title', { timeout: 10000 });

    // 1. Verify Dawn Gold Obsidian theme colors & zero outer document scrollbars
    const metrics = await page.evaluate(() => {
      const rootStyle = getComputedStyle(document.documentElement);
      const bodyStyle = getComputedStyle(document.body);
      return {
        accent: rootStyle.getPropertyValue('--accent').trim(),
        bgMain: rootStyle.getPropertyValue('--bg-main').trim(),
        docScrollW: document.documentElement.scrollWidth,
        docClientW: document.documentElement.clientWidth,
        docScrollH: document.documentElement.scrollHeight,
        docClientH: document.documentElement.clientHeight,
        bodyOverflow: bodyStyle.overflow,
      };
    });
    assert.equal(metrics.accent.toUpperCase(), '#E9B86D');
    assert.equal(metrics.bgMain.toUpperCase(), '#090B0F');
    assert.equal(metrics.bodyOverflow, 'hidden');
    assert.ok(metrics.docScrollW <= metrics.docClientW, `Horizontal overflow detected: ${metrics.docScrollW} > ${metrics.docClientW}`);
    assert.ok(metrics.docScrollH <= metrics.docClientH, `Outer vertical scrollbar detected: ${metrics.docScrollH} > ${metrics.docClientH}`);

    // Verify permanently compact 68px icon-only sidebar rail & sleek floating tooltip
    const railBox = await page.locator('.sol-rail').boundingBox();
    assert.ok(railBox && Math.round(railBox.width) === 68, `Expected compact 68px sidebar rail, got ${railBox?.width}`);
    assert.equal(await page.locator('button[title="Expand sidebar"], button[title="Collapse sidebar"]').count(), 0);
    assert.equal(await page.locator('.sol-rail button[title]').count(), 0, 'Rail buttons should not use slow native browser title attributes');
    await page.hover('.sol-rail-item[aria-label="Library"]');
    await page.waitForSelector('.sol-rail-tooltip[role="tooltip"]:has-text("Library")');
    const tipShadow = await page.locator('.sol-rail-tooltip').evaluate((el) => getComputedStyle(el).boxShadow);
    assert.ok(tipShadow && tipShadow !== 'none', `Expected valid box-shadow on rail tooltip, got ${tipShadow}`);

    // 2. Verify Home Dashboard shows Dawn SMP selected instance
    const homeTitle = await page.textContent('.sol-launch-title');
    assert.equal(homeTitle?.trim(), 'Dawn SMP');

    // 3. Navigate to Library, switch grid/list, create instance
    await page.click('.sol-rail-item[aria-label="Library"]');
    await page.waitForSelector('.sol-inst-grid');
    await page.click('button[aria-label="List view"]');
    await page.waitForSelector('.sol-list-table');
    await page.click('button[aria-label="Grid view"]');

    // Create instance modal
    await page.click('button:has-text("New Instance")');
    await page.waitForSelector('.sol-create-layout');
    await page.click('.sol-create-identity .sol-chip:has-text("Fabric")');
    const nameInput = page.locator('.sol-create-identity input.sol-input');
    const autoPlaceholder = await nameInput.getAttribute('placeholder');
    assert.equal(autoPlaceholder, 'Fabric 1.21.4');
    await nameInput.fill('E2E Fabric Test');
    await page.click('.sol-modal-footer button:has-text("Create Instance")');
    await page.waitForSelector('.sol-inst-title:has-text("E2E Fabric Test")');

    // 4. Test Instance Detail tabs (Overview, Content + Bulk Mixed Toggle, Worlds, Logs windowing/filters/clear, Files, Settings Launch Hooks warning)
    await page.click('.sol-inst-header-left >> .. >> .. >> button[role="tab"]:has-text("Content")');
    await page.waitForSelector('button:has-text("Mods")');
    await page.waitForSelector('text=Iris Shaders');
    const modCheckboxes = page.locator('.sol-content-row input[type="checkbox"]');
    await modCheckboxes.nth(0).check();
    await modCheckboxes.nth(1).check();
    await page.waitForSelector('.sol-bulk-bar:has-text("2 selected")');
    await page.click('.sol-bulk-bar button:has-text("Enable")');
    await page.waitForFunction(() => window.__SOLARA_TEST_STATE__.mods.every((m) => m.enabled === true));
    assert.equal(await page.locator('.sol-toast--error').count(), 0, 'Mixed-state bulk enable must not emit redundant-toggle errors');
    await page.click('button[role="tab"]:has-text("Worlds")');
    await page.waitForSelector('.sol-world-name:has-text("Dawn Survival")');
    await page.click('button[role="tab"]:has-text("Logs")');
    await page.waitForSelector('.sol-log-viewer');
    await page.waitForSelector('.sol-log-window-bar');
    assert.equal(await page.locator('.sol-log-line').count(), 300, 'Large log should window initial DOM lines to 300');
    await page.click('.sol-seg-btn:has-text("ERROR")');
    assert.equal(await page.locator('.sol-log-line').count(), 2, 'ERROR filter should show ERROR and FATAL lines');
    assert.ok((await page.locator('.sol-log-ts').count()) >= 2, 'Forge-style timestamps should parse into .sol-log-ts');
    await page.click('.sol-seg-btn:has-text("ALL")');
    await page.click('button:has-text("Timestamps")');
    assert.equal(await page.locator('.sol-log-ts').count(), 0, 'Toggling timestamps off should hide timestamp spans');
    await page.click('button:has-text("Pause")');
    await page.waitForSelector('button:has-text("Resume")');
    await page.click('button:has-text("Clear")');
    await page.waitForSelector('.sol-log-viewer:has-text("No matching log output.")');
    await page.click('button[role="tab"]:has-text("Files")');
    await page.waitForSelector('text=options.txt');
    await page.click('button[role="tab"]:has-text("Settings")');
    await page.click('.sol-settings-nav button:has-text("Launch Hooks")');
    await page.waitForSelector('.sol-banner--warning:has-text("without a system shell")');

    // 5. Test Discover page & Vanilla compatibility notice
    await page.click('.sol-rail-item[aria-label="Discover"]');
    await page.waitForSelector('.sol-discover-title:has-text("Sodium")');

    // 6. Test Skins 3D Viewer & Editor (verify empty library still initializes a saveable starter PNG)
    await page.click('.sol-rail-item[aria-label="Skin & Capes"]');
    await page.waitForSelector('.sol-skin-canvas');
    await page.click('button[role="tab"]:has-text("Skin Editor")');
    await page.waitForSelector('.sol-editor-canvas');
    await page.click('button:has-text("Save to Library")');
    await page.waitForFunction(() => window.__SOLARA_TEST_STATE__.savedSkins.length === 1);
    const savedSkinDataUrl = await page.evaluate(() => window.__SOLARA_TEST_STATE__.savedSkins[0]?.pngDataUrl || '');
    assert.ok(savedSkinDataUrl.startsWith('data:image/png'), 'Expected saved starter skin PNG data URL');

    // 7. Test Global Settings + Privacy cleanup + Unsaved SaveBar + Dirty Leave Guard + Confirm Launch
    await page.click('.sol-rail-item[aria-label="Settings"]');
    await page.waitForSelector('.sol-settings-sidebar');
    assert.equal(await page.locator('.sol-settings-nav-item:has-text("Privacy & Discord")').count(), 0);
    await page.click('.sol-settings-nav-item:has-text("Privacy")');
    assert.equal(await page.locator('text=Discord Rich Presence').count(), 0, 'Unimplemented Discord Rich Presence toggle must be removed');
    await page.click('.sol-settings-nav-item:has-text("Appearance")');
    await page.click('.sol-chip:has-text("Obsidian Slate")');
    await page.waitForSelector('.sol-savebar');
    // Try navigating away with unsaved changes -> leave dialog appears!
    await page.click('.sol-rail-item[aria-label="Home"]');
    await page.waitForSelector('.sol-modal-title:has-text("Unsaved Changes")');
    await page.click('button:has-text("Stay & Keep Editing")');
    await page.click('.sol-settings-nav-item:has-text("Behaviour")');
    await page.click('button[aria-label="Confirm launch"]');
    await page.click('.sol-savebar button:has-text("Save Changes")');
    await page.waitForTimeout(150);

    // Verify Confirm Launch dialog works on Home Play button
    await page.click('.sol-rail-item[aria-label="Home"]');
    await page.waitForSelector('.sol-launch-title');
    await page.click('.sol-launch-cta button:has-text("Play")');
    await page.waitForSelector('.sol-modal-title:has-text("Launch Instance?")');
    await page.click('.sol-modal-footer button:has-text("Launch Now")');
    await page.waitForSelector('.sol-launch-cta button:has-text("Stop")');
    await page.click('.sol-launch-cta button:has-text("Stop")');

    // Verify Screenshots page loads lazy previews on-demand, survives >72 LRU eviction on scroll-back, and lightbox shows Copy file path
    await page.click('.sol-rail-item[aria-label="Screenshots"]');
    await page.waitForSelector('.sol-shot-card img');
    await page.waitForTimeout(150);
    const initialCalls = await page.evaluate(() => window.__SOLARA_TEST_STATE__.screenshotImageCalls || 0);
    assert.ok(initialCalls > 0 && initialCalls < 80, `Expected lazy screenshot loading (<80 initial reads), got ${initialCalls}`);
    // Scroll down in steps so IntersectionObserver loads all 80 cards and evicts the earliest from 72-entry LRU
    await page.evaluate(async () => {
      const scroller = document.querySelector('.sol-page.sol-scroll');
      if (!scroller) return;
      for (let y = 0; y <= scroller.scrollHeight; y += 350) {
        scroller.scrollTop = y;
        await new Promise((r) => setTimeout(r, 40));
      }
      scroller.scrollTop = scroller.scrollHeight;
    });
    await page.waitForTimeout(250);
    const afterScrollDownCalls = await page.evaluate(() => window.__SOLARA_TEST_STATE__.screenshotImageCalls || 0);
    assert.ok(afterScrollDownCalls >= 80, `Expected remaining screenshots to load on scroll, got ${afterScrollDownCalls}`);
    // Scroll back to top and verify first card reloads cleanly instead of staying blank after LRU eviction
    await page.evaluate(() => {
      const scroller = document.querySelector('.sol-page.sol-scroll');
      if (scroller) scroller.scrollTop = 0;
    });
    await page.waitForSelector('.sol-shot-card:first-child img');
    const afterScrollUpCalls = await page.evaluate(() => window.__SOLARA_TEST_STATE__.screenshotImageCalls || 0);
    assert.ok(afterScrollUpCalls > afterScrollDownCalls, 'Expected evicted top screenshot to reload on scroll back');
    await page.click('.sol-shot-card:first-child');
    await page.waitForSelector('.sol-lightbox-img');
    await page.waitForSelector('.sol-modal-footer button:has-text("Copy file path")');
    await page.keyboard.press('Escape');

    // 8. Test Global Quick Search (Ctrl+K) & Tasks Popover & Window Controls
    await page.keyboard.press('Control+k');
    await page.waitForSelector('.sol-modal-title:has-text("Quick Jump")');
    await page.keyboard.press('Escape');

    await page.click('button[aria-label="Toggle tasks popover"]');
    await page.waitForSelector('.sol-tasks-popover');
    await page.click('button[aria-label="Toggle tasks popover"]');

    await page.click('button[aria-label="Minimize window"]');
    await page.click('button[aria-label="Maximize or restore window"]');
    const winActs = await page.evaluate(() => window.__SOLARA_TEST_STATE__.winActions);
    assert.deepEqual(winActs, ['minimize', 'maximize']);

    // 9. Verify responsive layout at 1100x720 and 940x560 (minimum window size)
    for (const vp of [
      { width: 1100, height: 720 },
      { width: 940, height: 560 },
    ]) {
      await page.setViewportSize(vp);
      await page.click('.sol-rail-item[aria-label="Home"]');
      await page.waitForTimeout(120);
      const vpCheck = await page.evaluate(() => ({
        sw: document.documentElement.scrollWidth,
        cw: document.documentElement.clientWidth,
        sh: document.documentElement.scrollHeight,
        ch: document.documentElement.clientHeight,
      }));
      assert.ok(vpCheck.sw <= vpCheck.cw, `Overflow at ${vp.width}x${vp.height}: ${vpCheck.sw} > ${vpCheck.cw}`);
      assert.ok(vpCheck.sh <= vpCheck.ch, `Outer scrollbar at ${vp.width}x${vp.height}: ${vpCheck.sh} > ${vpCheck.ch}`);
    }

    // 10. Verify 4-step Onboarding flow on fresh launch
    const onboardCtx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    const onboardPage = await onboardCtx.newPage();
    onboardPage.setDefaultTimeout(10000);
    onboardPage.on('pageerror', (err) => errors.push(`onboarding pageerror: ${err.message}`));
    onboardPage.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(`onboarding console.error: ${msg.text()}`);
    });
    await installTauriMock(onboardPage, { freshOnboarding: true });
    await installNetworkMocks(onboardPage);
    await onboardPage.goto(baseUrl, { waitUntil: 'domcontentloaded' });
    await onboardPage.waitForSelector('.sol-onboarding-title:has-text("Welcome to Solara Launcher")', { timeout: 10000 });
    await onboardPage.click('.sol-onboarding-footer button:has-text("Continue")');
    await onboardPage.waitForSelector('h2:has-text("Microsoft Account")');
    await onboardPage.click('.sol-onboarding-footer button:has-text("Continue")');
    await onboardPage.waitForSelector('h2:has-text("Initial Setup")');
    await onboardPage.click('.sol-onboarding-footer button:has-text("Create & Finish Setup")');
    await onboardPage.waitForSelector('h2:has-text("Everything is Ready")');
    await onboardPage.click('.sol-onboarding-footer button:has-text("Enter Solara Launcher")');
    await onboardPage.waitForSelector('.sol-launch-title', { timeout: 10000 });
    await onboardCtx.close();

    assert.equal(errors.length, 0, `Browser errors occurred:\n${errors.join('\n')}`);
    console.log('E2E verification passed: all routes, onboarding, controls, Dawn Gold tokens, responsive viewports, and zero console/page errors verified.');
  } finally {
    if (browser) await browser.close().catch(() => {});
    if (server && server.pid) {
      try {
        process.kill(-server.pid, 'SIGTERM');
      } catch {}
    }
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
