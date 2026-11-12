<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import {
  Minus, Maximize2, X, RefreshCw, Sparkles,
  ChevronLeft, ChevronRight, Play, Square, Download, Search,
  Plus, MoreVertical, Home, Layers3, Compass, Server, Settings,
  FolderOpen, ChevronDown, Discord, Heart, Globe, User
} from '../../packages/ui/icons';
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

// Navigation & Tab Management (Modrinth Header Breadcrumbs & History)
const activeTab = ref('home');
const navHistory = ref<string[]>(['home']);
const historyIndex = ref(0);

const currentTabTitle = computed(() => {
  switch (activeTab.value) {
    case 'home': return 'Home';
    case 'instances': return 'Instances';
    case 'mods': return 'Mods';
    case 'servers': return 'Servers';
    case 'settings': return 'Settings';
    default: return 'Home';
  }
});

function navigateTo(tab: string) {
  if (activeTab.value === tab) return;
  navHistory.value = navHistory.value.slice(0, historyIndex.value + 1);
  navHistory.value.push(tab);
  historyIndex.value = navHistory.value.length - 1;
  activeTab.value = tab;
}

function navigateBack() {
  if (historyIndex.value > 0) {
    historyIndex.value--;
    activeTab.value = navHistory.value[historyIndex.value];
  }
}

function navigateForward() {
  if (historyIndex.value < navHistory.value.length - 1) {
    historyIndex.value++;
    activeTab.value = navHistory.value[historyIndex.value];
  }
}

// Search & Instance State (Horizon & Modrinth Style)
interface Instance {
  id: string;
  name: string;
  version: string;
  loader: string;
  status: string;
  icon: string;
  playTime?: string;
  lastPlayed?: string;
}

const searchQuery = ref('');
const isGameRunning = ref(false);
const inviteCode = ref('');
const gameEdition = ref<'rust' | 'java'>('rust');

const activeInstance = ref<Instance>({
  id: 'solara-main',
  name: 'Solara Main',
  version: '1.21.4',
  loader: 'Fabric 26.3',
  status: 'Ready',
  icon: '/logo.svg',
  playTime: '27 hours',
  lastPlayed: 'Nov 12'
});

const instanceList = ref<Instance[]>([
  {
    id: 'solara-main',
    name: 'Solara Main',
    version: '1.21.4',
    loader: 'Fabric 26.3',
    status: 'Ready',
    icon: '/logo.svg',
    playTime: '27 hours',
    lastPlayed: 'Today'
  },
  {
    id: 'spinex-smp',
    name: 'Spinex SMP',
    version: '1.21.4',
    loader: 'Fabric 26.3',
    status: '0 online',
    icon: '/logo.svg',
    playTime: '12 hours',
    lastPlayed: '4 days ago'
  },
  {
    id: 'vanilla-exp',
    name: 'Minecraft 1.21.4',
    version: '1.21.4',
    loader: 'Vanilla',
    status: 'Ready',
    icon: '/logo.svg',
    playTime: '5 hours',
    lastPlayed: '2 weeks ago'
  }
]);

const hasUpdateAvailable = computed(() => {
  return isSuccess.value && updateMessage.value.toLowerCase().includes('available');
});

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

