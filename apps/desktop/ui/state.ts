import { reactive, computed, ref } from 'vue';
import { isDesktop, native, openExternal } from '../../../packages/shared/client';
import { getLatestAnnouncements, type LauncherAnnouncement } from '../../../packages/shared/announcements';
import type {
  DownloadJob,
  InstanceDetailTab,
  InstanceRecord,
  LauncherPreferences,
  MinecraftAccount,
  NavEntry,
  PageId,
  ToastNotice,
  UpdateStatus,
} from './types';

export { isDesktop, native, openExternal };

export const DEFAULT_PREFERENCES: LauncherPreferences = {
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
  initError: null as string | null,
  desktop: isDesktop,
  instances: [] as InstanceRecord[],
  accounts: [] as MinecraftAccount[],
  downloads: [] as DownloadJob[],
  preferences: { ...DEFAULT_PREFERENCES } as LauncherPreferences,
  selectedInstanceId: null as string | null,
  mcVersions: [] as McVersionSummary[],
  mcVersionsLoaded: false,
  updateStatus: { stage: 'idle', version: null, error: null } as UpdateStatus,
  announcements: [] as LauncherAnnouncement[],
  announcementsSource: 'loading' as 'loading' | 'live' | 'unavailable',
  activeAccountError: null as string | null,
  toasts: [] as ToastNotice[],
  activityFeed: [] as { id: string; time: string; title: string; tone: 'info' | 'success' | 'warning' | 'error' }[],

  // Navigation stack
  history: [{ page: 'home', instanceId: null, tab: 'overview' }] as NavEntry[],
  historyIndex: 0,

  // Global overlays
  showCreateModal: false,
  createModalDefaults: { loader: 'vanilla' as const, group: '' },
  showGlobalSearch: false,
  showTasksPopover: false,
  showAnnouncementsModal: false,
  activeAnnouncement: null as LauncherAnnouncement | null,

  // Unsaved-settings navigation guard
  dirtyChecker: null as null | (() => boolean),
  pendingNavAction: null as null | (() => void),
  showDirtyLeaveDialog: false,
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

function runWithDirtyGuard(action: () => void): void {
  if (store.dirtyChecker && store.dirtyChecker()) {
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
  store.dirtyChecker = null;
  action?.();
}

export function cancelLeaveStay(): void {
  store.showDirtyLeaveDialog = false;
  store.pendingNavAction = null;
}

export function navigate(
  page: PageId,
  opts: { instanceId?: string | null; tab?: InstanceDetailTab } = {},
): void {
  const entry: NavEntry = {
    page,
    instanceId: opts.instanceId ?? (page === 'instance' ? selectedInstance.value?.id ?? null : null),
    tab: opts.tab ?? 'overview',
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

export async function refreshSnapshot(): Promise<void> {
  if (!store.desktop) return;
  try {
    const snap = await native<{
      instances: InstanceRecord[];
      minecraftAccounts: MinecraftAccount[];
      downloads: DownloadJob[];
    }>('launcher_snapshot');
    store.instances = Array.isArray(snap.instances) ? snap.instances : [];
    store.accounts = Array.isArray(snap.minecraftAccounts) ? snap.minecraftAccounts : [];
    store.downloads = Array.isArray(snap.downloads) ? snap.downloads : [];
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
    store.preferences = { ...DEFAULT_PREFERENCES, ...prefs };
    applyAppearance(store.preferences);
  } catch (err) {
    console.warn('[preferences]', err);
  }
}

export async function savePreferencesFull(next: LauncherPreferences): Promise<LauncherPreferences> {
  applyAppearance(next);
  if (!store.desktop) {
    store.preferences = { ...next };
    return store.preferences;
  }
  const saved = await native<LauncherPreferences>('save_launcher_settings_full', {
    preferences: next,
  });
  store.preferences = { ...DEFAULT_PREFERENCES, ...saved };
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
    store.updateStatus = await native<UpdateStatus>('app_update_status');
  } catch {
    // ignore update check status failure
  }
}

export async function initializeEngine(): Promise<void> {
  store.desktop = isDesktop;
  try {
    if (store.desktop) {
      await native('initialize_launcher');
      await Promise.all([refreshPreferences(), refreshSnapshot(), refreshUpdateStatus()]);
    } else {
      applyAppearance(store.preferences);
    }
    store.initialized = true;
    store.initError = null;
    void loadAnnouncements();
  } catch (err) {
    store.initError = err instanceof Error ? err.message : String(err);
    store.initialized = true;
  }
}

export async function launchOrStopInstance(inst: InstanceRecord): Promise<void> {
  if (inst.running) {
    try {
      await native('stop_instance', { id: inst.id });
      notify('info', `Stopping ${inst.name}`, 'Saving world and closing Minecraft.');
      await refreshSnapshot();
    } catch (err) {
      notify('error', 'Could not stop Minecraft', String(err));
    }
    return;
  }
  if (store.preferences.confirmLaunch) {
    // caller can invoke confirm; direct launch handles standard play flow
  }
  if (!selectedAccount.value) {
    notify('warning', 'Sign in required', 'Connect a Microsoft Minecraft account before playing.');
    navigate('account');
    return;
  }
  try {
    notify('info', `Launching ${inst.name}`, `Preparing ${inst.loader} ${inst.gameVersion}...`);
    await native('launch_instance', { id: inst.id });
    if (store.preferences.onLaunch === 'minimize') {
      await native('window_action', { action: 'minimize' });
    } else if (store.preferences.onLaunch === 'hide') {
      await native('window_action', { action: 'hide' });
    }
    notify('success', `${inst.name} launched`, 'Minecraft is now running.');
    await refreshSnapshot();
  } catch (err) {
    notify('error', `Failed to launch ${inst.name}`, String(err));
  }
}

export async function patchInstanceMeta(
  id: string,
  patch: Record<string, unknown>,
): Promise<InstanceRecord | null> {
  if (!store.desktop) return null;
  const updated = await native<InstanceRecord>('update_instance_meta', { id, patch });
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
      return 'Discover';
    case 'skins':
      return 'Skin & Capes';
    case 'screenshots':
      return 'Screenshots';
    case 'tasks':
      return 'Downloads & Tasks';
    case 'settings':
      return 'Settings';
    case 'account':
      return 'Minecraft Accounts';
    case 'instance': {
      const inst = store.instances.find((i) => i.id === r.instanceId);
      return inst?.name ?? 'Instance';
    }
    default:
      return 'Solara';
  }
});

export const showModalOverlayRef = ref(false);
