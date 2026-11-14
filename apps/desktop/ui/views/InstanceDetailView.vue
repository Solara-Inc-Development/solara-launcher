<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import {
  Play,
  Pause,
  Square,
  Settings,
  MoreHorizontal,
  FolderOpen,
  Pin,
  Star,
  Plus,
  Upload,
  Search,
  RefreshCw,
  Trash2,
  RotateCcw,
  File,
  Folder,
  FileText,
  Archive,
  Wrench,
  Clock,
  Globe,
  Camera,
  Download,
  AlertTriangle,
  Copy,
  Check,
  Edit3,
  Package,
} from '../icons';
import {
  store,
  currentRoute,
  setInstanceTab,
  navigate,
  launchOrStopInstance,
  patchInstanceMeta,
  refreshSnapshot,
  formatBytes,
  formatDuration,
  formatRelativeTime,
  LOADER_LABEL,
  BUILTIN_ICON_PRESETS,
  activeJobs,
  native,
  notify,
} from '../state';
import {
  copyScreenshotPath,
  getCachedPreview,
  invalidateScreenshotPreview,
  loadFullScreenshot,
  loadScreenshotPreview,
  peekCachedPreview,
  screenshotKey,
} from '../screenshots';
import type {
  BackupEntry,
  DetectedJava,
  FileEntry,
  InstanceDetailTab,
  LogFileEntry,
  ModEntry,
  PackEntry,
  ScreenshotEntry,
  TrashEntry,
  WorldEntry,
} from '../types';
import SolaraTabs from '../components/SolaraTabs.vue';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraProgress from '../components/SolaraProgress.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import SolaraModal from '../components/SolaraModal.vue';
import SolaraSaveBar from '../components/SolaraSaveBar.vue';
import SolaraContextMenu, { type ContextMenuItem } from '../components/SolaraContextMenu.vue';
import ConfirmDialog from '../components/ConfirmDialog.vue';
import InstanceIcon from '../components/InstanceIcon.vue';
import ModUpdateModal from '../components/ModUpdateModal.vue';

const instance = computed(() =>
  store.instances.find((i) => i.id === currentRoute.value.instanceId) ?? null,
);
const activeTab = computed<InstanceDetailTab>({
  get: () => currentRoute.value.tab ?? 'overview',
  set: (t) => setInstanceTab(t),
});

const tabs = [
  { id: 'overview', label: 'Overview' },
  { id: 'content', label: 'Content' },
  { id: 'files', label: 'Files' },
  { id: 'worlds', label: 'Worlds' },
  { id: 'logs', label: 'Logs' },
  { id: 'screenshots', label: 'Screenshots' },
  { id: 'settings', label: 'Settings' },
];

// Overflow menu & Delete
const overflowMenu = ref({ open: false, x: 0, y: 0 });
const showDeleteDialog = ref(false);
const deleting = ref(false);
const showUpdateModal = ref(false);

const overflowItems = computed<ContextMenuItem[]>(() => {
  const inst = instance.value;
  if (!inst) return [];
  return [
    { id: 'pin', label: inst.pinned ? 'Unpin from Sidebar' : 'Pin to Sidebar' },
    { id: 'fav', label: inst.favourite ? 'Remove Favourite' : 'Favourite' },
    { id: 'folder', label: 'Open Instance Folder' },
    { id: 'duplicate', label: 'Duplicate Instance' },
    { id: 'repair', label: 'Repair Game Files', disabled: Boolean(inst.running) },
    { id: 'sep', label: '', separator: true },
    { id: 'delete', label: 'Delete Instance', danger: true, disabled: Boolean(inst.running) },
  ];
});

function openOverflow(e: MouseEvent) {
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  overflowMenu.value = { open: true, x: rect.left, y: rect.bottom + 6 };
}

async function handleOverflowAction(action: string) {
  const inst = instance.value;
  if (!inst) return;
  if (action === 'pin') await patchInstanceMeta(inst.id, { pinned: !inst.pinned });
  else if (action === 'fav') await patchInstanceMeta(inst.id, { favourite: !inst.favourite });
  else if (action === 'folder') {
    native('open_instance_folder', { id: inst.id }).catch((e) => notify('error', "Couldn't open folder", String(e)));
  } else if (action === 'duplicate') {
    try {
      const copy = await native<{ id: string }>('duplicate_instance', { id: inst.id });
      await refreshSnapshot();
      notify('success', 'Instance duplicated');
      navigate('instance', { instanceId: copy.id, tab: 'overview' });
    } catch (e) {
      notify('error', "Couldn't duplicate instance", String(e));
    }
  } else if (action === 'repair') {
    try {
      await native('repair_instance', { id: inst.id });
      await refreshSnapshot();
      notify('info', 'Repairing files', `Verifying runtime for ${inst.name}.`);
    } catch (e) {
      notify('error', "Couldn't repair files", String(e));
    }
  } else if (action === 'delete') {
    showDeleteDialog.value = true;
  }
}

async function confirmDelete() {
  if (!instance.value) return;
  deleting.value = true;
  try {
    await native('delete_instance', { id: instance.value.id });
    showDeleteDialog.value = false;
    await refreshSnapshot();
    notify('success', 'Instance deleted');
    navigate('library');
  } catch (e) {
    notify('error', "Couldn't delete instance", String(e));
  } finally {
    deleting.value = false;
  }
}

// ---------------------------------------------------------------------------
// Overview: Notes & Install Job
// ---------------------------------------------------------------------------
const notesText = ref('');
const savingNotes = ref(false);
const activeInstJob = computed(() =>
  instance.value ? activeJobs.value.find((j) => j.instance === instance.value!.id) ?? null : null,
);

watch(
  instance,
  (inst) => {
    notesText.value = inst?.notes ?? '';
  },
  { immediate: true },
);

async function saveNotes() {
  if (!instance.value) return;
  savingNotes.value = true;
  try {
    await patchInstanceMeta(instance.value.id, { notes: notesText.value });
    notify('success', 'Notes saved');
  } catch (e) {
    notify('error', "Couldn't save notes", String(e));
  } finally {
    savingNotes.value = false;
  }
}

// ---------------------------------------------------------------------------
// Content Tab (Mods / ResourcePacks / Shaders / Datapacks)
// ---------------------------------------------------------------------------
const contentSubTab = ref<'mod' | 'resourcepack' | 'shader' | 'datapack'>('mod');
const contentSearch = ref('');
const contentStateFilter = ref<'all' | 'enabled' | 'disabled'>('all');
const mods = ref<ModEntry[]>([]);
const packs = ref<PackEntry[]>([]);
const datapackWorld = ref<string>('');
const contentLoading = ref(false);
const selectedContentPaths = ref<Set<string>>(new Set());
const packFileInput = ref<HTMLInputElement | null>(null);

async function loadContentTab() {
  const inst = instance.value;
  if (!inst || !store.desktop) return;
  contentLoading.value = true;
  selectedContentPaths.value = new Set();
  try {
    if (contentSubTab.value === 'mod') {
      mods.value = await native<ModEntry[]>('instance_mods', { id: inst.id });
    } else {
      packs.value = await native<PackEntry[]>('instance_packs', {
        id: inst.id,
        kind: contentSubTab.value,
        world: contentSubTab.value === 'datapack' && datapackWorld.value ? datapackWorld.value : null,
      });
    }
  } catch (e) {
    notify('error', "Couldn't load content", String(e));
  } finally {
    contentLoading.value = false;
  }
}

const filteredContentRows = computed(() => {
  const q = contentSearch.value.trim().toLowerCase();
  if (contentSubTab.value === 'mod') {
    return mods.value.filter((m) => {
      if (contentStateFilter.value === 'enabled' && !m.enabled) return false;
      if (contentStateFilter.value === 'disabled' && m.enabled) return false;
      if (q && !`${m.name} ${m.fileName} ${m.modId ?? ''}`.toLowerCase().includes(q)) return false;
      return true;
    });
  }
  return packs.value.filter((p) => {
    if (contentStateFilter.value === 'enabled' && !p.enabled) return false;
    if (contentStateFilter.value === 'disabled' && p.enabled) return false;
    if (q && !`${p.name} ${p.fileName} ${p.description || ''}`.toLowerCase().includes(q)) return false;
    return true;
  });
});

async function setContentItemEnabled(path: string, enabled: boolean) {
  if (!instance.value) return;
  if (contentSubTab.value === 'mod') {
    await native('toggle_instance_mod', { id: instance.value.id, path, enabled });
  } else {
    await native('toggle_instance_pack', {
      id: instance.value.id,
      kind: contentSubTab.value,
      path,
      enabled,
    });
  }
}

