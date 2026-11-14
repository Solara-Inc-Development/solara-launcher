<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  ArrowLeft,
  ArrowRight,
  Search,
  Download,
  RefreshCw,
  Minus,
  Square,
  X,
  Play,
} from '../icons';
import {
  store,
  currentRoute,
  currentPageTitle,
  canGoBack,
  canGoForward,
  goBack,
  goForward,
  selectedInstance,
  runningInstances,
  activeJobs,
  navigate,
  native,
  notify,
  refreshUpdateStatus,
} from '../state';
import TasksPopover from './TasksPopover.vue';

const checkingUpdate = ref(false);
let lastWinClickAt = 0;
let lastWinAction: string | null = null;

const updateBadge = computed(() => {
  const s = store.updateStatus.stage;
  if (s === 'checking') return { label: 'Checking update...', tone: 'info' };
  if (s === 'downloading') return { label: 'Downloading update', tone: 'gold' };
  if (s === 'ready' || s === 'downloaded' || s === 'pending')
    return { label: `Update ${store.updateStatus.version || ''} ready`, tone: 'success' };
  if (s === 'available')
    return { label: `Update ${store.updateStatus.version || ''}`, tone: 'gold' };
  if (s === 'error') return { label: 'Update check failed', tone: 'warning' };
  return null;
});

async function handleCheckUpdate() {
  if (!store.desktop || checkingUpdate.value) return;
  checkingUpdate.value = true;
  try {
    await native('check_update');
    await refreshUpdateStatus();
    if (store.updateStatus.stage === 'idle' || store.updateStatus.stage === 'current') {
      notify('info', 'Solara is up to date', 'You are running the latest available version.');
    }
  } catch (err) {
    notify('warning', 'Update check unavailable', String(err));
  } finally {
    checkingUpdate.value = false;
  }
}

function onTitleMouseDown(e: MouseEvent) {
  if (!store.desktop || e.button !== 0) return;
  const target = e.target as HTMLElement | null;
  if (target?.closest('button, input, a, .sol-no-drag')) return;
  void native('start_dragging').catch(() => {});
}

function onTitleDblClick(e: MouseEvent) {
  if (!store.desktop) return;
  const target = e.target as HTMLElement | null;
  if (target?.closest('button, input, a, .sol-no-drag')) return;
  void native('window_action', { action: 'maximize' }).catch(() => {});
}

function onWindowAction(action: 'minimize' | 'maximize' | 'close') {
  const now = Date.now();
  if (lastWinAction === action && now - lastWinClickAt < 180) return;
  lastWinClickAt = now;
  lastWinAction = action;
  if (!store.desktop) return;
  void native('window_action', { action }).catch((err) => {
    notify('error', 'Window action failed', String(err));
  });
}
</script>

