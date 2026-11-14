<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import InstanceContentBrowserModal, {
  type BrowserContentTab,
} from '../components/InstanceContentBrowserModal.vue';
import InstanceIcon from '../components/InstanceIcon.vue';
import { copyScreenshotPath, getCachedPreview, loadFullScreenshot, peekCachedPreview, screenshotKey, useScreenshotObserver } from '../screenshots';
import ModUpdateModal from '../components/ModUpdateModal.vue';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import SolaraToggle from '../components/SolaraToggle.vue';
import SolaraModal from '../components/SolaraModal.vue';
import {
  AlertTriangle,
  Camera,
  Check,
  Compass,
  Copy,
  File,
  Edit3,
  Plus,
  Folder,
  FolderOpen,
  Globe,
  Info,
  MoreVertical,
  Package,
  Pin,
  Play,
  RefreshCw,
  Search,
  ShieldCheck,
  Sliders,
  Square,
  Terminal,
  Trash2,
  Upload,
  X,
} from '../icons';
import {
  currentRoute,
  registerUnsavedChanges,
  runWithDirtyGuard,
  formatBytes,
  formatDuration,
  formatRelativeTime,
  launchOrStopInstance,
  LOADER_LABEL,
  native,
  navigate,
  notify,
  openInstanceSettings,
  patchInstanceMeta,
  refreshSnapshot,
  selectInstance,
  setInstanceTab,
  store,
} from '../state';
import type {
  ContentUpdateItem,
  FileEntry,
  InstanceDetailTab,
  InstanceRecord,
  LogFileEntry,
  ModEntry,
  PackEntry,
  ScreenshotEntry,
  TrashEntry,
  WorldEntry,
} from '../types';

interface UnifiedContentRow {
  key: string;
  kind: 'mod' | 'resourcepacks' | 'shaderpacks' | 'datapacks';
  kindLabel: string;
  name: string;
  fileName: string;
  path: string;
  version: string | null;
  enabled: boolean;
  size: number;
  iconDataUrl: string | null;
  world: string | null;
  rawMod?: ModEntry;
  rawPack?: PackEntry;
}

const instance = computed<InstanceRecord | null>(() => {
  const id = currentRoute.value.instanceId ?? store.selectedInstanceId;
  return store.instances.find((i) => i.id === id) ?? null;
});

const activeTab = computed<InstanceDetailTab>(() => {
  const t = currentRoute.value.tab;
  if (!t || t === 'overview' || t === 'settings') return 'content';
  return t;
});

const launchState = computed(() => {
  if (!instance.value) return null;
  return store.launchStates[instance.value.id] ?? null;
});

const isLaunching = computed(() => {
  const st = launchState.value;
  return Boolean(st && st.stage !== 'running' && st.stage !== 'error');
});

const openHeaderMenu = ref(false);
const showBrowserModal = ref(false);
const browserInitialTab = ref<BrowserContentTab>('mod');
const uploadInput = ref<HTMLInputElement | null>(null);

// Content state
const contentCategory = ref<'all' | 'mod' | 'resourcepacks' | 'shaderpacks' | 'datapacks'>('all');
const contentSort = ref<'name' | 'enabled' | 'size' | 'kind'>('name');
const contentQuery = ref('');
const selectedContentKeys = ref<string[]>([]);
const bulkUpdating = ref(false);
const compatFindings = ref<string[] | null>(null);
const mods = ref<ModEntry[]>([]);
const resourcePacks = ref<PackEntry[]>([]);
const shaderPacks = ref<PackEntry[]>([]);
const datapacks = ref<PackEntry[]>([]);
const loadingContent = ref(false);
const contentUpdates = ref<ContentUpdateItem[]>([]);
const checkingUpdates = ref(false);
const showUpdatesModal = ref(false);

