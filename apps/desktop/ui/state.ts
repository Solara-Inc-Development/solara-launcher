import { reactive, computed, ref } from 'vue';
import { isDesktop, native, openExternal } from '../../../packages/shared/client';
import { getLatestAnnouncements, type LauncherAnnouncement } from '../../../packages/shared/announcements';
import type {
  DownloadJob,
  InstanceDetailTab,
  InstanceRecord,
  InstanceSettingsCategoryId,
  LauncherPreferences,
  LaunchProgressState,
  LocalServerRecord,
  MinecraftAccount,
  NavEntry,
  PageId,
  SettingsCategoryId,
  SolaraBootstrapData,
  SolaraProfile,
  ToastNotice,
  UpdateStatus,
} from './types';

export { isDesktop, native, openExternal };

export const DEFAULT_PREFERENCES: LauncherPreferences = {
  prefsVersion: 2,
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
  announcementsEnabled: true,
  solaraHubEnabled: true,
  autoUpdateEnabled: true,
  onboardingCompleted: false,
  groups: [],
};

export interface McVersionSummary {
  id: string;
  type: 'release' | 'snapshot' | 'old_beta' | 'old_alpha' | string;
  releaseTime?: string;
}

export const BUILTIN_ICON_PRESETS = [
  'preset:dawn-crest',
  'preset:obsidian-cube',
  'preset:amber-emerald',
  'preset:solar-compass',
  'preset:forge-anvil',
  'preset:nether-portal',
  'preset:redstone-core',
  'preset:end-crystal',
] as const;

export const store = reactive({
  initialized: false,
  splashDone: false,
  initPhase: 'Starting Solara...',
  initProgress: 12,
  initError: null as string | null,
  desktop: isDesktop,
  instances: [] as InstanceRecord[],
  accounts: [] as MinecraftAccount[],
  localServers: [] as LocalServerRecord[],
  downloads: [] as DownloadJob[],
  preferences: { ...DEFAULT_PREFERENCES } as LauncherPreferences,
  selectedInstanceId: null as string | null,
  mcVersions: [] as McVersionSummary[],
  mcVersionsLoaded: false,
  updateStatus: { stage: 'idle', version: null, error: null } as UpdateStatus,
  announcements: [] as LauncherAnnouncement[],
  announcementsSource: 'loading' as 'loading' | 'live' | 'unavailable',
  solaraProfile: { signedIn: false } as SolaraProfile,
  solaraBootstrap: null as SolaraBootstrapData | null,
  solaraBootstrapError: null as string | null,
  activeAccountError: null as string | null,
  launchStates: {} as Record<string, LaunchProgressState>,
  toasts: [] as ToastNotice[],
  activityFeed: [] as { id: string; time: string; title: string; tone: 'info' | 'success' | 'warning' | 'error' }[],

  // Navigation stack
  history: [{ page: 'home', instanceId: null, tab: 'content' }] as NavEntry[],
  historyIndex: 0,

  // Global overlays
  showCreateModal: false,
  createModalDefaults: { loader: 'vanilla' as const, group: '' },
  showGlobalSearch: false,
  showTasksPopover: false,
  showAnnouncementsModal: false,
  activeAnnouncement: null as LauncherAnnouncement | null,
  showSettingsModal: false,
  settingsModalCategory: 'appearance' as SettingsCategoryId,
  showInstanceSettingsModal: false,
  instanceSettingsId: null as string | null,
  instanceSettingsCategory: 'general' as InstanceSettingsCategoryId,

  // Unsaved-settings navigation guard
  dirtyChecker: null as null | (() => boolean),
  pendingNavAction: null as null | (() => void),
  showDirtyLeaveDialog: false,
  pendingLaunchConfirm: null as InstanceRecord | null,
});

export const currentRoute = computed<NavEntry>(() => store.history[store.historyIndex] ?? { page: 'home' });
export const canGoBack = computed(() => store.historyIndex > 0);
export const canGoForward = computed(() => store.historyIndex < store.history.length - 1);

export const selectedAccount = computed(
  () => store.accounts.find((a) => a.selected) ?? store.accounts[0] ?? null,
);

