<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import InstanceIcon from './InstanceIcon.vue';
import {
  Camera,
  Compass,
  Download,
  Home,
  Library,
  Plus,
  Settings,
  Shirt,
  User,
} from '../icons';
import {
  activeJobs,
  currentRoute,
  navigate,
  openCreateInstance,
  openGlobalSettings,
  pinnedInstances,
  playerAvatarUrl,
  selectedAccount,
  store,
} from '../state';
import type { PageId } from '../types';

interface RailNavEntry {
  id: PageId;
  label: string;
  sublabel?: string;
  icon: unknown;
}

interface TooltipState {
  label: string;
  sublabel?: string;
  x: number;
  y: number;
}

const primaryNav: RailNavEntry[] = [
  { id: 'home', label: 'Home', icon: Home },
  { id: 'library', label: 'Library', icon: Library },
  { id: 'discover', label: 'Discover Modpacks', icon: Compass },
  { id: 'skins', label: 'Skins & Capes', icon: Shirt },
  { id: 'screenshots', label: 'Screenshots', icon: Camera },
  { id: 'tasks', label: 'Downloads', icon: Download },
];

const tooltip = ref<TooltipState | null>(null);

const hasPendingUpdate = computed(
  () =>
    store.updateStatus?.stage === 'ready' ||
    store.updateStatus?.stage === 'available' ||
    store.updateStatus?.stage === 'downloading',
);

const accountLabel = computed(() =>
  selectedAccount.value ? selectedAccount.value.username : 'Minecraft Accounts',
);

const accountSublabel = computed(() =>
  selectedAccount.value ? 'Active Microsoft account' : 'Sign in with Microsoft',
);

function clampTooltipY(rawY: number): number {
  const minY = 76;
  const maxY = Math.max(minY, (typeof window !== 'undefined' ? window.innerHeight : 720) - 34);
  return Math.round(Math.min(maxY, Math.max(minY, rawY)));
}

function showTip(ev: Event, label: string, sublabel?: string): void {
  const el = ev.currentTarget as HTMLElement | null;
  if (!el || !label) return;
  const rect = el.getBoundingClientRect();
  if (rect.width === 0 && rect.height === 0) return;
  tooltip.value = {
    label,
    sublabel,
    x: Math.round(rect.right + 12),
    y: clampTooltipY(rect.top + rect.height / 2),
  };
}

function hideTip(): void {
  tooltip.value = null;
}

function goPage(page: PageId, extra?: { instanceId?: string; tab?: 'content' }): void {
  hideTip();
  navigate(page, extra);
}

function openCreateFromRail(): void {
  hideTip();
  openCreateInstance();
}

function openSettingsFromRail(): void {
  hideTip();
  openGlobalSettings('appearance');
}

function isActive(page: PageId): boolean {
  if (store.showSettingsModal) return false;
  return currentRoute.value.page === page;
}

function isPinnedActive(instId: string): boolean {
  if (store.showSettingsModal) return false;
  return currentRoute.value.page === 'instance' && currentRoute.value.instanceId === instId;
}

function onGlobalDismiss(ev?: KeyboardEvent): void {
  if (ev && ev.key !== 'Escape') return;
  hideTip();
}

onMounted(() => {
  window.addEventListener('blur', hideTip);
  window.addEventListener('resize', hideTip);
  window.addEventListener('pointerdown', hideTip);
  window.addEventListener('keydown', onGlobalDismiss);
});

onBeforeUnmount(() => {
  window.removeEventListener('blur', hideTip);
  window.removeEventListener('resize', hideTip);
  window.removeEventListener('pointerdown', hideTip);
  window.removeEventListener('keydown', onGlobalDismiss);
});
</script>