// Files + Trash + Inline Editor
const currentDir = ref('');
const fileEntries = ref<FileEntry[]>([]);
const trashEntries = ref<TrashEntry[]>([]);
const showTrash = ref(false);
const loadingFiles = ref(false);
const editingFile = ref<FileEntry | null>(null);
const editingContent = ref('');
const originalContent = ref('');
const editingSha256 = ref('');
const previewImageUrl = ref<string | null>(null);
const savingFile = ref(false);
const fileDialog = ref<{ action: 'mkdir' | 'rename' | 'trash'; entry?: FileEntry; id: string; directory: string } | null>(null);
const fileDialogName = ref('');
const fileDialogError = ref('');
const fileMutating = ref(false);
function openFileAction(action: 'mkdir' | 'rename' | 'trash', entry?: FileEntry): void {
  if (!instance.value) return;
  fileDialogName.value = action === 'rename' ? entry?.name ?? '' : '';
  fileDialogError.value = '';
  fileDialog.value = { action, entry, id: instance.value.id, directory: currentDir.value };
}
async function submitFileAction(): Promise<void> {
  const dialog = fileDialog.value;
  if (!dialog || fileMutating.value) return;
  const name = fileDialogName.value.trim();
  if (dialog.action !== 'trash' && (!name || /[\\/:*?"<>|]/.test(name) || name === '.' || name === '..')) {
    fileDialogError.value = 'Enter a name without slashes or special file characters.';
    return;
  }
  fileMutating.value = true;
  try {
    const parent = dialog.entry?.path.split('/').slice(0, -1).join('/') ?? dialog.directory;
    const target = [parent, name].filter(Boolean).join('/');
    await native('instance_file_action', {
      id: dialog.id, action: dialog.action,
      path: dialog.action === 'mkdir' ? target : dialog.entry?.path,
      target: dialog.action === 'rename' ? target : null,
    });
    fileDialog.value = null;
    if (instance.value?.id === dialog.id) await loadFiles();
    notify('success', dialog.action === 'trash' ? 'Moved to instance trash' : dialog.action === 'mkdir' ? 'Folder created' : 'Name updated');
  } catch (err) { fileDialogError.value = String(err); }
  finally { fileMutating.value = false; }
}

// Worlds
const worlds = ref<WorldEntry[]>([]);
const loadingWorlds = ref(false);
const editingWorld = ref<string | null>(null);
const worldRenameInput = ref('');

// Logs & Smart Crash Assistant
const logText = ref('');
const logSearch = ref('');
const logLevel = ref<'ALL' | 'INFO' | 'WARN' | 'ERROR'>('ALL');
const showTimestamps = ref(true);
const logPaused = ref(false);
const logWindowLimit = ref(300);
const showCrashAssistant = ref(false);
const diskLogs = ref<LogFileEntry[]>([]);
const selectedLogFile = ref<string>('live');
const loadingLogs = ref(false);
const copiedLogs = ref(false);
const logError = ref('');
const logViewer = ref<HTMLElement | null>(null);
let latestLogText = '';
let clearedLogPrefix = '';
let logPollTimer: ReturnType<typeof setInterval> | null = null;

// Screenshots
const screenshots = ref<ScreenshotEntry[]>([]);
const bindScreenshotRef = useScreenshotObserver();
const loadingShots = ref(false);
const lightboxShot = ref<{ entry: ScreenshotEntry; src: string } | null>(null);

const unifiedRows = computed<UnifiedContentRow[]>(() => {
  const rows: UnifiedContentRow[] = [
    ...mods.value.map((m) => ({
      key: `mod:${m.path}`,
      kind: 'mod' as const,
      kindLabel: 'Mod',
      name: m.name || m.fileName,
      fileName: m.fileName,
      path: m.path,
      version: m.version,
      enabled: m.enabled,
      size: m.size,
      iconDataUrl: null,
      world: null,
      rawMod: m,
    })),
    ...resourcePacks.value.map((p) => ({
      key: `rp:${p.path}`,
      kind: 'resourcepacks' as const,
      kindLabel: 'Resource Pack',
      name: p.name || p.fileName,
      fileName: p.fileName,
      path: p.path,
      version: p.packFormat ? `Pack v${p.packFormat}` : null,
      enabled: p.enabled,
      size: p.size,
      iconDataUrl: p.iconDataUrl,
      world: null,
      rawPack: p,
    })),
    ...shaderPacks.value.map((p) => ({
      key: `sp:${p.path}`,
      kind: 'shaderpacks' as const,
      kindLabel: 'Shader',
      name: p.name || p.fileName,
      fileName: p.fileName,
      path: p.path,
      version: null,
      enabled: p.enabled,
      size: p.size,
      iconDataUrl: p.iconDataUrl,
      world: null,
      rawPack: p,
    })),
    ...datapacks.value.map((p) => ({
      key: `dp:${p.path}`,
      kind: 'datapacks' as const,
      kindLabel: 'Datapack',
      name: p.name || p.fileName,
      fileName: p.fileName,
      path: p.path,
      version: p.world ? `World: ${p.world}` : null,
      enabled: p.enabled,
      size: p.size,
      iconDataUrl: p.iconDataUrl,
      world: p.world,
      rawPack: p,
    })),
  ];
  const q = contentQuery.value.trim().toLowerCase();
  return rows
    .filter((row) => {
      if (contentCategory.value !== 'all' && row.kind !== contentCategory.value) return false;
      if (q) {
        return `${row.name} ${row.fileName} ${row.world ?? ''}`.toLowerCase().includes(q);
      }
      return true;
    })
    .sort((a, b) => {
      if (contentSort.value === 'enabled') return Number(b.enabled) - Number(a.enabled) || a.name.localeCompare(b.name);
      if (contentSort.value === 'size') return b.size - a.size;
      if (contentSort.value === 'kind') return a.kindLabel.localeCompare(b.kindLabel) || a.name.localeCompare(b.name);
      return a.name.localeCompare(b.name);
    });
});

const installedContentNames = computed(() => unifiedRows.value.map((r) => r.name));

interface ParsedLogLine {
  id: number;
  ts: string;
  level: 'INFO' | 'WARN' | 'ERROR' | 'OTHER';
  body: string;
}

const parsedLogLines = computed<ParsedLogLine[]>(() => {
  if (!logText.value.trim()) return [];
  const raw = logText.value.split(/\r?\n/);
  const q = logSearch.value.trim().toLowerCase();
  const out: ParsedLogLine[] = [];
  let idx = 0;
  for (const line of raw) {
    if (!line) continue;
    const isErr = /\b(ERROR|FATAL|SEVERE|Exception|Error)\b/i.test(line);
    const isWarn = !isErr && /\/WARN\]|\bWARN\b/i.test(line);
    const isInfo = !isErr && !isWarn && /\/INFO\]|\bINFO\b/i.test(line);
    const level: ParsedLogLine['level'] = isErr ? 'ERROR' : isWarn ? 'WARN' : isInfo ? 'INFO' : 'OTHER';
    if (logLevel.value !== 'ALL' && level !== logLevel.value) continue;
    if (q && !line.toLowerCase().includes(q)) continue;
    const match = line.match(/^(\[[^\]]+\])\s*(.*)$/);
    out.push({
      id: idx++,
      ts: match ? match[1] : '',
      level,
      body: match ? match[2] : line,
    });
  }
  return out;
});

const windowedLogLines = computed(() =>
  parsedLogLines.value.slice(-logWindowLimit.value),
);

const crashInsights = computed(() => {
  const txt = logText.value;
  if (!txt.trim()) return [];
  const findings: Array<{ title: string; advice: string }> = [];
  if (/UnsupportedClassVersionError|compiled by a more recent version of the Java Runtime/i.test(txt)) {
    findings.push({
      title: 'Java version mismatch detected',
      advice: 'This Minecraft version or mod requires a newer Java runtime (e.g. Java 21 for MC 1.20.5+). Check Instance Settings -> Java & Memory.',
    });
  }
  if (/OutOfMemoryError|Java heap space/i.test(txt)) {
    findings.push({
      title: 'Minecraft ran out of memory',
      advice: 'Increase allocated RAM in Instance Settings -> Java & Memory (recommended 4096–6144 MB for modded instances).',
    });
  }
  if (/Missing or unsupported mandatory dependencies|ModResolutionException|requires version|Cannot find mod/i.test(txt)) {
    findings.push({
      title: 'Missing or incompatible mod dependency',
      advice: 'A mod requires a library mod or newer version (e.g. Fabric API, Cloth Config, Architectury). Install the required dependency without deleting existing mods.',
    });
  }
  if (/MixinApplyError|MixinTransformerError/i.test(txt)) {
    findings.push({
      title: 'Mixin conflict between installed mods',
      advice: 'Two mods modified the same Minecraft class. Check recent additions in the Content tab and toggle suspicious mods off to isolate.',
    });
  }
  if (/Address already in use|BindException/i.test(txt)) {
    findings.push({
      title: 'Network port already in use',
      advice: 'Another game or local server is using this port. Close the duplicate process or change the server port.',
    });
  }
  return findings;
});

function toggleSelectContent(key: string, checked: boolean): void {
  if (checked) {
    if (!selectedContentKeys.value.includes(key)) {
      selectedContentKeys.value = [...selectedContentKeys.value, key];
    }
  } else {
    selectedContentKeys.value = selectedContentKeys.value.filter((k) => k !== key);
  }
}

