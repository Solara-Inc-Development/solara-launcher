<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import {
  Minus,
  Maximize2,
  X,
  RefreshCw,
  Sparkles,
  Home,
  Discord,
  User,
  LogIn,
  LogOut,
  ShieldCheck,
  Check,
  Copy,
  ExternalLink,
  ChevronRight,
  ArrowLeft,
  FileText,
  Lock
} from '../../packages/ui/icons';
import { native, isDesktop, openExternal } from '../../packages/shared/client';
import {
  type LauncherAnnouncement,
  getLatestAnnouncements,
  getAllAnnouncements
} from '../../packages/shared/announcements';

// Auto-Updater States
const checking = ref(false);
const hasChecked = ref(false);
const updateMessage = ref('');
const isError = ref(false);
const isSuccess = ref(false);
const appUpdate = ref<any>({ state: 'idle', downloaded: 0, total: null });

// Canonical Animated Loading Screen States (5-10s Duration)
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
let unlistenLoginCode: (() => void) | null = null;

// Minecraft Account States
interface MinecraftAccount {
  id: string;
  name: string;
  active: boolean;
  owned: boolean;
}

const minecraftAccounts = ref<MinecraftAccount[]>([]);
const activeAccount = computed<MinecraftAccount | null>(() => {
  return minecraftAccounts.value.find((a) => a.active) || minecraftAccounts.value[0] || null;
});

const signingIn = ref(false);
const showLoginModal = ref(false);
const loginPrompt = ref<{ code: string; url: string; expiresIn: number } | null>(null);
const loginError = ref<string | null>(null);
const codeCopied = ref(false);

// Announcements States
const latestAnnouncements = ref<LauncherAnnouncement[]>(getLatestAnnouncements(3));
const allAnnouncements = ref<LauncherAnnouncement[]>(getAllAnnouncements(50));
const showAllAnnouncementsModal = ref(false);
const selectedAnnouncement = ref<LauncherAnnouncement | null>(null);
const activeCategoryFilter = ref<string>('All');
const categories = ['All', 'Release', 'Update', 'Feature', 'Security', 'Community'];

