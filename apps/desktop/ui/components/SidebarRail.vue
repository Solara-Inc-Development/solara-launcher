<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  Home,
  Library,
  Compass,
  Shirt,
  Camera,
  Download,
  Plus,
  Settings,
  User,
  PanelLeft,
} from '../icons';
import {
  store,
  currentRoute,
  pinnedInstances,
  selectedAccount,
  activeJobs,
  navigate,
  openCreateInstance,
  launchOrStopInstance,
  patchInstanceMeta,
  savePreferencesFull,
  playerAvatarUrl,
  native,
  notify,
} from '../state';
import type { InstanceRecord, PageId } from '../types';
import InstanceIcon from './InstanceIcon.vue';
import SolaraContextMenu, { type ContextMenuItem } from './SolaraContextMenu.vue';

const mainNav: { id: PageId; label: string; icon: unknown }[] = [
  { id: 'home', label: 'Home', icon: Home },
  { id: 'library', label: 'Library', icon: Library },
  { id: 'discover', label: 'Discover', icon: Compass },
  { id: 'skins', label: 'Skin & Capes', icon: Shirt },
  { id: 'screenshots', label: 'Screenshots', icon: Camera },
  { id: 'tasks', label: 'Downloads', icon: Download },
];

const expanded = computed(() => store.preferences.sidebarExpanded);

const sidebarInstances = computed(() => {
  const pinned = pinnedInstances.value;
  if (pinned.length > 0) return pinned;
  return store.instances.slice(0, 6);
});

const ctxMenu = ref<{
  open: boolean;
  x: number;
  y: number;
  inst: InstanceRecord | null;
}>({ open: false, x: 0, y: 0, inst: null });

const ctxItems = computed<ContextMenuItem[]>(() => {
  const inst = ctxMenu.value.inst;
  if (!inst) return [];
  return [
    { id: 'play', label: inst.running ? 'Stop Instance' : 'Launch Instance', disabled: inst.state !== 'installed' },
    { id: 'open', label: 'Open Instance' },
    { id: 'settings', label: 'Instance Settings' },
    { id: 'sep1', label: '', separator: true },
    { id: 'pin', label: inst.pinned ? 'Unpin from Sidebar' : 'Pin to Sidebar' },
    { id: 'folder', label: 'Open Folder' },
  ];
});

function openInstCtx(e: MouseEvent, inst: InstanceRecord) {
  e.preventDefault();
  ctxMenu.value = { open: true, x: e.clientX, y: e.clientY, inst };
}

async function handleCtxSelect(action: string) {
  const inst = ctxMenu.value.inst;
  if (!inst) return;
  if (action === 'play') await launchOrStopInstance(inst);
  else if (action === 'open') navigate('instance', { instanceId: inst.id, tab: 'overview' });
  else if (action === 'settings') navigate('instance', { instanceId: inst.id, tab: 'settings' });
  else if (action === 'pin') {
    await patchInstanceMeta(inst.id, { pinned: !inst.pinned });
  } else if (action === 'folder' && store.desktop) {
    native('open_instance_folder', { id: inst.id }).catch((err) =>
      notify('error', 'Could not open folder', String(err)),
    );
  }
}

async function toggleExpand() {
  const next = { ...store.preferences, sidebarExpanded: !store.preferences.sidebarExpanded };
  try {
    await savePreferencesFull(next);
  } catch {
    store.preferences.sidebarExpanded = next.sidebarExpanded;
  }
}
</script>

