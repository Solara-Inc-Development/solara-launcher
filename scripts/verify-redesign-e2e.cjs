const { chromium } = require('playwright');
const { spawn } = require('node:child_process');
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
  const server = spawn('npx', ['vite', '--port', '4173', '--host', '127.0.0.1'], {
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  let serverOut = '';
  server.stdout.on('data', (d) => (serverOut += d.toString()));
  server.stderr.on('data', (d) => (serverOut += d.toString()));

  let browser;
  try {
    const baseUrl = 'http://127.0.0.1:4173/desktop.html';
    await waitForServer(baseUrl);

    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    const page = await context.newPage();

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
            discord: false,
            theme: 'dawn-gold',
            uiScale: 100,
            reducedMotion: false,
            sidebarExpanded: false,
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
          accounts: [{ id: 'acc-1', username: 'SolaraTester', selected: true }],
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
              return [{ path: 'mods/sodium-0.6.jar', fileName: 'sodium-0.6.jar', name: 'Sodium', version: '0.6.0', modId: 'sodium', enabled: true, size: 950000 }];
            }
            if (cmd === 'instance_packs') return [];
            if (cmd === 'instance_worlds') {
              return [{ folder: 'New World', name: 'Dawn Survival', gameMode: 'Survival', difficulty: 'Normal', versionName: '1.21.4', lastPlayedMs: Date.now() - 10000, size: 12582912, iconDataUrl: null, locked: false, datapackCount: 1 }];
            }
            if (cmd === 'instance_files') {
              return [{ name: 'mods', path: 'mods', directory: true, size: 0, editable: false, image: false }, { name: 'options.txt', path: 'options.txt', directory: false, size: 1200, editable: true, image: false }];
            }
            if (cmd === 'instance_trash') return [];
            if (cmd === 'instance_disk_logs') return [{ name: 'latest.log', size: 512, modifiedMs: Date.now(), gzipped: false }];
            if (cmd === 'instance_disk_log') return '[10:00:00] [main/INFO]: Sound engine started\n[10:00:01] [main/WARN]: Example warn line';
            if (cmd === 'instance_logs') return '[10:00:00] [main/INFO]: Live process';
            if (cmd === 'launcher_screenshots') return [];
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
          body: JSON.stringify({ announcements: [] }),
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

    await page.goto(baseUrl, { waitUntil: 'domcontentloaded' });
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

    // 2. Verify Home Dashboard shows Dawn SMP selected instance
    const homeTitle = await page.textContent('.sol-launch-title');
    assert.equal(homeTitle?.trim(), 'Dawn SMP');

    // 3. Navigate to Library, switch grid/list, create instance
    await page.click('.sol-rail-item[title="Library"]');
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

    // 4. Test Instance Detail tabs (Overview, Content, Files, Worlds, Logs, Screenshots, Settings)
    await page.click('.sol-inst-header-left >> .. >> .. >> button[role="tab"]:has-text("Content")');
    await page.waitForSelector('button:has-text("Mods")');
    await page.click('button[role="tab"]:has-text("Worlds")');
    await page.waitForSelector('.sol-world-name:has-text("Dawn Survival")');
    await page.click('button[role="tab"]:has-text("Logs")');
    await page.waitForSelector('.sol-log-viewer');
    await page.click('button[role="tab"]:has-text("Files")');
    await page.waitForSelector('text=options.txt');

    // 5. Test Discover page & Vanilla compatibility notice
    await page.click('.sol-rail-item[title="Discover"]');
    await page.waitForSelector('.sol-discover-title:has-text("Sodium")');

    // 6. Test Skins 3D Viewer & Editor
    await page.click('.sol-rail-item[title="Skin & Capes"]');
    await page.waitForSelector('.sol-skin-canvas');
    await page.click('button[role="tab"]:has-text("Skin Editor")');
    await page.waitForSelector('.sol-editor-canvas');
    await page.click('button:has-text("Save to Library")');
    await page.waitForTimeout(150);

    // 7. Test Global Settings + Unsaved SaveBar + Dirty Leave Guard
    await page.click('.sol-rail-item[title="Settings"]');
    await page.waitForSelector('.sol-settings-sidebar');
    await page.click('.sol-chip:has-text("Obsidian Slate")');
    await page.waitForSelector('.sol-savebar');
    // Try navigating away with unsaved changes -> leave dialog appears!
    await page.click('.sol-rail-item[title="Home"]');
    await page.waitForSelector('.sol-modal-title:has-text("Unsaved Changes")');
    await page.click('button:has-text("Stay & Keep Editing")');
    await page.click('.sol-savebar button:has-text("Save Changes")');
    await page.waitForTimeout(150);

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
      await page.click('.sol-rail-item[title="Home"]');
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
    server.kill('SIGTERM');
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