const filteredAnnouncements = computed(() => {
  if (activeCategoryFilter.value === 'All') return allAnnouncements.value;
  return allAnnouncements.value.filter((a) => a.category === activeCategoryFilter.value);
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
    void refreshSnapshot().catch(() => {});
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
  const btn = e.currentTarget as HTMLButtonElement | null;
  if (!btn || btn.disabled) return;
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
      await new Promise((r) => setTimeout(r, 400));
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

async function refreshSnapshot() {
  if (!isDesktop) return;
  try {
    const snap = await native<any>('launcher_snapshot');
    if (snap && Array.isArray(snap.minecraftAccounts)) {
      minecraftAccounts.value = snap.minecraftAccounts;
    }
  } catch (e) {
    console.warn('launcher_snapshot error:', e);
  }
}

async function startMinecraftSignIn() {
  if (signingIn.value) return;
  signingIn.value = true;
  loginPrompt.value = null;
  loginError.value = null;
  codeCopied.value = false;
  showLoginModal.value = true;

  try {
    if (isDesktop) {
      await native('minecraft_sign_in');
      await refreshSnapshot();
      showLoginModal.value = false;
    } else {
      // Simulated preview mode
      loginPrompt.value = {
        code: 'QA-CODE',
        url: 'https://www.microsoft.com/link',
        expiresIn: 900
      };
    }
  } catch (err: any) {
    loginError.value = String(err?.message || err || 'Sign-in cancelled or failed');
  } finally {
    signingIn.value = false;
  }
}

async function cancelMinecraftSignIn() {
  showLoginModal.value = false;
  signingIn.value = false;
  loginPrompt.value = null;
  loginError.value = null;
  if (isDesktop) {
    try {
      await native('cancel_minecraft_sign_in');
    } catch {}
  }
}

async function signOutMinecraftAccount(id: string) {
  if (!id) return;
  try {
    if (isDesktop) {
      await native('minecraft_sign_out', { id });
      await refreshSnapshot();
    } else {
      minecraftAccounts.value = minecraftAccounts.value.filter((a) => a.id !== id);
    }
  } catch (err) {
    console.error('Sign out failed:', err);
  }
}

async function copyLoginCode() {
  if (!loginPrompt.value?.code) return;
  codeCopied.value = true;
  try {
    if (navigator?.clipboard?.writeText) {
      await navigator.clipboard.writeText(loginPrompt.value.code);
    }
  } catch {
    try {
      const input = document.createElement('input');
      input.value = loginPrompt.value.code;
      document.body.appendChild(input);
      input.select();
      document.execCommand('copy');
      document.body.removeChild(input);
    } catch {}
  }
  setTimeout(() => {
    codeCopied.value = false;
  }, 2000);
}

function openMicrosoftLink() {
  const url = loginPrompt.value?.url || 'https://www.microsoft.com/link';
  void openExternal(url);
}

function openDiscord() {
  void openExternal('https://discord.solara-inc.in/');
}

function openAllAnnouncements() {
  selectedAnnouncement.value = null;
  showAllAnnouncementsModal.value = true;
}

function closeAllAnnouncements() {
  showAllAnnouncementsModal.value = false;
  selectedAnnouncement.value = null;
}

function readAnnouncement(item: LauncherAnnouncement) {
  selectedAnnouncement.value = item;
  showAllAnnouncementsModal.value = true;
}

function backToAnnouncementsList() {
  selectedAnnouncement.value = null;
}

function formatDate(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  } catch {
    return dateStr;
  }
}

function onAvatarError(e: Event) {
  const target = e.target as HTMLImageElement;
  if (target) {
    target.style.display = 'none';
  }
}

function onContextMenu(e: MouseEvent) {
  e.preventDefault();
}

onMounted(async () => {
  window.addEventListener('contextmenu', onContextMenu, { capture: true });

  if (isDesktop) {
    try {
      const { listen } = await import('@tauri-apps/api/event');
      unlistenLoginCode = await listen<any>('minecraft-login-code', (event) => {
        if (event && event.payload) {
          loginPrompt.value = event.payload;
        }
      });
    } catch (err) {
      console.warn('Failed to listen for minecraft-login-code:', err);
    }
  }

  await runStartupSequence();
  if (isDesktop && !startupAbort) {
    await refreshSnapshot();
    pollTimer = setInterval(() => void pollStatus(), 3000);
  }
});

onUnmounted(() => {
  window.removeEventListener('contextmenu', onContextMenu, { capture: true });
  if (unlistenLoginCode) {
    unlistenLoginCode();
    unlistenLoginCode = null;
  }
  startupAbort = true;
  currentSequenceId++;
  if (startupInterval) clearInterval(startupInterval);
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
  <div class="shell" data-theme="dark" @contextmenu.prevent>
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

    <!-- App Body: Wider Horizon-inspired sidebar (~200px - 220px) + Main Home Page -->
    <div class="launcher-layout">
      <!-- Sidebar: Wider Horizon-inspired sidebar (~200px - 220px) with Icon + Text labels -->
      <aside class="horizon-sidebar" aria-label="Sidebar Navigation">
        <!-- Top Dock Item (Home active indicator with Icon + Text) -->
        <div class="dock-top">
          <button
            type="button"
            class="dock-item is-active"
            title="Home"
            aria-label="Home"
          >
            <Home :size="18" class="dock-icon" />
            <span class="dock-label">Home</span>
          </button>
        </div>

        <div class="dock-spacer"></div>

        <!-- Bottom Dock Item: Discord button opening https://discord.solara-inc.in/ (Icon + Text) -->
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
            <Discord :size="18" class="discord-icon dock-icon" />
            <span class="dock-label">Discord</span>
          </a>
        </div>
      </aside>

      <!-- Main Home Page View -->
      <main v-show="!loading || loadingLeaving" class="launcher-main home-viewport">
        <div class="home-container">
          <!-- 1. Top Greeting & Welcome Section -->
          <section class="welcome-section" aria-label="Welcome">
            <!-- Signed In Greeting with Avatar -->
            <div v-if="activeAccount" class="welcome-header has-avatar">
              <div class="welcome-avatar-wrapper">
                <img
                  :src="`https://mc-heads.net/avatar/${encodeURIComponent(activeAccount.name)}/72`"
                  :alt="activeAccount.name"
                  class="player-head"
                  @error="onAvatarError"
                />
                <span class="online-indicator" title="Minecraft account active"></span>
              </div>
              <div class="welcome-text">
                <div class="welcome-pill">
                  <span class="pill-dot"></span>
                  <span>Java Edition Entitled</span>
                </div>
                <h1 class="greeting">Welcome, {{ activeAccount.name }}</h1>
                <p class="welcome-sub">Your Minecraft account is connected locally and ready to play.</p>
              </div>
            </div>

            <!-- Unsigned Greeting -->
            <div v-else class="welcome-header">
              <div class="welcome-text">
                <h1 class="greeting">Welcome user</h1>
                <p class="welcome-sub">Connect your Minecraft account below to launch official worlds and verify purchase.</p>
              </div>
            </div>

            <!-- Minecraft Account Authentication Card -->
            <div class="minecraft-auth-card">
              <!-- When Signed In -->
              <div v-if="activeAccount" class="account-card signed-in">
                <div class="account-card-left">
                  <div class="account-badge">
                    <ShieldCheck :size="16" />
                    <span>Verified Minecraft Account</span>
                  </div>
                  <div class="account-details">
                    <span class="account-username">{{ activeAccount.name }}</span>
                    <span class="account-privacy-note">
                      <Lock :size="13" />
                      <span>Stored exclusively in your local Windows vault · Never uploaded to Solara</span>
                    </span>
                  </div>
                </div>
                <div class="account-card-right">
                  <button
                    type="button"
                    class="btn-account-signout"
                    title="Sign out of this Minecraft account"
                    @click="signOutMinecraftAccount(activeAccount.id)"
                  >
                    <LogOut :size="15" />
                    <span>Sign Out</span>
                  </button>
                </div>
              </div>

              <!-- When Unsigned -->
              <div v-else class="account-card unsigned">
                <div class="account-card-left">
                  <div class="account-lead">
                    <div class="account-icon-box">
                      <User :size="24" />
                    </div>
                    <div>
                      <h2 class="account-card-title">Login with your Minecraft account</h2>
                      <p class="account-card-desc">
                        Connect your official Microsoft account to verify genuine Minecraft Java Edition purchase.
                      </p>
                    </div>
                  </div>
                  <div class="account-security-pill">
                    <ShieldCheck :size="14" />
                    <span>Private & Local: Credentials and tokens remain stored exclusively on your PC, never sent to Solara servers.</span>
                  </div>
                </div>
                <div class="account-card-right">
                  <button
                    type="button"
                    class="btn-minecraft-login"
                    :disabled="signingIn"
                    @click="startMinecraftSignIn"
                  >
                    <LogIn :size="17" />
                    <span>Login with your Minecraft account</span>
                  </button>
                </div>
              </div>
            </div>
          </section>

          <!-- 2. Launcher Announcements Section (Latest 3 Cards + See All Announcements Button) -->
          <section class="home-section announcements-section" aria-label="Launcher Announcements">
            <div class="section-header">
              <div class="section-title-group">
                <h2 class="section-title">Launcher Announcements</h2>
                <span class="section-subtitle">Latest updates, features, and notices from the Solara project</span>
              </div>
              <button
                type="button"
                class="btn-see-all-announcements"
                title="See all announcements"
                @click="openAllAnnouncements"
              >
                <span>See all announcements</span>
                <ChevronRight :size="16" />
              </button>
            </div>

            <!-- Latest 3 Announcements Cards Grid -->
            <div class="announcements-grid">
              <article
                v-for="item in latestAnnouncements"
                :key="item.id"
                class="announcement-card"
                role="button"
                tabindex="0"
                @click="readAnnouncement(item)"
                @keydown.enter="readAnnouncement(item)"
              >
                <div class="announcement-meta">
                  <span class="announcement-badge" :data-category="item.category">{{ item.category }}</span>
                  <time class="announcement-date">{{ formatDate(item.date) }}</time>
                </div>
                <h3 class="announcement-title">{{ item.title }}</h3>
                <p class="announcement-summary">{{ item.summary }}</p>
                <div class="announcement-footer">
                  <span class="read-link">Read full announcement →</span>
                </div>
              </article>
            </div>
          </section>

          <!-- 3. Latest Update Info Section (Below Announcements) -->
          <section class="home-section update-info-section" aria-label="Latest Update Info">
            <div class="section-header">
              <div class="section-title-group">
                <h2 class="section-title">Latest Update Info</h2>
                <span class="section-subtitle">What was added and improved in this release</span>
              </div>
              <div class="update-version-tag">
                <span class="version-dot"></span>
                <span>Testing Preview V-0.0.32</span>
              </div>
            </div>

            <div class="update-info-card">
              <div class="update-card-header">
                <div>
                  <h3 class="update-release-title">Solara Launcher V-0.0.32 Highlights</h3>
                  <p class="update-release-date">Published November 12, 2026 · Windows x86_64</p>
                </div>
                <button
                  type="button"
                  class="update-btn"
                  :disabled="checking"
                  @click="checkUpdates"
                >
                  <span v-if="checking" class="spinner"></span>
                  <RefreshCw v-else :size="15" />
                  <span>{{ checking ? 'Checking for updates…' : 'Check for updates' }}</span>
                </button>
              </div>

              <!-- Update Status Message -->
              <div
                v-if="updateMessage"
                class="update-status"
                :class="{ error: isError, success: isSuccess }"
              >
                <span class="status-dot"></span>
                <span>{{ updateMessage }}</span>
              </div>

              <!-- Download Progress Bar -->
              <div v-if="appUpdate?.state === 'downloading'" class="download-progress">
                <div class="progress-bar">
                  <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
                </div>
                <span class="progress-info">{{ updateProgressText }}</span>
              </div>

              <!-- Highlights of what was added -->
              <ul class="update-highlights-list">
                <li>
                  <span class="check-icon"><Check :size="13" /></span>
                  <div>
                    <strong>Minecraft Java Account Sign-In:</strong>
                    <span>Native Microsoft OAuth flow with automatic Java entitlement verification. Credentials remain stored exclusively on your PC in the Windows vault and never reach Solara.</span>
                  </div>
                </li>
                <li>
                  <span class="check-icon"><Check :size="13" /></span>
                  <div>
                    <strong>Launcher Announcements Section:</strong>
                    <span>Live announcement cards on the home page with a comprehensive 50-item archive and clean built-in reader.</span>
                  </div>
                </li>
                <li>
                  <span class="check-icon"><Check :size="13" /></span>
                  <div>
                    <strong>Disabled Default Browser Context Menu:</strong>
                    <span>Global suppression of webview right-click menus (Back, Reload, Save as, Print) for an authentic native desktop feel.</span>
                  </div>
                </li>
                <li>
                  <span class="check-icon"><Check :size="13" /></span>
                  <div>
                    <strong>Wider Navigation Sidebar (~210px):</strong>
                    <span>Generous navigation dock featuring clear Icon + Text labels for Home and Discord.</span>
                  </div>
                </li>
                <li>
                  <span class="check-icon"><Check :size="13" /></span>
                  <div>
                    <strong>Modrinth-Inspired Top Bar:</strong>
                    <span>Clean frameless header with brand typography, window dragging regions, and deduplicated controls.</span>
                  </div>
                </li>
                <li>
                  <span class="check-icon"><Check :size="13" /></span>
                  <div>
                    <strong>Live Cryptographic Auto-Updater:</strong>
                    <span>Automatic Minisign verification ensures seamless background updates with zero manual installer downloads.</span>
                  </div>
                </li>
              </ul>

              <div class="update-actions">
                <button type="button" class="preview-anim-btn" @click="replayLoading">
                  <Sparkles :size="14" />
                  <span>Preview loading animation</span>
                </button>
              </div>
            </div>
          </section>
        </div>
      </main>
    </div>

    <!-- Modal 1: Minecraft Sign-In Device Flow Modal -->
    <div
      v-if="showLoginModal"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-signin-title"
      @click.self="cancelMinecraftSignIn"
    >
      <div class="modal-card signin-modal">
        <div class="modal-header">
          <div class="modal-header-title">
            <User :size="18" />
            <h2 id="modal-signin-title">Sign in to Minecraft</h2>
          </div>
          <button
            type="button"
            class="modal-close-btn"
            title="Cancel sign-in"
            aria-label="Cancel sign-in"
            @click="cancelMinecraftSignIn"
          >
            <X :size="18" />
          </button>
        </div>

        <div class="modal-body">
          <!-- Error State -->
          <div v-if="loginError" class="login-error-box">
            <span class="error-badge">Sign-In Notice</span>
            <p class="error-text">{{ loginError }}</p>
            <button type="button" class="btn-retry" @click="startMinecraftSignIn">
              Try Again
            </button>
          </div>

          <!-- Prompt with Device Code -->
          <div v-else-if="loginPrompt" class="device-prompt-box">
            <p class="prompt-instruction">
              To link your Minecraft Java Edition account, open the official Microsoft sign-in page and enter the code below:
            </p>

            <div class="device-code-display">
              <span class="minecraft-device-code">{{ loginPrompt.code }}</span>
              <button
                type="button"
                class="btn-copy-code"
                :title="codeCopied ? 'Copied!' : 'Copy Code'"
                @click="copyLoginCode"
              >
                <Check v-if="codeCopied" :size="15" />
                <Copy v-else :size="15" />
                <span>{{ codeCopied ? 'Copied' : 'Copy Code' }}</span>
              </button>
            </div>

            <div class="prompt-steps">
              <div class="step-row">
                <span class="step-num">1</span>
                <span>Copy the one-time code shown above</span>
              </div>
              <div class="step-row">
                <span class="step-num">2</span>
                <span>Click the button below to visit Microsoft device login</span>
              </div>
              <div class="step-row">
                <span class="step-num">3</span>
                <span>Sign in with the Microsoft account owning Minecraft Java Edition</span>
              </div>
            </div>

            <div class="prompt-actions">
              <button
                type="button"
                class="btn-open-microsoft"
                @click="openMicrosoftLink"
              >
                <span>Open Microsoft Login Page</span>
                <ExternalLink :size="16" />
              </button>
            </div>

            <div class="security-disclaimer">
              <ShieldCheck :size="15" />
              <span>Strictly Local & Private: You only enter credentials on Microsoft’s official website. Your game tokens remain stored exclusively in your local Windows vault and never touch Solara servers.</span>
            </div>
          </div>

          <!-- Connecting / Waiting State -->
          <div v-else class="connecting-state">
            <span class="spinner large"></span>
            <p>Connecting to Microsoft authentication service…</p>
            <small>Awaiting one-time device code</small>
          </div>
        </div>

        <div class="modal-footer">
          <button
            type="button"
            class="btn-cancel"
            @click="cancelMinecraftSignIn"
          >
            Cancel sign-in
          </button>
        </div>
      </div>
    </div>

    <!-- Modal 2: All Announcements GUI & Reader Modal -->
    <div
      v-if="showAllAnnouncementsModal"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-announcements-title"
      @click.self="closeAllAnnouncements"
    >
      <div class="modal-card announcements-modal">
        <!-- Announcements Modal Header -->
        <div class="modal-header">
          <div class="modal-header-title">
            <FileText :size="18" />
            <h2 id="modal-announcements-title">
              {{ selectedAnnouncement ? 'Announcement Details' : 'All Announcements' }}
            </h2>
            <span v-if="!selectedAnnouncement" class="header-count-pill">
              {{ filteredAnnouncements.length }} announcements
            </span>
          </div>
          <button
            type="button"
            class="modal-close-btn"
            title="Close announcements"
            aria-label="Close announcements"
            @click="closeAllAnnouncements"
          >
            <X :size="18" />
          </button>
        </div>

        <!-- Detail Reader View -->
        <div v-if="selectedAnnouncement" class="modal-body announcement-reader">
          <button
            type="button"
            class="btn-back-to-list"
            @click="backToAnnouncementsList"
          >
            <ArrowLeft :size="16" />
            <span>Back to all announcements</span>
          </button>

          <header class="reader-header">
            <div class="reader-meta">
              <span class="announcement-badge" :data-category="selectedAnnouncement.category">
                {{ selectedAnnouncement.category }}
              </span>
              <time class="reader-date">{{ formatDate(selectedAnnouncement.date) }}</time>
              <span class="reader-author">By {{ selectedAnnouncement.author }}</span>
            </div>
            <h1 class="reader-title">{{ selectedAnnouncement.title }}</h1>
          </header>

          <div class="reader-content">
            <p
              v-for="(para, idx) in selectedAnnouncement.content.split('\n\n')"
              :key="idx"
              class="reader-para"
            >
              {{ para }}
            </p>
          </div>
        </div>

        <!-- 50 Announcements List View -->
        <div v-else class="modal-body announcements-archive">
          <!-- Category Filter Pills -->
          <div class="category-filters">
            <button
              v-for="cat in categories"
              :key="cat"
              type="button"
              class="filter-pill"
              :class="{ 'is-active': activeCategoryFilter === cat }"
              @click="activeCategoryFilter = cat"
            >
              {{ cat }}
            </button>
          </div>

          <!-- Scrollable List of Announcements -->
          <div class="announcements-archive-list">
            <article
              v-for="ann in filteredAnnouncements"
              :key="ann.id"
              class="archive-item"
              role="button"
              tabindex="0"
              @click="readAnnouncement(ann)"
              @keydown.enter="readAnnouncement(ann)"
            >
              <div class="archive-item-main">
                <div class="archive-item-meta">
                  <span class="announcement-badge" :data-category="ann.category">{{ ann.category }}</span>
                  <time class="archive-item-date">{{ formatDate(ann.date) }}</time>
                  <span v-if="ann.tag" class="archive-tag">{{ ann.tag }}</span>
                </div>
                <h3 class="archive-item-title">{{ ann.title }}</h3>
                <p class="archive-item-summary">{{ ann.summary }}</p>
              </div>
              <div class="archive-item-arrow">
                <ChevronRight :size="18" />
              </div>
            </article>
          </div>
        </div>

        <div class="modal-footer">
          <button
            type="button"
            class="btn-cancel"
            @click="closeAllAnnouncements"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