<template>
  <aside :class="['sol-rail', { 'is-expanded': expanded }]" aria-label="Primary navigation">
    <!-- Main Navigation Group -->
    <nav class="sol-rail-section">
      <button
        v-for="item in mainNav"
        :key="item.id"
        type="button"
        :title="item.label"
        :class="['sol-rail-item', { 'is-active': currentRoute.page === item.id }]"
        @click="navigate(item.id)"
      >
        <span class="sol-rail-indicator" aria-hidden="true" />
        <span class="sol-rail-icon">
          <component :is="item.icon" :size="18" />
          <span
            v-if="item.id === 'tasks' && activeJobs.length > 0"
            class="sol-rail-badge"
          >{{ activeJobs.length }}</span>
        </span>
        <span v-if="expanded" class="sol-rail-label">{{ item.label }}</span>
      </button>
    </nav>

    <div class="sol-rail-divider" />

    <!-- Quick Instances & Create -->
    <div class="sol-rail-instances sol-scroll">
      <button
        v-for="inst in sidebarInstances"
        :key="inst.id"
        type="button"
        :title="`${inst.name} (${inst.gameVersion} ${inst.loader})`"
        :class="[
          'sol-rail-inst',
          {
            'is-active': currentRoute.page === 'instance' && currentRoute.instanceId === inst.id,
            'is-running': inst.running,
          },
        ]"
        @click="navigate('instance', { instanceId: inst.id, tab: 'overview' })"
        @contextmenu="openInstCtx($event, inst)"
      >
        <span class="sol-rail-indicator" aria-hidden="true" />
        <div class="sol-rail-inst-avatar">
          <InstanceIcon
            :name="inst.name"
            :loader="inst.loader"
            :icon="inst.icon"
            :custom-icon="inst.customIcon"
            :size="32"
            :rounded="9"
          />
          <span v-if="inst.running" class="sol-running-dot" title="Running" />
        </div>
        <span v-if="expanded" class="sol-rail-label sol-truncate">{{ inst.name }}</span>
      </button>

      <button
        type="button"
        class="sol-rail-add"
        title="Create New Instance"
        aria-label="Create new instance"
        @click="openCreateInstance()"
      >
        <Plus :size="17" />
        <span v-if="expanded" class="sol-rail-label">New Instance</span>
      </button>
    </div>

    <div class="sol-rail-divider" />

    <!-- Bottom Controls: Expand Toggle, Settings, Account -->
    <div class="sol-rail-bottom">
      <button
        type="button"
        class="sol-rail-item"
        :title="expanded ? 'Collapse sidebar' : 'Expand sidebar'"
        :aria-label="expanded ? 'Collapse sidebar' : 'Expand sidebar'"
        @click="toggleExpand"
      >
        <span class="sol-rail-icon"><PanelLeft :size="18" /></span>
        <span v-if="expanded" class="sol-rail-label">Collapse</span>
      </button>

      <button
        type="button"
        title="Settings"
        :class="['sol-rail-item', { 'is-active': currentRoute.page === 'settings' }]"
        @click="navigate('settings')"
      >
        <span class="sol-rail-indicator" aria-hidden="true" />
        <span class="sol-rail-icon"><Settings :size="18" /></span>
        <span v-if="expanded" class="sol-rail-label">Settings</span>
      </button>

      <button
        type="button"
        :title="selectedAccount ? `Account: ${selectedAccount.username}` : 'Sign in with Microsoft'"
        :class="['sol-rail-item', 'sol-rail-account', { 'is-active': currentRoute.page === 'account' }]"
        @click="navigate('account')"
      >
        <span class="sol-rail-indicator" aria-hidden="true" />
        <span class="sol-rail-icon">
          <img
            v-if="selectedAccount"
            :src="playerAvatarUrl(selectedAccount.username)"
            :alt="selectedAccount.username"
            class="sol-rail-avatar"
          />
          <User v-else :size="18" />
        </span>
        <span v-if="expanded" class="sol-rail-label sol-truncate">
          {{ selectedAccount ? selectedAccount.username : 'Sign In' }}
        </span>
      </button>
    </div>

    <SolaraContextMenu
      :open="ctxMenu.open"
      :x="ctxMenu.x"
      :y="ctxMenu.y"
      :items="ctxItems"
      @close="ctxMenu.open = false"
      @select="handleCtxSelect"
    />
  </aside>
</template>