async function applyBulkToggle(targetEnabled: boolean): Promise<void> {
  if (!instance.value || bulkUpdating.value) return;
  const id = instance.value.id;
  const targets = unifiedRows.value.filter((r) => selectedContentKeys.value.includes(r.key) && r.enabled !== targetEnabled);
  const failed: string[] = [];
  bulkUpdating.value = true;
  try {
    for (const row of targets) {
      try {
        if (row.kind === 'mod') await native('toggle_instance_mod', { id, path: row.path, enabled: targetEnabled });
        else await native('toggle_instance_pack', { id, kind: row.kind, path: row.path, enabled: targetEnabled });
      } catch (err) { failed.push(row.key); notify('error', `Couldn't change ${row.name}`, String(err)); }
    }
    if (instance.value?.id === id) {
      selectedContentKeys.value = failed;
      await loadAllContent();
    }
  } finally { bulkUpdating.value = false; }
}

function runCompatibilityCheck(): void {
  if (!instance.value) return;
  const issues: string[] = [];
  const enabledMods = mods.value.filter((m) => m.enabled);
  if (instance.value.loader === 'vanilla' && enabledMods.length > 0) {
    issues.push(`Vanilla instances cannot load ${enabledMods.length} mod(s) in mods/. Switch to Fabric/NeoForge or disable mods.`);
  }
  const seenIds = new Map<string, string>();
  for (const m of enabledMods) {
    if (m.issue) issues.push(`${m.fileName}: ${m.issue}. Reinstall this file or inspect it before playing.`);
    const loaders = m.loaders ?? [];
    if (loaders.length > 0 && !loaders.includes(instance.value.loader) && !(instance.value.loader === 'quilt' && loaders.includes('fabric'))) {
      issues.push(`${m.name} is marked for ${loaders.join(' / ')}, but this instance uses ${LOADER_LABEL[instance.value.loader]}. Install a matching release or disable this mod.`);
    }
    const id = m.modId?.toLowerCase();
    if (id && seenIds.has(id)) {
      issues.push(`Duplicate mod ID "${m.modId}" found in ${seenIds.get(id)} and ${m.fileName}.`);
    } else if (id) {
      seenIds.set(id, m.fileName);
    }
  }
  const enabledShaders = shaderPacks.value.filter((s) => s.enabled);
  const hasShaderMod = enabledMods.some((m) => /iris|oculus|optifine/i.test(`${m.name} ${m.modId ?? ''} ${m.fileName}`));
  if (enabledShaders.length > 0 && !hasShaderMod) {
    issues.push('Shader packs are enabled, but no Iris / Oculus shader mod was detected in mods/.');
  }
  compatFindings.value = issues;
}

async function loadAllContent(): Promise<void> {
  if (!store.desktop || !instance.value) return;
  loadingContent.value = true;
  const id = instance.value.id;
  try {
    const [m, rp, sp, dp, w] = await Promise.all([
      native<ModEntry[]>('instance_mods', { id }).catch(() => []),
      native<PackEntry[]>('instance_packs', { id, kind: 'resourcepacks', world: null }).catch(() => []),
      native<PackEntry[]>('instance_packs', { id, kind: 'shaderpacks', world: null }).catch(() => []),
      native<PackEntry[]>('instance_packs', { id, kind: 'datapacks', world: null }).catch(() => []),
      native<WorldEntry[]>('instance_worlds', { id }).catch(() => []),
    ]);
    if (instance.value?.id !== id) return;
    mods.value = Array.isArray(m) ? m : [];
    resourcePacks.value = Array.isArray(rp) ? rp : [];
    shaderPacks.value = Array.isArray(sp) ? sp : [];
    datapacks.value = Array.isArray(dp) ? dp : [];
    worlds.value = Array.isArray(w) ? w : [];
  } finally {
    loadingContent.value = false;
  }
}

async function toggleContentRow(row: UnifiedContentRow, enabled: boolean): Promise<void> {
  if (!instance.value) return;
  try {
    if (row.kind === 'mod') {
      await native('toggle_instance_mod', { id: instance.value.id, path: row.path, enabled });
    } else {
      await native('toggle_instance_pack', {
        id: instance.value.id,
        kind: row.kind,
        path: row.path,
        enabled,
      });
    }
    await loadAllContent();
  } catch (err) {
    notify('error', "Couldn't toggle content", String(err));
  }
}

async function removeContentRow(row: UnifiedContentRow): Promise<void> {
  if (!instance.value) return;
  try {
    await native('instance_file_action', {
      id: instance.value.id,
      action: 'trash',
      path: row.path,
      target: null,
      content: null,
      expectedSha256: null,
    });
    notify('info', 'Moved to instance trash', row.fileName);
    await loadAllContent();
  } catch (err) {
    notify('error', "Couldn't remove file", String(err));
  }
}

async function onUploadFiles(e: Event): Promise<void> {
  const input = e.target as HTMLInputElement;
  const files = Array.from(input.files ?? []);
  input.value = '';
  if (!files.length || !instance.value) return;

  for (const file of files) {
    try {
      const buf = await file.arrayBuffer();
      const bytes = new Uint8Array(buf);
      let bin = '';
      for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
      const b64 = btoa(bin);
      const lower = file.name.toLowerCase();
      if (lower.endsWith('.jar')) {
        await native('import_instance_mod', {
          id: instance.value.id,
          fileName: file.name,
          bytesBase64: b64,
          sourcePath: null,
        });
      } else {
        const targetKind =
          contentCategory.value === 'shaderpacks'
            ? 'shaderpacks'
            : contentCategory.value === 'datapacks'
              ? 'datapacks'
              : 'resourcepacks';
        await native('import_instance_pack', {
          id: instance.value.id,
          kind: targetKind,
          world: targetKind === 'datapacks' ? (worlds.value[0]?.folder ?? null) : null,
          fileName: file.name,
          bytesBase64: b64,
          sourcePath: null,
        });
      }
      notify('success', 'Uploaded', file.name);
    } catch (err) {
      notify('error', `Failed to upload ${file.name}`, String(err));
    }
  }
  await loadAllContent();
}

async function checkUpdates(): Promise<void> {
  if (!instance.value || !store.desktop) return;
  showUpdatesModal.value = true;
}

function openBrowserForCurrentCategory(): void {
  browserInitialTab.value =
    contentCategory.value === 'resourcepacks'
      ? 'resourcepack'
      : contentCategory.value === 'shaderpacks'
        ? 'shader'
        : contentCategory.value === 'datapacks'
          ? 'datapack'
          : 'mod';
  showBrowserModal.value = true;
}