async function toggleContentItem(path: string, enabled: boolean) {
  if (!instance.value) return;
  try {
    await setContentItemEnabled(path, enabled);
    await loadContentTab();
  } catch (e) {
    notify('error', "Couldn't update item", String(e));
  }
}

async function deleteContentItem(path: string) {
  if (!instance.value) return;
  try {
    await native('instance_file_action', { id: instance.value.id, action: 'trash', path });
    notify('info', 'Moved to instance trash');
    await loadContentTab();
  } catch (e) {
    notify('error', "Couldn't move item to trash", String(e));
  }
}

async function bulkToggle(enable: boolean) {
  if (!instance.value) return;
  const rows = contentSubTab.value === 'mod' ? mods.value : packs.value;
  const targets = Array.from(selectedContentPaths.value).filter((p) => {
    const row = rows.find((item) => item.path === p);
    return !row || row.enabled !== enable;
  });
  let changed = 0;
  let firstErr: unknown = null;
  for (const p of targets) {
    try {
      await setContentItemEnabled(p, enable);
      changed++;
    } catch (e) {
      if (!firstErr) firstErr = e;
    }
  }
  selectedContentPaths.value = new Set();
  await loadContentTab();
  if (firstErr) {
    notify('error', changed > 0 ? "Some items couldn't be updated" : "Couldn't update items", String(firstErr));
  }
}

async function bulkDelete() {
  if (!instance.value) return;
  const targets = Array.from(selectedContentPaths.value);
  let moved = 0;
  let firstErr: unknown = null;
  for (const p of targets) {
    try {
      await native('instance_file_action', { id: instance.value.id, action: 'trash', path: p });
      moved++;
    } catch (e) {
      if (!firstErr) firstErr = e;
    }
  }
  selectedContentPaths.value = new Set();
  await loadContentTab();
  if (firstErr) {
    notify('error', moved > 0 ? `Moved ${moved} item(s); some couldn't be moved` : "Couldn't move items to trash", String(firstErr));
  } else if (moved > 0) {
    notify('info', 'Selected content moved to trash');
  }
}

function triggerImportContent() {
  packFileInput.value?.click();
}

async function onImportContentFile(e: Event) {
  const input = e.target as HTMLInputElement;
  const files = Array.from(input.files ?? []);
  input.value = '';
  if (!files.length || !instance.value) return;
  for (const file of files) {
    const buf = await file.arrayBuffer();
    const bytes = new Uint8Array(buf);
    let bin = '';
    for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
    const b64 = btoa(bin);
    try {
      if (contentSubTab.value === 'mod') {
        await native('import_instance_mod', {
          id: instance.value.id,
          fileName: file.name,
          bytesBase64: b64,
        });
      } else {
        await native('import_instance_pack', {
          id: instance.value.id,
          kind: contentSubTab.value,
          world: contentSubTab.value === 'datapack' && datapackWorld.value ? datapackWorld.value : null,
          fileName: file.name,
          bytesBase64: b64,
        });
      }
      notify('success', `Imported ${file.name}`);
    } catch (err) {
      notify('error', `Couldn't import ${file.name}`, String(err));
    }
  }
  await loadContentTab();
}

// ---------------------------------------------------------------------------
// Files Tab
// ---------------------------------------------------------------------------
const filesPath = ref('');
const filesList = ref<FileEntry[]>([]);
const trashList = ref<TrashEntry[]>([]);
const showTrash = ref(false);
const filesLoading = ref(false);
const editingFile = ref<{ path: string; content: string; sha256: string; dirty: boolean } | null>(null);
const previewImage = ref<{ path: string; dataUrl: string } | null>(null);
const newFolderModal = ref(false);
const newFolderName = ref('');

async function loadFiles(path = filesPath.value) {
  if (!instance.value || !store.desktop) return;
  filesLoading.value = true;
  filesPath.value = path;
  try {
    filesList.value = await native<FileEntry[]>('instance_files', {
      id: instance.value.id,
      path,
    });
    if (showTrash.value) {
      trashList.value = await native<TrashEntry[]>('instance_trash', { id: instance.value.id });
    }
  } catch (e) {
    notify('error', "Couldn't read folder", String(e));
  } finally {
    filesLoading.value = false;
  }
}

async function openFileEntry(entry: FileEntry) {
  if (!instance.value) return;
  if (entry.directory) {
    await loadFiles(entry.path);
    return;
  }
  if (entry.editable) {
    try {
      const res = await native<{ content: string; sha256: string }>('instance_text', {
        id: instance.value.id,
        path: entry.path,
      });
      editingFile.value = { path: entry.path, content: res.content, sha256: res.sha256, dirty: false };
    } catch (e) {
      notify('error', "Couldn't open text file", String(e));
    }
  } else if (entry.image) {
    try {
      const dataUrl = await native<string>('instance_image', {
        id: instance.value.id,
        path: entry.path,
      });
      previewImage.value = { path: entry.path, dataUrl };
    } catch (e) {
      notify('error', "Couldn't preview image", String(e));
    }
  } else {
    native('open_instance_file', { id: instance.value.id, path: entry.path }).catch((e) =>
      notify('error', "Couldn't open file", String(e)),
    );
  }
}

async function saveEditedFile() {
  if (!instance.value || !editingFile.value) return;
  try {
    const res = await native<{ sha256: string }>('instance_file_action', {
      id: instance.value.id,
      action: 'save',
      path: editingFile.value.path,
      content: editingFile.value.content,
      expectedSha256: editingFile.value.sha256,
    });
    editingFile.value.sha256 = res.sha256;
    editingFile.value.dirty = false;
    notify('success', 'File saved');
  } catch (e) {
    notify('error', "Couldn't save file", String(e));
  }
}

async function trashFileEntry(entry: FileEntry) {
  if (!instance.value) return;
  try {
    await native('instance_file_action', {
      id: instance.value.id,
      action: 'trash',
      path: entry.path,
    });
    notify('info', `Moved ${entry.name} to trash`);
    await loadFiles();
  } catch (e) {
    notify('error', "Couldn't move to trash", String(e));
  }
}

async function restoreTrashItem(item: TrashEntry) {
  if (!instance.value) return;
  try {
    await native('instance_file_action', {
      id: instance.value.id,
      action: 'restore',
      path: item.id,
    });
    notify('success', `Restored ${item.name}`);
    await loadFiles();
  } catch (e) {
    notify('error', "Couldn't restore item", String(e));
  }
}

async function createFolder() {
  if (!instance.value || !newFolderName.value.trim()) return;
  const path = filesPath.value
    ? `${filesPath.value}/${newFolderName.value.trim()}`
    : newFolderName.value.trim();
  try {
    await native('instance_file_action', { id: instance.value.id, action: 'mkdir', path });
    newFolderName.value = '';
    newFolderModal.value = false;
    await loadFiles();
  } catch (e) {
    notify('error', "Couldn't create folder", String(e));
  }
}

// ---------------------------------------------------------------------------
// Worlds Tab
// ---------------------------------------------------------------------------
const worlds = ref<WorldEntry[]>([]);
const worldsLoading = ref(false);
const renameWorldTarget = ref<WorldEntry | null>(null);
const renameWorldFolder = ref('');

async function loadWorlds() {
  if (!instance.value || !store.desktop) return;
  worldsLoading.value = true;
  try {
    worlds.value = await native<WorldEntry[]>('instance_worlds', { id: instance.value.id });
  } catch (e) {
    notify('error', "Couldn't load worlds", String(e));
  } finally {
    worldsLoading.value = false;
  }
}

async function confirmRenameWorld() {
  if (!instance.value || !renameWorldTarget.value || !renameWorldFolder.value.trim()) return;
  try {
    await native('rename_instance_world', {
      id: instance.value.id,
      folder: renameWorldTarget.value.folder,
      newName: renameWorldFolder.value.trim(),
    });
    renameWorldTarget.value = null;
    await loadWorlds();
    notify('success', 'World folder renamed');
  } catch (e) {
    notify('error', "Couldn't rename world", String(e));
  }
}

async function deleteWorld(w: WorldEntry) {
  if (!instance.value) return;
  try {
    await native('instance_file_action', {
      id: instance.value.id,
      action: 'trash',
      path: `saves/${w.folder}`,
    });
    notify('info', `World '${w.name}' moved to instance trash`);
    await loadWorlds();
  } catch (e) {
    notify('error', "Couldn't delete world", String(e));
  }
}

