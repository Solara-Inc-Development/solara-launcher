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
import type { PageId } from '../types';

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
  <aside class="sol-rail" aria-label="Primary Navigation">
    <nav class="sol-rail-top">
      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('home') }"
        data-tooltip="Home"
        aria-label="Home"
        @click="navigate('home')"
      >
        <Home :size="19" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('library') }"
        data-tooltip="Library"
        aria-label="Library"
        @click="navigate('library')"
      >
        <Library :size="19" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('discover') }"
        data-tooltip="Discover Modpacks"
        aria-label="Discover Modpacks"
        @click="navigate('discover')"
      >
        <Compass :size="19" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('skins') }"
        data-tooltip="Skins &amp; Capes"
        aria-label="Skins &amp; Capes"
        @click="navigate('skins')"
      >
        <Shirt :size="19" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('screenshots') }"
        data-tooltip="Screenshots"
        aria-label="Screenshots"
        @click="navigate('screenshots')"
      >
        <Camera :size="19" />
      </button>

      <button
        v-if="store.preferences.solaraHubEnabled !== false"
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('solara') }"
        data-tooltip="Solara Hub &amp; Servers"
        aria-label="Solara Hub &amp; Servers"
        @click="navigate('solara')"
      >
        <Server :size="19" />
      </button>

      <button
        type="button"
        class="sol-rail-item"
        :class="{ 'is-active': isActive('tasks') }"
        data-tooltip="Downloads"
        aria-label="Downloads"
        @click="navigate('tasks')"
      >
        <Download :size="19" />
        <span v-if="activeJobs.length > 0" class="sol-rail-dot" />
      </button>
    </nav>

    <div class="sol-rail-sep" />

    <!-- Pinned instances + Create Instance (+) -->
    <div class="sol-rail-pinned">
      <button
        v-for="inst in pinnedInstances"
        :key="inst.id"
        type="button"
        class="sol-rail-inst"
        :class="{ 'is-active': isPinnedActive(inst.id), 'is-running': inst.running }"
        :data-tooltip="inst.name"
        :aria-label="inst.name"
        @click="navigate('instance', { instanceId: inst.id, tab: 'content' })"
      >
        <InstanceIcon
          :name="inst.name"
          :loader="inst.loader"
          :icon="inst.icon"
          :custom-icon="inst.customIcon"
          :size="34"
          :rounded="9"
        />
        <span v-if="inst.running" class="sol-rail-running-dot" />
      </button>

      <button
        type="button"
        class="sol-rail-item sol-rail-item--add"
        data-tooltip="Create Instance"
        aria-label="Create Instance"
        @click="openCreateInstance()"
      >
        <Plus :size="19" />
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
        @click="openGlobalSettings('appearance')"
      >
        <Settings :size="19" />
      </button>

      <button
        type="button"
        class="sol-rail-item sol-rail-avatar sol-rail-account"
        :class="{ 'is-active': isActive('account') }"
        :data-tooltip="selectedAccount ? selectedAccount.username : 'Accounts'"
        aria-label="Accounts"
        @click="navigate('account')"
      >
        <img
          v-if="selectedAccount"
          :src="playerAvatarUrl(selectedAccount.username)"
          :alt="selectedAccount.username"
          class="sol-rail-avatar-img"
        />
        <User v-else :size="19" />
      </button>
    </div>
  </aside>
</template>