// Files
async function loadFiles(dir = currentDir.value): Promise<void> {
  if (!store.desktop || !instance.value) return;
  loadingFiles.value = true;
  const id = instance.value.id;
  currentDir.value = dir;
  try {
    const [list, trash] = await Promise.all([
      native<FileEntry[]>('instance_files', { id: instance.value.id, path: dir }),
      native<TrashEntry[]>('instance_trash', { id: instance.value.id }).catch(() => []),
    ]);
    if (instance.value?.id !== id) return;
    fileEntries.value = Array.isArray(list) ? list.map((f) => ({
      ...f,
      editable: !f.directory && /\.(txt|json|toml|ya?ml|properties|log|cfg|ini|conf|md|csv|xml|mcmeta)$/i.test(f.name),
      image: !f.directory && /\.(png|jpe?g|webp)$/i.test(f.name),
    })) : [];
    trashEntries.value = Array.isArray(trash) ? trash.map((t) => ({
      ...t, name: t.originalPath.split('/').at(-1) || t.originalPath,
    })) : [];
  } catch (err) {
    notify('error', 'Failed to list files', String(err));
  } finally {
    loadingFiles.value = false;
  }
}

async function openFileEntry(entry: FileEntry): Promise<void> {
  if (!instance.value) return;
  if (entry.directory) {
    await loadFiles(entry.path);
    return;
  }
  if (entry.editable) {
    try {
      const content = await native<{ content: string; sha256: string }>('instance_text', { id: instance.value.id, path: entry.path });
      editingFile.value = entry;
      editingContent.value = content.content;
      originalContent.value = content.content;
      editingSha256.value = content.sha256;
      previewImageUrl.value = null;
    } catch (err) {
      notify('error', 'Could not open file', String(err));
    }
  } else if (entry.image) {
    try {
      previewImageUrl.value = await native<string>('instance_image', { id: instance.value.id, path: entry.path });
      editingFile.value = entry;
    } catch (err) {
      notify('error', 'Could not preview image', String(err));
    }
  } else {
    await native('open_instance_file', { id: instance.value.id, path: entry.path }).catch(() => {});
  }
}

async function saveEditedFile(): Promise<void> {
  if (!instance.value || !editingFile.value) return;
  savingFile.value = true;
  try {
    await native('instance_file_action', {
      id: instance.value.id,
      action: 'save',
      path: editingFile.value.path,
      target: null,
      content: editingContent.value,
      expectedSha256: editingSha256.value,
    });
    notify('success', 'Saved', editingFile.value.name);
    editingFile.value = null;
  } catch (err) {
    notify('error', 'Save failed', String(err));
  } finally {
    savingFile.value = false;
  }
}

async function restoreTrashItem(t: TrashEntry): Promise<void> {
  if (!instance.value) return;
  try {
    await native('instance_file_action', {
      id: instance.value.id,
      action: 'restore',
      path: t.id,
      target: null,
      content: null,
      expectedSha256: null,
    });
    notify('success', 'Restored', t.name);
    await loadFiles();
  } catch (err) {
    notify('error', 'Restore failed', String(err));
  }
}

// Worlds
async function loadWorlds(): Promise<void> {
  if (!store.desktop || !instance.value) return;
  loadingWorlds.value = true;
  try {
    const list = await native<WorldEntry[]>('instance_worlds', { id: instance.value.id });
    worlds.value = Array.isArray(list) ? list : [];
  } finally {
    loadingWorlds.value = false;
  }
}

async function saveWorldRename(folder: string): Promise<void> {
  if (!instance.value || !worldRenameInput.value.trim()) return;
  try {
    await native('rename_instance_world', {
      id: instance.value.id,
      folder,
      newName: worldRenameInput.value.trim(),
    });
    editingWorld.value = null;
    await loadWorlds();
    notify('success', 'World renamed', worldRenameInput.value.trim());
  } catch (err) {
    notify('error', 'Rename failed', String(err));
  }
}

// Logs
async function loadLogs(): Promise<void> {
  if (!store.desktop || !instance.value || loadingLogs.value || logPaused.value) return;
  const id = instance.value.id;
  const source = selectedLogFile.value;
  loadingLogs.value = true;
  logError.value = '';
  const followEnd = !logViewer.value || logViewer.value.scrollHeight - logViewer.value.scrollTop - logViewer.value.clientHeight < 48;
  try {
    if (diskLogs.value.length === 0) {
      const files = await native<LogFileEntry[]>('instance_disk_logs', { id }).catch(() => []);
      if (instance.value?.id !== id) return;
      diskLogs.value = Array.isArray(files) ? files : [];
    }
    const text = source === 'live'
      ? await native<string>('instance_logs', { id }).catch(() => '')
      : await native<string>('instance_disk_log', { id, name: source });
    if (instance.value?.id !== id || selectedLogFile.value !== source || logPaused.value) return;
    latestLogText = text;
    logText.value = clearedLogPrefix && text.startsWith(clearedLogPrefix) ? text.slice(clearedLogPrefix.length) : text;
    if (followEnd) {
      await nextTick();
      if (logViewer.value) logViewer.value.scrollTop = logViewer.value.scrollHeight;
    }
  } catch (err) {
    if (instance.value?.id === id) logError.value = String(err);
  } finally { loadingLogs.value = false; }
}
function clearLogView(): void { clearedLogPrefix = latestLogText; logText.value = ''; }
watch(selectedLogFile, () => { latestLogText = ''; clearedLogPrefix = ''; logText.value = ''; void loadLogs(); });
watch(logPaused, (paused) => { if (!paused) void loadLogs(); });

async function copyAllLogs(): Promise<void> {
  try {
    await navigator.clipboard.writeText(logText.value);
    copiedLogs.value = true;
    window.setTimeout(() => {
      copiedLogs.value = false;
    }, 1500);
  } catch {
    notify('info', 'Copy failed', 'Select text manually in the log console.');
  }
}

// Screenshots
async function loadScreenshots(): Promise<void> {
  if (!store.desktop || !instance.value) return;
  const id = instance.value.id;
  loadingShots.value = true;
  try {
    const list = await native<ScreenshotEntry[]>('launcher_screenshots', { instanceId: id });
    if (instance.value?.id === id) screenshots.value = Array.isArray(list) ? list : [];
  } finally { loadingShots.value = false; }
}
async function openScreenshot(shot: ScreenshotEntry): Promise<void> {
  lightboxShot.value = { entry: shot, src: getCachedPreview(shot) ?? '' };
  try {
    const full = await loadFullScreenshot(shot);
    if (lightboxShot.value && screenshotKey(lightboxShot.value.entry) === screenshotKey(shot)) lightboxShot.value.src = full;
  } catch (err) { notify('error', "Couldn't load screenshot", String(err)); }
}

async function loadTabData(): Promise<void> {
  if (activeTab.value === 'content') await loadAllContent();
  else if (activeTab.value === 'files') await loadFiles(currentDir.value);
  else if (activeTab.value === 'worlds') await loadWorlds();
  else if (activeTab.value === 'logs') await loadLogs();
  else if (activeTab.value === 'screenshots') await loadScreenshots();
}