<template>
  <header
    class="sol-titlebar"
    data-tauri-drag-region
    @mousedown="onTitleMouseDown"
    @dblclick="onTitleDblClick"
  >
    <!-- Brand & Navigation -->
    <div class="sol-titlebar-left">
      <button
        type="button"
        class="sol-brand-btn sol-no-drag"
        title="Go to Home"
        @click="navigate('home')"
      >
        <span class="sol-brand-mark" aria-hidden="true">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path d="M12 2L20.5 7V17L12 22L3.5 17V7L12 2Z" stroke="#E9B86D" stroke-width="2" />
            <circle cx="12" cy="12" r="3.2" fill="#E9B86D" />
          </svg>
        </span>
        <span class="sol-brand-name">Solara</span>
      </button>

      <div class="sol-nav-arrows sol-no-drag">
        <button
          type="button"
          class="sol-icon-btn sol-icon-btn--sm"
          :disabled="!canGoBack"
          title="Back (Alt+Left)"
          aria-label="Go back"
          @click="goBack"
        >
          <ArrowLeft :size="15" />
        </button>
        <button
          type="button"
          class="sol-icon-btn sol-icon-btn--sm"
          :disabled="!canGoForward"
          title="Forward (Alt+Right)"
          aria-label="Go forward"
          @click="goForward"
        >
          <ArrowRight :size="15" />
        </button>
      </div>

      <div class="sol-breadcrumb">
        <span class="sol-breadcrumb-page">{{ currentPageTitle }}</span>
        <template v-if="currentRoute.page === 'instance' && selectedInstance">
          <span class="sol-breadcrumb-sep">/</span>
          <span class="sol-breadcrumb-sub">{{ selectedInstance.gameVersion }} {{ selectedInstance.loader }}</span>
        </template>
        <template v-else-if="selectedInstance && currentRoute.page !== 'settings' && currentRoute.page !== 'account'">
          <button
            type="button"
            class="sol-active-inst-pill sol-no-drag"
            title="Open selected instance"
            @click="navigate('instance', { instanceId: selectedInstance.id })"
          >
            <span class="sol-pill-dot" :class="{ 'is-running': selectedInstance.running }" />
            <span class="sol-pill-text">{{ selectedInstance.name }}</span>
          </button>
        </template>
      </div>
    </div>

    <!-- Center Quick Search -->
    <div class="sol-titlebar-center">
      <button
        type="button"
        class="sol-cmd-trigger sol-no-drag"
        title="Quick Search (Ctrl+K)"
        @click="store.showGlobalSearch = true"
      >
        <Search :size="13" />
        <span>Search instances & pages...</span>
        <kbd class="sol-kbd">Ctrl+K</kbd>
      </button>
    </div>

    <!-- Right Indicators & Native Controls -->
    <div class="sol-titlebar-right sol-no-drag">
      <button
        v-if="runningInstances.length > 0"
        type="button"
        class="sol-status-pill sol-status-pill--running"
        :title="`Minecraft running: ${runningInstances.map((i) => i.name).join(', ')}`"
        @click="navigate('instance', { instanceId: runningInstances[0].id, tab: 'logs' })"
      >
        <Play :size="11" />
        <span>{{ runningInstances.length === 1 ? 'Game Running' : `${runningInstances.length} Running` }}</span>
      </button>

      <button
        v-if="updateBadge"
        type="button"
        :class="['sol-status-pill', `sol-status-pill--${updateBadge.tone}`]"
        :disabled="checkingUpdate"
        title="Application update status"
        @click="handleCheckUpdate"
      >
        <RefreshCw :size="12" :class="{ 'is-spinning': checkingUpdate || store.updateStatus.stage === 'checking' }" />
        <span>{{ updateBadge.label }}</span>
      </button>
      <button
        v-else-if="store.desktop"
        type="button"
        class="sol-icon-btn sol-icon-btn--sm"
        :disabled="checkingUpdate"
        title="Check for launcher updates"
        aria-label="Check for updates"
        @click="handleCheckUpdate"
      >
        <RefreshCw :size="14" :class="{ 'is-spinning': checkingUpdate }" />
      </button>

      <div class="sol-tasks-anchor">
        <button
          type="button"
          :class="['sol-icon-btn', 'sol-icon-btn--sm', { 'is-active': store.showTasksPopover || activeJobs.length > 0 }]"
          title="Downloads & Tasks"
          aria-label="Toggle tasks popover"
          @click="store.showTasksPopover = !store.showTasksPopover"
        >
          <Download :size="15" />
          <span v-if="activeJobs.length > 0" class="sol-count-dot">{{ activeJobs.length }}</span>
        </button>
        <TasksPopover v-if="store.showTasksPopover" @close="store.showTasksPopover = false" />
      </div>

      <div class="sol-win-controls">
        <button
          type="button"
          class="sol-win-btn"
          title="Minimize"
          aria-label="Minimize window"
          @click="onWindowAction('minimize')"
        >
          <Minus :size="14" />
        </button>
        <button
          type="button"
          class="sol-win-btn"
          title="Maximize or Restore"
          aria-label="Maximize or restore window"
          @click="onWindowAction('maximize')"
        >
          <Square :size="11" />
        </button>
        <button
          type="button"
          class="sol-win-btn sol-win-btn--close"
          title="Close"
          aria-label="Close window"
          @click="onWindowAction('close')"
        >
          <X :size="14" />
        </button>
      </div>
    </div>
  </header>
</template>
