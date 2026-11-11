<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { Minus, Maximize2, X, RefreshCw } from '../../packages/ui/icons';
import { native, isDesktop } from '../../packages/shared/client';

const checking = ref(false);
const hasChecked = ref(false);
const updateMessage = ref('');
const isError = ref(false);
const isSuccess = ref(false);
const appUpdate = ref<any>({ state: 'idle', downloaded: 0, total: null });

let pollTimer: ReturnType<typeof setInterval> | null = null;
let lastWindowActionTime = 0;
let lastWindowActionType = '';

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
  if (isDesktop) {
    try {
      await native('initialize_launcher');
    } catch (e) {
      console.warn('Initialize launcher error:', e);
    }
    await pollStatus();
    pollTimer = setInterval(() => void pollStatus(), 3000);
  }
});

onUnmounted(() => {
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

    <main class="clean-container">
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
    </main>
  </div>
</template>