async function duplicateCurrent(): Promise<void> {
  if (!instance.value) return;
  openHeaderMenu.value = false;
  try {
    await native('duplicate_instance', { id: instance.value.id, name: `${instance.value.name} (Copy)` });
    notify('success', 'Duplicated', `${instance.value.name} (Copy) added.`);
    await refreshSnapshot();
  } catch (err) {
    notify('error', 'Duplicate failed', String(err));
  }
}

const unregisterFileGuard = registerUnsavedChanges(
  () => Boolean(editingFile.value && !previewImageUrl.value && editingContent.value !== originalContent.value),
  () => { editingFile.value = null; },
);
watch(() => instance.value?.id, () => {
  currentDir.value = '';
  editingFile.value = null;
  fileEntries.value = []; trashEntries.value = [];
  selectedContentKeys.value = []; compatFindings.value = null;
  mods.value = []; resourcePacks.value = []; shaderPacks.value = []; datapacks.value = [];
  worlds.value = []; screenshots.value = [];
  selectedLogFile.value = 'live'; logText.value = ''; diskLogs.value = [];
  latestLogText = ''; clearedLogPrefix = ''; logWindowLimit.value = 300;
});
watch(() => [instance.value?.id, activeTab.value], () => { void loadTabData().catch((err) => notify('error', "Couldn't load this tab", String(err))); });
onMounted(() => {
  void loadTabData().catch((err) => notify('error', "Couldn't load this tab", String(err)));
  logPollTimer = setInterval(() => {
    if (activeTab.value === 'logs' && selectedLogFile.value === 'live' && !logPaused.value && !document.hidden) void loadLogs();
  }, 1500);
});
onUnmounted(() => { unregisterFileGuard(); if (logPollTimer) clearInterval(logPollTimer); });
</script>

