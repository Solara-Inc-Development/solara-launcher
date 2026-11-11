<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Minus, Maximize2, X, RefreshCw, Sparkles } from '../../packages/ui/icons';
import { native, isDesktop } from '../../packages/shared/client';

const checking = ref(false);
const hasChecked = ref(false);
const updateMessage = ref('');
const isError = ref(false);
const isSuccess = ref(false);
const appUpdate = ref<any>({ state: 'idle', downloaded: 0, total: null });

// Animated Loading Screen States
const loading = ref(true);
const loadingStage = ref('Starting launcher services…');
const loadingProgress = ref(15);
const loadingFading = ref(false);

let pollTimer: ReturnType<typeof setInterval> | null = null;
let lastWindowActionTime = 0;
let lastWindowActionType = '';
let startupAbort = false;

const progressPercent = computed(() => {
  if (appUpdate.value?.total && appUpdate.value.total > 0 && typeof appUpdate.value.downloaded === 'number') {
    return Math.min(100, Math.max(0, Math.round((appUpdate.value.downloaded / appUpdate.value.total) * 100)));
  }
  return 0;
});

const updateProgressText = computed(() => {
  if (appUpdate.value?.total && appUpdate.value.total > 0) {
    const dl = (appUpdate.value.downloaded / 1048576).toFixed(1);
    const tot = (appUpdate.value.total / 1048576).toFixed(1);
    return `${dl} MB of ${tot} MB (${progressPercent.value}%)`;
  }
  return 'Downloading signed update…';
});

async function runStartupSequence() {
  startupAbort = false;
  loading.value = true;
  loadingFading.value = false;
  loadingProgress.value = 15;
  loadingStage.value = 'Starting launcher services…';

  const p1 = new Promise(r => setTimeout(r, 350));
  let initPromise = Promise.resolve();
  if (isDesktop) {
    initPromise = native('initialize_launcher').catch((e) => {
      console.warn('Initialize launcher error:', e);
    });
  }
  await Promise.all([p1, initPromise]);
  if (startupAbort) return;

  loadingProgress.value = 50;
  loadingStage.value = 'Connecting to Solara network…';

  const p2 = new Promise(r => setTimeout(r, 380));
  let pollPromise = Promise.resolve();
  if (isDesktop) {
    pollPromise = pollStatus().catch(() => {});
  }
  await Promise.all([p2, pollPromise]);
  if (startupAbort) return;

  loadingProgress.value = 85;
  loadingStage.value = 'Verifying workspace readiness…';

  await new Promise(r => setTimeout(r, 320));
  if (startupAbort) return;

  loadingProgress.value = 100;
  loadingStage.value = 'Ready';

  await new Promise(r => setTimeout(r, 260));
  if (startupAbort) return;

  loadingFading.value = true;
  setTimeout(() => {
    if (!startupAbort) {
      loading.value = false;
      loadingFading.value = false;
    }
  }, 420);
}

function skipLoading() {
  startupAbort = true;
  loadingFading.value = false;
  loading.value = false;
}

function replayLoading() {
  void runStartupSequence();
}

async function windowAction(action: string) {
  const now = Date.now();
  if (lastWindowActionType === action && now - lastWindowActionTime < 300) return;
  lastWindowActionTime = now;
  lastWindowActionType = action;

  if (isDesktop) {
    try {
      await native('window_action', { action });
    } catch {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const win = getCurrentWindow();
        if (action === 'close') await win.close();
        else if (action === 'minimize') await win.minimize();
        else if (action === 'maximize') await win.toggleMaximize();
      } catch (err) {
        console.error('Window action failed:', err);
      }
    }
  }
}

function onWindowButtonUp(action: string, e: MouseEvent | PointerEvent) {
  const btn = e.currentTarget as HTMLElement | null;
  if (!btn) return;
  const rect = btn.getBoundingClientRect();
  if (e.clientX >= rect.left && e.clientX <= rect.right && e.clientY >= rect.top && e.clientY <= rect.bottom) {
    void windowAction(action);
  }
}

async function onTitlebarMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  const target = e.target as HTMLElement | null;
  if (target?.closest('.window-buttons,button')) return;
  if (isDesktop) {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().startDragging();
    } catch {
      void native('start_dragging').catch(() => {});
    }
  }
}

function onTitlebarDblClick(e: MouseEvent) {
  if ((e.target as HTMLElement)?.closest('.window-buttons,button') === null) {
    void windowAction('maximize');
  }
}

async function checkUpdates() {
  if (checking.value) return;
  checking.value = true;
  hasChecked.value = true;
  isError.value = false;
  isSuccess.value = false;
  updateMessage.value = 'Checking for updates…';

  try {
    if (isDesktop) {
      const res = await native('check_update');
      if (res?.available) {
        isSuccess.value = true;
        updateMessage.value = `Update ${res.version ? 'v' + res.version + ' ' : ''}is available! Downloading…`;
      } else if (res?.error) {
        isError.value = true;
        updateMessage.value = res.error === 'unconfigured' ? 'Updater is not configured.' : String(res.error);
      } else {
        isSuccess.value = true;
        updateMessage.value = 'You are up to date.';
      }
    } else {
      // Browser preview mode
      await new Promise(r => setTimeout(r, 400));
      isSuccess.value = true;
      updateMessage.value = 'You are up to date.';
    }
  } catch (err: any) {
    isError.value = true;
    updateMessage.value = String(err?.message || err || 'Failed to check for updates');
  } finally {
    checking.value = false;
  }
}

