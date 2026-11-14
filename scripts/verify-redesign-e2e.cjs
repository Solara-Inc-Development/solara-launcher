#!/usr/bin/env node
/**
 * End-to-end Playwright verification for Solara Launcher V3 (Obsidian Gold).
 * Spawns `vite preview` over the built dist/ bundle, injects a realistic Tauri
 * IPC mock + offline-safe Modrinth/Solara fixtures, verifies every V3 feature
 * and layout invariant across 1440x900, 1280x720, and 940x560, and writes
 * visual inspection screenshots into scratch/ (plus public/screenshots/dashboard.png).
 */

const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const assert = require('assert');

const ROOT = path.resolve(__dirname, '..');
const PORT = Number(process.env.SOLARA_E2E_PORT || 4179);
const BASE_URL = `http://127.0.0.1:${PORT}/desktop.html`;
const SHOT_DIR =
  process.env.SOLARA_E2E_SHOT_DIR ||
  path.join(ROOT, 'checks/continuation');
const DASHBOARD_SHOT = path.join(ROOT, 'public/screenshots/dashboard.png');

function findPlaywright() {
  for (const candidate of [
    'playwright',
    path.join(ROOT, 'node_modules/playwright'),
    '/usr/lib/node_modules/playwright',
    '/root/.npm-global/lib/node_modules/playwright',
  ]) {
    try {
      return require(candidate);
    } catch {
      // try next
    }
  }
  throw new Error('Playwright module not found.');
}

async function waitForServer(url, timeoutMs = 15000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {
      // retry
    }
    await new Promise((r) => setTimeout(r, 150));
  }
  throw new Error(`Timed out waiting for Vite server at ${url}`);
}

