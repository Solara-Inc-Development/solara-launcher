<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import {
  store,
  currentRoute,
  shouldShowOnboarding,
  initializeEngine,
  refreshSnapshot,
  refreshUpdateStatus,
  goBack,
  goForward,
  navigate,
  openCreateInstance,
  openExternal,
  confirmLeaveDiscard,
  cancelLeaveStay,
  launchOrStopInstance,
  native,
  notify,
} from './ui/state';
import TitleBar from './ui/components/TitleBar.vue';
import SidebarRail from './ui/components/SidebarRail.vue';
import CreateInstanceModal from './ui/components/CreateInstanceModal.vue';
import GlobalSearchModal from './ui/components/GlobalSearchModal.vue';
import ConfirmDialog from './ui/components/ConfirmDialog.vue';
import SolaraModal from './ui/components/SolaraModal.vue';
import SolaraButton from './ui/components/SolaraButton.vue';
import SolaraToasts from './ui/components/SolaraToasts.vue';

import OnboardingView from './ui/views/OnboardingView.vue';
import HomeView from './ui/views/HomeView.vue';
import LibraryView from './ui/views/LibraryView.vue';
import DiscoverView from './ui/views/DiscoverView.vue';
import SkinsView from './ui/views/SkinsView.vue';
import ScreenshotsView from './ui/views/ScreenshotsView.vue';
import TasksView from './ui/views/TasksView.vue';
import SettingsView from './ui/views/SettingsView.vue';
import InstanceDetailView from './ui/views/InstanceDetailView.vue';
import AccountView from './ui/views/AccountView.vue';

// Direct Microsoft Browser PKCE Sign-In state
const signInModalOpen = ref(false);
const signInUrl = ref('');
const signInError = ref('');
const signInPending = ref(false);
const signInWaiting = ref(false);

let pollTimer: ReturnType<typeof setInterval> | null = null;
const unlistenFns: (() => void)[] = [];

async function startMinecraftSignIn() {
  if (!store.desktop) {
    notify('info', 'Desktop required', 'Microsoft PKCE sign-in runs inside the Solara desktop app.');
    return;
  }
  if (signInPending.value) return;
  signInModalOpen.value = true;
  signInUrl.value = '';
  signInError.value = '';
  signInPending.value = true;
  signInWaiting.value = true;
  try {
    await native('minecraft_sign_in');
    signInModalOpen.value = false;
    await refreshSnapshot();
    notify('success', 'Microsoft account connected', 'Minecraft Java ownership verified.');
  } catch (err) {
    const msg = String(err);
    if (!msg.toLowerCase().includes('cancel')) {
      signInError.value = msg;
    } else {
      signInModalOpen.value = false;
    }
  } finally {
    signInPending.value = false;
    signInWaiting.value = false;
  }
}

async function cancelMinecraftSignIn() {
  try {
    if (store.desktop) await native('cancel_minecraft_sign_in');
  } catch {
    // ignore
  }
  signInModalOpen.value = false;
  signInPending.value = false;
  signInWaiting.value = false;
}

function reopenSignInBrowser() {
  if (!signInUrl.value) return;
  openExternal(signInUrl.value);
}

const activeViewComponent = computed(() => {
  if (shouldShowOnboarding.value) return OnboardingView;
  switch (currentRoute.value.page) {
    case 'home':
      return HomeView;
    case 'library':
      return LibraryView;
    case 'discover':
      return DiscoverView;
    case 'skins':
      return SkinsView;
    case 'screenshots':
      return ScreenshotsView;
    case 'tasks':
      return TasksView;
    case 'settings':
      return SettingsView;
    case 'instance':
      return InstanceDetailView;
    case 'account':
      return AccountView;
    default:
      return HomeView;
  }
});

function onGlobalKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    store.showGlobalSearch = !store.showGlobalSearch;
  } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'n') {
    e.preventDefault();
    openCreateInstance();
  } else if ((e.ctrlKey || e.metaKey) && e.key === ',') {
    e.preventDefault();
    navigate('settings');
  } else if (e.altKey && e.key === 'ArrowLeft') {
    e.preventDefault();
    goBack();
  } else if (e.altKey && e.key === 'ArrowRight') {
    e.preventDefault();
    goForward();
  }
}

function onGlobalContextMenu(e: MouseEvent) {
  const target = e.target as HTMLElement | null;
  if (target?.closest('input, textarea, [contenteditable="true"], .selectable')) return;
  e.preventDefault();
}