async function pollStatus() {
  if (!isDesktop) return;
  try {
    const s = await native('app_update_status');
    if (s && typeof s === 'object') {
      appUpdate.value = s;
      if (s.state === 'downloading') {
        updateMessage.value = 'Downloading update…';
      } else if (s.state === 'installing') {
        updateMessage.value = 'Installing update…';
      } else if (s.state === 'waiting-for-idle') {
        updateMessage.value = 'Update downloaded. Will install when idle.';
      } else if (s.state === 'up-to-date') {
        if (hasChecked.value && !checking.value) {
          updateMessage.value = 'You are up to date.';
        }
      }
    }
  } catch {}
}

onMounted(async () => {
  await runStartupSequence();
  if (isDesktop) {
    pollTimer = setInterval(() => void pollStatus(), 3000);
  }
});

onUnmounted(() => {
  startupAbort = true;
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
  <div class="shell" data-theme="dark">
    <div class="titlebar">
      <div class="titlebar-drag-area" data-tauri-drag-region @mousedown="onTitlebarMouseDown" @dblclick="onTitlebarDblClick">
        <span class="name" data-tauri-drag-region>
          <img src="/logo.svg" alt="Solara">
          <span>solara</span>
        </span>
      </div>
      <div class="window-buttons" data-tauri-drag-region="false" @pointerdown.stop @mousedown.stop>
        <button type="button" data-tauri-drag-region="false" title="Minimize" aria-label="Minimize" :disabled="!isDesktop" @pointerdown.stop @mousedown.stop @pointerup.stop="onWindowButtonUp('minimize',$event)" @mouseup.stop="onWindowButtonUp('minimize',$event)" @click.stop="windowAction('minimize')"><Minus/></button>
        <button type="button" data-tauri-drag-region="false" title="Maximize or restore" aria-label="Maximize or restore" :disabled="!isDesktop" @pointerdown.stop @mousedown.stop @pointerup.stop="onWindowButtonUp('maximize',$event)" @mouseup.stop="onWindowButtonUp('maximize',$event)" @click.stop="windowAction('maximize')"><Maximize2/></button>
        <button type="button" data-tauri-drag-region="false" title="Close Solara" aria-label="Close Solara" :disabled="!isDesktop" @pointerdown.stop @mousedown.stop @pointerup.stop="onWindowButtonUp('close',$event)" @mouseup.stop="onWindowButtonUp('close',$event)" @click.stop="windowAction('close')"><X/></button>
      </div>
    </div>

    <!-- Animated Loading Screen Overlay -->
    <div
      v-if="loading"
      class="loading-screen"
      :class="{ 'fade-out': loadingFading }"
      role="status"
      aria-live="polite"
    >
      <div class="loading-ambient-glow" aria-hidden="true"></div>
      <div class="loading-content">
        <div class="loading-emblem-wrap">
          <div class="loading-halo" aria-hidden="true"></div>
          <div class="loading-emblem">
            <img src="/logo.svg" alt="Solara" class="loading-logo" />
          </div>
        </div>

        <div class="loading-brand">
          <h2 class="loading-title">solara</h2>
          <span class="loading-subtitle">Launcher</span>
        </div>

        <div class="loading-track" role="progressbar" :aria-valuenow="loadingProgress" aria-valuemin="0" aria-valuemax="100">
          <div class="loading-progress" :style="{ width: loadingProgress + '%' }">
            <div class="loading-shimmer" aria-hidden="true"></div>
          </div>
        </div>

        <div class="loading-meta">
          <span class="loading-pulse-dot" aria-hidden="true"></span>
          <span class="loading-status-text">{{ loadingStage }}</span>
        </div>

        <button type="button" class="loading-skip-btn" @click="skipLoading" title="Skip animation">
          Skip
        </button>
      </div>
    </div>

    <!-- Main Clean Container -->
    <main v-show="!loading || loadingFading" class="clean-container">
      <h1 class="greeting">HI ;)</h1>
      <button class="update-btn" :disabled="checking" @click="checkUpdates">
        <span v-if="checking" class="spinner"></span>
        <RefreshCw v-else :size="16" />
        <span>{{ checking ? 'Checking for updates…' : 'Check for updates' }}</span>
      </button>

      <div v-if="updateMessage" class="update-status" :class="{ error: isError, success: isSuccess }">
        <span class="status-dot"></span>
        <span>{{ updateMessage }}</span>
      </div>

      <div v-if="appUpdate?.state === 'downloading'" class="download-progress">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
        </div>
        <span class="progress-info">{{ updateProgressText }}</span>
      </div>

      <button type="button" class="preview-anim-btn" @click="replayLoading">
        <Sparkles :size="14" />
        <span>Preview loading animation</span>
      </button>
    </main>
  </div>
</template>