export const selectedInstance = computed<InstanceRecord | null>(() => {
  if (store.selectedInstanceId) {
    const match = store.instances.find((i) => i.id === store.selectedInstanceId);
    if (match) return match;
  }
  return store.instances[0] ?? null;
});

export const pinnedInstances = computed(() => store.instances.filter((i) => i.pinned));
export const runningInstances = computed(() => store.instances.filter((i) => i.running));
export const activeJobs = computed(() =>
  store.downloads.filter((d) => d.status === 'queued' || d.status === 'running'),
);
export const shouldShowOnboarding = computed(
  () => store.initialized && !store.preferences.onboardingCompleted && store.instances.length === 0,
);

export function applyAppearance(prefs: LauncherPreferences): void {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  root.setAttribute('data-theme', prefs.theme || 'dawn-gold');
  root.setAttribute('data-density', prefs.density || 'comfortable');
  root.setAttribute('data-reduced-motion', prefs.reducedMotion ? 'true' : 'false');
  const scale = Math.min(125, Math.max(80, prefs.uiScale || 100));
  root.style.setProperty('--ui-scale', String(scale / 100));
  root.style.fontSize = `${(13.5 * scale) / 100}px`;
}

let toastSeq = 0;
export function notify(
  tone: ToastNotice['tone'],
  title: string,
  message?: string,
  durationMs = 4200,
): void {
  const id = `toast-${++toastSeq}`;
  store.toasts.push({ id, tone, title, message });
  recordActivity(title + (message ? ` — ${message}` : ''), tone);
  if (durationMs > 0) {
    setTimeout(() => dismissToast(id), durationMs);
  }
}

export function dismissToast(id: string): void {
  const idx = store.toasts.findIndex((t) => t.id === id);
  if (idx !== -1) store.toasts.splice(idx, 1);
}