<template>
  <div class="sol-page sol-scroll" @click="openHeaderMenu = false">
    <div v-if="!instance" class="sol-view-wrap">
      <SolaraEmptyState
        title="No instance selected"
        description="Choose a Minecraft instance from your Library or create a new one."
      >
        <template #icon><Package :size="28" /></template>
        <template #actions>
          <SolaraButton variant="primary" size="sm" @click="navigate('library')">Open Library</SolaraButton>
        </template>
      </SolaraEmptyState>
    </div>

    <div v-else class="sol-view-wrap">
      <!-- Compact Modrinth-style Instance Header -->
      <header class="sol-inst-header">
        <div class="sol-inst-header-left">
          <InstanceIcon
            :name="instance.name"
            :loader="instance.loader"
            :icon="instance.icon"
            :custom-icon="instance.customIcon"
            :size="54"
            :rounded="12"
          />
          <div class="sol-inst-header-meta">
            <div class="sol-row-gap">
              <h1 class="sol-inst-title">{{ instance.name }}</h1>
              <span class="sol-badge sol-badge--gold">
                {{ LOADER_LABEL[instance.loader] ?? instance.loader }} {{ instance.gameVersion }}
              </span>
              <span v-if="instance.running" class="sol-badge sol-badge--success">Running</span>
            </div>
            <div class="sol-muted sol-text-xs">
              {{ formatDuration(instance.playtimeSeconds) }}
              <template v-if="instance.lastPlayed"> · Last played {{ formatRelativeTime(instance.lastPlayed) }}</template>
              <template v-if="instance.notes"> · {{ instance.notes }}</template>
            </div>
            <div
              v-if="launchState && (isLaunching || launchState.stage === 'error')"
              class="sol-launch-stage"
            >
              <span v-if="launchState.stage === 'error'" class="sol-launch-stage-err">
                {{ launchState.error || launchState.detail }}
                <button type="button" class="sol-link-btn" @click="setInstanceTab('logs')">View Logs</button>
              </span>
              <span v-else class="sol-row-gap">
                <span class="sol-dot sol-dot--gold" />
                <span>{{ launchState.detail }}</span>
              </span>
            </div>
          </div>
        </div>

        <div class="sol-inst-header-right" @click.stop>
          <SolaraButton
            :variant="instance.running ? 'danger' : 'primary'"
            size="md"
            :loading="isLaunching"
            @click="launchOrStopInstance(instance)"
          >
            <Square v-if="instance.running" :size="14" />
            <Play v-else :size="15" />
            {{
              instance.running
                ? 'Stop'
                : isLaunching
                  ? launchState?.detail || 'Launching...'
                  : 'Play'
            }}
          </SolaraButton>

          <SolaraButton variant="secondary" size="md" @click="openInstanceSettings(instance.id)">
            <Sliders :size="14" /> Settings
          </SolaraButton>

          <div class="sol-menu-wrap">
            <button
              type="button"
              class="sol-icon-btn"
              aria-label="More instance actions"
              @click="openHeaderMenu = !openHeaderMenu"
            >
              <MoreVertical :size="16" />
            </button>
            <div v-if="openHeaderMenu" class="sol-menu-dropdown">
              <button
                type="button"
                @click="openHeaderMenu = false; patchInstanceMeta(instance.id, { pinned: !instance.pinned })"
              >
                <Pin :size="13" /> {{ instance.pinned ? 'Unpin Sidebar' : 'Pin to Sidebar' }}
              </button>
              <button
                type="button"
                @click="openHeaderMenu = false; native('open_instance_folder', { id: instance.id }).catch(() => {})"
              >
                <FolderOpen :size="13" /> Open Instance Folder
              </button>
              <button type="button" @click="duplicateCurrent">
                <Copy :size="13" /> Duplicate
              </button>
              <button
                type="button"
                class="is-danger"
                @click="openHeaderMenu = false; openInstanceSettings(instance.id, 'advanced')"
              >
                <Trash2 :size="13" /> Delete...
              </button>
            </div>
          </div>
        </div>
      </header>

      <!-- Rounded Navigation Tabs (Content Default, NO Overview card bloat) -->
      <nav class="sol-inst-tabs" aria-label="Instance sections">
        <button
          v-for="tab in [
            { id: 'content', label: 'Content', icon: Package },
            { id: 'files', label: 'Files', icon: Folder },
            { id: 'worlds', label: 'Worlds', icon: Globe },
            { id: 'logs', label: 'Logs', icon: Terminal },
            { id: 'screenshots', label: 'Screenshots', icon: Camera },
          ] as const"
          :key="tab.id"
          type="button"
          :class="['sol-inst-tab', { 'is-active': activeTab === tab.id }]"
          @click="setInstanceTab(tab.id)"
        >
          <component :is="tab.icon" :size="14" />
          <span>{{ tab.label }}</span>
        </button>

        <button
          type="button"
          class="sol-inst-tab sol-inst-tab--settings"
          @click="openInstanceSettings(instance.id)"
        >
          <Sliders :size="14" />
          <span>Settings</span>
        </button>
      </nav>

      <!-- CONTENT TAB -->
      <section v-if="activeTab === 'content'" class="sol-inst-pane">
        <div class="sol-toolbar-bar">
          <div class="sol-search-box sol-search-box--grow">
            <Search :size="14" class="sol-muted" />
            <input
              v-model="contentQuery"
              type="text"
              class="sol-search-input"
              placeholder="Filter installed mods, resource packs, shaders, datapacks..."
            />
            <button v-if="contentQuery" type="button" class="sol-icon-btn sol-icon-btn--xs" @click="contentQuery = ''">
              <X :size="12" />
            </button>
          </div>

          <input
            ref="uploadInput"
            type="file"
            accept=".jar,.zip"
            multiple
            hidden
            @change="onUploadFiles"
          />

          <SolaraButton variant="secondary" size="sm" @click="uploadInput?.click()">
            <Upload :size="13" /> Upload Files
          </SolaraButton>

          <SolaraButton variant="primary" size="sm" @click="openBrowserForCurrentCategory">
            <Compass :size="13" /> Browse Content
          </SolaraButton>

          <SolaraButton variant="secondary" size="sm" :loading="checkingUpdates" @click="checkUpdates">
            Update All
          </SolaraButton>

          <button type="button" class="sol-icon-btn" title="Refresh content" @click="loadAllContent">
            <RefreshCw :size="14" />
          </button>
        </div>

        <div class="sol-toolbar-bar">
          <div class="sol-chip-row">
            <button
              v-for="cat in [
                { id: 'all', label: `All (${mods.length + resourcePacks.length + shaderPacks.length + datapacks.length})` },
                { id: 'mod', label: `Mods (${mods.length})` },
                { id: 'resourcepacks', label: `Resource Packs (${resourcePacks.length})` },
                { id: 'shaderpacks', label: `Shaders (${shaderPacks.length})` },
                { id: 'datapacks', label: `Datapacks (${datapacks.length})` },
              ] as const"
              :key="cat.id"
              type="button"
              :class="['sol-chip', { 'is-active': contentCategory === cat.id }]"
              @click="contentCategory = cat.id"
            >
              {{ cat.label }}
            </button>
          </div>

          <div class="sol-row-gap" style="margin-left: auto">
            <select v-model="contentSort" class="sol-select" aria-label="Sort content">
              <option value="name">Sort: Name</option>
              <option value="enabled">Sort: Enabled first</option>
              <option value="size">Sort: Size</option>
              <option value="kind">Sort: Type</option>
            </select>
            <SolaraButton variant="ghost" size="xs" @click="runCompatibilityCheck">
              <ShieldCheck :size="13" /> Compatibility Check
            </SolaraButton>
          </div>
        </div>

        <div v-if="compatFindings !== null" :class="compatFindings.length > 0 ? 'sol-banner-warn' : 'sol-banner-info'">
          <div class="sol-stack" style="gap: 3px">
            <strong v-if="compatFindings.length === 0">Basic checks passed. Dependencies and version ranges have not been verified.</strong>
            <template v-else>
              <strong>Compatibility Check ({{ compatFindings.length }} notice{{ compatFindings.length > 1 ? 's' : '' }})</strong>
              <span v-for="msg in compatFindings" :key="msg" class="sol-text-xs">• {{ msg }}</span>
            </template>
          </div>
          <button type="button" class="sol-icon-btn sol-icon-btn--xs" @click="compatFindings = null"><X :size="12" /></button>
        </div>

        <div v-if="selectedContentKeys.length > 0" class="sol-bulk-bar">
          <span>{{ selectedContentKeys.length }} selected</span>
          <div class="sol-row-gap">
            <SolaraButton variant="secondary" size="xs" :disabled="bulkUpdating" @click="applyBulkToggle(true)">Enable</SolaraButton>
            <SolaraButton variant="secondary" size="xs" :disabled="bulkUpdating" @click="applyBulkToggle(false)">Disable</SolaraButton>
            <SolaraButton variant="ghost" size="xs" @click="selectedContentKeys = []">Clear</SolaraButton>
          </div>
        </div>

        <SolaraEmptyState
          v-if="!loadingContent && unifiedRows.length === 0"
          title="No content installed yet"
          description="Browse Modrinth content tailored to this Minecraft version & loader, or upload .jar / .zip files."
        >
          <template #icon><Package :size="26" /></template>
          <template #actions>
            <SolaraButton variant="primary" size="sm" @click="openBrowserForCurrentCategory">
              <Compass :size="14" /> Browse Content
            </SolaraButton>
            <SolaraButton variant="secondary" size="sm" @click="uploadInput?.click()">
              <Upload :size="14" /> Upload Files
            </SolaraButton>
          </template>
        </SolaraEmptyState>

        <div v-else class="sol-row-list">
          <div v-for="row in unifiedRows" :key="row.key" :class="['sol-content-row', { 'is-disabled': !row.enabled }]">
            <input
              type="checkbox"
              :checked="selectedContentKeys.includes(row.key)"
              :aria-label="`Select ${row.name}`"
              @change="toggleSelectContent(row.key, ($event.target as HTMLInputElement).checked)"
            />
            <img v-if="row.iconDataUrl" :src="row.iconDataUrl" :alt="row.name" class="sol-content-icon" />
            <div v-else class="sol-content-icon sol-content-icon--empty">
              {{ row.name.slice(0, 2).toUpperCase() }}
            </div>

            <div class="sol-content-meta">
              <div class="sol-row-gap">
                <strong class="sol-truncate">{{ row.name }}</strong>
                <span class="sol-badge">{{ row.kindLabel }}</span>
                <span v-if="row.version" class="sol-muted sol-text-xs">{{ row.version }}</span>
              </div>
              <div class="sol-muted sol-text-xs sol-truncate">
                {{ row.fileName }} · {{ formatBytes(row.size) }}
              </div>
            </div>

            <div class="sol-row-gap">
              <SolaraToggle
                :model-value="row.enabled"
                :label="`Toggle ${row.name}`"
                @update:model-value="toggleContentRow(row, $event)"
              />
              <button
                type="button"
                class="sol-icon-btn sol-icon-btn--xs"
                title="Move to instance trash"
                @click="removeContentRow(row)"
              >
                <Trash2 :size="13" />
              </button>
            </div>
          </div>
        </div>
      </section>

      <!-- FILES TAB -->
      <section v-else-if="activeTab === 'files'" class="sol-inst-pane">
        <div class="sol-toolbar-bar">
          <div class="sol-row-gap">
            <SolaraButton v-if="currentDir" variant="secondary" size="xs" @click="loadFiles(currentDir.split('/').slice(0, -1).join('/'))">
              Up Level
            </SolaraButton>
            <span class="sol-mono sol-text-xs">/{{ currentDir }}</span>
          </div>
          <div class="sol-row-gap">
            <SolaraButton variant="secondary" size="xs" @click="openFileAction('mkdir')">
              <Plus :size="12" /> New Folder
            </SolaraButton>
            <SolaraButton variant="secondary" size="xs" @click="showTrash = !showTrash">
              <Trash2 :size="12" /> {{ showTrash ? 'Hide Trash' : `Trash (${trashEntries.length})` }}
            </SolaraButton>
            <SolaraButton variant="secondary" size="xs" @click="native('open_instance_folder', { id: instance.id }).catch(() => {})">
              <FolderOpen :size="12" /> Open Folder
            </SolaraButton>
          </div>
        </div>

        <div v-if="editingFile" class="sol-editor-pane">
          <div class="sol-row-between">
            <strong class="sol-mono sol-text-xs">{{ editingFile.path }}</strong>
            <div class="sol-row-gap">
              <SolaraButton variant="ghost" size="xs" @click="runWithDirtyGuard(() => { editingFile = null; })">Cancel</SolaraButton>
              <SolaraButton v-if="!previewImageUrl" variant="primary" size="xs" :loading="savingFile" @click="saveEditedFile">
                Save File
              </SolaraButton>
            </div>
          </div>
          <img v-if="previewImageUrl" :src="previewImageUrl" :alt="editingFile.name" class="sol-file-preview-img" />
          <textarea v-else v-model="editingContent" rows="16" class="sol-code-textarea sol-scroll selectable" />
        </div>

        <div v-else-if="showTrash" class="sol-row-list">
          <div v-if="trashEntries.length === 0" class="sol-muted sol-text-xs">Instance trash is empty.</div>
          <div v-for="t in trashEntries" :key="t.id" class="sol-row-item">
            <div>
              <strong>{{ t.name }}</strong>
              <div class="sol-muted sol-text-xs">{{ t.originalPath }}</div>
            </div>
            <SolaraButton variant="secondary" size="xs" @click="restoreTrashItem(t)">Restore</SolaraButton>
          </div>
        </div>

        <div v-else class="sol-row-list">
          <div
            v-for="f in fileEntries"
            :key="f.path"
            class="sol-row-item"
            role="button"
            tabindex="0"
            @keydown.enter.self="openFileEntry(f)"
            @click="openFileEntry(f)"
          >
            <div class="sol-row-gap">
              <Folder v-if="f.directory" :size="15" class="sol-gold" />
              <File v-else :size="15" class="sol-muted" />
              <span>{{ f.name }}</span>
            </div>
            <div class="sol-row-gap">
              <span class="sol-muted sol-text-xs">{{ f.directory ? 'Folder' : formatBytes(f.size) }}</span>
              <button type="button" class="sol-icon-btn sol-icon-btn--xs" :aria-label="`Rename ${f.name}`" title="Rename" @click.stop="openFileAction('rename', f)"><Edit3 :size="13" /></button>
              <button type="button" class="sol-icon-btn sol-icon-btn--xs" :aria-label="`Move ${f.name} to trash`" title="Move to instance trash" @click.stop="openFileAction('trash', f)"><Trash2 :size="13" /></button>
            </div>
          </div>
        </div>
      </section>

      <!-- WORLDS TAB -->
      <section v-else-if="activeTab === 'worlds'" class="sol-inst-pane">
        <div class="sol-toolbar-bar">
          <span class="sol-muted sol-text-xs">Saved worlds inside {{ instance.name }}/saves</span>
          <div class="sol-row-gap">
            <SolaraButton
              variant="secondary"
              size="xs"
              @click="browserInitialTab = 'datapack'; showBrowserModal = true"
            >
              <Compass :size="12" /> Browse Datapacks
            </SolaraButton>
            <SolaraButton variant="ghost" size="xs" @click="loadWorlds">
              <RefreshCw :size="12" /> Refresh
            </SolaraButton>
          </div>
        </div>

        <SolaraEmptyState
          v-if="!loadingWorlds && worlds.length === 0"
          title="No worlds saved yet"
          description="Launch Minecraft and create a singleplayer world to manage it or install datapacks into it."
        >
          <template #icon><Globe :size="26" /></template>
        </SolaraEmptyState>

        <div v-else class="sol-row-list">
          <div v-for="w in worlds" :key="w.folder" class="sol-row-item">
            <div class="sol-row-gap">
              <img v-if="w.iconDataUrl" :src="w.iconDataUrl" :alt="w.name" class="sol-content-icon" />
              <div v-else class="sol-content-icon sol-content-icon--empty"><Globe :size="15" /></div>

              <div>
                <div v-if="editingWorld === w.folder" class="sol-row-gap">
                  <input v-model="worldRenameInput" type="text" class="sol-input sol-input--sm" />
                  <SolaraButton variant="primary" size="xs" @click="saveWorldRename(w.folder)">Save</SolaraButton>
                  <SolaraButton variant="ghost" size="xs" @click="editingWorld = null">Cancel</SolaraButton>
                </div>
                <template v-else>
                  <strong>{{ w.name }}</strong>
                  <div class="sol-muted sol-text-xs">
                    {{ w.gameMode }} · {{ w.versionName }} · {{ w.datapackCount }} datapacks · {{ formatBytes(w.size) }}
                  </div>
                </template>
              </div>
            </div>

            <div class="sol-row-gap">
              <SolaraButton
                variant="ghost"
                size="xs"
                @click="editingWorld = w.folder; worldRenameInput = w.name"
              >
                Rename
              </SolaraButton>
              <SolaraButton variant="primary" size="xs" @click="launchOrStopInstance(instance)">
                <Play :size="12" /> Play
              </SolaraButton>
            </div>
          </div>
        </div>
      </section>

      <!-- LOGS TAB -->
      <section v-else-if="activeTab === 'logs'" class="sol-inst-pane">
        <div class="sol-toolbar-bar">
          <select v-model="selectedLogFile" class="sol-select" @change="loadLogs">
            <option value="live">Live Session Output</option>
            <option v-for="lf in diskLogs" :key="lf.name" :value="lf.name">
              {{ lf.name }} ({{ formatBytes(lf.size) }})
            </option>
          </select>

          <div class="sol-seg-group">
            <button
              v-for="lvl in (['ALL', 'INFO', 'WARN', 'ERROR'] as const)"
              :key="lvl"
              type="button"
              :class="['sol-seg-btn', { 'is-active': logLevel === lvl }]"
              @click="logLevel = lvl"
            >
              {{ lvl }}
            </button>
          </div>

          <div class="sol-search-box sol-search-box--grow">
            <Search :size="13" class="sol-muted" />
            <input v-model="logSearch" type="text" class="sol-search-input" placeholder="Filter log lines..." />
          </div>

          <SolaraButton variant="ghost" size="xs" @click="showTimestamps = !showTimestamps">
            {{ showTimestamps ? 'Timestamps: On' : 'Timestamps: Off' }}
          </SolaraButton>
          <SolaraButton variant="ghost" size="xs" @click="logPaused = !logPaused">
            {{ logPaused ? 'Resume' : 'Pause' }}
          </SolaraButton>
          <SolaraButton variant="ghost" size="xs" @click="clearLogView">Clear</SolaraButton>
          <SolaraButton variant="secondary" size="xs" @click="showCrashAssistant = !showCrashAssistant">
            <Info :size="12" /> Diagnose Log
          </SolaraButton>
          <SolaraButton variant="secondary" size="xs" @click="copyAllLogs">
            <Check v-if="copiedLogs" :size="12" />
            <Copy v-else :size="12" />
            {{ copiedLogs ? 'Copied' : 'Copy Log' }}
          </SolaraButton>
          <SolaraButton variant="ghost" size="xs" :loading="loadingLogs" @click="!logPaused && loadLogs()">
            <RefreshCw :size="12" /> Refresh
          </SolaraButton>
        </div>

        <!-- Smart Crash Assistant Panel -->
        <div v-if="showCrashAssistant" :class="crashInsights.length > 0 ? 'sol-banner-warn' : 'sol-banner-info'">
          <div class="sol-stack" style="gap: 4px">
            <strong v-if="crashInsights.length === 0">Smart Crash Assistant: No known fatal crash patterns found in this log.</strong>
            <template v-else>
              <strong>Smart Crash Assistant ({{ crashInsights.length }} finding{{ crashInsights.length > 1 ? 's' : '' }})</strong>
              <div v-for="ci in crashInsights" :key="ci.title" class="sol-text-xs">
                <strong>{{ ci.title }}:</strong> {{ ci.advice }}
              </div>
            </template>
          </div>
          <button type="button" class="sol-icon-btn sol-icon-btn--xs" @click="showCrashAssistant = false"><X :size="12" /></button>
        </div>

        <div v-if="parsedLogLines.length > logWindowLimit" class="sol-log-window-bar">
          <span class="sol-muted sol-text-xs">Showing latest {{ logWindowLimit }} of {{ parsedLogLines.length }} lines</span>
          <SolaraButton variant="ghost" size="xs" @click="logWindowLimit += 300">Show earlier lines</SolaraButton>
        </div>

        <p v-if="logError" class="sol-inline-error">{{ logError }}</p>
        <div ref="logViewer" class="sol-log-viewer sol-scroll selectable">
          <div v-if="windowedLogLines.length === 0" class="sol-muted">No matching log output.</div>
          <div
            v-for="ln in windowedLogLines"
            :key="ln.id"
            :class="['sol-log-line', `sol-log-line--${ln.level.toLowerCase()}`]"
          >
            <span v-if="showTimestamps && ln.ts" class="sol-log-ts">{{ ln.ts }} </span>
            <span>{{ ln.body }}</span>
          </div>
        </div>
      </section>

      <!-- SCREENSHOTS TAB -->
      <section v-else-if="activeTab === 'screenshots'" class="sol-inst-pane">
        <SolaraEmptyState
          v-if="!loadingShots && screenshots.length === 0"
          title="No screenshots captured yet"
          description="Press F2 in Minecraft and your screenshots for this instance will appear here."
        >
          <template #icon><Camera :size="26" /></template>
        </SolaraEmptyState>

        <div v-else class="sol-shots-grid">
          <article
            v-for="s in screenshots"
            :key="screenshotKey(s)"
            :ref="(el) => bindScreenshotRef(s, el)"
            class="sol-shot-card"
            @click="openScreenshot(s)"
          >
            <img v-if="peekCachedPreview(s)" :src="peekCachedPreview(s)" :alt="s.fileName" class="sol-shot-img" />
            <div v-else class="sol-shot-placeholder">Loading...</div>
            <div class="sol-shot-foot">
              <span class="sol-truncate sol-text-xs">{{ s.fileName }}</span>
            </div>
          </article>
        </div>
      </section>
    </div>

    <SolaraModal
      :open="Boolean(fileDialog)"
      :title="fileDialog?.action === 'mkdir' ? 'New Folder' : fileDialog?.action === 'rename' ? 'Rename' : 'Move to Instance Trash'"
      width="sm"
      :closable="!fileMutating"
      @close="fileDialog = null"
    >
      <p v-if="fileDialog?.action === 'trash'" class="sol-muted">Move {{ fileDialog.entry?.name }} to the instance trash? You can restore it from the Files tab.</p>
      <form v-else id="instance-file-action" class="sol-field" @submit.prevent="submitFileAction">
        <label for="instance-file-name" class="sol-label">Name</label>
        <input id="instance-file-name" v-model="fileDialogName" class="sol-input" maxlength="120" :disabled="fileMutating" />
      </form>
      <p v-if="fileDialogError" class="sol-inline-error">{{ fileDialogError }}</p>
      <template #footer>
        <SolaraButton variant="ghost" :disabled="fileMutating" @click="fileDialog = null">Cancel</SolaraButton>
        <SolaraButton :variant="fileDialog?.action === 'trash' ? 'danger' : 'primary'" :loading="fileMutating" @click="submitFileAction">
          {{ fileDialog?.action === 'trash' ? 'Move to Trash' : 'Save Name' }}
        </SolaraButton>
      </template>
    </SolaraModal>

    <!-- Built-in Instance Content Browser Modal -->
    <InstanceContentBrowserModal
      v-if="instance"
      :open="showBrowserModal"
      :instance="instance"
      :initial-tab="browserInitialTab"
      :worlds="worlds"
      :installed-names="installedContentNames"
      @close="showBrowserModal = false"
      @installed="loadAllContent"
    />

    <!-- Content Update Modal -->
    <ModUpdateModal
      v-if="instance"
      :open="showUpdatesModal"
      :instance-id="instance.id"
      :instance-name="instance.name"
      @close="showUpdatesModal = false"
      @updated="loadAllContent"
    />

    <!-- Screenshot Lightbox -->
    <div v-if="lightboxShot" class="sol-overlay" @click.self="lightboxShot = null">
      <div class="sol-dialog-box" style="max-width: 900px">
        <div class="sol-row-between" style="margin-bottom: 10px">
          <strong>{{ lightboxShot.entry.fileName }}</strong>
          <SolaraButton variant="secondary" size="xs" @click="copyScreenshotPath(lightboxShot.entry)">Copy file path</SolaraButton>
          <SolaraButton variant="ghost" size="xs" @click="lightboxShot = null">Close</SolaraButton>
        </div>
        <img v-if="lightboxShot.src" :src="lightboxShot.src" :alt="lightboxShot.entry.fileName" class="sol-lightbox-img" />
      </div>
    </div>
  </div>
</template>