// ---------------------------------------------------------------------------
// Logs Tab
// ---------------------------------------------------------------------------
const LOG_WINDOW_STEP = 300;
const MAX_LOG_LINE_CHARS = 2000;
const diskLogs = ref<LogFileEntry[]>([]);
const selectedLogSource = ref<'live' | string>('live');
const rawLogText = ref('');
const clearedLogPrefix = ref('');
const logLevelFilter = ref<'ALL' | 'INFO' | 'WARN' | 'ERROR'>('ALL');
const logSearch = ref('');
const logAutoScroll = ref(true);
const logPaused = ref(false);
const logShowTimestamps = ref(true);
const logWindowLimit = ref(LOG_WINDOW_STEP);
const logBoxRef = ref<HTMLElement | null>(null);
const copiedLog = ref(false);
let logTimer: ReturnType<typeof setInterval> | null = null;

async function loadLogsMeta() {
  if (!instance.value || !store.desktop) return;
  try {
    diskLogs.value = await native<LogFileEntry[]>('instance_disk_logs', { id: instance.value.id });
    if (!instance.value.running && selectedLogSource.value === 'live' && diskLogs.value.length > 0) {
      selectedLogSource.value = diskLogs.value[0].name;
    }
    await fetchCurrentLog(true);
  } catch {
    // ignore missing logs
  }
}

async function fetchCurrentLog(force = false) {
  if (!instance.value || !store.desktop) return;
  if (!force && logPaused.value) return;
  try {
    const nextText =
      selectedLogSource.value === 'live'
        ? await native<string>('instance_logs', { id: instance.value.id })
        : await native<string>('instance_disk_log', {
            id: instance.value.id,
            name: selectedLogSource.value,
          });
    rawLogText.value = nextText;
    if (clearedLogPrefix.value && !nextText.startsWith(clearedLogPrefix.value)) {
      clearedLogPrefix.value = '';
    }
    if (logAutoScroll.value) {
      await nextTick();
      if (logBoxRef.value) {
        logBoxRef.value.scrollTop = logBoxRef.value.scrollHeight;
      }
    }
  } catch (e) {
    if (selectedLogSource.value !== 'live') {
      rawLogText.value = `Couldn't read log: ${String(e)}`;
    } else {
      rawLogText.value = 'No live game process log yet. Launch this instance or select latest.log.';
    }
    clearedLogPrefix.value = '';
  }
}

function handleLogSourceChange() {
  clearedLogPrefix.value = '';
  logWindowLimit.value = LOG_WINDOW_STEP;
  void fetchCurrentLog(true);
}

function clearLogView() {
  clearedLogPrefix.value = rawLogText.value;
  logWindowLimit.value = LOG_WINDOW_STEP;
}

function toggleLogPause() {
  logPaused.value = !logPaused.value;
  if (!logPaused.value) {
    void fetchCurrentLog(true);
  }
}

const activeRawLog = computed(() => {
  if (clearedLogPrefix.value && rawLogText.value.startsWith(clearedLogPrefix.value)) {
    return rawLogText.value.slice(clearedLogPrefix.value.length);
  }
  return rawLogText.value;
});

const matchedLogLines = computed(() => {
  const source = activeRawLog.value;
  if (!source) return [];
  const lines = source.split('\n');
  const q = logSearch.value.trim().toLowerCase();
  const lvl = logLevelFilter.value;
  return lines.filter((line) => {
    if (lvl !== 'ALL') {
      const upper = line.toUpperCase();
      if (lvl === 'ERROR') {
        if (!upper.includes('/ERROR') && !upper.includes('[ERROR') && !upper.includes('/FATAL') && !upper.includes('[FATAL')) {
          return false;
        }
      } else if (!upper.includes(`/${lvl}`) && !upper.includes(`[${lvl}`)) {
        return false;
      }
    }
    if (q && !line.toLowerCase().includes(q)) return false;
    return true;
  });
});

watch([logLevelFilter, logSearch], () => {
  logWindowLimit.value = LOG_WINDOW_STEP;
});

const hiddenLogCount = computed(() => Math.max(0, matchedLogLines.value.length - logWindowLimit.value));

const visibleLogLines = computed(() => {
  const sliceStart = Math.max(0, matchedLogLines.value.length - logWindowLimit.value);
  return matchedLogLines.value.slice(sliceStart).map((rawLine, idx) => {
    const safeLine =
      rawLine.length > MAX_LOG_LINE_CHARS
        ? `${rawLine.slice(0, MAX_LOG_LINE_CHARS)}… (${rawLine.length - MAX_LOG_LINE_CHARS} chars truncated)`
        : rawLine;
    const upper = safeLine.toUpperCase();
    const level: 'error' | 'warn' | 'info' =
      upper.includes('/ERROR') || upper.includes('[ERROR') || upper.includes('/FATAL') || upper.includes('[FATAL')
        ? 'error'
        : upper.includes('/WARN') || upper.includes('[WARN')
          ? 'warn'
          : 'info';
    const tsMatch = /^(\[(?:\d{4}-\d{2}-\d{2}[ T]|\d{1,2}[A-Za-z]{3}\d{4}\s+)?\d{2}:\d{2}:\d{2}(?:[.,]\d+)?\])\s*(.*)$/.exec(safeLine);
    return {
      key: sliceStart + idx,
      level,
      ts: logShowTimestamps.value && tsMatch ? tsMatch[1] : '',
      body: tsMatch ? tsMatch[2] : safeLine,
    };
  });
});

async function copyLogs() {
  try {
    await navigator.clipboard.writeText(matchedLogLines.value.join('\n'));
    copiedLog.value = true;
    setTimeout(() => (copiedLog.value = false), 1500);
  } catch (err) {
    notify('error', "Couldn't copy logs", String(err));
  }
}