function openExternal(url: string) {
  if (typeof window !== 'undefined') {
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}

function openDiscord() {
  openExternal('https://discord.gg');
}

function handlePlay() {
  isGameRunning.value = !isGameRunning.value;
}

function openGameFolder() {
  if (isDesktop) {
    native('open_game_dir').catch(() => {});
  }
}

function handleJoinInvite() {
  if (!inviteCode.value.trim()) return;
  // Feedback
  console.log('Joining server:', inviteCode.value);
}

function toggleInstanceMenu() {
  // Instance menu action
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
    <!-- Top Bar / Header: Inspired by Modrinth's Header Design -->
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

        <!-- History Navigation Arrows (< >) -->
        <div class="header-nav-history" data-tauri-drag-region="false" @mousedown.stop>
          <button
            type="button"
            class="nav-history-btn"
            :disabled="historyIndex <= 0"
            title="Go back"
            aria-label="Go back"
            @click="navigateBack"
          >
            <ChevronLeft :size="15" />
          </button>
          <button
            type="button"
            class="nav-history-btn"
            :disabled="historyIndex >= navHistory.length - 1"
            title="Go forward"
            aria-label="Go forward"
            @click="navigateForward"
          >
            <ChevronRight :size="15" />
          </button>
        </div>

        <!-- Breadcrumb / Active Page Indicator (Modrinth Style: ▷ Home) -->
        <div class="header-breadcrumb" data-tauri-drag-region>
          <span class="breadcrumb-arrow" data-tauri-drag-region>▷</span>
          <span class="breadcrumb-current" data-tauri-drag-region>{{ currentTabTitle }}</span>
        </div>

        <!-- Header Draggable Spacer -->
        <div class="titlebar-spacer" data-tauri-drag-region></div>

        <!-- Clean Integrated Search Box -->
        <div class="header-search-box" data-tauri-drag-region="false" @mousedown.stop>
          <Search :size="13" class="search-icon" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search Solara…"
            class="header-search-input"
            spellcheck="false"
            data-tauri-drag-region="false"
          />
          <kbd class="search-badge">Ctrl K</kbd>
        </div>

        <!-- Header Draggable Spacer -->
        <div class="titlebar-spacer" data-tauri-drag-region></div>

        <!-- Modrinth Header Right Actions -->
        <div class="header-actions" data-tauri-drag-region="false" @mousedown.stop>
          <!-- Downloads Tray Button -->
          <button
            type="button"
            class="header-icon-btn"
            title="Downloads & Tasks"
            aria-label="Downloads & Tasks"
            @click="checkUpdates"
          >
            <Download :size="15" />
            <span v-if="appUpdate?.state === 'downloading'" class="badge-dot"></span>
          </button>

          <!-- Modrinth Instances Status Pill: ● No instances running -->
          <div
            class="instance-status-pill"
            :class="{ 'is-running': isGameRunning }"
            title="Instance running status"
          >
            <span class="status-indicator-dot"></span>
            <span class="status-indicator-text">
              {{ isGameRunning ? '1 instance running' : 'No instances running' }}
            </span>
          </div>
        </div>
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

    <!-- App Body: Sidebar + Main Content Layout -->
    <div class="launcher-layout">
      <!-- Sidebar: Inspired by Horizon Launcher's Sidebar Design -->
      <aside class="horizon-sidebar" aria-label="Sidebar Navigation">
        <!-- Top Instances Section (Horizon Style) -->
        <div class="sidebar-section">
          <div class="section-header">
            <span class="section-title">Instances</span>
            <button
              type="button"
              class="section-add-btn"
              title="Create instance"
              aria-label="Create instance"
              @click="navigateTo('instances')"
            >
              <Plus :size="14" />
            </button>
          </div>

          <!-- Active Instance Card -->
          <div
            class="horizon-instance-card"
            :class="{ 'is-selected': activeTab === 'instances' }"
            @click="navigateTo('instances')"
          >
            <div class="instance-card-icon">
              <img src="/logo.svg" alt="Solara" class="instance-card-logo" />
            </div>
            <div class="instance-card-info">
              <div class="instance-card-title">{{ activeInstance.name }}</div>
              <div class="instance-card-status">
                <span class="status-pulse-dot"></span>
                <span>{{ activeInstance.status }}</span>
              </div>
            </div>
            <button
              type="button"
              class="instance-card-menu-btn"
              title="Instance options"
              aria-label="Instance options"
              @click.stop="toggleInstanceMenu"
            >
              <MoreVertical :size="14" />
            </button>
          </div>
        </div>

        <!-- Navigation Rail Tabs (Modern Horizon Tabs) -->
        <nav class="sidebar-nav">
          <button
            type="button"
            class="nav-tab-btn"
            :class="{ 'active': activeTab === 'home' }"
            @click="navigateTo('home')"
          >
            <span class="nav-tab-indicator"></span>
            <Home :size="18" class="nav-tab-icon" />
            <span class="nav-tab-label">Home</span>
          </button>

          <button
            type="button"
            class="nav-tab-btn"
            :class="{ 'active': activeTab === 'instances' }"
            @click="navigateTo('instances')"
          >
            <span class="nav-tab-indicator"></span>
            <Layers3 :size="18" class="nav-tab-icon" />
            <span class="nav-tab-label">Instances</span>
          </button>

          <button
            type="button"
            class="nav-tab-btn"
            :class="{ 'active': activeTab === 'mods' }"
            @click="navigateTo('mods')"
          >
            <span class="nav-tab-indicator"></span>
            <Compass :size="18" class="nav-tab-icon" />
            <span class="nav-tab-label">Mods</span>
          </button>

          <button
            type="button"
            class="nav-tab-btn"
            :class="{ 'active': activeTab === 'servers' }"
            @click="navigateTo('servers')"
          >
            <span class="nav-tab-indicator"></span>
            <Server :size="18" class="nav-tab-icon" />
            <span class="nav-tab-label">Servers</span>
          </button>

          <button
            type="button"
            class="nav-tab-btn"
            :class="{ 'active': activeTab === 'settings' }"
            @click="navigateTo('settings')"
          >
            <span class="nav-tab-indicator"></span>
            <Settings :size="18" class="nav-tab-icon" />
            <span class="nav-tab-label">Settings</span>
          </button>
        </nav>

        <!-- Vertical Spacer pushes footer widgets to bottom -->
        <div class="sidebar-spacer"></div>

        <!-- Horizon Sidebar Bottom Section -->
        <div class="sidebar-footer">
          <!-- Horizon Update Status Widget -->
          <div class="horizon-update-card">
            <div class="update-card-heading">
              <span class="update-card-title">
                {{ hasUpdateAvailable ? 'A new version is available' : 'Launcher Status' }}
              </span>
              <span class="update-card-ver">V-0.0.30</span>
            </div>
            <button
              type="button"
              class="sidebar-update-action-btn"
              :disabled="checking"
              @click="checkUpdates"
            >
              <span v-if="checking" class="spinner-small"></span>
              <RefreshCw v-else :size="13" />
              <span>{{ checking ? 'Checking…' : hasUpdateAvailable ? 'Update' : 'Check status' }}</span>
            </button>
          </div>

          <!-- Join the Discord Button (Horizon Pill Button) -->
          <a
            href="https://discord.gg"
            target="_blank"
            rel="noopener noreferrer"
            class="horizon-discord-btn"
            @click.prevent="openDiscord"
          >
            <Discord :size="16" class="discord-icon" />
            <span>Join the Discord</span>
          </a>

          <!-- Sponsor & Website Quick Links -->
          <div class="sidebar-aux-links">
            <button
              type="button"
              class="aux-link-btn"
              @click="openExternal('https://solara-inc.in')"
            >
              <Heart :size="13" />
              <span>Sponsor</span>
            </button>
            <button
              type="button"
              class="aux-link-btn"
              @click="openExternal('https://app.solara-inc.in')"
            >
              <Globe :size="13" />
              <span>Website</span>
            </button>
          </div>

          <!-- Version Footer -->
          <div class="sidebar-version-tag">Launcher V-0.0.30</div>
        </div>
      </aside>

      <!-- Main Content Area -->
      <main class="launcher-main" v-show="!loading || loadingLeaving">
        <!-- HOME VIEW -->
        <section v-if="activeTab === 'home'" class="view-home">
          <!-- Horizon Hero Section -->
          <div class="hero-card">
            <div class="hero-top-row">
              <div class="hero-title-area">
                <h1 class="greeting">HI ;)</h1>
                <div class="edition-badges">
                  <span
                    class="badge-edition"
                    :class="{ active: gameEdition === 'rust' }"
                    @click="gameEdition = 'rust'"
                  >
                    ⚡ Rust Edition
                  </span>
                  <span
                    class="badge-edition"
                    :class="{ active: gameEdition === 'java' }"
                    @click="gameEdition = 'java'"
                  >
                    ☕ Java Edition
                  </span>
                </div>
              </div>

              <!-- Horizon User Profile Chip -->
              <div class="hero-user-chip" title="Solara Profile">
                <div class="user-avatar-circle">
                  <User :size="14" />
                </div>
                <div class="user-meta">
                  <span class="user-username">SolaraPlayer</span>
                  <span class="user-sub">Microsoft account</span>
                </div>
              </div>
            </div>

            <p class="hero-stats">
              Ready to launch • Clean slate engine • Fast & secure
            </p>

            <!-- Horizon Play Action Bar -->
            <div class="hero-action-bar">
              <button
                type="button"
                class="hero-play-btn"
                :class="{ 'is-active': isGameRunning }"
                @click="handlePlay"
              >
                <Square v-if="isGameRunning" :size="18" />
                <Play v-else :size="20" />
                <span>{{ isGameRunning ? 'Stop' : 'Play' }}</span>
              </button>

              <div class="hero-version-selector" @click="navigateTo('instances')">
                <span class="ver-label">Version</span>
                <span class="ver-val">{{ activeInstance.loader }} • {{ activeInstance.version }}</span>
                <ChevronDown :size="14" class="ver-arrow" />
              </div>

              <button
                type="button"
                class="hero-folder-btn"
                title="Open Game Folder"
                aria-label="Open Game Folder"
                @click="openGameFolder"
              >
                <FolderOpen :size="18" />
              </button>
            </div>

            <!-- Server Invite / Direct Join Box -->
            <div class="hero-invite-row">
              <div class="invite-input-wrap">
                <input
                  v-model="inviteCode"
                  type="text"
                  placeholder="Paste an invite code or server IP"
                  class="invite-input"
                  spellcheck="false"
                />
              </div>
              <button type="button" class="invite-join-btn" @click="handleJoinInvite">
                Join
              </button>
            </div>
          </div>

          <!-- Clean Container for Updater & Animation Preview -->
          <div class="clean-container">
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
          </div>
        </section>

        <!-- INSTANCES VIEW -->
        <section v-else-if="activeTab === 'instances'" class="view-instances">
          <div class="view-header">
            <div>
              <h2 class="view-heading">Instances</h2>
              <p class="view-subheading">Manage and customize your Minecraft instances</p>
            </div>
            <button type="button" class="view-primary-btn" @click="navigateTo('mods')">
              <Plus :size="15" />
              <span>New Instance</span>
            </button>
          </div>

          <div class="instances-grid">
            <div
              v-for="inst in instanceList"
              :key="inst.id"
              class="instance-grid-card"
              :class="{ 'is-current': inst.id === activeInstance.id }"
            >
              <div class="grid-card-head">
                <div class="grid-card-avatar">
                  <img :src="inst.icon" alt="Instance" class="grid-avatar-img" />
                </div>
                <div class="grid-card-meta">
                  <div class="grid-card-title">{{ inst.name }}</div>
                  <div class="grid-card-sub">{{ inst.loader }} • {{ inst.version }}</div>
                </div>
              </div>
              <div class="grid-card-footer">
                <span class="grid-card-status">
                  <span class="status-pulse-dot"></span>
                  {{ inst.status }}
                </span>
                <button
                  type="button"
                  class="grid-card-play-btn"
                  @click="activeInstance = inst; handlePlay()"
                >
                  <Play :size="14" />
                  <span>Launch</span>
                </button>
              </div>
            </div>
          </div>
        </section>

        <!-- MODS VIEW -->
        <section v-else-if="activeTab === 'mods'" class="view-mods">
          <div class="view-header">
            <div>
              <h2 class="view-heading">Mods & Modpacks</h2>
              <p class="view-subheading">Discover high-performance modpacks, shaders, and optimizations</p>
            </div>
          </div>

          <div class="mods-feature-grid">
            <div class="mod-feature-card">
              <div class="mod-badge">Optimization</div>
              <h3 class="mod-title">Solara Performance Suite</h3>
              <p class="mod-desc">Pre-configured Fabric stack with Sodium, Lithium, and FerriteCore for peak FPS.</p>
              <button type="button" class="mod-install-btn">Install to Active</button>
            </div>
            <div class="mod-feature-card">
              <div class="mod-badge">Shaders</div>
              <h3 class="mod-title">Iris Shader Support</h3>
              <p class="mod-desc">Modern shader rendering engine compatible with existing OptiFine shader packs.</p>
              <button type="button" class="mod-install-btn">Install to Active</button>
            </div>
          </div>
        </section>

        <!-- SERVERS VIEW -->
        <section v-else-if="activeTab === 'servers'" class="view-servers">
          <div class="view-header">
            <div>
              <h2 class="view-heading">Multiplayer Servers</h2>
              <p class="view-subheading">Connect directly to Solara SMP and verified partner networks</p>
            </div>
          </div>

          <div class="servers-list">
            <div class="server-card">
              <div class="server-icon-box">
                <Server :size="24" />
              </div>
              <div class="server-info-col">
                <div class="server-title-row">
                  <h3 class="server-name">Solara Official SMP</h3>
                  <span class="server-ping">32 ms</span>
                </div>
                <p class="server-desc">Official survival multiplayer realm with vanilla enhancements.</p>
              </div>
              <button type="button" class="server-join-btn" @click="handlePlay">Connect</button>
            </div>
          </div>
        </section>

        <!-- SETTINGS VIEW -->
        <section v-else-if="activeTab === 'settings'" class="view-settings">
          <div class="view-header">
            <div>
              <h2 class="view-heading">Launcher Settings</h2>
              <p class="view-subheading">Fine-tune memory allocation, JVM parameters, and client options</p>
            </div>
          </div>

          <div class="settings-panel">
            <div class="settings-row">
              <div class="settings-label-col">
                <span class="settings-title">Allocated RAM</span>
                <span class="settings-sub">Memory dedicated to Minecraft Java instances</span>
              </div>
              <div class="settings-control">
                <span class="settings-value">4096 MB</span>
              </div>
            </div>

            <div class="settings-row">
              <div class="settings-label-col">
                <span class="settings-title">Discord Rich Presence</span>
                <span class="settings-sub">Show playing status and game version in Discord</span>
              </div>
              <div class="settings-control">
                <span class="badge-edition active">Enabled</span>
              </div>
            </div>

            <div class="settings-row">
              <div class="settings-label-col">
                <span class="settings-title">Automatic Updates</span>
                <span class="settings-sub">Download signed launcher updates in background</span>
              </div>
              <div class="settings-control">
                <span class="badge-edition active">Always On</span>
              </div>
            </div>
          </div>
        </section>
      </main>
    </div>
  </div>
</template>