export function recordActivity(title: string, tone: 'info' | 'success' | 'warning' | 'error' = 'info'): void {
  store.activityFeed.unshift({
    id: `act-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
    time: new Date().toISOString(),
    title,
    tone,
  });
  if (store.activityFeed.length > 30) store.activityFeed.length = 30;
}

const unsavedGuards = new Set<{ check: () => boolean; discard: () => void }>();
export function registerUnsavedChanges(check: () => boolean, discard: () => void): () => void {
  const entry = { check, discard };
  unsavedGuards.add(entry);
  return () => { unsavedGuards.delete(entry); };
}
export function runWithDirtyGuard(action: () => void): void {
  if ([...unsavedGuards].some((g) => g.check()) || (store.dirtyChecker && store.dirtyChecker())) {
    store.pendingNavAction = action;
    store.showDirtyLeaveDialog = true;
    return;
  }
  action();
}

export function confirmLeaveDiscard(): void {
  store.showDirtyLeaveDialog = false;
  const action = store.pendingNavAction;
  store.pendingNavAction = null;
  for (const guard of unsavedGuards) if (guard.check()) guard.discard();
  store.dirtyChecker = null;
  action?.();
}

export function cancelLeaveStay(): void {
  store.showDirtyLeaveDialog = false;
  store.pendingNavAction = null;
}

export function openGlobalSettings(category: SettingsCategoryId = 'appearance'): void {
  const open = () => {
    store.showInstanceSettingsModal = false;
    store.settingsModalCategory = category;
    store.showSettingsModal = true;
  };
  if (store.showSettingsModal) open(); else runWithDirtyGuard(open);
}

export function closeGlobalSettings(): void {
  runWithDirtyGuard(() => {
    store.showSettingsModal = false;
  });
}

export function openInstanceSettings(
  instanceId?: string | null,
  category: InstanceSettingsCategoryId = 'general',
): void {
  const targetId = instanceId ?? selectedInstance.value?.id ?? null;
  if (!targetId) return;
  const open = () => {
    store.showSettingsModal = false;
    store.instanceSettingsId = targetId;
    store.instanceSettingsCategory = category;
    store.showInstanceSettingsModal = true;
  };
  if (store.showInstanceSettingsModal && store.instanceSettingsId === targetId) open();
  else runWithDirtyGuard(open);
}

export function closeInstanceSettings(): void {
  runWithDirtyGuard(() => {
    store.showInstanceSettingsModal = false;
  });
}

export function navigate(
  page: PageId,
  opts: { instanceId?: string | null; tab?: InstanceDetailTab } = {},
): void {
  if (page === 'settings') {
    openGlobalSettings();
    return;
  }
  const targetInstanceId = opts.instanceId ?? (page === 'instance' ? selectedInstance.value?.id ?? null : null);
  if (page === 'instance' && opts.tab === 'settings') {
    openInstanceSettings(targetInstanceId);
    return;
  }
  const entry: NavEntry = {
    page,
    instanceId: targetInstanceId,
    tab: opts.tab ?? 'content',
  };
  const cur = currentRoute.value;
  if (
    cur.page === entry.page &&
    cur.instanceId === entry.instanceId &&
    cur.tab === entry.tab
  ) {
    return;
  }
  runWithDirtyGuard(() => {
    if (entry.instanceId) {
      selectInstance(entry.instanceId);
    }
    store.history = store.history.slice(0, store.historyIndex + 1);
    store.history.push(entry);
    store.historyIndex = store.history.length - 1;
  });
}

export function setInstanceTab(tab: InstanceDetailTab): void {
  const cur = currentRoute.value;
  if (cur.page !== 'instance') return;
  if (tab === 'settings') {
    openInstanceSettings(cur.instanceId);
    return;
  }
  runWithDirtyGuard(() => {
    store.history = store.history.slice(0, store.historyIndex + 1);
    store.history.push({ page: 'instance', instanceId: cur.instanceId, tab });
    store.historyIndex = store.history.length - 1;
  });
}

export function goBack(): void {
  if (!canGoBack.value) return;
  runWithDirtyGuard(() => {
    store.historyIndex -= 1;
  });
}

export function goForward(): void {
  if (!canGoForward.value) return;
  runWithDirtyGuard(() => {
    store.historyIndex += 1;
  });
}

export function selectInstance(id: string | null): void {
  store.selectedInstanceId = id;
  try {
    if (id) localStorage.setItem('solara.selectedInstance', id);
    else localStorage.removeItem('solara.selectedInstance');
  } catch {
    // ignore storage limits
  }
}

export function openCreateInstance(defaults?: { loader?: 'vanilla'; group?: string }): void {
  store.createModalDefaults = {
    loader: defaults?.loader ?? 'vanilla',
    group: defaults?.group ?? '',
  };
  store.showCreateModal = true;
}

let snapshotRequest: Promise<void> | null = null;
export async function refreshSnapshot(): Promise<void> {
  if (!store.desktop) return;
  if (!snapshotRequest) snapshotRequest = readSnapshot().finally(() => { snapshotRequest = null; });
  await snapshotRequest;
}

// Retain unchanged objects so periodic polling does not redraw every page.
function reconcileRecords<T extends { id: string }>(current: T[], incoming: T[]): T[] {
  const prior = new Map(current.map((item) => [item.id, item]));
  const next = incoming.map((item) => {
    const old = prior.get(item.id);
    return old && JSON.stringify(old) === JSON.stringify(item) ? old : item;
  });
  return next.length === current.length && next.every((item, index) => item === current[index]) ? current : next;
}

async function readSnapshot(): Promise<void> {
  if (!store.desktop) return;
  try {
    const snap = await native<{
      instances: InstanceRecord[];
      minecraftAccounts: Array<Partial<MinecraftAccount> & { name?: string; active?: boolean }>;
      localServers?: LocalServerRecord[];
      downloads: Array<Partial<DownloadJob> & { instanceId?: string; detail?: string; downloaded?: number }>;
    } | null>('launcher_snapshot');
    if (!snap || typeof snap !== 'object') return;
    store.instances = reconcileRecords(store.instances, Array.isArray(snap.instances) ? snap.instances : []);
    store.localServers = reconcileRecords(store.localServers, Array.isArray(snap.localServers) ? snap.localServers : []);
    store.accounts = reconcileRecords(store.accounts, Array.isArray(snap.minecraftAccounts)
      ? snap.minecraftAccounts
          .filter((a) => Boolean(a?.id))
          .map((a) => ({
            id: String(a.id),
            username: String(a.username ?? a.name ?? ''),
            selected: Boolean(a.selected ?? a.active ?? false),
          }))
      : []);
    store.downloads = reconcileRecords(store.downloads, Array.isArray(snap.downloads)
      ? snap.downloads.filter((j) => Boolean(j.id)).map((j) => ({
          id: String(j.id), name: j.name ?? 'Download', kind: j.kind ?? 'server',
          status: j.status ?? 'queued', instance: j.instanceId ?? j.instance ?? '',
          phase: j.detail ?? j.phase ?? '', done: j.downloaded ?? j.done ?? 0,
          total: j.total ?? 0, actions: j.actions ?? [],
        })) : []);
    if (!store.selectedInstanceId || !store.instances.some((i) => i.id === store.selectedInstanceId)) {
      let saved: string | null = null;
      try {
        saved = localStorage.getItem('solara.selectedInstance');
      } catch {
        saved = null;
      }
      store.selectedInstanceId =
        (saved && store.instances.some((i) => i.id === saved) ? saved : store.instances[0]?.id) ?? null;
    }
  } catch (err) {
    console.warn('[snapshot]', err);
  }
}

export async function refreshPreferences(): Promise<void> {
  if (!store.desktop) {
    applyAppearance(store.preferences);
    return;
  }
  try {
    const prefs = await native<LauncherPreferences>('launcher_settings');
    const normalized: LauncherPreferences = {
      ...DEFAULT_PREFERENCES,
      ...prefs,
      onLaunch: prefs?.onLaunch === 'minimize' ? 'minimize' : 'keep',
    };
    store.preferences = normalized;
    applyAppearance(store.preferences);
  } catch (err) {
    console.warn('[preferences]', err);
  }
}

export async function savePreferencesFull(next: LauncherPreferences): Promise<LauncherPreferences> {
  const cleanNext: LauncherPreferences = {
    ...next,
    onLaunch: next.onLaunch === 'minimize' ? 'minimize' : 'keep',
  };
  applyAppearance(cleanNext);
  if (!store.desktop) {
    store.preferences = { ...cleanNext };
    return store.preferences;
  }
  const saved = await native<LauncherPreferences>('save_launcher_settings_full', {
    preferences: cleanNext,
  });
  store.preferences = {
    ...DEFAULT_PREFERENCES,
    ...saved,
    onLaunch: saved?.onLaunch === 'minimize' ? 'minimize' : 'keep',
  };
  applyAppearance(store.preferences);
  return store.preferences;
}

export async function loadMinecraftVersions(force = false): Promise<McVersionSummary[]> {
  if (store.mcVersionsLoaded && !force && store.mcVersions.length > 0) {
    return store.mcVersions;
  }
  if (store.desktop) {
    const list = await native<McVersionSummary[]>('minecraft_versions');
    store.mcVersions = Array.isArray(list) ? list : [];
    store.mcVersionsLoaded = true;
    return store.mcVersions;
  }
  const res = await fetch('https://piston-meta.mojang.com/mc/game/version_manifest_v2.json');
  const data = await res.json();
  store.mcVersions = Array.isArray(data?.versions) ? data.versions : [];
  store.mcVersionsLoaded = true;
  return store.mcVersions;
}

export async function loadAnnouncements(): Promise<void> {
  store.announcementsSource = 'loading';
  try {
    const res = await fetch('https://api.solara-inc.in/announcements', { credentials: 'omit' });
    if (res.ok) {
      const data = await res.json();
      if (Array.isArray(data?.items) && data.items.length > 0) {
        store.announcements = data.items;
        store.announcementsSource = 'live';
        return;
      }
    }
    store.announcements = getLatestAnnouncements(4);
    store.announcementsSource = 'unavailable';
  } catch {
    store.announcements = getLatestAnnouncements(4);
    store.announcementsSource = 'unavailable';
  }
}

export async function refreshUpdateStatus(): Promise<void> {
  if (!store.desktop) return;
  try {
    const next = await native<(Partial<UpdateStatus> & { state?: string }) | null>('app_update_status');
    if (next && typeof next === 'object' && (next.stage || next.state)) {
      store.updateStatus = { stage: next.stage || next.state || 'idle', version: next.version ?? null, error: next.error ?? null };
    }
  } catch {
    // ignore update check status failure
  }
}

export function setLaunchProgress(
  instanceId: string,
  stage: LaunchProgressState['stage'],
  detail: string,
  error?: string,
): void {
  store.launchStates[instanceId] = {
    stage,
    detail,
    error,
    updatedAt: Date.now(),
  };
}

export function clearLaunchProgress(instanceId: string): void {
  delete store.launchStates[instanceId];
}

export async function refreshSolaraProfile(): Promise<void> {
  if (!store.desktop) {
    store.solaraProfile = { signedIn: false };
    return;
  }
  try {
    const profile = await native<SolaraProfile>('solara_profile');
    store.solaraProfile = profile ?? { signedIn: false };
  } catch {
    store.solaraProfile = { signedIn: false };
  }
}

export async function refreshSolaraBootstrap(): Promise<void> {
  store.solaraBootstrapError = null;
  try {
    const res = await fetch('https://api.solara-inc.in/v1/launcher/bootstrap', {
      headers: { Accept: 'application/json' },
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = (await res.json()) as SolaraBootstrapData;
    store.solaraBootstrap = data;
  } catch (err) {
    store.solaraBootstrapError = err instanceof Error ? err.message : String(err);
  }
}

export async function initializeEngine(): Promise<void> {
  store.desktop = isDesktop;
  store.initError = null;
  store.initPhase = 'Starting Solara...';
  store.initProgress = 18;
  try {
    if (store.desktop) {
      await native('initialize_launcher');
      store.initPhase = 'Loading your instances...';
      store.initProgress = 48;
      await Promise.all([refreshPreferences(), refreshSnapshot()]);
      store.initPhase = 'Checking your account...';
      store.initProgress = 78;
      await Promise.allSettled([refreshSolaraProfile(), refreshUpdateStatus()]);
    } else {
      store.initPhase = 'Loading your instances...';
      store.initProgress = 65;
      applyAppearance(store.preferences);
    }
    store.initPhase = 'Almost ready...';
    store.initProgress = 100;
    store.initialized = true;
    store.splashDone = true;
    void loadAnnouncements();
    void refreshSolaraBootstrap();
  } catch (err) {
    store.initError = err instanceof Error ? err.message : String(err);
    store.initPhase = 'Startup failed';
    store.initialized = false;
  }
}

const pendingGameActions = new Set<string>();
export async function launchOrStopInstance(inst: InstanceRecord, skipConfirm = false): Promise<void> {
  if (pendingGameActions.has(inst.id)) return;
  inst = store.instances.find((item) => item.id === inst.id) ?? inst;
  if (!inst.running && !selectedAccount.value) {
    notify('warning', 'Sign in required', 'Connect a Microsoft Minecraft account before playing.');
    navigate('account');
    return;
  }
  if (!inst.running && store.preferences.confirmLaunch && !skipConfirm) {
    store.pendingLaunchConfirm = inst;
    return;
  }
  pendingGameActions.add(inst.id);
  try {
    if (inst.running) {
      await native('stop_instance', { id: inst.id });
      clearLaunchProgress(inst.id);
      await refreshSnapshot();
      notify('info', `${inst.name} stopped`, 'Minecraft has closed.');
      return;
    }
    setLaunchProgress(inst.id, 'checking', 'Checking files...');
    notify('info', `Launching ${inst.name}`, `Preparing ${inst.loader} ${inst.gameVersion}...`);
    await native('launch_instance', { id: inst.id });
    await refreshSnapshot();
    // A quick crash/exit can arrive before the launch command returns. Never
    // hide the launcher or overwrite that error with a stale running state.
    if (!store.instances.find((item) => item.id === inst.id)?.running) {
      if (store.launchStates[inst.id]?.stage !== 'error') clearLaunchProgress(inst.id);
      return;
    }
    setLaunchProgress(inst.id, 'running', 'Minecraft is running');
    if (store.preferences.onLaunch === 'minimize') {
      try { await native('window_action', { action: 'minimize' }); }
      catch (err) { notify('warning', 'Minecraft is running', `Couldn't minimize the launcher: ${String(err)}`); }
    }
    notify('success', `${inst.name} is running`, 'Minecraft has started.');
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    if (inst.running) notify('error', "Couldn't stop Minecraft", msg);
    else {
      setLaunchProgress(inst.id, 'error', 'Launch failed', msg);
      notify('error', `Couldn't launch ${inst.name}`, msg);
    }
  } finally {
    pendingGameActions.delete(inst.id);
  }
}

