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

const sidebarInstances = computed(() => {
  const pinned = pinnedInstances.value;
  if (pinned.length > 0) return pinned;
  return store.instances.slice(0, 6);
});

const tip = ref<{
  visible: boolean;
  label: string;
  sub: string;
  top: number;
  left: number;
}>({ visible: false, label: '', sub: '', top: 0, left: 0 });

function showTip(e: MouseEvent | FocusEvent, label: string, sub = '') {
  const target = e.currentTarget as HTMLElement | null;
  if (!target) return;
  const rect = target.getBoundingClientRect();
  const rawTop = Math.round(rect.top + rect.height / 2);
  const maxTop = typeof window !== 'undefined' ? Math.max(28, window.innerHeight - 36) : rawTop;
  tip.value = {
    visible: true,
    label,
    sub,
    top: Math.max(28, Math.min(maxTop, rawTop)),
    left: Math.round(rect.right + 10),
  };
}

function hideTip() {
  if (tip.value.visible) tip.value.visible = false;
}

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
      notify('error', "Couldn't open folder", String(err)),
    );
  }
}

</script>

<template>
  <aside class="sol-rail" aria-label="Primary navigation">
    <!-- Main Navigation Group -->
    <nav class="sol-rail-section">
      <button
        v-for="item in mainNav"
        :key="item.id"
        type="button"
        :aria-label="item.label"
        :class="['sol-rail-item', { 'is-active': currentRoute.page === item.id }]"
        @mouseenter="showTip($event, item.label)"
        @mouseleave="hideTip"
        @focus="showTip($event, item.label)"
        @blur="hideTip"
        @click="hideTip(); navigate(item.id)"
      >
        <span class="sol-rail-indicator" aria-hidden="true" />
        <span class="sol-rail-icon">
          <component :is="item.icon" :size="18" />
          <span
            v-if="item.id === 'tasks' && activeJobs.length > 0"
            class="sol-rail-badge"
          >{{ activeJobs.length }}</span>
        </span>
      </button>
    </nav>

    <div class="sol-rail-divider" />

    <!-- Quick Instances & Create -->
    <div class="sol-rail-instances sol-scroll" @scroll="hideTip">
      <button
        v-for="inst in sidebarInstances"
        :key="inst.id"
        type="button"
        :aria-label="`${inst.name} (${inst.gameVersion} ${inst.loader})`"
        :class="[
          'sol-rail-inst',
          {
            'is-active': currentRoute.page === 'instance' && currentRoute.instanceId === inst.id,
            'is-running': inst.running,
          },
        ]"
        @mouseenter="showTip($event, inst.name, inst.running ? `${inst.gameVersion} · ${inst.loader} · Running` : `${inst.gameVersion} · ${inst.loader}`)"
        @mouseleave="hideTip"
        @focus="showTip($event, inst.name, inst.running ? `${inst.gameVersion} · ${inst.loader} · Running` : `${inst.gameVersion} · ${inst.loader}`)"
        @blur="hideTip"
        @click="hideTip(); navigate('instance', { instanceId: inst.id, tab: 'overview' })"
        @contextmenu="hideTip(); openInstCtx($event, inst)"
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
          <span v-if="inst.running" class="sol-running-dot" aria-hidden="true" />
        </div>
      </button>

      <button
        type="button"
        class="sol-rail-add"
        aria-label="Create new instance"
        @mouseenter="showTip($event, 'New Instance', 'Create a modpack or vanilla profile')"
        @mouseleave="hideTip"
        @focus="showTip($event, 'New Instance', 'Create a modpack or vanilla profile')"
        @blur="hideTip"
        @click="hideTip(); openCreateInstance()"
      >
        <Plus :size="17" />
      </button>
    </div>

    <div class="sol-rail-divider" />

    <!-- Bottom Controls: Settings, Account -->
    <div class="sol-rail-bottom">
      <button
        type="button"
        aria-label="Settings"
        :class="['sol-rail-item', { 'is-active': currentRoute.page === 'settings' }]"
        @mouseenter="showTip($event, 'Settings')"
        @mouseleave="hideTip"
        @focus="showTip($event, 'Settings')"
        @blur="hideTip"
        @click="hideTip(); navigate('settings')"
      >
        <span class="sol-rail-indicator" aria-hidden="true" />
        <span class="sol-rail-icon"><Settings :size="18" /></span>
      </button>

      <button
        type="button"
        :aria-label="selectedAccount ? `Account: ${selectedAccount.username}` : 'Sign in with Microsoft'"
        :class="['sol-rail-item', 'sol-rail-account', { 'is-active': currentRoute.page === 'account' }]"
        @mouseenter="showTip($event, selectedAccount ? selectedAccount.username : 'Microsoft Account', selectedAccount ? 'Active profile' : 'Sign in to play')"
        @mouseleave="hideTip"
        @focus="showTip($event, selectedAccount ? selectedAccount.username : 'Microsoft Account', selectedAccount ? 'Active profile' : 'Sign in to play')"
        @blur="hideTip"
        @click="hideTip(); navigate('account')"
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
      </button>
    </div>

    <div
      v-if="tip.visible"
      role="tooltip"
      class="sol-rail-tooltip"
      :style="{ top: `${tip.top}px`, left: `${tip.left}px` }"
    >
      <span class="sol-rail-tooltip-label">{{ tip.label }}</span>
      <span v-if="tip.sub" class="sol-rail-tooltip-sub">{{ tip.sub }}</span>
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
