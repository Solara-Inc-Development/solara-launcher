<script setup lang="ts">
import InstanceIcon from './InstanceIcon.vue';
import {
  Camera,
  Compass,
  Download,
  Home,
  Library,
  Plus,
  Server,
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
import { ref } from 'vue';
import type { PageId } from '../types';

const tooltip = ref<{ label: string; x: number; y: number } | null>(null);

function showTip(ev: Event): void {
  const el = ev.currentTarget as HTMLElement | null;
  const label = el?.getAttribute('data-tooltip');
  if (!el || !label) return;
  const rect = el.getBoundingClientRect();
  tooltip.value = {
    label,
    x: Math.round(rect.right + 12),
    y: Math.round(rect.top + rect.height / 2),
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
</script>

<template>
  <aside class="sol-rail" aria-label="Primary Navigation" @mouseleave="hideTip">
    <nav class="sol-rail-top">
      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('home') }"
        data-tooltip="Home"
        aria-label="Home"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="goPage('home')"
      >
        <Home :size="22" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('library') }"
        data-tooltip="Library"
        aria-label="Library"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="goPage('library')"
      >
        <Library :size="22" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('discover') }"
        data-tooltip="Discover Modpacks"
        aria-label="Discover Modpacks"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="goPage('discover')"
      >
        <Compass :size="22" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('skins') }"
        data-tooltip="Skins &amp; Capes"
        aria-label="Skins &amp; Capes"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="goPage('skins')"
      >
        <Shirt :size="22" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('screenshots') }"
        data-tooltip="Screenshots"
        aria-label="Screenshots"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="goPage('screenshots')"
      >
        <Camera :size="22" />
      </button>

      <button
        v-if="store.preferences.solaraHubEnabled !== false"
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('solara') }"
        data-tooltip="Solara Hub &amp; Servers"
        aria-label="Solara Hub &amp; Servers"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="goPage('solara')"
      >
        <Server :size="22" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('tasks') }"
        data-tooltip="Downloads"
        aria-label="Downloads"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="goPage('tasks')"
      >
        <Download :size="22" />
        <span v-if="activeJobs.length > 0" class="sol-rail-dot" />
      </button>
    </nav>

    <div class="sol-rail-sep" />

    <!-- Pinned instances + Create Instance (+) -->
    <div class="sol-rail-pinned" @scroll.passive="hideTip">
      <button
        v-for="inst in pinnedInstances"
        :key="inst.id"
        type="button"
        class="sol-rail-inst"
        :class="{ 'is-active': isPinnedActive(inst.id), 'is-running': inst.running }"
        :data-tooltip="inst.name"
        :aria-label="inst.name"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
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

      <button
        type="button"
        class="sol-rail-item sol-rail-item--add"
        data-tooltip="Create Instance"
        aria-label="Create Instance"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="openCreateFromRail()"
      >
        <Plus :size="22" />
      </button>
    </div>

    <div class="sol-rail-bottom">
      <div class="sol-rail-sep" />
      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': store.showSettingsModal }"
        data-tooltip="Settings"
        aria-label="Settings"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="openSettingsFromRail()"
      >
        <Settings :size="22" />
      </button>

      <button
        type="button"
        class="sol-rail-item sol-rail-avatar sol-rail-account"
        :class="{ 'is-active': isActive('account') }"
        :data-tooltip="selectedAccount ? selectedAccount.username : 'Accounts'"
        aria-label="Accounts"
        @mouseenter="showTip"
        @mouseleave="hideTip"
        @focusin="showTip"
        @focusout="hideTip"
        @click="goPage('account')"
      >
        <img
          v-if="selectedAccount"
          :src="playerAvatarUrl(selectedAccount.username)"
          :alt="selectedAccount.username"
          class="sol-rail-avatar-img"
        />
        <User v-else :size="22" />
      </button>
    </div>

    <div
      v-if="tooltip"
      class="sol-rail-tooltip"
      role="tooltip"
      :style="{ top: `${tooltip.y}px`, left: `${tooltip.x}px` }"
    >
      <span class="sol-rail-tooltip-label">{{ tooltip.label }}</span>
    </div>
  </aside>
</template>