export function handleGameExit(payload: { instanceId?: string; success?: boolean; exitCode?: number | null }): void {
  const id = payload.instanceId;
  if (!id) return;
  const inst = store.instances.find((item) => item.id === id);
  if (inst) inst.running = false;
  if (payload.success === false) {
    const detail = payload.exitCode == null ? 'Minecraft stopped unexpectedly.' : `Minecraft exited with code ${payload.exitCode}.`;
    setLaunchProgress(id, 'error', 'Minecraft closed unexpectedly', `${detail} Open Logs for details.`);
    notify('error', `${inst?.name ?? 'Minecraft'} closed unexpectedly`, detail);
  } else clearLaunchProgress(id);
}

export async function patchInstanceMeta(
  id: string,
  patch: Record<string, unknown>,
): Promise<InstanceRecord | null> {
  if (!store.desktop) return null;
  const updated = await native<InstanceRecord>('update_instance_meta', { id, patch });
  if (updated.icon !== 'custom') updated.customIcon = null;
  const idx = store.instances.findIndex((i) => i.id === id);
  if (idx !== -1) store.instances[idx] = { ...store.instances[idx], ...updated };
  return updated;
}

export function formatBytes(bytes: number | null | undefined): string {
  if (!Number.isFinite(bytes) || (bytes ?? 0) <= 0) return '0 B';
  const n = Number(bytes);
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let idx = 0;
  let val = n;
  while (val >= 1024 && idx < units.length - 1) {
    val /= 1024;
    idx++;
  }
  return `${val >= 10 || idx === 0 ? val.toFixed(0) : val.toFixed(1)} ${units[idx]}`;
}

