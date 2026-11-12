<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Minus, Maximize2, X, RefreshCw, Sparkles, Home, Discord } from '../../packages/ui/icons';
import { native, isDesktop } from '../../packages/shared/client';

const checking = ref(false);
const hasChecked = ref(false);
const updateMessage = ref('');
const isError = ref(false);
const isSuccess = ref(false);
const appUpdate = ref<any>({ state: 'idle', downloaded: 0, total: null });

// Canonical Animated Loading Screen States
const loading = ref(true);
const loadingProgress = ref(0);
const loadingLeaving = ref(false);

const loadingStatusText = computed(() => {
  if (loadingProgress.value < 25) return 'Reading Files…';
  if (loadingProgress.value < 55) return 'Checking Integrity…';
  if (loadingProgress.value < 85) return 'Preparing Environment…';
  return 'Launching Solara-Launcher…';
});

let startupInterval: ReturnType<typeof setInterval> | null = null;
let pollTimer: ReturnType<typeof setInterval> | null = null;
let lastWindowActionTime = 0;
let lastWindowActionType = '';
let startupAbort = false;
let currentSequenceId = 0;

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
  const seqId = ++currentSequenceId;
  startupAbort = false;
  if (startupInterval) {
    clearInterval(startupInterval);
    startupInterval = null;
  }
  loading.value = true;
  loadingLeaving.value = false;
  loadingProgress.value = 0;

  let initPromise = Promise.resolve();
  if (isDesktop) {
    initPromise = native('initialize_launcher').catch((e) => {
      console.warn('Initialize launcher error:', e);
    });
  }

  // Strict 5 to 10 seconds random duration
  const totalDuration = Math.floor(5000 + Math.random() * 5000);
  const startTime = Date.now();

  await new Promise<void>((resolve) => {
    startupInterval = setInterval(() => {
      if (startupAbort || seqId !== currentSequenceId) {
        if (startupInterval) clearInterval(startupInterval);
        resolve();
        return;
      }
      const elapsed = Date.now() - startTime;
      const progressFraction = Math.min(1, elapsed / totalDuration);
      loadingProgress.value = Math.min(100, Math.round(progressFraction * 100));

      if (elapsed >= totalDuration || loadingProgress.value >= 100) {
        loadingProgress.value = 100;
        if (startupInterval) {
          clearInterval(startupInterval);
          startupInterval = null;
        }
        resolve();
      }
    }, 40);
  });

  if (startupAbort || seqId !== currentSequenceId) return;

  await initPromise;
  if (seqId !== currentSequenceId) return;

  if (isDesktop) {
    void pollStatus().catch(() => {});
  }

  // Hold 100% briefly so user perceives completion
  await new Promise((r) => setTimeout(r, 300));
  if (startupAbort || seqId !== currentSequenceId) return;

  // Split curtain exit animation
  loadingLeaving.value = true;
  await new Promise((r) => setTimeout(r, 600));
  if (!startupAbort && seqId === currentSequenceId) {
    loading.value = false;
    loadingLeaving.value = false;
  }
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
  if (target?.closest('.window-buttons,button,input,a')) return;
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
  if ((e.target as HTMLElement)?.closest('.window-buttons,button,input,a') === null) {
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

async function openExternal(url: string) {
  if (isDesktop) {
    try {
      await native('open_url', { url });
      return;
    } catch {}
    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener');
      await openUrl(url);
      return;
    } catch {}
  }
  if (typeof window !== 'undefined') {
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}

function openDiscord() {
  void openExternal('https://discord.solara-inc.in/');
}

onMounted(async () => {
  await runStartupSequence();
  if (isDesktop) {
    pollTimer = setInterval(() => void pollStatus(), 3000);
  }
});

onUnmounted(() => {
  startupAbort = true;
  currentSequenceId++;
  if (startupInterval) clearInterval(startupInterval);
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
  <div class="shell" data-theme="dark">
    <!-- Top Bar / Header: Clean Minimal Modrinth-Inspired Top Bar -->
    <header class="titlebar">
      <div
        class="titlebar-drag-area"
        data-tauri-drag-region
        @mousedown="onTitlebarMouseDown"
        @dblclick="onTitlebarDblClick"
      >
        <!-- Modrinth Brand Logo + Wordmark -->
        <div class="header-brand" data-tauri-drag-region>
          <img src="/logo.svg" alt="Solara" class="brand-logo" data-tauri-drag-region />
          <span class="brand-name" data-tauri-drag-region>solara</span>
        </div>

        <!-- Draggable Spacer filling remaining header width -->
        <div class="titlebar-spacer" data-tauri-drag-region></div>
      </div>

      <!-- Window Control Buttons (Minimize, Maximize, Close) -->
      <div
        class="window-buttons"
        data-tauri-drag-region="false"
        @pointerdown.stop
        @mousedown.stop
      >
        <button
          type="button"
          data-tauri-drag-region="false"
          title="Minimize"
          aria-label="Minimize"
          :disabled="!isDesktop"
          @pointerdown.stop
          @mousedown.stop
          @pointerup.stop="onWindowButtonUp('minimize', $event)"
          @mouseup.stop="onWindowButtonUp('minimize', $event)"
          @click.stop="windowAction('minimize')"
        >
          <Minus />
        </button>
        <button
          type="button"
          data-tauri-drag-region="false"
          title="Maximize or restore"
          aria-label="Maximize or restore"
          :disabled="!isDesktop"
          @pointerdown.stop
          @mousedown.stop
          @pointerup.stop="onWindowButtonUp('maximize', $event)"
          @mouseup.stop="onWindowButtonUp('maximize', $event)"
          @click.stop="windowAction('maximize')"
        >
          <Maximize2 />
        </button>
        <button
          type="button"
          data-tauri-drag-region="false"
          title="Close Solara"
          aria-label="Close Solara"
          :disabled="!isDesktop"
          @pointerdown.stop
          @mousedown.stop
          @pointerup.stop="onWindowButtonUp('close', $event)"
          @mouseup.stop="onWindowButtonUp('close', $event)"
          @click.stop="windowAction('close')"
        >
          <X />
        </button>
      </div>
    </header>

    <!-- Canonical Animated Loading Screen Overlay (5-10s duration) -->
    <div
      v-if="loading"
      id="solara-website-loader"
      class="solara-loading-screen loading-screen solara-loading-full"
      :class="{ 'is-leaving': loadingLeaving }"
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <div class="solara-loader-content">
        <!-- Central prominent glowing Solara brand logo (/logo-512.png) -->
        <div class="solara-loader-mark" aria-hidden="true">
          <div class="solara-logo-glow" aria-hidden="true"></div>
          <img
            src="/logo-512.png"
            alt="Solara"
            class="loading-logo"
            width="96"
            height="96"
          />
        </div>

        <!-- Sleek, minimalist loading progress bar below the logo -->
        <div
          class="solara-loader-track loading-track"
          role="progressbar"
          :aria-valuenow="Math.round(loadingProgress)"
          aria-valuemin="0"
          aria-valuemax="100"
        >
          <div
            class="loading-progress-bar"
            :style="{ width: Math.min(100, Math.max(0, loadingProgress)) + '%' }"
          ></div>
        </div>

        <!-- Clean, simple status lines below the loading bar -->
        <div class="solara-loader-status loading-status-text">
          <span>{{ loadingStatusText }}</span>
        </div>

        <!-- Accurate numeric percentage below/with the bar -->
        <div class="solara-loader-meta-progress loading-percentage">
          <span>{{ Math.round(loadingProgress) }}%</span>
        </div>
      </div>
    </div>

    <!-- App Body: Sleek Horizon-inspired icon dock Sidebar + Minimal Clean Main View -->
    <div class="launcher-layout">
      <!-- Sidebar: Sleek minimal Horizon-inspired icon dock with a Discord button that opens https://discord.solara-inc.in/ -->
      <aside class="horizon-sidebar" aria-label="Sidebar Navigation">
        <!-- Top Dock Item (Home active indicator) -->
        <div class="dock-top">
          <button
            type="button"
            class="dock-item is-active"
            title="Home"
            aria-label="Home"
          >
            <Home :size="20" />
          </button>
        </div>

        <div class="dock-spacer"></div>

        <!-- Bottom Dock Item: Discord button opening https://discord.solara-inc.in/ -->
        <div class="dock-bottom">
          <a
            href="https://discord.solara-inc.in/"
            target="_blank"
            rel="noopener noreferrer"
            class="horizon-discord-btn dock-item"
            title="Join the Discord"
            aria-label="Join the Discord"
            @click.prevent="openDiscord"
          >
            <Discord :size="20" class="discord-icon" />
          </a>
        </div>
      </aside>

      <!-- Main Clean View with "HI ;)" and "Check for updates" button -->
      <main v-show="!loading || loadingLeaving" class="launcher-main clean-container">
        <h1 class="greeting">HI ;)</h1>
        <button class="update-btn" :disabled="checking" @click="checkUpdates">
          <span v-if="checking" class="spinner"></span>
          <RefreshCw v-else :size="16" />
          <span>{{ checking ? 'Checking for updates…' : 'Check for updates' }}</span>
        </button>

        <div
          v-if="updateMessage"
          class="update-status"
          :class="{ error: isError, success: isSuccess }"
        >
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
  </div>
</template>