function buildMockInitScript() {
  return () => {
    const samplePng =
      'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAAbElEQVR42u3QMQEAAAgDoGlyo1jBTwI4c2bs3QYBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQGBqwEn2gABpT8c6wAAAABJRU5ErkJggg==';

    const state = {
      prefs: {
        prefsVersion: 2,
        memory: 4096,
        javaPath: '/usr/lib/jvm/java-21-openjdk/bin/java',
        theme: 'obsidian-blue',
        uiScale: 100,
        reducedMotion: false,
        density: 'comfortable',
        onLaunch: 'keep',
        confirmLaunch: false,
        maxConcurrentDownloads: 6,
        defaultWidth: 1280,
        defaultHeight: 720,
        fullscreen: false,
        jvmArgs: '-XX:+UseG1GC',
        diagnosticsEnabled: false,
        announcementsEnabled: true,
        solaraHubEnabled: true,
        autoUpdateEnabled: true,
        onboardingCompleted: true,
        groups: ['Survival', 'Testing'],
      },
      snapshot: {
        dataDir: '/home/solara/.local/share/solara-launcher',
        minecraftAccounts: [
          {
            id: '11111111-2222-3333-4444-555555555555',
            username: 'SolaraPlayer',
            selected: true,
          },
        ],
        instances: [
          {
            schemaVersion: 1,
            id: 'inst-dawn-fabric',
            name: 'Dawn Survival 1.21',
            gameVersion: '1.21.4',
            loader: 'fabric',
            loaderVersion: '0.16.10',
            memory: 4096,
            javaPath: '',
            state: 'installed',
            running: false,
            icon: 'gold',
            customIcon: null,
            group: 'Survival',
            pinned: true,
            favourite: true,
            createdAt: '2026-11-01T00:00:00Z',
            lastPlayed: '2026-11-14T10:00:00Z',
            playtimeSeconds: 18420,
            jvmArgs: null,
            resolution: [1280, 720],
            fullscreen: false,
            preLaunch: null,
            postExit: null,
            updateChannel: 'release',
            notes: 'Main SMP pack with Sodium + Iris.',
          },
          {
            schemaVersion: 1,
            id: 'inst-vanilla-121',
            name: 'Clean Vanilla 1.21.4',
            gameVersion: '1.21.4',
            loader: 'vanilla',
            loaderVersion: null,
            memory: 2048,
            javaPath: '',
            state: 'installed',
            running: false,
            icon: 'ember',
            customIcon: null,
            group: 'Testing',
            pinned: false,
            favourite: false,
            createdAt: '2026-11-05T00:00:00Z',
            lastPlayed: '2026-11-12T15:30:00Z',
            playtimeSeconds: 3600,
            jvmArgs: null,
            resolution: [1280, 720],
            fullscreen: false,
            preLaunch: null,
            postExit: null,
            updateChannel: 'release',
            notes: 'Pure vanilla snapshot verifier.',
          },
        ],
        localServers: [
          {
            id: 'srv-dawn-1',
            name: 'Dawn SMP Local',
            version: '1.21.4',
            port: 25565,
            memory: 4096,
            eulaAccepted: true,
            state: 'stopped',
            createdAt: '2026-11-10T00:00:00Z',
          },
        ],
        downloads: [
          {
            id: 'dl-1',
            instanceId: 'inst-dawn-fabric',
            name: 'Fabric Loader 0.16.10',
            kind: 'install',
            status: 'completed',
            detail: 'Verified',
            downloaded: 100,
            actions: [],
            total: 100,
            createdAt: '2026-11-14T09:00:00Z',
          },
        ],
        javaRuntimes: [
          {
            path: '/usr/lib/jvm/java-21-openjdk/bin/java',
            version: '21.0.5',
            major: 21,
            vendor: 'Eclipse Temurin',
            managed: false,
          },
        ],
      },
      modsByInst: {
        'inst-dawn-fabric': [
          {
            path: 'mods/sodium-fabric-0.6.5.jar',
            fileName: 'sodium-fabric-0.6.5.jar',
            name: 'Sodium',
            version: '0.6.5+mc1.21.4',
            modId: 'sodium',
            loaders: ['forge'],
            enabled: true,
            size: 1240320,
          },
          {
            path: 'mods/fabric-api-0.114.0.jar.disabled',
            fileName: 'fabric-api-0.114.0.jar.disabled',
            name: 'Fabric API',
            version: '0.114.0+1.21.4',
            modId: 'fabric-api',
            enabled: false,
            size: 2450112,
          },
        ],
        'inst-vanilla-121': [],
      },
      packsByInst: {
        'inst-dawn-fabric': [
          {
            kind: 'resourcepacks',
            path: 'resourcepacks/Solara-Dark-UI.zip',
            fileName: 'Solara-Dark-UI.zip',
            name: 'Solara Dark UI',
            description: 'Obsidian Gold dark HUD pack',
            packFormat: 46,
            iconDataUrl: null,
            enabled: true,
            directory: false,
            size: 512000,
            world: null,
          },
        ],
        'inst-vanilla-121': [],
      },
      worldsByInst: {
        'inst-dawn-fabric': [
          {
            folder: 'Dawn_Realm',
            name: 'Dawn Realm',
            gameMode: 'Survival',
            difficulty: 'Hard',
            versionName: '1.21.4',
            lastPlayedMs: 1763113800000,
            size: 14800210,
            iconDataUrl: null,
            locked: false,
            datapackCount: 1,
          },
        ],
        'inst-vanilla-121': [
          {
            folder: 'Vanilla_World',
            name: 'Vanilla World',
            gameMode: 'Survival',
            difficulty: 'Normal',
            versionName: '1.21.4',
            lastPlayedMs: 1762950000000,
            size: 6400100,
            iconDataUrl: null,
            locked: false,
            datapackCount: 0,
          },
        ],
      },
      logsByInst: {
        'inst-dawn-fabric': [
          '[10:12:01] [main/INFO]: Loading Minecraft 1.21.4 with Fabric Loader 0.16.10',
          ...Array.from({ length: 315 }, (_, i) => `[10:12:${String(i % 60).padStart(2, '0')}] [main/INFO]: Loaded resource entry #${i + 1}`),
          '[10:12:02] [main/WARN]: Configuration for sodium uses default low-latency profile',
          '[10:12:03] [main/ERROR]: ModResolutionException: Cannot find mod fabric-api (requires version >=0.114.0)',
        ].join('\n'),
        'inst-vanilla-121': '[09:00:01] [main/INFO]: Sound engine started\n',
      },
      savedSkins: [
        {
          id: 'skin-dawn-knight',
          name: 'Dawn Knight',
          variant: 'classic',
          pngDataUrl: samplePng,
          updatedAt: '2026-11-14T10:00:00Z',
        },
      ],
      solaraProfile: {
        signedIn: true,
        displayName: 'Solara Dev',
        handle: 'solaradev',
        email: 'dev@solara-inc.in',
        avatarUrl: null,
        role: 'Creator',
      },
      invoked: [],
      listeners: new Map(),
      fileEntries: [{ path: 'options.txt', name: 'options.txt', directory: false, size: 2140 }, { path: 'config', name: 'config', directory: true, size: 0 }],
      textFiles: { 'options.txt': { content: 'renderDistance:12', sha256: 'fixture-original-hash' } },
      trash: [{ id: 'trash-fixture', originalPath: 'config/old.cfg', removedAt: '2026-11-14T10:00:00Z', directory: false }],
    };

    window.__SOLARA_TEST_STATE__ = state;
    state.renderCalls = 0;
    state.bufferCreates = 0;
    const drawElements = WebGLRenderingContext.prototype.drawElements;
    WebGLRenderingContext.prototype.drawElements = function (...args) { state.renderCalls++; return drawElements.apply(this, args); };
    const createBuffer = WebGLRenderingContext.prototype.createBuffer;
    WebGLRenderingContext.prototype.createBuffer = function (...args) { state.bufferCreates++; return createBuffer.apply(this, args); };

    const callbacks = new Map();
    let cbId = 1;
    state.emit = (event, payload) => {
      for (const handler of state.listeners.get(event) || []) callbacks.get(handler)?.callback({ event, id: handler, payload });
    };

    window.__TAURI_INTERNALS__ = {
      transformCallback(callback, once) {
        const id = cbId++;
        callbacks.set(id, { callback, once: Boolean(once) });
        return id;
      },
      unregisterCallback(id) {
        callbacks.delete(id);
      },
      convertFileSrc(src) {
        return src;
      },
      metadata: {
        currentWindow: { label: 'main' },
        currentWebview: { label: 'main' },
      },
      async invoke(cmd, args = {}) {
        state.invoked.push({ cmd, args });

        if (cmd === 'plugin:event|listen') {
          const arr = state.listeners.get(args.event) || [];
          arr.push(args.handler);
          state.listeners.set(args.event, arr);
          return args.handler;
        }
        if (cmd.startsWith('plugin:')) return null;

        switch (cmd) {
          case 'initialize_launcher':
            await new Promise((r) => setTimeout(r, 120));
            return state.snapshot;
          case 'launcher_snapshot':
            return state.snapshot;
          case 'launcher_settings':
            return state.prefs;
          case 'save_launcher_settings_full':
            state.prefs = { ...state.prefs, ...(args.preferences || args.prefs || args) };
            return state.prefs;
          case 'app_update_status':
          case 'check_update':
            return (
              state.updateStatus || {
                available: false,
                state: 'up-to-date',
                stage: 'up-to-date',
                version: null,
                publicVersion: null,
                currentVersion: 'V-0.0.38',
                notes: null,
                downloaded: 0,
                total: null,
                error: null,
                checkedAt: '2026-11-14T10:00:00Z',
              }
            );
          case 'install_update':
            state.updateStatus = {
              ...(state.updateStatus || {}),
              state: 'installing',
              stage: 'installing',
              error: null,
            };
            return state.updateStatus;
          case 'solara_profile':
            return state.solaraProfile;
          case 'solara_request':
            if (String(args.path || '').includes('/launcher/bootstrap')) {
              return {
                schemaVersion: 1,
                generatedAt: '2026-11-14T10:00:00Z',
                network: {
                  ip: 'play.solaranetwork.in',
                  address: 'play.solaranetwork.in',
                  name: 'Solara Network',
                  status: 'online',
                  version: '1.21.4',
                  playersOnline: 48,
                  maxPlayers: 200,
                },
                events: [
                  {
                    id: 'ann-1',
                    title: 'Dawn Gold V3 Early Build',
                    summary: 'Clean OLED obsidian workspace with per-world datapack support.',
                    publishedAt: '2026-11-14',
                    url: 'https://app.solara-inc.in',
                  },
                ],
                services: [
                  { id: 'sso', name: 'Solara Auth SSO', url: 'https://auth.solara-inc.in', status: 'operational' },
                  { id: 'api', name: 'Solara Core API', url: 'https://api.solara-inc.in', status: 'operational' },
                ],
              };
            }
            return { ok: true };
          case 'detect_java_runtimes':
            return state.snapshot.javaRuntimes;
          case 'minecraft_versions':
            return [
              { id: '1.21.4', type: 'release', releaseTime: '2024-12-03T10:00:00Z' },
              { id: '1.20.4', type: 'release', releaseTime: '2023-12-07T10:00:00Z' },
            ];
          case 'loader_versions':
            return [
              { version: '0.16.10', stable: true },
              { version: '0.16.9', stable: true },
            ];
          case 'create_instance': {
            const id = 'inst-' + (state.snapshot.instances.length + 1);
            state.snapshot.instances.push({
              id,
              name: args.name || 'New Instance',
              mcVersion: args.mcVersion || '1.21.4',
              loader: args.loader || 'fabric',
              loaderVersion: args.loaderVersion || '0.16.10',
              group: args.group || null,
              iconPreset: args.iconPreset || 'gold-anvil',
              notes: '',
              lastPlayedAt: null,
              totalPlaySeconds: 0,
              running: false,
              javaPath: null,
              javaMemoryMb: 4096,
              jvmArgs: null,
              resolutionWidth: 1280,
              resolutionHeight: 720,
              fullscreen: false,
              preLaunchCommand: null,
              postExitCommand: null,
              envVars: {},
            });
            state.modsByInst[id] = [];
            state.packsByInst[id] = [];
            state.worldsByInst[id] = [];
            state.logsByInst[id] = '';
            return { jobId: 'job-' + id, instanceId: id };
          }
          case 'update_instance_meta': {
            const instId = args.instanceId || args.id;
            const inst = state.snapshot.instances.find((i) => i.id === instId);
            if (inst) Object.assign(inst, args.patch || args);
            return inst;
          }
          case 'launch_instance': {
            const instId = args.instanceId || args.id;
            const inst = state.snapshot.instances.find((i) => i.id === instId);
            await new Promise((r) => setTimeout(r, 80));
            if (state.launchFailure) throw new Error('Fixture Java failed before spawning');
            if (inst) inst.running = true;
            return { processId: 42420 };
          }
          case 'stop_instance': {
            const instId = args.instanceId || args.id;
            const inst = state.snapshot.instances.find((i) => i.id === instId);
            if (inst) inst.running = false;
            state.emit('game-exited', { instanceId: instId, success: true, exitCode: 0 });
            return null;
          }
          case 'instance_mods':
            return state.modsByInst[args.instanceId || args.id] || [];
          case 'toggle_instance_mod': {
            const mods = state.modsByInst[args.instanceId || args.id] || [];
            const m = mods.find((x) => x.fileName === args.fileName || x.path === args.path);
            if (!m) throw new Error('Mod not found');
            if (m.enabled === Boolean(args.enabled)) throw new Error('Redundant toggle');
            m.enabled = Boolean(args.enabled);
            return null;
          }
          case 'check_instance_content_updates':
            return state.updateCandidates || [];
          case 'apply_instance_content_updates': {
            if (state.updateFailure) throw new Error('Fixture content update interrupted');
            for (const item of args.items) {
              const mod = state.modsByInst[args.id].find((m) => m.path === item.currentPath);
              if (!mod) throw new Error('Update target missing');
              mod.version = item.latestVersionNumber;
            }
            state.updateCandidates = [];
            return { updated: args.items.length };
          }
          case 'instance_packs': {
            const list = state.packsByInst[args.instanceId || args.id] || [];
            return args.kind ? list.filter((p) => p.kind === args.kind) : list;
          }
          case 'instance_worlds':
            return state.worldsByInst[args.instanceId || args.id] || [];
          case 'instance_files': return state.fileEntries;
          case 'instance_text':
            return { path: args.path, ...state.textFiles[args.path] };
          case 'instance_trash': return state.trash;
          case 'instance_file_action': {
            if (args.action === 'save') {
              const file = state.textFiles[args.path];
              if (args.expectedSha256 !== file.sha256) throw new Error('This file changed since you opened it. Reload it before saving.');
              file.content = args.content;
              file.sha256 = 'fixture-saved-hash';
              return { sha256: file.sha256 };
            }
            if (args.action === 'mkdir') {
              if (state.fileEntries.some((f) => f.path === args.path)) throw new Error('Folder already exists');
              state.fileEntries.push({ path: args.path, name: args.path.split('/').at(-1), directory: true, size: 0 }); return { ok: true };
            }
            if (args.action === 'rename') {
              const file = state.fileEntries.find((f) => f.path === args.path);
              if (!file) throw new Error('Missing file');
              file.path = args.target; file.name = args.target.split('/').at(-1); return { ok: true };
            }
            if (args.action === 'restore') {
              const item = state.trash.find((t) => t.id === args.path);
              if (item?.entry) state.fileEntries.push(item.entry);
              state.trash = state.trash.filter((t) => t.id !== args.path); return { ok: true };
            }
            if (args.action === 'trash') {
              const item = state.fileEntries.find((f) => f.path === args.path);
              state.fileEntries = state.fileEntries.filter((f) => f.path !== args.path);
              state.trash.push({ id: 'fixture-trash-id', originalPath: args.path, directory: item?.directory ?? false, entry: item });
              return { trashId: 'fixture-trash-id' };
            }
            throw new Error('Unsupported file action: ' + args.action);
          }
          case 'instance_backups':
            return [];
          case 'instance_logs':
            return state.logsByInst[args.instanceId || args.id] || '';
          case 'instance_disk_logs':
            return [{ name: 'latest.log', size: 1200, modifiedMs: 1763115123000, gzipped: false }];
          case 'instance_disk_log':
            return state.logsByInst[args.instanceId || args.id] || '';
          case 'launcher_screenshots':
            return Array.from({ length: 80 }, (_, index) => ({
              instanceId: 'inst-dawn-fabric', instanceName: 'Dawn Survival 1.21',
              fileName: `fixture-${index}.png`, path: `screenshots/fixture-${index}.png`,
              fullPath: `/fixture/screenshots/fixture-${index}.png`, size: 1024, modifiedMs: index,
            }));
          case 'launcher_screenshot_image': {
            const canvas = document.createElement('canvas'); canvas.width = 640; canvas.height = 360;
            const ctx = canvas.getContext('2d'); ctx.fillStyle = '#E9B86D'; ctx.fillRect(0, 0, 640, 360);
            return canvas.toDataURL('image/png');
          }
          case 'saved_skins_list':
            return state.savedSkins;
          case 'saved_skins_save': {
            const item = {
              id: args.id || 'skin-' + (state.savedSkins.length + 1),
              name: args.name || 'Saved Skin',
              variant: args.variant || 'classic',
              pngDataUrl: args.pngDataUrl || samplePng,
            };
            const idx = state.savedSkins.findIndex((s) => s.id === item.id);
            if (idx >= 0) state.savedSkins[idx] = item;
            else state.savedSkins.push(item);
            return item;
          }
          case 'profile_skins_and_capes':
            return {
              skins: [{ id: 'mc-skin-1', state: 'ACTIVE', variant: 'CLASSIC', dataUrl: samplePng }],
              capes: [{ id: 'cape-migrator', state: 'ACTIVE', alias: 'Migrator', dataUrl: samplePng }],
            };
          case 'launcher_storage_summary':
            return {
              dataDir: state.snapshot.dataDir,
              totalBytes: 420_000_000,
              instancesBytes: 300_000_000,
              cacheBytes: 64_000_000,
              logsBytes: 12_000_000,
            };
          case 'install_content': {
            if (!args.instanceId || !args.versionId) throw new Error('Missing content install target/version');
            if (args.contentType === 'datapack' && !args.world) throw new Error('Target world required');
            const jobId = 'fixture-content-' + state.snapshot.downloads.length;
            state.snapshot.downloads.push({ id: jobId, instanceId: args.instanceId, name: 'Terralith Worldgen', kind: 'content', status: 'queued', detail: 'Waiting to start', downloaded: 0, total: 1024, actions: ['cancel'] });
            return { instanceId: args.instanceId, jobId };
          }
          case 'download_action': {
            const job = state.snapshot.downloads.find((j) => j.id === args.id);
            if (!job || !job.actions.includes(args.action)) throw new Error('Unsupported task action');
            job.status = args.action === 'cancel' ? 'cancelled' : 'queued';
            job.actions = args.action === 'cancel' ? ['retry'] : ['cancel'];
            return null;
          }
          case 'clear_finished_downloads':
            state.snapshot.downloads = state.snapshot.downloads.filter((j) => ['queued', 'running'].includes(j.status));
            return null;
          case 'create_local_server': {
            const srv = {
              id: 'srv-' + (state.snapshot.localServers.length + 1),
              name: args.name || 'Local Server',
              software: args.software || 'paper',
              mcVersion: args.mcVersion || '1.21.4',
              port: Number(args.port || 25566),
              memoryMb: Number(args.memoryMb || 2048),
              running: false,
              eulaAccepted: Boolean(args.eula),
            };
            state.snapshot.localServers.push(srv);
            return srv;
          }
          case 'local_server_logs':
            return '[10:00:00] [Server thread/INFO]: Done (2.14s)! For help, type "help"\n';
          case 'window_action':
          case 'start_dragging':
          case 'open_instance_folder':
          case 'open_data_folder':
            return null;
          default:
            throw new Error('Unmocked native command: ' + cmd);
        }
      },
    };
    const invoke = window.__TAURI_INTERNALS__.invoke;
    window.__TAURI_INTERNALS__.invoke = async (...args) => structuredClone(await invoke(...args));
  };
}