export function formatDuration(seconds: number | null | undefined): string {
  const s = Math.max(0, Math.floor(seconds ?? 0));
  if (s < 60) return `${s}s`;
  const mins = Math.floor(s / 60);
  if (mins < 60) return `${mins}m`;
  const hrs = Math.floor(mins / 60);
  const rem = mins % 60;
  return rem > 0 ? `${hrs}h ${rem}m` : `${hrs}h`;
}

export function formatRelativeTime(isoOrMs: string | number | null | undefined): string {
  if (!isoOrMs) return 'Never';
  const ts = typeof isoOrMs === 'number' ? isoOrMs : Date.parse(isoOrMs);
  if (!Number.isFinite(ts) || ts <= 0) return 'Never';
  const diff = Date.now() - ts;
  if (diff < 60_000) return 'Just now';
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`;
  const days = Math.floor(diff / 86_400_000);
  if (days < 30) return `${days}d ago`;
  return new Date(ts).toLocaleDateString();
}

export function playerAvatarUrl(username: string): string {
  const clean = (username || 'Steve').replace(/[^A-Za-z0-9_]/g, '').slice(0, 16) || 'Steve';
  return `https://mc-heads.net/avatar/${encodeURIComponent(clean)}/64`;
}

export const LOADER_LABEL: Record<string, string> = {
  vanilla: 'Vanilla',
  fabric: 'Fabric',
  quilt: 'Quilt',
  forge: 'Forge',
  neoforge: 'NeoForge',
};

export const currentPageTitle = computed(() => {
  const r = currentRoute.value;
  switch (r.page) {
    case 'home':
      return 'Home';
    case 'library':
      return 'Library';
    case 'discover':
      return 'Discover Modpacks';
    case 'skins':
      return 'Skins & Capes';
    case 'screenshots':
      return 'Screenshots';
    case 'solara':
      return 'Solara Hub';
    case 'tasks':
      return 'Downloads & Tasks';
    case 'settings':
      return 'Settings';
    case 'account':
      return 'Accounts';
    case 'instance': {
      const inst = store.instances.find((i) => i.id === r.instanceId);
      return inst?.name ?? 'Instance';
    }
    default:
      return 'Solara';
  }
});

export const showModalOverlayRef = ref(false);
