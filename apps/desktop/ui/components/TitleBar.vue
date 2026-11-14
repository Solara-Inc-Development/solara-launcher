<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import SolaraLogo from './SolaraLogo.vue';
import {
  Activity,
  ChevronLeft,
  ChevronRight,
  Minus,
  Maximize2,
  Play,
  RefreshCw,
  Search,
  Square,
  X,
} from '../icons';
import {
  activeJobs,
  canGoBack,
  canGoForward,
  currentPageTitle,
  currentRoute,
  formatPublicVersion,
  goBack,
  goForward,
  installAppUpdate,
  native,
  navigate,
  notify,
  runningInstances,
  selectedInstance,
  store,
} from '../state';

const isMaximized = ref(false);
let winEventTimer: number | null = null;
let lastWinActionAt = 0;

async function syncMaximizedState(): Promise<void> {
  if (!store.desktop) return;
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    isMaximized.value = await getCurrentWindow().isMaximized();
  } catch {
    // ignore outside Tauri webview
  }
}

async function winAction(action: 'minimize' | 'maximize' | 'close'): Promise<void> {
  if (!store.desktop) return;
  const now = Date.now();
  if (now - lastWinActionAt < 180) return;
  lastWinActionAt = now;
  try {
    await native('window_action', { action });
    if (action === 'maximize') {
      isMaximized.value = !isMaximized.value;
    }
  } catch (err) {
    notify('error', 'Window action failed', String(err));
  }
}

async function startDrag(e: MouseEvent): Promise<void> {
  if (e.button !== 0 || !store.desktop) return;
  const target = e.target as HTMLElement | null;
  if (target?.closest('button, input, a, .sol-no-drag')) return;
  try {
    if (e.detail >= 2) {
      await winAction('maximize');
    } else {
      await native('start_dragging');
    }
  } catch {
    // ignore in browser preview
  }
}

const activeLaunchEntry = computed(() => {
  for (const [id, st] of Object.entries(store.launchStates)) {
    if (st && st.stage !== 'running') {
      const inst = store.instances.find((i) => i.id === id);
      return { id, name: inst?.name ?? 'Minecraft', ...st };
    }
  }
  return null;
});

const breadcrumbExtra = computed(() => {
  const r = currentRoute.value;
  if (r.page === 'instance' && selectedInstance.value) {
    const inst = store.instances.find((i) => i.id === r.instanceId) ?? selectedInstance.value;
    return `${inst.loader} ${inst.gameVersion}`;
  }
  return null;
});

const updateAvailable = computed(
  () =>
    store.updateStatus?.stage === 'ready' ||
    store.updateStatus?.stage === 'available' ||
    store.updateStatus?.stage === 'downloading' ||
    store.updateStatus?.stage === 'installing' ||
    store.updateStatus?.stage === 'waiting-for-idle',
);

const updatePillLabel = computed(() => {
  const st = store.updateStatus?.stage;
  if (st === 'installing') return 'Restarting...';
  if (st === 'downloading') return 'Downloading Update...';
  return 'Restart to Update';
});

const updatePillTitle = computed(() => {
  const ver = store.updateStatus?.publicVersion || store.updateStatus?.version;
  return ver ? `Restart to update to ${formatPublicVersion(ver)}` : 'Restart to update Solara Launcher';
});

async function handleUpdatePillClick(): Promise<void> {
  if (!store.desktop) return;
  if (store.updateStatus?.stage === 'downloading' || store.updateStatus?.stage === 'installing') {
    store.showUpdateModal = true;
    return;
  }
  await installAppUpdate();
}

onMounted(() => {
  void syncMaximizedState();
  winEventTimer = window.setInterval(() => {
    if (!document.hidden) void syncMaximizedState();
  }, 4000);
});

onBeforeUnmount(() => {
  if (winEventTimer !== null) window.clearInterval(winEventTimer);
});
</script>