onMounted(async () => {
  window.addEventListener('keydown', onGlobalKeydown);
  window.addEventListener('contextmenu', onGlobalContextMenu);

  await initializeEngine();

  if (store.desktop) {
    try {
      const u1 = await listen<{ url?: string }>('minecraft-login-code', (ev) => {
        if (ev.payload?.url) {
          signInUrl.value = ev.payload.url;
          signInWaiting.value = false;
        }
      });
      unlistenFns.push(u1);
    } catch {
      // ignore outside tauri runtime
    }
    try {
      const u2 = await listen<string>('shutdown-error', (ev) => {
        notify('warning', 'Hold on', String(ev.payload || 'A background task is finishing.'));
      });
      unlistenFns.push(u2);
    } catch {
      // ignore
    }

    pollTimer = setInterval(() => {
      if (!document.hidden) {
        void refreshSnapshot();
        void refreshUpdateStatus();
      }
    }, 3000);
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', onGlobalKeydown);
  window.removeEventListener('contextmenu', onGlobalContextMenu);
  if (pollTimer) clearInterval(pollTimer);
  for (const fn of unlistenFns) fn();
});

async function confirmPendingLaunch() {
  const inst = store.pendingLaunchConfirm;
  store.pendingLaunchConfirm = null;
  if (inst) await launchOrStopInstance(inst, true);
}

function cancelPendingLaunch() {
  store.pendingLaunchConfirm = null;
}
</script>

<template>
  <div class="sol-shell">
    <TitleBar />

    <div class="sol-shell-body">
      <SidebarRail v-if="!shouldShowOnboarding" />

      <main class="sol-workspace">
        <div v-if="!store.initialized" class="sol-boot-state">
          <div class="sol-boot-spinner" />
          <div class="sol-muted">Initializing Solara Launcher...</div>
        </div>

        <div v-else-if="store.initError" class="sol-boot-state">
          <div class="sol-card sol-stack" style="max-width: 460px; padding: 24px">
            <h2 class="sol-card-title">Startup Error</h2>
            <p class="sol-inline-error">{{ store.initError }}</p>
            <SolaraButton variant="primary" @click="initializeEngine">Retry Initialization</SolaraButton>
          </div>
        </div>

        <component
          :is="activeViewComponent"
          v-else
          @start-signin="startMinecraftSignIn"
        />
      </main>
    </div>

    <!-- Global Modals -->
    <CreateInstanceModal />
    <GlobalSearchModal />
    <SolaraToasts />

    <!-- Direct Browser Microsoft PKCE Sign-In Modal -->
    <SolaraModal
      :open="signInModalOpen"
      title="Microsoft Sign-In"
      subtitle="Official browser authentication"
      width="sm"
      :closable="true"
      @close="cancelMinecraftSignIn"
    >
      <div class="direct-signin-box">
        <div class="browser-pulse-ring" aria-hidden="true" />
        <p class="sol-signin-prompt">
          Opening Microsoft Login in your browser... Please sign in and approve Solara Launcher.
        </p>
        <p v-if="signInError" class="sol-inline-error">{{ signInError }}</p>
        <div class="sol-signin-actions">
          <button
            type="button"
            class="btn-reopen-browser"
            :disabled="!signInUrl && signInWaiting"
            @click="reopenSignInBrowser"
          >
            Re-open browser
          </button>
          <SolaraButton variant="ghost" @click="cancelMinecraftSignIn">
            Cancel
          </SolaraButton>
        </div>
      </div>
    </SolaraModal>

    <!-- Announcement Reader Modal -->
    <SolaraModal
      :open="store.showAnnouncementsModal && Boolean(store.activeAnnouncement)"
      :title="store.activeAnnouncement?.title"
      :subtitle="store.activeAnnouncement?.category || 'Solara Announcement'"
      width="md"
      @close="store.showAnnouncementsModal = false"
    >
      <div v-if="store.activeAnnouncement" class="sol-stack selectable">
        <p style="white-space: pre-wrap; margin: 0; line-height: 1.6">
          {{ store.activeAnnouncement.content || store.activeAnnouncement.summary }}
        </p>
      </div>
      <template #footer>
        <SolaraButton variant="secondary" @click="store.showAnnouncementsModal = false">Close</SolaraButton>
      </template>
    </SolaraModal>

    <!-- Unsaved Settings Leave Protection Dialog -->
    <SolaraModal
      :open="store.showDirtyLeaveDialog"
      title="Unsaved Changes"
      width="sm"
      @close="cancelLeaveStay"
    >
      <p class="sol-confirm-msg">
        You have unsaved changes on this page. Leaving now will discard them.
      </p>
      <template #footer>
        <SolaraButton variant="ghost" @click="cancelLeaveStay">Stay & Keep Editing</SolaraButton>
        <SolaraButton variant="danger" @click="confirmLeaveDiscard">Discard & Leave</SolaraButton>
      </template>
    </SolaraModal>

    <ConfirmDialog
      :open="Boolean(store.pendingLaunchConfirm)"
      title="Launch Instance?"
      :message="
        store.pendingLaunchConfirm
          ? `Start ${store.pendingLaunchConfirm.name} (${store.pendingLaunchConfirm.loader} ${store.pendingLaunchConfirm.gameVersion})?`
          : ''
      "
      confirm-label="Launch Now"
      @confirm="confirmPendingLaunch"
      @cancel="cancelPendingLaunch"
    />
  </div>
</template>