async function run() {
  fs.mkdirSync(SHOT_DIR, { recursive: true });
  const { chromium } = findPlaywright();

  const vite = spawn(
    'npx',
    ['vite', 'preview', '--host', '127.0.0.1', '--port', String(PORT), '--strictPort'],
    { cwd: ROOT, stdio: 'ignore' },
  );

  let browser;
  try {
    await waitForServer(BASE_URL);
    browser = await chromium.launch({ headless: true });

    const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });

    // Offline-safe deterministic Solara API, avatar & Modrinth fixtures
    await context.route('https://api.solara-inc.in/**', async (route) => {
      const url = route.request().url();
      const headers = { 'access-control-allow-origin': '*' };
      if (url.includes('/v1/launcher/bootstrap')) {
        return route.fulfill({
          status: 200,
          headers,
          contentType: 'application/json',
          body: JSON.stringify({
            schemaVersion: 1,
            generatedAt: '2026-11-14T10:00:00Z',
            network: {
              ip: 'play.solaranetwork.in',
              address: 'play.solaranetwork.in',
              name: 'Solara Network',
              status: 'online',
              version: '1.21.4',
              playersOnline: 48,
              maxPlayers: 200,
            },
            events: [
              {
                id: 'ann-1',
                title: 'Dawn Gold V3 Early Build',
                summary: 'Clean OLED obsidian workspace with per-world datapack support.',
                publishedAt: '2026-11-14',
                url: 'https://app.solara-inc.in',
              },
            ],
            services: [
              { id: 'sso', name: 'Solara Auth SSO', url: 'https://auth.solara-inc.in', status: 'operational' },
              { id: 'api', name: 'Solara Core API', url: 'https://api.solara-inc.in', status: 'operational' },
            ],
          }),
        });
      }
      return route.fulfill({
        status: 200,
        headers,
        contentType: 'application/json',
        body: JSON.stringify({
          announcements: [
            {
              id: 'ann-1',
              title: 'Dawn Gold V3 Ready',
              body: 'All Solara Launcher services online.',
              publishedAt: '2026-11-14',
            },
          ],
        }),
      });
    });

    await context.route(/https:\/\/(mc-heads\.net|minotar\.net|crafatar\.com)\/.*/, async (route) => {
      // 1x1 Dawn Gold #E9B86D PNG
      const goldPng = Buffer.from(
        'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8f5nhPwAHqgLh6h6f8QAAAABJRU5ErkJggg==',
        'base64',
      );
      return route.fulfill({ status: 200, contentType: 'image/png', body: goldPng });
    });

    await context.route('https://api.modrinth.com/**', async (route) => {
      const url = decodeURIComponent(route.request().url());
      if (url.includes('/v2/search')) {
        const isModpack = url.includes('project_type:modpack');
        const isShader = url.includes('project_type:shader');
        const isDatapack = url.includes('project_type:datapack');
        const hit = isModpack
          ? {
              project_id: 'mp-solara-opt',
              slug: 'fabulously-optimized',
              title: 'Dawn Performance Pack',
              description: 'Curated Fabric optimization pack ready for 1.21.4.',
              downloads: 4200000,
              follows: 52000,
              icon_url: '',
              project_type: 'modpack',
              categories: ['optimization'],
              versions: ['1.21.4'],
            }
          : isShader
          ? {
              project_id: 'sh-complementary',
              slug: 'complementary-reimagined',
              title: 'Solstice Dawn Shaders',
              description: 'Balanced atmospheric shaders requiring Iris or Oculus.',
              downloads: 1850000,
              follows: 21000,
              icon_url: '',
              project_type: 'shader',
              categories: ['atmospheric'],
              versions: ['1.21.4'],
            }
          : isDatapack
          ? {
              project_id: 'dp-terralith',
              slug: 'terralith',
              title: 'Terralith Worldgen',
              description: 'Overworld biome overhaul installed per world save.',
              downloads: 2900000,
              follows: 33000,
              icon_url: '',
              project_type: 'datapack',
              categories: ['worldgen'],
              versions: ['1.21.4'],
            }
          : {
              project_id: 'mod-lithium',
              slug: 'lithium',
              title: 'Lithium',
              description: 'No-compromises game logic optimization mod.',
              downloads: 9100000,
              follows: 48000,
              icon_url: '',
              project_type: 'mod',
              categories: ['optimization'],
              versions: ['1.21.4'],
            };
        return route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ hits: [hit], total_hits: 1, limit: 20, offset: 0 }),
        });
      }
      if (url.includes('/v2/project/')) {
        const parts = url.split('/v2/project/')[1].split('/');
        const id = parts[0];
        if (parts[1]?.startsWith('version')) {
          return route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify([
              {
                id: 'ver-' + id,
                name: '1.21.4 Release',
                version_number: '1.2.0',
                game_versions: ['1.21.4'],
                loaders: [id.startsWith('sh-') ? 'iris' : id.startsWith('dp-') ? 'datapack' : 'fabric'],
                date_published: '2026-11-01T00:00:00Z',
              },
            ]),
          });
        }
        return route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            id,
            title: id,
            description: 'Verified fixture project.',
            body: '<p>Full Markdown/HTML details preview.</p>',
            downloads: 120000,
            followers: 4200,
            gallery: [],
          }),
        });
      }
      return route.fulfill({ status: 200, contentType: 'application/json', body: '{}' });
    });

    await context.addInitScript(buildMockInitScript());
    const page = await context.newPage();
    page.setDefaultTimeout(8000);

    const errors = [];
    page.on('pageerror', (e) => {
      errors.push(`pageerror: ${e.message}`);
      console.error('PAGEERROR:', e.message);
    });
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        errors.push(`console.error: ${msg.text()}`);
        console.error('CONSOLE ERROR:', msg.text());
      }
    });

    await page.goto(BASE_URL, { waitUntil: 'domcontentloaded' });

    // Verify splash screen appears first with #000000 OLED ground
    const splashSeen = await page.locator('.sol-splash').isVisible().catch(() => false);
    assert(splashSeen, 'Expected Startup SplashScreen on initial boot');

    // Wait for Home Hero after initialization settles
    try {
      await page.waitForSelector('.sol-launch-hero-actions button:has-text("Play")', { timeout: 10000 });
    } catch (e) {
      const diag = await page.evaluate(() => ({
        text: document.body.innerText.slice(0, 400),
        html: document.body.innerHTML.slice(0, 600),
        invoked: window.__SOLARA_TEST_STATE__?.invoked?.map((x) => x.cmd),
      }));
      console.error('TIMEOUT DIAG:', JSON.stringify(diag, null, 2));
      throw e;
    }
    await page.waitForTimeout(150);

    // Verify Theme Palette, 84px rail, 56px TitleBar, and Zero Document Scroll
    const layoutMetrics = await page.evaluate(() => {
      const rootStyle = getComputedStyle(document.documentElement);
      const bodyStyle = getComputedStyle(document.body);
      const rail = document.querySelector('.sol-rail');
      const titlebar = document.querySelector('.sol-titlebar');
      return {
        bgRoot: rootStyle.getPropertyValue('--bg-main').trim().toLowerCase(),
        accent: rootStyle.getPropertyValue('--accent').trim().toLowerCase(),
        bodyBg: bodyStyle.backgroundColor,
        railW: rail ? rail.getBoundingClientRect().width : 0,
        titlebarH: titlebar ? titlebar.getBoundingClientRect().height : 0,
        docOverflowX: document.documentElement.scrollWidth > window.innerWidth,
        docOverflowY: document.documentElement.scrollHeight > window.innerHeight,
        hasTasksAnchor: Boolean(document.querySelector('.sol-tasks-anchor')),
        hasAccountBtn: Boolean(document.querySelector('.sol-rail-account')),
      };
    });

    assert.strictEqual(layoutMetrics.bgRoot, '#000000', 'OLED --bg-main must be #000000');
    assert.strictEqual(layoutMetrics.accent, '#3b82f6', 'Cobalt Blue --accent must be #3B82F6');
    assert.strictEqual(layoutMetrics.bodyBg, 'rgb(0, 0, 0)', 'Body background must be pure OLED black');
    assert(Math.abs(layoutMetrics.railW - 84) <= 2, `Expected 84px SidebarRail, got ${layoutMetrics.railW}`);
    assert(Math.abs(layoutMetrics.titlebarH - 56) <= 2, `Expected 56px TitleBar, got ${layoutMetrics.titlebarH}`);
    assert(!layoutMetrics.docOverflowX && !layoutMetrics.docOverflowY, 'Zero document-level overflow expected');
    assert(layoutMetrics.hasTasksAnchor, 'TitleBar must expose .sol-tasks-anchor');
    assert(layoutMetrics.hasAccountBtn, 'SidebarRail must expose .sol-rail-account');

    // Home Screenshot (1440x900)
    await page.screenshot({ path: path.join(SHOT_DIR, 'e2e-home.png') });
    await page.screenshot({ path: DASHBOARD_SHOT });

    // Verify Play launches with Keep Launcher Open default
    await page.click('.sol-launch-hero-actions button:has-text("Play")');
    await page.waitForTimeout(100);
    const launchCalls = await page.evaluate(() =>
      window.__SOLARA_TEST_STATE__.invoked.filter((c) => c.cmd === 'launch_instance'),
    );
    assert(launchCalls.length >= 1, 'Expected launch_instance IPC when clicking Hero Play');
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.filter((c) => c.cmd === 'window_action' && ['close', 'hide', 'minimize'].includes(c.args.action)).length), 0, 'Default Play must leave the launcher open');
    await page.click('.sol-launch-hero-actions button:has-text("Stop")');
    await page.waitForSelector('.sol-launch-hero-actions button:has-text("Play")');
    // Two entry points cannot start duplicate sessions during the same launch.
    const beforeLaunchCount = launchCalls.length;
    await page.evaluate(() => { const button = document.querySelector('.sol-jump-play'); button.click(); button.click(); });
    await page.waitForSelector('.sol-launch-hero-actions button:has-text("Stop")');
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.filter((c) => c.cmd === 'launch_instance').length), beforeLaunchCount + 1);
    await page.evaluate(() => {
      const state = window.__SOLARA_TEST_STATE__;
      state.snapshot.instances[0].running = false;
      state.emit('game-exited', { instanceId: state.snapshot.instances[0].id, success: false, exitCode: 7 });
    });
    await page.waitForSelector('.sol-launch-stage-err:has-text("code 7")');
    assert(await page.locator('.sol-launch-stage button:has-text("View Logs")').isVisible(), 'Crash must retain a useful logs action');
    await page.evaluate(() => { window.__SOLARA_TEST_STATE__.launchFailure = true; });
    await page.click('.sol-launch-hero-actions button:has-text("Play")');
    await page.locator('.sol-launch-stage-err').filter({ hasText: 'Fixture Java failed before spawning' }).waitFor();
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.filter((c) => c.cmd === 'window_action' && ['close', 'hide', 'minimize'].includes(c.args.action)).length), 0, 'A failed launch must also leave the launcher open');
    await page.evaluate(() => { window.__SOLARA_TEST_STATE__.launchFailure = false; });



    // Library Page + New Group + Right-click Context Menu
    await page.click('button[data-tooltip="Library"]');
    await page.waitForSelector('.sol-inst-grid', { timeout: 5000 });
    await page.click('.sol-inst-card >> nth=0', { button: 'right' });
    await page.waitForSelector('.sol-context-fixed', { timeout: 3000 });
    const ctxWidth = await page.locator('.sol-context-fixed').evaluate((el) => el.getBoundingClientRect().width);
    assert(ctxWidth <= 240, `Expected compact context menu width <= 240px, got ${ctxWidth}`);
    await page.screenshot({ path: path.join(SHOT_DIR, 'e2e-library.png') });
    await page.keyboard.press('Escape').catch(() => {});
    await page.click('body', { position: { x: 10, y: 100 } });

    // Open Create Instance Modal briefly
    await page.click('button:has-text("New Instance")');
    await page.waitForSelector('.sol-modal-title', { timeout: 3000 });
    await page.click('button[aria-label="Close dialog"]');

    // Open Instance Detail -> Content Default Tab -> Bulk Bar + Compatibility Check
    await page.click('.sol-inst-card >> nth=0');
    await page.waitForSelector('.sol-inst-header', { timeout: 5000 });
    const firstTabActive = await page.locator('.sol-inst-tab.is-active').textContent();
    assert(firstTabActive && firstTabActive.includes('Content'), `Expected Content default tab, got ${firstTabActive}`);
    await page.click('button:has-text("Compatibility")');
    await page.getByText(/Sodium is marked for forge, but this instance uses Fabric/).waitFor();
    await page.screenshot({ path: path.join(SHOT_DIR, 'e2e-instance-content.png') });
    await page.locator('input[aria-label="Select Sodium"]').check();
    await page.locator('input[aria-label="Select Fabric API"]').check();
    await page.click('.sol-bulk-bar button:has-text("Enable")');
    await page.waitForFunction(() => window.__SOLARA_TEST_STATE__.modsByInst['inst-dawn-fabric'].every((m) => m.enabled));
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.filter((c) => c.cmd === 'toggle_instance_mod').length), 1, 'Bulk Enable should only toggle disabled mods');
    await page.locator('.sol-inst-header-right').getByRole('button', { name: 'Settings', exact: true }).click();
    await page.locator('.sol-settings-dialog textarea').fill('');
    await page.getByRole('button', { name: 'Save Changes', exact: true }).click();
    await page.waitForSelector('.sol-savebar', { state: 'detached' });
    const notePatch = await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.findLast((c) => c.cmd === 'update_instance_meta').args.patch);
    assert.deepStrictEqual(notePatch, { notes: null }, 'Saving notes must clear them without changing memory/window overrides');
    await page.click('button[aria-label="Close dialog"]');
    await page.evaluate(() => {
      const state = window.__SOLARA_TEST_STATE__;
      state.updateCandidates = state.modsByInst['inst-dawn-fabric'].map((m, i) => ({
        currentPath: m.path, currentFileName: m.fileName, projectId: 'fixture-project-' + i, projectName: m.name,
        currentVersionId: 'old-' + i, currentVersionNumber: m.version, latestVersionId: 'new-' + i,
        latestVersionNumber: '2.0.0', latestVersionName: 'Fixture update', releaseChannel: 'release',
        changelog: 'Fixture changelog', destination: m.path, downloadUrl: 'https://cdn.modrinth.com/fixture.jar', sha1: 'a'.repeat(40), sha512: 'b'.repeat(128), size: 1024,
      }));
      state.updateFailure = true;
    });
    await page.getByRole('button', { name: 'Update All', exact: true }).click();
    await page.locator('.sol-updates-grid').waitFor();
    await page.getByRole('button', { name: 'Update 2 Selected', exact: true }).click();
    await page.getByRole('button', { name: 'Check Again', exact: true }).waitFor();
    assert(await page.locator('.sol-updates-grid').isVisible(), 'A failed update must retain recovery controls');
    await page.evaluate(() => { window.__SOLARA_TEST_STATE__.updateFailure = false; });
    await page.getByRole('button', { name: 'Check Again', exact: true }).click();
    await page.getByRole('button', { name: 'Update 2 Selected', exact: true }).click();
    await page.waitForSelector('.sol-updates-grid', { state: 'detached' });
    assert(await page.evaluate(() => window.__SOLARA_TEST_STATE__.modsByInst['inst-dawn-fabric'].every((m) => m.version === '2.0.0')));



    // Built-in Instance Content Browser -> Check Shaders Iris note & Datapacks Target World selector
    await page.click('button:has-text("Browse Content")');
    await page.waitForSelector('.sol-browser-tabs', { timeout: 5000 });
    await page.click('.sol-browser-tab:has-text("Shaders")');
    await page.waitForSelector('text=Iris', { timeout: 3000 });
    await page.click('.sol-browser-tab:has-text("Datapacks")');
    await page.waitForSelector('.sol-datapack-target', { timeout: 5000 });
    await page.screenshot({ path: path.join(SHOT_DIR, 'e2e-content-browser.png') });
    const datapackRow = page.locator('.sol-browser-row').filter({ hasText: 'Terralith Worldgen' });
    await datapackRow.getByRole('button', { name: 'Install', exact: true }).click();
    await datapackRow.getByRole('button', { name: 'Queued', exact: true }).waitFor();
    assert.strictEqual(await datapackRow.getByRole('button', { name: 'Installed', exact: true }).count(), 0, 'Queueing is not installation');
    const installArgs = await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.findLast((c) => c.cmd === 'install_content').args);
    assert.strictEqual(installArgs.contentType, 'datapack'); assert.strictEqual(installArgs.world, 'Dawn_Realm'); assert.strictEqual(installArgs.instanceId, 'inst-dawn-fabric');
    await page.evaluate(() => { const j = window.__SOLARA_TEST_STATE__.snapshot.downloads.at(-1); j.status = 'failed'; j.detail = 'Fixture network interruption'; j.actions = ['retry']; });
    await datapackRow.getByRole('button', { name: 'Install', exact: true }).waitFor({ timeout: 6500 });
    await datapackRow.getByRole('button', { name: 'Install', exact: true }).click();
    await datapackRow.getByRole('button', { name: 'Queued', exact: true }).waitFor();
    await page.evaluate(() => { const j = window.__SOLARA_TEST_STATE__.snapshot.downloads.at(-1); j.status = 'completed'; j.downloaded = j.total; j.actions = []; });
    await datapackRow.getByRole('button', { name: 'Installed', exact: true }).waitFor({ timeout: 6500 });
    await page.click('button[aria-label="Close dialog"]');

    // File editor must read the native object and send an optimistic save.
    await page.click('.sol-inst-tab:has-text("Files")');
    await page.click('.sol-row-item:has-text("options.txt")');
    assert.strictEqual(await page.locator('.sol-code-textarea').inputValue(), 'renderDistance:12');
    await page.locator('.sol-code-textarea').fill('renderDistance:16');
    await page.click('button:has-text("Save File")');
    await page.waitForSelector('.sol-editor-pane', { state: 'detached' });
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.textFiles['options.txt'].content), 'renderDistance:16');
    await page.click('.sol-row-item:has-text("options.txt")');
    await page.locator('.sol-code-textarea').fill('renderDistance:20');
    await page.evaluate(() => { window.__SOLARA_TEST_STATE__.textFiles['options.txt'].sha256 = 'external-change-hash'; });
    await page.click('button:has-text("Save File")');
    await page.getByText(/This file changed since you opened it/).waitFor();
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.textFiles['options.txt'].content), 'renderDistance:16', 'Conflicting save must preserve existing file');
    await page.click('.sol-inst-tab:has-text("Content")');
    await page.getByRole('dialog', { name: 'Unsaved Changes', exact: true }).waitFor();
    await page.getByRole('button', { name: 'Discard & Leave', exact: true }).click();
    await page.click('.sol-inst-tab:has-text("Files")');
    await page.getByRole('button', { name: 'Trash (1)', exact: true }).click();
    await page.getByRole('button', { name: 'Restore', exact: true }).click();
    await page.getByText('Instance trash is empty.', { exact: true }).waitFor();
    await page.getByRole('button', { name: 'Hide Trash', exact: true }).click();
    await page.getByRole('button', { name: 'New Folder', exact: true }).click();
    await page.locator('#instance-file-name').fill('QA Folder');
    await page.getByRole('button', { name: 'Save Name', exact: true }).click();
    await page.getByRole('button', { name: 'Rename QA Folder', exact: true }).click();
    await page.locator('#instance-file-name').fill('QA Renamed');
    await page.getByRole('button', { name: 'Save Name', exact: true }).click();
    await page.getByRole('button', { name: 'Move QA Renamed to trash', exact: true }).click();
    await page.getByRole('button', { name: 'Move to Trash', exact: true }).click();
    await page.getByRole('button', { name: 'Trash (1)', exact: true }).click();
    await page.getByRole('button', { name: 'Restore', exact: true }).click();
    await page.getByRole('button', { name: 'Hide Trash', exact: true }).click();
    await page.getByRole('button', { name: 'Rename QA Renamed', exact: true }).waitFor();



    // Logs Tab -> Level filter + Timestamps + Diagnose Log (Smart Crash Assistant)
    await page.click('.sol-inst-tab:has-text("Logs")');
    await page.waitForSelector('.sol-log-window-bar', { timeout: 5000 });
    await page.click('button:has-text("ERROR")');
    await page.click('button:has-text("Diagnose Log")');
    await page.waitForSelector('text=Missing or incompatible mod dependency', { timeout: 3000 });
    await page.evaluate(() => { window.__SOLARA_TEST_STATE__.logsByInst['inst-dawn-fabric'] += '\n[10:13:00] [main/FATAL]: Fixture live crash line'; });
    await page.locator('.sol-log-line').filter({ hasText: 'Fixture live crash line' }).waitFor({ timeout: 4500 });
    await page.getByRole('button', { name: 'Pause', exact: true }).click();
    await page.evaluate(() => { window.__SOLARA_TEST_STATE__.logsByInst['inst-dawn-fabric'] += '\n[10:13:01] [main/ERROR]: Fixture paused line'; });
    await page.waitForTimeout(1700);
    assert.strictEqual(await page.locator('.sol-log-line').filter({ hasText: 'Fixture paused line' }).count(), 0);
    await page.getByRole('button', { name: 'Resume', exact: true }).click();
    await page.locator('.sol-log-line').filter({ hasText: 'Fixture paused line' }).waitFor();
    await page.getByRole('button', { name: 'Clear', exact: true }).click();
    await page.waitForTimeout(1700);
    assert.strictEqual(await page.locator('.sol-log-line').count(), 0, 'Clear must survive the next poll');
    await page.evaluate(() => { window.__SOLARA_TEST_STATE__.logsByInst['inst-dawn-fabric'] += '\n[10:13:02] [main/ERROR]: Fixture after clear'; });
    await page.locator('.sol-log-line').filter({ hasText: 'Fixture after clear' }).waitFor({ timeout: 4500 });


    // Instance screenshots use the existing bounded cache, including images after #24.
    await page.click('.sol-inst-tab:has-text("Screenshots")');
    await page.waitForSelector('.sol-shot-img');
    const readsAtTop = await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.filter((c) => c.cmd === 'launcher_screenshot_image').length);
    assert(readsAtTop < 80, 'Opening screenshots must not eagerly read all files');
    await page.locator('.sol-shot-card').nth(79).scrollIntoViewIfNeeded();
    await page.locator('.sol-shot-card').nth(79).locator('img').waitFor();
    await page.locator('.sol-shot-card').nth(79).click();
    await page.locator('.sol-lightbox-img').waitFor();
    assert(await page.getByRole('button', { name: 'Copy file path', exact: true }).isVisible());
    await page.locator('.sol-dialog-box').getByRole('button', { name: 'Close', exact: true }).click();

    // Discover Page -> Modpacks ONLY (and verify rail hover tooltip opens + dismisses on click)
    await page.hover('button[data-tooltip="Discover Modpacks"]');
    await page.waitForSelector('.sol-rail-tooltip', { timeout: 2000 });
    await page.click('button[data-tooltip="Discover Modpacks"]');
    await page.waitForSelector('h1:has-text("Discover Modpacks")', { timeout: 5000 });
    assert.strictEqual(await page.locator('.sol-rail-tooltip').count(), 0, 'Rail tooltip must dismiss immediately on click');
    await page.waitForSelector('text=Dawn Performance Pack', { timeout: 5000 });
    const hasModTypeTabs = await page.locator('button:has-text("Shaders"), button:has-text("Datapacks")').count();
    assert.strictEqual(hasModTypeTabs, 0, 'Global Discover must ONLY browse Modpacks');
    await page.screenshot({ path: path.join(SHOT_DIR, 'e2e-discover.png') });

    // Skin Studio -> Edit Skin button, Add Skin tile, Default Starter Skins, Local 3D Preview Capes
    await page.click('button[data-tooltip="Skins & Capes"]');
    await page.waitForSelector('.sol-skin-add-tile', { timeout: 5000 });
    await page.waitForSelector('text=Default Starter Skins', { timeout: 5000 });
    await page.click('button:has-text("Capes")');
    await page.waitForSelector('text=Local 3D Preview Capes', { timeout: 5000 });
    await page.waitForSelector('.sol-cape-preview-badge', { timeout: 3000 });
    await page.click('.sol-capes-grid >> nth=1 >> article >> nth=0');
    await page.waitForSelector('.sol-cape-preview-note', { timeout: 3000 });
    const capeTitleWrapHeight = await page.locator('.sol-capes-grid .sol-ellipsis').first().evaluate((el) => el.getBoundingClientRect().height);
    assert(capeTitleWrapHeight <= 28, `Expected single-line .sol-ellipsis cape title, got height ${capeTitleWrapHeight}`);
    await page.screenshot({ path: path.join(SHOT_DIR, 'e2e-skins.png') });
    await page.getByRole('button', { name: 'Pause Walk', exact: true }).click();
    await page.waitForTimeout(150);
    const pausedRender = await page.evaluate(() => ({ draws: window.__SOLARA_TEST_STATE__.renderCalls, buffers: window.__SOLARA_TEST_STATE__.bufferCreates }));
    await page.waitForTimeout(400);
    assert.deepStrictEqual(await page.evaluate(() => ({ draws: window.__SOLARA_TEST_STATE__.renderCalls, buffers: window.__SOLARA_TEST_STATE__.bufferCreates })), pausedRender, 'Paused skin must stop drawing and allocating GPU buffers');
    assert(pausedRender.draws > 0, 'The skin must have actually rendered in WebGL');
    await page.getByRole('button', { name: 'Zoom in', exact: true }).click();
    await page.waitForFunction((before) => window.__SOLARA_TEST_STATE__.renderCalls > before, pausedRender.draws);
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.bufferCreates), pausedRender.buffers, 'Camera changes should reuse GPU mesh buffers');
    await page.getByRole('button', { name: 'Edit Skin', exact: true }).click();
    await page.locator('.sol-editor-canvas').waitFor();
    await page.waitForTimeout(100);
    await page.locator('.sol-editor-canvas').click({ position: { x: 18, y: 18 } });
    await page.getByText('Unsaved pixels', { exact: true }).waitFor();
    await page.getByRole('button', { name: 'Save to Library', exact: true }).click();
    await page.waitForFunction(() => window.__SOLARA_TEST_STATE__.invoked.some((c) => c.cmd === 'saved_skins_save' && c.args.pngDataUrl.startsWith('data:image/png;base64,')));
    await page.waitForSelector('text=Unsaved pixels', { state: 'detached' });


    // Solara Hub -> Network IP + Create Server
    await page.click('button[data-tooltip="Solara Hub & Servers"]');
    await page.waitForSelector('text=play.solaranetwork.in', { timeout: 5000 });
    await page.screenshot({ path: path.join(SHOT_DIR, 'e2e-solara-hub.png') });

    // Centered Global Settings Modal -> Check Behaviour & Privacy tabs & Save Bar
    await page.click('button[data-tooltip="Settings"]');
    await page.waitForSelector('.sol-settings-dialog', { timeout: 5000 });
    await page.click('.sol-settings-nav-btn:has-text("Behaviour")');
    await page.waitForSelector('button:has-text("Keep Launcher Open")', { timeout: 3000 });
    assert.strictEqual(await page.locator('.sol-settings-dialog button:has-text("Close Launcher")').count(), 0, 'Close Launcher option must not exist without tray mode');
    assert.strictEqual(await page.locator('.sol-settings-dialog button:has-text("Hide")').count(), 0, 'Hide option must not exist without tray mode');
    await page.locator('.sol-settings-dialog button:has-text("Minimize Launcher")').click();
    await page.getByRole('button', { name: 'Save Changes', exact: true }).click();
    await page.waitForSelector('.sol-savebar', { state: 'detached' });
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.prefs.onLaunch), 'minimize');
    await page.click('button[aria-label="Close dialog"]');

    // With Minimize Launcher enabled, Play from Home triggers window_action minimize (and never close/hide) after running
    await page.click('button[data-tooltip="Home"]');
    await page.waitForSelector('.sol-launch-hero-actions button:has-text("Play")', { timeout: 5000 });
    const beforeMinCalls = await page.evaluate(() =>
      window.__SOLARA_TEST_STATE__.invoked.filter((c) => c.cmd === 'window_action' && c.args.action === 'minimize').length,
    );
    await page.click('.sol-launch-hero-actions button:has-text("Play")');
    await page.waitForSelector('.sol-launch-hero-actions button:has-text("Stop")');
    const afterMinCalls = await page.evaluate(() =>
      window.__SOLARA_TEST_STATE__.invoked.filter((c) => c.cmd === 'window_action' && c.args.action === 'minimize').length,
    );
    assert.strictEqual(afterMinCalls, beforeMinCalls + 1, 'Minimize Launcher preference must trigger window_action minimize on launch');
    assert.strictEqual(
      await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.filter((c) => c.cmd === 'window_action' && ['close', 'hide'].includes(c.args.action)).length),
      0,
      'Play must never invoke close or hide window_action',
    );
    await page.click('.sol-launch-hero-actions button:has-text("Stop")');
    await page.waitForSelector('.sol-launch-hero-actions button:has-text("Play")');

    // Reopen Settings, restore Keep Launcher Open, then check Privacy tab & Save Bar
    await page.click('button[data-tooltip="Settings"]');
    await page.waitForSelector('.sol-settings-dialog', { timeout: 5000 });
    await page.click('.sol-settings-nav-btn:has-text("Behaviour")');
    await page.locator('.sol-settings-dialog button:has-text("Keep Launcher Open")').click();
    await page.getByRole('button', { name: 'Save Changes', exact: true }).click();
    await page.waitForSelector('.sol-savebar', { state: 'detached' });
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.prefs.onLaunch), 'keep');

    await page.click('.sol-settings-nav-btn:has-text("Privacy")');
    await page.waitForSelector('text=Local Redacted Diagnostics Only', { timeout: 3000 });
    const navFitsWithoutClip = await page.locator('.sol-settings-dialog-nav').evaluate((el) => el.scrollHeight <= el.clientHeight + 2);
    assert(navFitsWithoutClip, 'Settings navigation list must fit without vertical clipping at 1440x900');
    await page.screenshot({ path: path.join(SHOT_DIR, 'e2e-settings.png') });
    await page.getByRole('switch', { name: 'Local diagnostics', exact: true }).click();
    await page.click('button[aria-label="Close dialog"]');
    await page.getByRole('dialog', { name: 'Unsaved Changes', exact: true }).waitFor();
    await page.keyboard.press('Escape');
    assert(await page.locator('.sol-settings-dialog').isVisible(), 'Escape on discard prompt must keep Settings open');
    assert.strictEqual(await page.getByRole('dialog', { name: 'Unsaved Changes', exact: true }).count(), 0);
    assert(await page.locator('.sol-savebar').isVisible(), 'Stay must preserve unsaved settings');
    await page.getByRole('button', { name: 'Reset', exact: true }).click();
    await page.waitForSelector('.sol-savebar', { state: 'detached' });
    await page.getByRole('switch', { name: 'Local diagnostics', exact: true }).click();
    await page.getByRole('button', { name: 'Save Changes', exact: true }).click();
    await page.waitForSelector('.sol-savebar', { state: 'detached' });
    assert.strictEqual(await page.evaluate(() => window.__SOLARA_TEST_STATE__.prefs.diagnosticsEnabled), true);
    await page.click('button[aria-label="Close dialog"]');
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await page.click('.sol-settings-nav-btn:has-text("Privacy")');
    assert.strictEqual(await page.getByRole('switch', { name: 'Local diagnostics', exact: true }).getAttribute('aria-checked'), 'true', 'Saved preference must survive reopening');
    await page.click('button[aria-label="Close dialog"]');

    // Task page uses native byte counts, details, IDs and supported actions.
    await page.getByRole('button', { name: 'Downloads', exact: true }).click();
    await page.getByRole('heading', { name: 'Downloads & Tasks', exact: true }).waitFor();
    assert((await page.locator('.sol-task-card').filter({ hasText: 'Fabric Loader 0.16.10' }).innerText()).includes('100 B / 100 B'));
    assert(await page.getByRole('button', { name: 'Open Instance', exact: true }).first().isVisible());


    // Verify Solara Blue theme token & non-automatic startup update check + modal + topbar "Restart to Update" flow
    const accentHex = await page.evaluate(() =>
      getComputedStyle(document.documentElement).getPropertyValue('--accent').trim().toLowerCase(),
    );
    assert.strictEqual(accentHex, '#3b82f6', `Expected Solara Blue --accent #3b82f6, got ${accentHex}`);

    const invokedPreUpdate = await page.evaluate(() =>
      window.__SOLARA_TEST_STATE__.invoked.map((i) => i.cmd),
    );
    assert(invokedPreUpdate.includes('check_update'), 'Startup must automatically check for updates on launch');
    assert(!invokedPreUpdate.includes('install_update'), 'Updates must never auto-install without user confirmation');

    await page.evaluate(() => {
      const next = {
        available: true,
        state: 'available',
        stage: 'available',
        version: '0.0.99',
        publicVersion: 'V-0.0.99',
        currentVersion: 'V-0.0.38',
        notes: 'Blue UI & Restart-to-Update check',
        downloaded: 0,
        total: null,
        error: null,
      };
      window.__SOLARA_TEST_STATE__.updateStatus = next;
      window.__SOLARA_TEST_STATE__.emit('app-update-status', next);
    });
    const updateDialog = page.getByRole('dialog', { name: 'New Version Available', exact: true });
    await updateDialog.waitFor({ state: 'visible', timeout: 3000 });
    assert(await updateDialog.getByText('V-0.0.99').first().isVisible(), 'Update modal must show V-0.0.99');
    assert(await updateDialog.getByRole('button', { name: 'Update Now' }).isVisible(), 'Update modal must show Update Now button');
    assert(await updateDialog.getByRole('button', { name: "I'll do it later" }).isVisible(), "Update modal must show I'll do it later button");

    await updateDialog.getByRole('button', { name: "I'll do it later" }).click();
    await updateDialog.waitFor({ state: 'detached', timeout: 3000 });
    assert.strictEqual(
      await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.filter((i) => i.cmd === 'install_update').length),
      0,
      'Clicking later must not trigger install_update',
    );
    const restartBtn = page.locator('.sol-titlebar').getByRole('button', { name: 'Restart to Update' });
    await restartBtn.waitFor({ state: 'visible', timeout: 3000 });
    await restartBtn.click();
    assert.strictEqual(
      await page.evaluate(() => window.__SOLARA_TEST_STATE__.invoked.filter((i) => i.cmd === 'install_update').length),
      1,
      'Clicking Restart to Update must trigger install_update',
    );
    // Verify checking/waiting-for-idle preserve V-0.0.99 cleanly without V-0.0.00 flash
    await page.evaluate(() => {
      const checking = { stage: 'checking', state: 'checking', version: null, publicVersion: null, notes: null, error: null };
      window.__SOLARA_TEST_STATE__.emit('app-update-status', checking);
    });
    assert(await updateDialog.getByText('V-0.0.99').first().isVisible(), 'Transitional checking stage must preserve V-0.0.99');
    assert.strictEqual(await updateDialog.getByText('V-0.0.00').count(), 0, 'Checking stage must never render V-0.0.00');
    await page.evaluate(() => {
      const waiting = { stage: 'waiting-for-idle', state: 'waiting-for-idle', version: '0.0.99', publicVersion: 'V-0.0.99', error: null };
      window.__SOLARA_TEST_STATE__.emit('app-update-status', waiting);
    });
    assert(await updateDialog.getByText(/waiting for your running game/i).isVisible(), 'waiting-for-idle stage must display idle-wait guidance');
    // Reset mock update state cleanly before viewport overflow check
    await page.evaluate(() => {
      const done = { stage: 'up-to-date', state: 'up-to-date', version: null, publicVersion: null, error: null };
      window.__SOLARA_TEST_STATE__.updateStatus = done;
      window.__SOLARA_TEST_STATE__.emit('app-update-status', done);
    });

    // Responsive Overflow Check at 1280x720 and 940x560
    for (const vp of [
      { width: 1280, height: 720, label: '1280x720' },
      { width: 940, height: 560, label: '940x560' },
    ]) {
      await page.setViewportSize({ width: vp.width, height: vp.height });
      await page.click('button[data-tooltip="Home"]');
      await page.waitForTimeout(100);
      const ov = await page.evaluate(() => ({
        x: document.documentElement.scrollWidth > window.innerWidth,
        y: document.documentElement.scrollHeight > window.innerHeight,
      }));
      assert(!ov.x && !ov.y, `Document overflow detected at ${vp.label}: ${JSON.stringify(ov)}`);
    }

    // Verify public website (/) uses True OLED Black + Cobalt Blue and scrolls properly
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.goto(`http://127.0.0.1:${PORT}/`, { waitUntil: 'domcontentloaded', timeout: 25000 });
    await page.waitForSelector('.web-hero h1', { timeout: 15000 });
    await page.waitForTimeout(200);
    const webTheme = await page.evaluate(() => {
      const site = document.querySelector('.website');
      const siteStyle = site ? getComputedStyle(site) : null;
      const bodyStyle = getComputedStyle(document.body);
      return {
        bodyBg: bodyStyle.backgroundColor,
        siteBg: siteStyle ? siteStyle.backgroundColor : '',
        accent: siteStyle ? siteStyle.getPropertyValue('--accent').trim().toLowerCase() : '',
        scrollHeight: document.documentElement.scrollHeight,
        innerHeight: window.innerHeight,
      };
    });
    assert.strictEqual(webTheme.bodyBg, 'rgb(0, 0, 0)', 'Website body background must be OLED #000000');
    assert.strictEqual(webTheme.siteBg, 'rgb(0, 0, 0)', 'Website .website background must be OLED #000000');
    assert.strictEqual(webTheme.accent, '#3b82f6', 'Website Cobalt Blue --accent must be #3b82f6');
    assert(webTheme.scrollHeight > webTheme.innerHeight + 500, `Website must scroll vertically (got scrollHeight=${webTheme.scrollHeight})`);
    await page.screenshot({ path: path.join(SHOT_DIR, 'e2e-website.png') });

    assert.strictEqual(errors.length, 0, `Runtime errors encountered:\n${errors.join('\n')}`);
    console.log('SOLARA V3 E2E VERIFICATION PASSED (all assertions + screenshots captured).');
  } finally {
    if (browser) await browser.close().catch(() => {});
    vite.kill('SIGTERM');
  }
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