<template>
  <header class="sol-titlebar" data-tauri-drag-region @mousedown="startDrag">
    <div class="sol-titlebar-left" data-tauri-drag-region>
      <button
        type="button"
        class="sol-brand-btn sol-no-drag"
        title="Go to Home"
        @click="navigate('home')"
      >
        <SolaraLogo :size="26" />
        <span class="sol-brand-name">Solara</span>
      </button>

      <div class="sol-nav-history sol-no-drag">
        <button
          type="button"
          class="sol-icon-btn"
          :disabled="!canGoBack"
          title="Back (Alt+Left)"
          aria-label="Back"
          @click="goBack"
        >
          <ChevronLeft :size="18" />
        </button>
        <button
          type="button"
          class="sol-icon-btn"
          :disabled="!canGoForward"
          title="Forward (Alt+Right)"
          aria-label="Forward"
          @click="goForward"
        >
          <ChevronRight :size="18" />
        </button>
      </div>

      <div class="sol-titlebar-crumb" data-tauri-drag-region>
        <span class="sol-crumb-title">{{ currentPageTitle }}</span>
        <span v-if="breadcrumbExtra" class="sol-crumb-sub">{{ breadcrumbExtra }}</span>
      </div>
    </div>

    <div class="sol-titlebar-center" data-tauri-drag-region>
      <button
        type="button"
        class="sol-search-trigger sol-no-drag"
        title="Search instances &amp; modpacks (Ctrl+K)"
        @click="store.showGlobalSearch = true"
      >
        <Search :size="15" />
        <span>Search instances, modpacks, tools...</span>
        <kbd>Ctrl K</kbd>
      </button>
    </div>

    <div class="sol-titlebar-right" data-tauri-drag-region>
      <!-- Live launch / running status pill -->
      <button
        v-if="activeLaunchEntry"
        type="button"
        class="sol-status-pill sol-no-drag"
        :class="activeLaunchEntry.stage === 'error' ? 'sol-status-pill--error' : 'sol-status-pill--launching'"
        :title="activeLaunchEntry.error || activeLaunchEntry.detail"
        @click="navigate('instance', { instanceId: activeLaunchEntry.id, tab: 'logs' })"
      >
        <span class="sol-dot" />
        <span class="sol-truncate">
          {{ activeLaunchEntry.stage === 'error' ? `Launch Failed · View Logs` : `${activeLaunchEntry.name}: ${activeLaunchEntry.detail}` }}
        </span>
      </button>
      <button
        v-else-if="runningInstances.length > 0"
        type="button"
        class="sol-status-pill sol-status-pill--running sol-no-drag"
        title="View running instance logs"
        @click="navigate('instance', { instanceId: runningInstances[0].id, tab: 'logs' })"
      >
        <Play :size="11" />
        <span>Running: {{ runningInstances[0].name }}</span>
      </button>

      <!-- Restart to Update Action -->
      <button
        v-if="updateAvailable"
        type="button"
        class="sol-status-pill sol-status-pill--accent sol-no-drag"
        :title="updatePillTitle"
        @click="handleUpdatePillClick"
      >
        <RefreshCw :size="14" />
        <span>{{ updatePillLabel }}</span>
      </button>

      <!-- Tasks Popover Trigger -->
      <button
        type="button"
        class="sol-icon-btn sol-tasks-btn sol-tasks-anchor sol-no-drag"
        :class="{ 'is-active': store.showTasksPopover || activeJobs.length > 0 }"
        title="Downloads &amp; Tasks (Ctrl+J)"
        aria-label="Tasks"
        @click="store.showTasksPopover = !store.showTasksPopover"
      >
        <Activity :size="18" />
        <span v-if="activeJobs.length > 0" class="sol-badge-count">{{ activeJobs.length }}</span>
      </button>

      <div class="sol-win-controls sol-no-drag">
        <button
          type="button"
          class="sol-win-btn"
          title="Minimize"
          aria-label="Minimize window"
          @click="winAction('minimize')"
        >
          <Minus :size="15" />
        </button>
        <button
          type="button"
          class="sol-win-btn"
          :title="isMaximized ? 'Restore' : 'Maximize'"
          aria-label="Maximize or restore window"
          @click="winAction('maximize')"
        >
          <Square v-if="isMaximized" :size="13" />
          <Maximize2 v-else :size="14" />
        </button>
        <button
          type="button"
          class="sol-win-btn sol-win-btn--close"
          title="Close"
          aria-label="Close window"
          @click="winAction('close')"
        >
          <X :size="16" />
        </button>
      </div>
    </div>
  </header>
</template>