<template>
  <aside class="sol-rail" aria-label="Primary Navigation" @mouseleave="hideTip">
    <nav class="sol-rail-top" aria-label="Launcher Sections">
      <div v-for="item in primaryNav" :key="item.id" class="sol-rail-row">
        <span
          class="sol-rail-indicator"
          :class="{ 'is-active': isActive(item.id) }"
          aria-hidden="true"
        />
        <button
          type="button"
          class="sol-rail-item"
          :class="{ 'is-active': isActive(item.id) }"
          :data-tooltip="item.label"
          :aria-label="item.label"
          :aria-current="isActive(item.id) ? 'page' : undefined"
          @mouseenter="
            showTip(
              $event,
              item.label,
              item.id === 'tasks' && activeJobs.length > 0
                ? `${activeJobs.length} active ${activeJobs.length === 1 ? 'task' : 'tasks'}`
                : item.sublabel,
            )
          "
          @mouseleave="hideTip"
          @focusin="showTip($event, item.label, item.sublabel)"
          @focusout="hideTip"
          @click="goPage(item.id)"
        >
          <component :is="item.icon" :size="21" />
          <span
            v-if="item.id === 'tasks' && activeJobs.length > 0"
            class="sol-rail-badge"
          >
            {{ activeJobs.length > 9 ? '9+' : activeJobs.length }}
          </span>
        </button>
      </div>
    </nav>

    <div class="sol-rail-divider" role="separator" />

    <div class="sol-rail-pinned" aria-label="Instances" @scroll.passive="hideTip">
      <div v-for="inst in pinnedInstances" :key="inst.id" class="sol-rail-row">
        <span
          class="sol-rail-indicator"
          :class="{ 'is-active': isPinnedActive(inst.id) }"
          aria-hidden="true"
        />
        <button
          type="button"
          class="sol-rail-inst"
          :class="{ 'is-active': isPinnedActive(inst.id), 'is-running': inst.running }"
          :data-tooltip="inst.name"
          :aria-label="inst.name"
          :aria-current="isPinnedActive(inst.id) ? 'page' : undefined"
          @mouseenter="
            showTip(
              $event,
              inst.name,
              inst.running ? `${inst.loader} ${inst.gameVersion} · Running` : `${inst.loader} ${inst.gameVersion}`,
            )
          "
          @mouseleave="hideTip"
          @focusin="
            showTip(
              $event,
              inst.name,
              inst.running ? `${inst.loader} ${inst.gameVersion} · Running` : `${inst.loader} ${inst.gameVersion}`,
            )
          "
          @focusout="hideTip"
          @click="goPage('instance', { instanceId: inst.id, tab: 'content' })"
        >
          <InstanceIcon
            :name="inst.name"
            :loader="inst.loader"
            :icon="inst.icon"
            :custom-icon="inst.customIcon"
            :size="38"
            :rounded="11"
          />
          <span v-if="inst.running" class="sol-rail-running-dot" />
        </button>
      </div>

      <div class="sol-rail-row">
        <button
          type="button"
          class="sol-rail-item sol-rail-item--add"
          data-tooltip="Create Instance"
          aria-label="Create Instance"
          @mouseenter="showTip($event, 'Create Instance', 'New Minecraft profile')"
          @mouseleave="hideTip"
          @focusin="showTip($event, 'Create Instance', 'New Minecraft profile')"
          @focusout="hideTip"
          @click="openCreateFromRail()"
        >
          <Plus :size="21" />
        </button>
      </div>
    </div>

    <div class="sol-rail-bottom">
      <div class="sol-rail-divider" role="separator" />

      <div class="sol-rail-row">
        <span
          class="sol-rail-indicator"
          :class="{ 'is-active': store.showSettingsModal }"
          aria-hidden="true"
        />
        <button
          type="button"
          class="sol-rail-item"
          :class="{ 'is-active': store.showSettingsModal }"
          data-tooltip="Settings"
          aria-label="Settings"
          @mouseenter="
            showTip($event, 'Settings', hasPendingUpdate ? 'Launcher update ready' : 'Appearance, Java & storage')
          "
          @mouseleave="hideTip"
          @focusin="
            showTip($event, 'Settings', hasPendingUpdate ? 'Launcher update ready' : 'Appearance, Java & storage')
          "
          @focusout="hideTip"
          @click="openSettingsFromRail()"
        >
          <Settings :size="21" />
          <span v-if="hasPendingUpdate" class="sol-rail-update-dot" />
        </button>
      </div>

      <div class="sol-rail-row">
        <span
          class="sol-rail-indicator"
          :class="{ 'is-active': isActive('account') }"
          aria-hidden="true"
        />
        <button
          type="button"
          class="sol-rail-item sol-rail-account"
          :class="{ 'is-active': isActive('account') }"
          :data-tooltip="accountLabel"
          aria-label="Accounts"
          :aria-current="isActive('account') ? 'page' : undefined"
          @mouseenter="showTip($event, accountLabel, accountSublabel)"
          @mouseleave="hideTip"
          @focusin="showTip($event, accountLabel, accountSublabel)"
          @focusout="hideTip"
          @click="goPage('account')"
        >
          <img
            v-if="selectedAccount"
            :src="playerAvatarUrl(selectedAccount.username)"
            :alt="selectedAccount.username"
            class="sol-rail-avatar-img"
          />
          <User v-else :size="21" />
          <span v-if="selectedAccount" class="sol-rail-account-dot" />
        </button>
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="tooltip"
        class="sol-rail-tooltip"
        role="tooltip"
        :style="{ top: `${tooltip.y}px`, left: `${tooltip.x}px` }"
      >
        <span class="sol-rail-tooltip-label">{{ tooltip.label }}</span>
        <span v-if="tooltip.sublabel" class="sol-rail-tooltip-sub">{{ tooltip.sublabel }}</span>
      </div>
    </Teleport>
  </aside>
</template>