function downloadLogFile() {
  const blob = new Blob([matchedLogLines.value.join('\n')], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${instance.value?.name || 'solara'}-${selectedLogSource.value}.log`;
  a.click();
  URL.revokeObjectURL(url);
}

// ---------------------------------------------------------------------------
// Screenshots Tab
// ---------------------------------------------------------------------------
const screenshots = ref<ScreenshotEntry[]>([]);
const lightboxShot = ref<ScreenshotEntry | null>(null);
const lightboxShotSrc = ref<string | null>(null);
const lightboxLoading = ref(false);
const lightboxError = ref<string | null>(null);

const shotObservedMap = new WeakMap<Element, ScreenshotEntry>();
const shotObservedElements = new WeakSet<Element>();
let shotObserver: IntersectionObserver | null = null;

function ensureShotObserver() {
  if (shotObserver || typeof IntersectionObserver === 'undefined') return;
  shotObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const shot = shotObservedMap.get(entry.target);
        if (shot && !peekCachedPreview(shot)) {
          void loadScreenshotPreview(shot);
        }
      }
    },
    { rootMargin: '160px' },
  );
}

function bindShotCardRef(shot: ScreenshotEntry, el: unknown) {
  if (!(el instanceof Element)) return;
  if (typeof IntersectionObserver === 'undefined') {
    if (!peekCachedPreview(shot)) void loadScreenshotPreview(shot);
    return;
  }
  ensureShotObserver();
  shotObservedMap.set(el, shot);
  if (!shotObservedElements.has(el)) {
    shotObservedElements.add(el);
    shotObserver?.observe(el);
  }
}

async function openShotLightbox(shot: ScreenshotEntry) {
  previewImage.value = null;
  lightboxShot.value = shot;
  lightboxShotSrc.value = getCachedPreview(shot) ?? null;
  lightboxError.value = null;
  lightboxLoading.value = true;
  try {
    const full = await loadFullScreenshot(shot);
    if (lightboxShot.value && screenshotKey(lightboxShot.value) === screenshotKey(shot)) {
      lightboxShotSrc.value = full;
    }
  } catch (e) {
    if (lightboxShot.value && screenshotKey(lightboxShot.value) === screenshotKey(shot)) {
      lightboxError.value = String(e);
    }
  } finally {
    if (lightboxShot.value && screenshotKey(lightboxShot.value) === screenshotKey(shot)) {
      lightboxLoading.value = false;
    }
  }
}

function closeMediaLightbox() {
  previewImage.value = null;
  lightboxShot.value = null;
  lightboxShotSrc.value = null;
  lightboxError.value = null;
  lightboxLoading.value = false;
}

async function loadScreenshots() {
  if (!instance.value || !store.desktop) return;
  try {
    screenshots.value = await native<ScreenshotEntry[]>('launcher_screenshots', {
      instanceId: instance.value.id,
    });
  } catch {
    screenshots.value = [];
  }
}

async function deleteScreenshot(shot: ScreenshotEntry) {
  try {
    await native('instance_file_action', {
      id: shot.instanceId,
      action: 'trash',
      path: shot.path,
    });
    invalidateScreenshotPreview(shot);
    closeMediaLightbox();
    await loadScreenshots();
    notify('info', 'Moved screenshot to trash');
  } catch (e) {
    notify('error', "Couldn't move screenshot to trash", String(e));
  }
}

// ---------------------------------------------------------------------------
// Instance Settings + Backups Tab
// ---------------------------------------------------------------------------
const settingsSection = ref<'general' | 'install' | 'java' | 'window' | 'hooks' | 'backups' | 'danger'>('general');
const draft = ref({
  name: '',
  icon: '' as string | null,
  customIcon: null as string | null,
  group: '',
  updateChannel: 'release' as 'release' | 'beta' | 'alpha',
  memory: 4096,
  javaPath: 'auto',
  jvmArgs: '',
  width: 854,
  height: 480,
  fullscreen: false,
  preLaunch: '',
  postExit: '',
});
const baselineJson = ref('');
const savingSettings = ref(false);
const detectedJava = ref<DetectedJava[]>([]);
const backups = ref<BackupEntry[]>([]);
const newBackupName = ref('');
const creatingBackup = ref(false);
const iconUploadInput = ref<HTMLInputElement | null>(null);

function syncDraftFromInstance() {
  const inst = instance.value;
  if (!inst) return;
  draft.value = {
    name: inst.name,
    icon: inst.icon ?? null,
    customIcon: inst.customIcon ?? null,
    group: inst.group ?? '',
    updateChannel: inst.updateChannel ?? 'release',
    memory: inst.memory,
    javaPath: inst.javaPath,
    jvmArgs: inst.jvmArgs ?? '',
    width: inst.resolution?.[0] ?? 854,
    height: inst.resolution?.[1] ?? 480,
    fullscreen: Boolean(inst.fullscreen),
    preLaunch: inst.preLaunch ?? '',
    postExit: inst.postExit ?? '',
  };
  baselineJson.value = JSON.stringify(draft.value);
}

const isSettingsDirty = computed(() => JSON.stringify(draft.value) !== baselineJson.value);

watch(
  () => [currentRoute.value.tab, currentRoute.value.instanceId],
  async ([tab]) => {
    store.dirtyChecker = tab === 'settings' ? () => isSettingsDirty.value : null;
    if (logTimer) {
      clearInterval(logTimer);
      logTimer = null;
    }
    if (tab === 'content') {
      await loadWorlds();
      await loadContentTab();
    } else if (tab === 'files') {
      await loadFiles('');
    } else if (tab === 'worlds') {
      await loadWorlds();
    } else if (tab === 'logs') {
      await loadLogsMeta();
      logTimer = setInterval(() => void fetchCurrentLog(false), 2500);
    } else if (tab === 'screenshots') {
      await loadScreenshots();
    } else if (tab === 'settings') {
      syncDraftFromInstance();
      if (store.desktop) {
        native<DetectedJava[]>('detect_java_runtimes').then((j) => (detectedJava.value = j)).catch(() => {});
        loadBackups();
      }
    }
  },
  { immediate: true },
);

onMounted(() => {
  if (activeTab.value === 'settings') {
    store.dirtyChecker = () => isSettingsDirty.value;
  }
});

onUnmounted(() => {
  store.dirtyChecker = null;
  if (logTimer) clearInterval(logTimer);
  shotObserver?.disconnect();
  shotObserver = null;
});

async function loadBackups() {
  if (!instance.value || !store.desktop) return;
  try {
    backups.value = await native<BackupEntry[]>('instance_backups', { id: instance.value.id });
  } catch {
    backups.value = [];
  }
}

async function createBackup() {
  if (!instance.value) return;
  creatingBackup.value = true;
  try {
    await native('instance_backup_action', {
      id: instance.value.id,
      action: 'create',
      name: newBackupName.value.trim() || undefined,
    });
    newBackupName.value = '';
    await loadBackups();
    notify('success', 'Backup created');
  } catch (e) {
    notify('error', "Couldn't create backup", String(e));
  } finally {
    creatingBackup.value = false;
  }
}

async function restoreBackup(b: BackupEntry) {
  if (!instance.value) return;
  try {
    await native('instance_backup_action', {
      id: instance.value.id,
      action: 'restore',
      backupId: b.id,
    });
    await refreshSnapshot();
    notify('success', 'Backup restored', b.name);
  } catch (e) {
    notify('error', "Couldn't restore backup", String(e));
  }
}

async function deleteBackup(b: BackupEntry) {
  if (!instance.value) return;
  try {
    await native('instance_backup_action', {
      id: instance.value.id,
      action: 'delete',
      backupId: b.id,
    });
    await loadBackups();
    notify('info', 'Backup deleted');
  } catch (e) {
    notify('error', "Couldn't delete backup", String(e));
  }
}

function onSettingsIconPicked(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  (e.target as HTMLInputElement).value = '';
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    const res = typeof reader.result === 'string' ? reader.result : '';
    if (res.startsWith('data:image/')) {
      draft.value.customIcon = res;
      draft.value.icon = res;
    }
  };
  reader.readAsDataURL(file);
}

async function saveInstanceSettings() {
  if (!instance.value) return;
  savingSettings.value = true;
  try {
    const iconPatch =
      draft.value.customIcon && draft.value.customIcon.startsWith('data:image/')
        ? draft.value.customIcon
        : draft.value.icon;
    await patchInstanceMeta(instance.value.id, {
      name: draft.value.name.trim(),
      icon: iconPatch,
      group: draft.value.group.trim() || null,
      updateChannel: draft.value.updateChannel,
      memory: draft.value.memory,
      javaPath: draft.value.javaPath,
      jvmArgs: draft.value.jvmArgs.trim() || null,
      resolution: [draft.value.width, draft.value.height],
      fullscreen: draft.value.fullscreen,
      preLaunch: draft.value.preLaunch.trim() || null,
      postExit: draft.value.postExit.trim() || null,
    });
    await refreshSnapshot();
    syncDraftFromInstance();
    notify('success', 'Instance settings saved');
  } catch (e) {
    notify('error', "Couldn't save settings", String(e));
  } finally {
    savingSettings.value = false;
  }
}
</script>

<template>
  <div v-if="instance" class="sol-page sol-page--column">
    <!-- Instance Header -->
    <header class="sol-inst-header">
      <div class="sol-inst-header-left">
        <InstanceIcon
          :name="instance.name"
          :loader="instance.loader"
          :icon="instance.icon"
          :custom-icon="instance.customIcon"
          :size="48"
          :rounded="12"
        />
        <div>
          <div class="sol-row-gap">
            <h1 class="sol-inst-title">{{ instance.name }}</h1>
            <span v-if="instance.running" class="sol-badge sol-badge--success">Running</span>
            <span v-else-if="instance.state !== 'installed'" :class="['sol-badge', `sol-badge--${instance.state}`]">
              {{ instance.state }}
            </span>
            <span v-if="instance.group" class="sol-group-chip">{{ instance.group }}</span>
          </div>
          <div class="sol-inst-subtitle">
            <span>Minecraft {{ instance.gameVersion }}</span>
            <span>•</span>
            <span>{{ LOADER_LABEL[instance.loader] }}<template v-if="instance.loaderVersion"> {{ instance.loaderVersion }}</template></span>
            <span>•</span>
            <span class="sol-inline-flex"><Clock :size="12" /> {{ formatDuration(instance.playtimeSeconds) }}</span>
            <span>•</span>
            <span>Last played {{ formatRelativeTime(instance.lastPlayed) }}</span>
          </div>
        </div>
      </div>

      <div class="sol-row-gap">
        <SolaraButton
          v-if="instance.state === 'installed'"
          :variant="instance.running ? 'danger' : 'primary'"
          size="md"
          @click="launchOrStopInstance(instance)"
        >
          <Square v-if="instance.running" :size="14" />
          <Play v-else :size="14" />
          {{ instance.running ? 'Stop Game' : 'Play' }}
        </SolaraButton>
        <SolaraButton
          v-else-if="instance.state === 'failed' || instance.state === 'cancelled'"
          variant="primary"
          size="md"
          @click="native('retry_instance', { id: instance.id }).then(() => refreshSnapshot()).catch((e) => notify('error', &quot;Couldn't retry install&quot;, String(e)))"
        >
          <RefreshCw :size="14" /> Retry Install
        </SolaraButton>
        <SolaraButton variant="secondary" size="md" @click="activeTab = 'settings'">
          <Settings :size="14" /> Settings
        </SolaraButton>
        <button type="button" class="sol-icon-btn" title="More instance actions" @click="openOverflow">
          <MoreHorizontal :size="16" />
        </button>
      </div>
    </header>

    <!-- Tabs -->
    <div class="sol-inst-tabs-bar">
      <SolaraTabs v-model="activeTab" :tabs="tabs" />
    </div>

    <!-- Scrollable Tab Body -->
    <div class="sol-inst-body sol-scroll">
      <div class="sol-page-inner">
        <!-- OVERVIEW TAB -->
        <div v-if="activeTab === 'overview'" class="sol-overview-grid">
          <div class="sol-card sol-overview-main">
            <h2 class="sol-card-title">Instance Summary</h2>
            <div v-if="activeInstJob" class="sol-launch-progress" style="margin-bottom: 14px">
              <div class="sol-launch-progress-row">
                <span>{{ activeInstJob.phase || 'Installing...' }}</span>
                <span v-if="activeInstJob.total > 0">{{ formatBytes(activeInstJob.done) }} / {{ formatBytes(activeInstJob.total) }}</span>
              </div>
              <SolaraProgress :value="activeInstJob.done" :max="activeInstJob.total" :indeterminate="activeInstJob.total <= 0" />
            </div>

            <div class="sol-kv-grid">
              <div class="sol-kv"><span class="sol-muted">Game Version</span><strong>{{ instance.gameVersion }}</strong></div>
              <div class="sol-kv"><span class="sol-muted">Loader</span><strong>{{ LOADER_LABEL[instance.loader] }} {{ instance.loaderVersion || 'Native' }}</strong></div>
              <div class="sol-kv"><span class="sol-muted">Allocated Memory</span><strong>{{ instance.memory }} MB</strong></div>
              <div class="sol-kv"><span class="sol-muted">Java Runtime</span><strong class="sol-truncate">{{ instance.javaPath === 'auto' ? 'Managed Auto' : instance.javaPath }}</strong></div>
              <div class="sol-kv"><span class="sol-muted">Created</span><strong>{{ formatRelativeTime(instance.createdAt) }}</strong></div>
              <div class="sol-kv"><span class="sol-muted">Total Playtime</span><strong>{{ formatDuration(instance.playtimeSeconds) }}</strong></div>
            </div>

            <div class="sol-quick-actions-row">
              <SolaraButton variant="secondary" size="sm" @click="navigate('discover')">
                <Plus :size="14" /> Browse Modrinth Content
              </SolaraButton>
              <SolaraButton variant="secondary" size="sm" @click="native('open_instance_folder', { id: instance.id }).catch(() => {})">
                <FolderOpen :size="14" /> Open Folder
              </SolaraButton>
              <SolaraButton variant="secondary" size="sm" @click="activeTab = 'settings'; settingsSection = 'backups'">
                <Archive :size="14" /> Backups
              </SolaraButton>
              <SolaraButton variant="ghost" size="sm" :disabled="Boolean(instance.running)" @click="native('repair_instance', { id: instance.id }).then(() => notify('info', 'Repair queued')).catch((e) => notify('error', &quot;Couldn't repair files&quot;, String(e)))">
                <Wrench :size="14" /> Repair Runtime
              </SolaraButton>
            </div>
          </div>

          <div class="sol-card">
            <div class="sol-side-head">
              <h2 class="sol-card-title">Instance Notes</h2>
              <SolaraButton variant="secondary" size="xs" :loading="savingNotes" @click="saveNotes">Save Notes</SolaraButton>
            </div>
            <textarea
              v-model="notesText"
              class="sol-textarea"
              rows="6"
              placeholder="Write server IP, mod notes, shader settings, or seed notes for this instance..."
              maxlength="4000"
            />
          </div>
        </div>

        <!-- CONTENT TAB -->
        <div v-else-if="activeTab === 'content'" class="sol-stack">
          <div class="sol-toolbar">
            <div class="sol-seg-group">
              <button
                v-for="k in [
                  { id: 'mod', label: 'Mods' },
                  { id: 'resourcepack', label: 'Resource Packs' },
                  { id: 'shader', label: 'Shaders' },
                  { id: 'datapack', label: 'Datapacks' },
                ] as const"
                :key="k.id"
                type="button"
                :class="['sol-seg-btn', { 'is-active': contentSubTab === k.id }]"
                @click="
                  contentSubTab = k.id;
                  loadContentTab();
                "
              >
                {{ k.label }}
              </button>
            </div>

            <div class="sol-search-box">
              <Search :size="13" class="sol-muted" />
              <input v-model="contentSearch" type="text" class="sol-search-input" placeholder="Search installed..." />
            </div>

            <select v-model="contentStateFilter" class="sol-select">
              <option value="all">All Status</option>
              <option value="enabled">Enabled</option>
              <option value="disabled">Disabled</option>
            </select>

            <select
              v-if="contentSubTab === 'datapack'"
              v-model="datapackWorld"
              class="sol-select"
              @change="loadContentTab"
            >
              <option value="">Instance datapacks/</option>
              <option v-for="w in worlds" :key="w.folder" :value="w.folder">World: {{ w.name }}</option>
            </select>

            <div class="sol-row-gap">
              <input
                ref="packFileInput"
                type="file"
                multiple
                :accept="contentSubTab === 'mod' ? '.jar' : '.zip'"
                hidden
                @change="onImportContentFile"
              />
              <SolaraButton variant="secondary" size="sm" @click="triggerImportContent">
                <Upload :size="13" /> Import {{ contentSubTab === 'mod' ? '.jar' : '.zip' }}
              </SolaraButton>
              <SolaraButton
                v-if="contentSubTab === 'mod' && instance.loader !== 'vanilla'"
                variant="secondary"
                size="sm"
                @click="showUpdateModal = true"
              >
                <RefreshCw :size="13" /> Check Updates
              </SolaraButton>
              <SolaraButton variant="primary" size="sm" @click="navigate('discover')">
                <Plus :size="13" /> Add Content
              </SolaraButton>
            </div>
          </div>

          <!-- Honest Compatibility Notices -->
          <div v-if="contentSubTab === 'mod' && instance.loader === 'vanilla'" class="sol-banner sol-banner--warning">
            <AlertTriangle :size="15" />
            <span>Vanilla instances do not load mods. Create or use a Fabric, Quilt, Forge, or NeoForge instance to run mods.</span>
          </div>
          <div v-else-if="contentSubTab === 'shader' && instance.loader === 'vanilla'" class="sol-banner sol-banner--info">
            <AlertTriangle :size="15" />
            <span>Shaders require a shader mod (such as Iris on Fabric/NeoForge or Oculus on Forge) to render in-game.</span>
          </div>

          <!-- Bulk Actions -->
          <div v-if="selectedContentPaths.size > 0" class="sol-bulk-bar">
            <span>{{ selectedContentPaths.size }} selected</span>
            <div class="sol-row-gap">
              <SolaraButton variant="secondary" size="xs" @click="bulkToggle(true)">Enable</SolaraButton>
              <SolaraButton variant="secondary" size="xs" @click="bulkToggle(false)">Disable</SolaraButton>
              <SolaraButton variant="danger" size="xs" @click="bulkDelete">Move to Trash</SolaraButton>
            </div>
          </div>

          <div class="sol-card sol-list-table">
            <div v-if="contentLoading" class="sol-center-state">Loading {{ contentSubTab }}s...</div>
            <SolaraEmptyState
              v-else-if="filteredContentRows.length === 0"
              :title="`No ${contentSubTab}s installed`"
              description="Import local files or browse verified content on Discover."
            >
              <template #icon><Package :size="24" /></template>
              <template #actions>
                <SolaraButton variant="primary" size="sm" @click="navigate('discover')">Browse Discover</SolaraButton>
              </template>
            </SolaraEmptyState>

            <div v-for="item in filteredContentRows" :key="item.path" class="sol-content-row">
              <input
                type="checkbox"
                :checked="selectedContentPaths.has(item.path)"
                @change="
                  ($event.target as HTMLInputElement).checked
                    ? selectedContentPaths.add(item.path)
                    : selectedContentPaths.delete(item.path)
                "
              />
              <div class="sol-content-info">
                <div class="sol-content-name">{{ item.name }}</div>
                <div class="sol-muted sol-text-xs">
                  {{ item.fileName }}
                  <template v-if="'version' in item && item.version"> • v{{ item.version }}</template>
                  <template v-if="item.size"> • {{ formatBytes(item.size) }}</template>
                </div>
              </div>
              <div class="sol-row-gap">
                <SolaraButton
                  :variant="item.enabled ? 'secondary' : 'ghost'"
                  size="xs"
                  @click="toggleContentItem(item.path, !item.enabled)"
                >
                  {{ item.enabled ? 'Enabled' : 'Disabled' }}
                </SolaraButton>
                <button
                  type="button"
                  class="sol-icon-btn sol-icon-btn--xs"
                  title="Move to trash"
                  @click="deleteContentItem(item.path)"
                >
                  <Trash2 :size="13" />
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- FILES TAB -->
        <div v-else-if="activeTab === 'files'" class="sol-stack">
          <div class="sol-toolbar">
            <div class="sol-row-gap">
              <SolaraButton variant="ghost" size="xs" @click="loadFiles('')">game/</SolaraButton>
              <template v-for="(seg, idx) in filesPath.split('/').filter(Boolean)" :key="idx">
                <span class="sol-muted">/</span>
                <SolaraButton
                  variant="ghost"
                  size="xs"
                  @click="loadFiles(filesPath.split('/').filter(Boolean).slice(0, idx + 1).join('/'))"
                >
                  {{ seg }}
                </SolaraButton>
              </template>
            </div>
            <div class="sol-row-gap">
              <SolaraButton variant="secondary" size="xs" @click="newFolderModal = true">New Folder</SolaraButton>
              <SolaraButton
                :variant="showTrash ? 'primary' : 'secondary'"
                size="xs"
                @click="
                  showTrash = !showTrash;
                  loadFiles();
                "
              >
                <Trash2 :size="13" /> Trash ({{ trashList.length }})
              </SolaraButton>
              <SolaraButton variant="ghost" size="xs" @click="native('open_instance_folder', { id: instance.id, path: filesPath }).catch(() => {})">
                <FolderOpen :size="13" /> Open in Explorer
              </SolaraButton>
            </div>
          </div>

          <!-- Trash View -->
          <div v-if="showTrash" class="sol-card sol-list-table">
            <div v-if="trashList.length === 0" class="sol-center-state">Instance trash is empty.</div>
            <div v-for="t in trashList" :key="t.id" class="sol-content-row">
              <div class="sol-content-info">
                <div class="sol-content-name">{{ t.name }}</div>
                <div class="sol-muted sol-text-xs">From {{ t.originalPath }} • {{ formatBytes(t.size) }}</div>
              </div>
              <SolaraButton variant="secondary" size="xs" @click="restoreTrashItem(t)">
                <RotateCcw :size="12" /> Restore
              </SolaraButton>
            </div>
          </div>

          <!-- Files Table -->
          <div v-else class="sol-card sol-list-table">
            <div v-if="filesLoading" class="sol-center-state">Reading directory...</div>
            <div v-else-if="filesList.length === 0" class="sol-center-state">This folder is empty.</div>
            <div v-for="entry in filesList" :key="entry.path" class="sol-content-row">
              <div class="sol-file-main" @click="openFileEntry(entry)">
                <Folder v-if="entry.directory" :size="16" class="sol-accent" />
                <FileText v-else-if="entry.editable" :size="16" class="sol-info" />
                <File v-else :size="16" class="sol-muted" />
                <span class="sol-content-name">{{ entry.name }}</span>
              </div>
              <div class="sol-row-gap">
                <span v-if="!entry.directory" class="sol-muted sol-text-xs">{{ formatBytes(entry.size) }}</span>
                <button
                  type="button"
                  class="sol-icon-btn sol-icon-btn--xs"
                  title="Move to trash"
                  @click="trashFileEntry(entry)"
                >
                  <Trash2 :size="13" />
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- WORLDS TAB -->
        <div v-else-if="activeTab === 'worlds'" class="sol-stack">
          <div class="sol-toolbar">
            <h2 class="sol-card-title">Saved Worlds ({{ worlds.length }})</h2>
            <SolaraButton variant="secondary" size="xs" @click="loadWorlds">
              <RefreshCw :size="13" /> Refresh
            </SolaraButton>
          </div>
          <div v-if="worldsLoading" class="sol-center-state">Reading level.dat world metadata...</div>
          <SolaraEmptyState
            v-else-if="worlds.length === 0"
            title="No Minecraft worlds yet"
            description="Singleplayer worlds created inside this instance will appear here with metadata and datapack controls."
          >
            <template #icon><Globe :size="24" /></template>
          </SolaraEmptyState>
          <div v-else class="sol-worlds-grid">
            <article v-for="w in worlds" :key="w.folder" class="sol-card sol-world-card">
              <div class="sol-world-top">
                <img v-if="w.iconDataUrl" :src="w.iconDataUrl" :alt="w.name" class="sol-world-icon" />
                <div v-else class="sol-world-icon sol-world-icon--empty"><Globe :size="20" /></div>
                <div class="sol-world-info">
                  <div class="sol-world-name">{{ w.name }}</div>
                  <div class="sol-muted sol-text-xs">
                    {{ w.gameMode }} • {{ w.difficulty }} • {{ w.versionName }}
                  </div>
                  <div class="sol-muted sol-text-xs">
                    {{ formatBytes(w.size) }} • {{ w.datapackCount }} datapack(s)
                    <span v-if="w.locked" class="sol-badge sol-badge--warning">Locked</span>
                  </div>
                </div>
              </div>
              <div class="sol-world-actions">
                <SolaraButton
                  variant="ghost"
                  size="xs"
                  @click="
                    datapackWorld = w.folder;
                    contentSubTab = 'datapack';
                    activeTab = 'content';
                  "
                >
                  Datapacks
                </SolaraButton>
                <SolaraButton
                  variant="ghost"
                  size="xs"
                  :disabled="Boolean(instance.running) || w.locked"
                  @click="
                    renameWorldTarget = w;
                    renameWorldFolder = w.folder;
                  "
                >
                  <Edit3 :size="12" /> Rename
                </SolaraButton>
                <SolaraButton
                  variant="danger"
                  size="xs"
                  :disabled="Boolean(instance.running) || w.locked"
                  @click="deleteWorld(w)"
                >
                  <Trash2 :size="12" /> Delete
                </SolaraButton>
              </div>
            </article>
          </div>
        </div>

        <!-- LOGS TAB -->
        <div v-else-if="activeTab === 'logs'" class="sol-stack">
          <div class="sol-toolbar">
            <select v-model="selectedLogSource" class="sol-select" @change="handleLogSourceChange">
              <option value="live">Live Game Process Log</option>
              <option v-for="lf in diskLogs" :key="lf.name" :value="lf.name">
                {{ lf.name }} ({{ formatBytes(lf.size) }})
              </option>
            </select>

            <div class="sol-seg-group">
              <button
                v-for="lvl in (['ALL', 'INFO', 'WARN', 'ERROR'] as const)"
                :key="lvl"
                type="button"
                :class="['sol-seg-btn', { 'is-active': logLevelFilter === lvl }]"
                @click="logLevelFilter = lvl"
              >
                {{ lvl }}
              </button>
            </div>

            <div class="sol-search-box">
              <Search :size="13" class="sol-muted" />
              <input v-model="logSearch" type="text" class="sol-search-input" placeholder="Search log lines..." />
            </div>

            <div class="sol-row-gap">
              <SolaraButton
                :variant="logShowTimestamps ? 'secondary' : 'ghost'"
                size="xs"
                @click="logShowTimestamps = !logShowTimestamps"
              >
                <Clock :size="12" /> {{ logShowTimestamps ? 'Timestamps' : 'No timestamps' }}
              </SolaraButton>
              <SolaraButton
                :variant="logPaused ? 'primary' : 'secondary'"
                size="xs"
                @click="toggleLogPause"
              >
                <Play v-if="logPaused" :size="12" />
                <Pause v-else :size="12" />
                {{ logPaused ? 'Resume' : 'Pause' }}
              </SolaraButton>
              <SolaraButton
                :variant="logAutoScroll ? 'primary' : 'secondary'"
                size="xs"
                @click="logAutoScroll = !logAutoScroll"
              >
                {{ logAutoScroll ? 'Auto-scroll: On' : 'Auto-scroll: Off' }}
              </SolaraButton>
              <SolaraButton variant="secondary" size="xs" @click="copyLogs">
                <Check v-if="copiedLog" :size="12" />
                <Copy v-else :size="12" />
                {{ copiedLog ? 'Copied' : 'Copy' }}
              </SolaraButton>
              <SolaraButton variant="secondary" size="xs" @click="downloadLogFile">
                <Download :size="12" /> Export
              </SolaraButton>
              <SolaraButton variant="ghost" size="xs" @click="clearLogView">
                <Trash2 :size="12" /> Clear
              </SolaraButton>
            </div>
          </div>

          <div ref="logBoxRef" class="sol-log-viewer sol-scroll selectable">
            <div v-if="hiddenLogCount > 0" class="sol-log-window-bar">
              <span class="sol-muted sol-text-xs">
                Showing latest {{ visibleLogLines.length }} of {{ matchedLogLines.length }} matching lines ({{ hiddenLogCount }} older hidden).
              </span>
              <div class="sol-row-gap">
                <SolaraButton variant="secondary" size="xs" @click="logWindowLimit += LOG_WINDOW_STEP">
                  Show {{ Math.min(LOG_WINDOW_STEP, hiddenLogCount) }} older
                </SolaraButton>
                <SolaraButton v-if="logWindowLimit > LOG_WINDOW_STEP" variant="ghost" size="xs" @click="logWindowLimit = LOG_WINDOW_STEP">
                  Latest only
                </SolaraButton>
              </div>
            </div>
            <template v-if="visibleLogLines.length > 0">
              <div
                v-for="entry in visibleLogLines"
                :key="entry.key"
                :class="['sol-log-line', { 'is-error': entry.level === 'error', 'is-warn': entry.level === 'warn' }]"
              ><span v-if="entry.ts" class="sol-log-ts">{{ entry.ts }}</span>{{ entry.body }}</div>
            </template>
            <div v-else class="sol-muted">No matching log output.</div>
          </div>
        </div>

        <!-- SCREENSHOTS TAB -->
        <div v-else-if="activeTab === 'screenshots'" class="sol-stack">
          <div class="sol-toolbar">
            <h2 class="sol-card-title">Instance Screenshots ({{ screenshots.length }})</h2>
            <SolaraButton variant="secondary" size="xs" @click="native('open_instance_folder', { id: instance.id, path: 'screenshots' }).catch(() => {})">
              <FolderOpen :size="13" /> Open Screenshots Folder
            </SolaraButton>
          </div>
          <SolaraEmptyState
            v-if="screenshots.length === 0"
            title="No screenshots captured yet"
            description="Press F2 in Minecraft to capture screenshots. They will appear here automatically."
          >
            <template #icon><Camera :size="24" /></template>
          </SolaraEmptyState>
          <div v-else class="sol-shots-grid">
            <div
              v-for="shot in screenshots"
              :key="screenshotKey(shot)"
              :ref="(el) => bindShotCardRef(shot, el)"
              class="sol-shot-card"
              @click="openShotLightbox(shot)"
            >
              <img
                v-if="peekCachedPreview(shot)"
                :src="peekCachedPreview(shot)"
                :alt="shot.fileName"
                class="sol-shot-img"
              />
              <div v-else class="sol-shot-placeholder">
                <Camera :size="20" />
              </div>
              <div class="sol-shot-meta">
                <span class="sol-truncate">{{ shot.fileName }}</span>
                <span class="sol-muted sol-text-xs">{{ formatBytes(shot.size) }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- INSTANCE SETTINGS TAB -->
        <div v-else-if="activeTab === 'settings'" class="sol-settings-split">
          <nav class="sol-settings-nav">
            <button
              v-for="sec in [
                { id: 'general', label: 'General' },
                { id: 'install', label: 'Installation' },
                { id: 'java', label: 'Java & Memory' },
                { id: 'window', label: 'Window' },
                { id: 'hooks', label: 'Launch Hooks' },
                { id: 'backups', label: 'Backups' },
                { id: 'danger', label: 'Danger Zone' },
              ] as const"
              :key="sec.id"
              type="button"
              :class="['sol-settings-nav-item', { 'is-active': settingsSection === sec.id }]"
              @click="settingsSection = sec.id"
            >
              {{ sec.label }}
            </button>
          </nav>

          <div class="sol-settings-pane">
            <!-- General -->
            <section v-if="settingsSection === 'general'" class="sol-card sol-stack">
              <h2 class="sol-card-title">General</h2>
              <div class="sol-row-gap">
                <InstanceIcon
                  :name="draft.name"
                  :loader="instance.loader"
                  :icon="draft.icon"
                  :custom-icon="draft.customIcon"
                  :size="56"
                  :rounded="12"
                />
                <input ref="iconUploadInput" type="file" accept="image/png,image/jpeg,image/webp" hidden @change="onSettingsIconPicked" />
                <SolaraButton variant="secondary" size="xs" @click="iconUploadInput?.click()">Upload Icon</SolaraButton>
                <div class="sol-preset-strip">
                  <button
                    v-for="preset in BUILTIN_ICON_PRESETS"
                    :key="preset"
                    type="button"
                    :class="['sol-preset-btn', { 'is-active': !draft.customIcon && draft.icon === preset }]"
                    @click="
                      draft.customIcon = null;
                      draft.icon = preset;
                    "
                  >
                    <InstanceIcon :icon="preset" :size="22" :rounded="5" />
                  </button>
                </div>
              </div>

              <div class="sol-split-2">
                <div class="sol-field">
                  <label class="sol-label">Instance Name</label>
                  <input v-model="draft.name" type="text" class="sol-input" maxlength="80" />
                </div>
                <div class="sol-field">
                  <label class="sol-label">Group</label>
                  <input v-model="draft.group" type="text" class="sol-input" placeholder="None" maxlength="60" />
                </div>
              </div>

              <div class="sol-field">
                <label class="sol-label">Update Channel</label>
                <select v-model="draft.updateChannel" class="sol-input">
                  <option value="release">Release</option>
                  <option value="beta">Beta</option>
                  <option value="alpha">Alpha</option>
                </select>
              </div>
            </section>

            <!-- Installation -->
            <section v-else-if="settingsSection === 'install'" class="sol-card sol-stack">
              <h2 class="sol-card-title">Installation & Runtime Integrity</h2>
              <div class="sol-kv-grid">
                <div class="sol-kv"><span class="sol-muted">Minecraft Version</span><strong>{{ instance.gameVersion }}</strong></div>
                <div class="sol-kv"><span class="sol-muted">Mod Loader</span><strong>{{ LOADER_LABEL[instance.loader] }} {{ instance.loaderVersion || '' }}</strong></div>
                <div class="sol-kv"><span class="sol-muted">Install State</span><strong>{{ instance.state }}</strong></div>
              </div>
              <div>
                <SolaraButton
                  variant="secondary"
                  size="sm"
                  :disabled="Boolean(instance.running)"
                  @click="native('repair_instance', { id: instance.id }).then(() => { refreshSnapshot(); notify('info', 'Repair queued'); }).catch((e) => notify('error', &quot;Couldn't repair files&quot;, String(e)))"
                >
                  <Wrench :size="14" /> Verify & Repair Game Files
                </SolaraButton>
              </div>
            </section>

            <!-- Java & Memory -->
            <section v-else-if="settingsSection === 'java'" class="sol-card sol-stack">
              <h2 class="sol-card-title">Java & Memory</h2>
              <div class="sol-field">
                <label class="sol-label">Memory Allocation ({{ draft.memory }} MB)</label>
                <input v-model.number="draft.memory" type="range" min="512" max="16384" step="512" class="sol-range" />
              </div>
              <div class="sol-field">
                <label class="sol-label">Java Runtime</label>
                <select v-model="draft.javaPath" class="sol-input">
                  <option value="auto">Automatic (Solara Managed)</option>
                  <option v-for="j in detectedJava" :key="j.path" :value="j.path">
                    Java {{ j.major ?? '?' }} ({{ j.vendor }}) — {{ j.path }}
                  </option>
                </select>
              </div>
              <div class="sol-field">
                <label class="sol-label">Custom JVM Arguments</label>
                <input
                  v-model="draft.jvmArgs"
                  type="text"
                  class="sol-input"
                  placeholder="e.g. -XX:+UseG1GC -XX:+UnlockExperimentalVMOptions"
                />
              </div>
            </section>

            <!-- Window -->
            <section v-else-if="settingsSection === 'window'" class="sol-card sol-stack">
              <h2 class="sol-card-title">Game Window</h2>
              <div class="sol-split-2">
                <div class="sol-field">
                  <label class="sol-label">Width (px)</label>
                  <input v-model.number="draft.width" type="number" min="320" max="7680" class="sol-input" />
                </div>
                <div class="sol-field">
                  <label class="sol-label">Height (px)</label>
                  <input v-model.number="draft.height" type="number" min="240" max="4320" class="sol-input" />
                </div>
              </div>
              <label class="sol-checkbox-label">
                <input v-model="draft.fullscreen" type="checkbox" />
                <span>Start Minecraft in Fullscreen</span>
              </label>
            </section>

            <!-- Hooks -->
            <section v-else-if="settingsSection === 'hooks'" class="sol-card sol-stack">
              <h2 class="sol-card-title">Launch Hooks</h2>
              <div class="sol-banner sol-banner--warning">
                <AlertTriangle :size="15" />
                <span>
                  Hook commands run directly without a system shell, split by spaces and quotes, inside this instance's
                  <code>game/</code> folder (60s limit). Only run trusted local commands—if the pre-launch command returns an error, Minecraft will not start.
                </span>
              </div>
              <div class="sol-field">
                <label class="sol-label">Pre-launch Command</label>
                <input v-model="draft.preLaunch" type="text" class="sol-input" placeholder='e.g. "C:\Tools\sync.exe" --mode fast' />
              </div>
              <div class="sol-field">
                <label class="sol-label">Post-exit Command</label>
                <input v-model="draft.postExit" type="text" class="sol-input" placeholder='e.g. "C:\Tools\cleanup.exe"' />
              </div>
            </section>

            <!-- Backups -->
            <section v-else-if="settingsSection === 'backups'" class="sol-card sol-stack">
              <div class="sol-side-head">
                <h2 class="sol-card-title">Instance Backups ({{ backups.length }})</h2>
                <div class="sol-row-gap">
                  <input v-model="newBackupName" type="text" class="sol-input sol-input--sm" placeholder="Optional label" />
                  <SolaraButton variant="primary" size="sm" :loading="creatingBackup" @click="createBackup">
                    Create Backup
                  </SolaraButton>
                </div>
              </div>
              <div v-if="backups.length === 0" class="sol-muted sol-text-xs">
                No backups created yet for this instance.
              </div>
              <div v-for="b in backups" :key="b.id" class="sol-content-row">
                <div class="sol-content-info">
                  <div class="sol-content-name">{{ b.name }}</div>
                  <div class="sol-muted sol-text-xs">{{ formatRelativeTime(b.createdAt) }} • {{ formatBytes(b.size) }}</div>
                </div>
                <div class="sol-row-gap">
                  <SolaraButton variant="secondary" size="xs" @click="restoreBackup(b)">Restore</SolaraButton>
                  <SolaraButton variant="danger" size="xs" @click="deleteBackup(b)">Delete</SolaraButton>
                </div>
              </div>
            </section>

            <!-- Danger Zone -->
            <section v-else-if="settingsSection === 'danger'" class="sol-card sol-stack sol-danger-card">
              <h2 class="sol-card-title">Danger Zone</h2>
              <p class="sol-muted sol-text-xs">
                Deleting an instance removes its files, worlds, and mods permanently from disk.
              </p>
              <div class="sol-row-gap">
                <SolaraButton variant="secondary" size="sm" @click="handleOverflowAction('duplicate')">
                  Duplicate Instance
                </SolaraButton>
                <SolaraButton
                  variant="danger"
                  size="sm"
                  :disabled="Boolean(instance.running)"
                  @click="showDeleteDialog = true"
                >
                  Delete Instance
                </SolaraButton>
              </div>
            </section>
          </div>
        </div>
      </div>
    </div>

    <!-- Sticky Save Bar inside Instance Settings -->
    <SolaraSaveBar
      v-if="activeTab === 'settings'"
      :dirty="isSettingsDirty"
      :saving="savingSettings"
      @reset="syncDraftFromInstance"
      @save="saveInstanceSettings"
    />

    <!-- Context / Overflow Menu & Modals -->
    <SolaraContextMenu
      :open="overflowMenu.open"
      :x="overflowMenu.x"
      :y="overflowMenu.y"
      :items="overflowItems"
      @close="overflowMenu.open = false"
      @select="handleOverflowAction"
    />

    <ModUpdateModal
      :open="showUpdateModal"
      :instance-id="instance.id"
      :instance-name="instance.name"
      @close="showUpdateModal = false"
      @updated="loadContentTab"
    />

    <ConfirmDialog
      :open="showDeleteDialog"
      title="Delete Instance"
      :message="`Permanently delete '${instance.name}' and all its saves, mods, and configs?`"
      confirm-label="Delete Instance"
      :danger="true"
      :require-text="instance.name"
      :loading="deleting"
      @cancel="showDeleteDialog = false"
      @confirm="confirmDelete"
    />

    <!-- File Text Editor Modal -->
    <SolaraModal
      :open="Boolean(editingFile)"
      :title="editingFile?.path || 'Edit File'"
      width="lg"
      @close="editingFile = null"
    >
      <textarea
        v-if="editingFile"
        v-model="editingFile.content"
        class="sol-code-editor selectable"
        rows="18"
        @input="editingFile.dirty = true"
      />
      <template #footer>
        <SolaraButton variant="ghost" @click="editingFile = null">Close</SolaraButton>
        <SolaraButton variant="primary" :disabled="!editingFile?.dirty" @click="saveEditedFile">
          Save File
        </SolaraButton>
      </template>
    </SolaraModal>

    <!-- Image / Screenshot Lightbox Modal -->
    <SolaraModal
      :open="Boolean(previewImage || lightboxShot)"
      :title="previewImage?.path || lightboxShot?.fileName || 'Preview'"
      width="lg"
      @close="closeMediaLightbox"
    >
      <img
        v-if="previewImage?.dataUrl || lightboxShotSrc"
        :src="previewImage?.dataUrl || lightboxShotSrc || undefined"
        alt="Screenshot preview"
        class="sol-lightbox-img"
      />
      <div v-else-if="lightboxShot" class="sol-lightbox-placeholder">
        <Camera :size="26" />
        <span v-if="lightboxLoading">Loading full-resolution image...</span>
        <span v-else-if="lightboxError" class="sol-muted sol-text-xs">{{ lightboxError }}</span>
      </div>
      <template v-if="lightboxShot" #footer>
        <SolaraButton variant="secondary" size="sm" @click="copyScreenshotPath(lightboxShot)">
          <Copy :size="13" /> Copy file path
        </SolaraButton>
        <SolaraButton
          variant="secondary"
          size="sm"
          @click="native('open_instance_folder', { id: lightboxShot.instanceId, path: 'screenshots' }).catch(() => {})"
        >
          <FolderOpen :size="13" /> Open Folder
        </SolaraButton>
        <SolaraButton variant="danger" size="sm" @click="deleteScreenshot(lightboxShot)">
          <Trash2 :size="13" /> Move to Trash
        </SolaraButton>
        <SolaraButton variant="secondary" size="sm" @click="closeMediaLightbox">Close</SolaraButton>
      </template>
    </SolaraModal>

    <!-- New Folder Modal -->
    <SolaraModal :open="newFolderModal" title="Create Folder" width="sm" @close="newFolderModal = false">
      <div class="sol-field">
        <label class="sol-label">Folder Name</label>
        <input v-model="newFolderName" type="text" class="sol-input" @keydown.enter="createFolder" />
      </div>
      <template #footer>
        <SolaraButton variant="ghost" @click="newFolderModal = false">Cancel</SolaraButton>
        <SolaraButton variant="primary" :disabled="!newFolderName.trim()" @click="createFolder">Create</SolaraButton>
      </template>
    </SolaraModal>

    <!-- Rename World Modal -->
    <SolaraModal :open="Boolean(renameWorldTarget)" title="Rename World Folder" width="sm" @close="renameWorldTarget = null">
      <div class="sol-field">
        <label class="sol-label">Folder Name</label>
        <input v-model="renameWorldFolder" type="text" class="sol-input" @keydown.enter="confirmRenameWorld" />
      </div>
      <template #footer>
        <SolaraButton variant="ghost" @click="renameWorldTarget = null">Cancel</SolaraButton>
        <SolaraButton variant="primary" :disabled="!renameWorldFolder.trim()" @click="confirmRenameWorld">Rename</SolaraButton>
      </template>
    </SolaraModal>
  </div>

  <SolaraEmptyState
    v-else
    title="Instance not found"
    description="Select an instance from your Library or create a new one."
  >
    <template #actions>
      <SolaraButton variant="primary" @click="navigate('library')">Open Library</SolaraButton>
    </template>
  </SolaraEmptyState>
</template>
