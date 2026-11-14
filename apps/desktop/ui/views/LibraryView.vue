<script setup lang="ts">
import { computed, ref } from 'vue';
import InstanceIcon from '../components/InstanceIcon.vue';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import {
  Copy,
  FolderOpen,
  Grid,
  Library,
  List,
  MoreVertical,
  Package,
  Pin,
  Play,
  Plus,
  Search,
  Sliders,
  Square,
  Star,
  Trash2,
  X,
} from '../icons';
import {
  formatDuration,
  formatRelativeTime,
  launchOrStopInstance,
  LOADER_LABEL,
  native,
  navigate,
  notify,
  openCreateInstance,
  openInstanceSettings,
  patchInstanceMeta,
  refreshSnapshot,
  savePreferencesFull,
  selectInstance,
  store,
} from '../state';
import type { InstanceRecord } from '../types';

const searchQuery = ref('');
const groupFilter = ref<'all' | 'favourites' | 'modded' | 'vanilla' | string>('all');
const sortBy = ref<'lastPlayed' | 'name' | 'playtime' | 'version'>('lastPlayed');
const viewMode = ref<'grid' | 'list'>('grid');
const openMenuId = ref<string | null>(null);
const deleteConfirm = ref<InstanceRecord | null>(null);
const deleting = ref(false);

// Inline New Group creator
const showNewGroupInput = ref(false);
const newGroupName = ref('');

// Right-click context menu
const contextMenu = ref<{ x: number; y: number; inst: InstanceRecord } | null>(null);

const groups = computed(() => {
  const fromInstances = store.instances
    .map((i) => i.group)
    .filter((g): g is string => Boolean(g && g.trim()));
  const merged = new Set([...(store.preferences.groups ?? []), ...fromInstances]);
  return Array.from(merged).sort();
});

const filteredInstances = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  return [...store.instances]
    .filter((inst) => {
      if (q) {
        const hay = `${inst.name} ${inst.gameVersion} ${inst.loader}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      if (groupFilter.value === 'favourites') return Boolean(inst.favourite);
      if (groupFilter.value === 'modded') return inst.loader !== 'vanilla';
      if (groupFilter.value === 'vanilla') return inst.loader === 'vanilla';
      if (groupFilter.value !== 'all') return inst.group === groupFilter.value;
      return true;
    })
    .sort((a, b) => {
      if (sortBy.value === 'name') return a.name.localeCompare(b.name);
      if (sortBy.value === 'playtime') return (b.playtimeSeconds ?? 0) - (a.playtimeSeconds ?? 0);
      if (sortBy.value === 'version') return b.gameVersion.localeCompare(a.gameVersion);
      const aT = a.lastPlayed ? new Date(a.lastPlayed).getTime() : 0;
      const bT = b.lastPlayed ? new Date(b.lastPlayed).getTime() : 0;
      return bT - aT;
    });
});

function openInstance(inst: InstanceRecord): void {
  selectInstance(inst.id);
  navigate('instance', { instanceId: inst.id, tab: 'content' });
}

function openContextMenu(e: MouseEvent, inst: InstanceRecord): void {
  openMenuId.value = null;
  contextMenu.value = {
    x: Math.min(e.clientX, window.innerWidth - 200),
    y: Math.min(e.clientY, window.innerHeight - 250),
    inst,
  };
}

function closeMenus(): void {
  openMenuId.value = null;
  contextMenu.value = null;
}

async function createGroup(): Promise<void> {
  const name = newGroupName.value.trim();
  if (!name) return;
  if (!store.preferences.groups.includes(name)) {
    try {
      await savePreferencesFull({
        ...store.preferences,
        groups: [...store.preferences.groups, name],
      });
      notify('success', 'Group created', `Added "${name}" to Library groups.`);
    } catch (err) {
      notify('error', "Couldn't save group", String(err));
      return;
    }
  }
  groupFilter.value = name;
  newGroupName.value = '';
  showNewGroupInput.value = false;
}

async function togglePin(inst: InstanceRecord): Promise<void> {
  closeMenus();
  await patchInstanceMeta(inst.id, { pinned: !inst.pinned });
}

async function toggleFav(inst: InstanceRecord): Promise<void> {
  closeMenus();
  await patchInstanceMeta(inst.id, { favourite: !inst.favourite });
}

async function duplicateInstance(inst: InstanceRecord): Promise<void> {
  closeMenus();
  try {
    await native('duplicate_instance', { id: inst.id, name: `${inst.name} (Copy)` });
    notify('success', 'Instance duplicated', `${inst.name} (Copy) is ready.`);
    await refreshSnapshot();
  } catch (err) {
    notify('error', "Couldn't duplicate instance", String(err));
  }
}

async function confirmDelete(): Promise<void> {
  if (!deleteConfirm.value) return;
  deleting.value = true;
  try {
    const victim = deleteConfirm.value;
    await native('delete_instance', { id: victim.id });
    deleteConfirm.value = null;
    notify('success', 'Instance removed', `${victim.name} was deleted.`);
    await refreshSnapshot();
  } catch (err) {
    notify('error', "Couldn't delete instance", String(err));
  } finally {
    deleting.value = false;
  }
}
</script>

<template>
  <div class="sol-page sol-scroll" @click="closeMenus">
    <div class="sol-view-wrap">
      <!-- Row 0 & Row 1: Title + Search | New Group | New Instance -->
      <header class="sol-stack" style="gap: 10px">
        <div class="sol-section-head">
          <h1 class="sol-page-title">Library</h1>
          <span class="sol-muted sol-text-xs">
            {{ filteredInstances.length }} of {{ store.instances.length }} instances
          </span>
        </div>

        <div class="sol-toolbar-bar">
          <div class="sol-search-box sol-search-box--grow">
            <Search :size="14" class="sol-muted" />
            <input
              v-model="searchQuery"
              type="text"
              class="sol-search-input"
              placeholder="Search instances by name, version, or loader..."
            />
            <button v-if="searchQuery" type="button" class="sol-icon-btn sol-icon-btn--xs" @click="searchQuery = ''">
              <X :size="12" />
            </button>
          </div>

          <form v-if="showNewGroupInput" class="sol-row-gap" @submit.prevent="createGroup" @click.stop>
            <input
              v-model="newGroupName"
              type="text"
              class="sol-input sol-input--sm"
              placeholder="Group name..."
              maxlength="40"
              autofocus
            />
            <SolaraButton variant="secondary" size="sm" type="submit">Add</SolaraButton>
            <SolaraButton variant="ghost" size="sm" @click="showNewGroupInput = false">Cancel</SolaraButton>
          </form>
          <SolaraButton v-else variant="secondary" size="sm" @click.stop="showNewGroupInput = true">
            <Plus :size="13" /> New Group
          </SolaraButton>

          <SolaraButton variant="primary" size="sm" @click="openCreateInstance()">
            <Plus :size="14" /> New Instance
          </SolaraButton>
        </div>

        <!-- Row 2: Sort | Group | Filter | Grid/List toggle -->
        <div class="sol-toolbar-bar">
          <select v-model="sortBy" class="sol-select" aria-label="Sort instances">
            <option value="lastPlayed">Sort: Last played</option>
            <option value="name">Sort: Name (A–Z)</option>
            <option value="playtime">Sort: Playtime</option>
            <option value="version">Sort: MC Version</option>
          </select>

          <select v-model="groupFilter" class="sol-select" aria-label="Filter by group">
            <option value="all">Group: All ({{ store.instances.length }})</option>
            <option value="favourites">Filter: Favourites</option>
            <option value="modded">Filter: Modded</option>
            <option value="vanilla">Filter: Vanilla</option>
            <option v-for="g in groups" :key="g" :value="g">Group: {{ g }}</option>
          </select>

          <div class="sol-chip-row">
            <button
              v-for="chip in [
                { id: 'all', label: 'All' },
                { id: 'favourites', label: 'Favourites' },
                { id: 'modded', label: 'Modded' },
                { id: 'vanilla', label: 'Vanilla' },
              ]"
              :key="chip.id"
              type="button"
              :class="['sol-chip', { 'is-active': groupFilter === chip.id }]"
              @click="groupFilter = chip.id"
            >
              {{ chip.label }}
            </button>
          </div>

          <div class="sol-seg-group" style="margin-left: auto">
            <button
              type="button"
              :class="['sol-seg-btn', { 'is-active': viewMode === 'grid' }]"
              aria-label="Grid view"
              title="Grid view"
              @click="viewMode = 'grid'"
            >
              <Grid :size="14" />
            </button>
            <button
              type="button"
              :class="['sol-seg-btn', { 'is-active': viewMode === 'list' }]"
              aria-label="List view"
              title="List view"
              @click="viewMode = 'list'"
            >
              <List :size="14" />
            </button>
          </div>
        </div>
      </header>

      <SolaraEmptyState
        v-if="filteredInstances.length === 0"
        :title="store.instances.length === 0 ? 'No instances yet' : 'No matching instances'"
        :description="
          store.instances.length === 0
            ? 'Create a fresh Minecraft instance or install a modpack from Discover.'
            : 'Adjust your search query or filter chips to see instances.'
        "
      >
        <template #icon><Library :size="28" /></template>
        <template #actions>
          <SolaraButton variant="primary" size="sm" @click="openCreateInstance()">
            <Plus :size="14" /> New Instance
          </SolaraButton>
          <SolaraButton variant="secondary" size="sm" @click="navigate('discover')">
            Discover Modpacks
          </SolaraButton>
        </template>
      </SolaraEmptyState>

      <!-- Grid View -->
      <div v-else-if="viewMode === 'grid'" class="sol-inst-grid">
        <article
          v-for="inst in filteredInstances"
          :key="inst.id"
          :class="[
            'sol-inst-card',
            { 'is-selected': store.selectedInstanceId === inst.id, 'is-running': inst.running },
          ]"
          @click="openInstance(inst)"
          @contextmenu.prevent="openContextMenu($event, inst)"
        >
          <div class="sol-inst-card-top">
            <InstanceIcon
              :name="inst.name"
              :loader="inst.loader"
              :icon="inst.icon"
              :custom-icon="inst.customIcon"
              :size="48"
              :rounded="11"
            />
            <div class="sol-inst-card-titlewrap">
              <div class="sol-inst-card-name sol-truncate">{{ inst.name }}</div>
              <div class="sol-muted sol-text-xs">
                {{ LOADER_LABEL[inst.loader] ?? inst.loader }} {{ inst.gameVersion }}
              </div>
            </div>

            <div class="sol-menu-wrap" @click.stop>
              <button
                type="button"
                class="sol-icon-btn sol-icon-btn--xs"
                aria-label="Instance menu"
                @click="openMenuId = openMenuId === inst.id ? null : inst.id"
              >
                <MoreVertical :size="14" />
              </button>
              <div v-if="openMenuId === inst.id" class="sol-menu-dropdown">
                <button type="button" @click="closeMenus(); openInstanceSettings(inst.id)">
                  <Sliders :size="13" /> Instance Settings
                </button>
                <button type="button" @click="togglePin(inst)">
                  <Pin :size="13" /> {{ inst.pinned ? 'Unpin from Sidebar' : 'Pin to Sidebar' }}
                </button>
                <button type="button" @click="toggleFav(inst)">
                  <Star :size="13" /> {{ inst.favourite ? 'Remove Favourite' : 'Add Favourite' }}
                </button>
                <button type="button" @click="closeMenus(); native('open_instance_folder', { id: inst.id }).catch(() => {})">
                  <FolderOpen :size="13" /> Open Folder
                </button>
                <button type="button" @click="duplicateInstance(inst)">
                  <Copy :size="13" /> Duplicate
                </button>
                <button type="button" class="is-danger" @click="closeMenus(); deleteConfirm = inst">
                  <Trash2 :size="13" /> Delete
                </button>
              </div>
            </div>
          </div>

          <div class="sol-inst-card-bottom">
            <span class="sol-muted sol-text-xs">
              {{ formatDuration(inst.playtimeSeconds) }} · {{ formatRelativeTime(inst.lastPlayed) }}
            </span>

            <div class="sol-row-gap" @click.stop>
              <button
                type="button"
                class="sol-icon-btn sol-icon-btn--xs sol-card-hover-act"
                title="Instance Settings"
                @click="openInstanceSettings(inst.id)"
              >
                <Sliders :size="13" />
              </button>
              <SolaraButton
                :variant="inst.running ? 'danger' : 'primary'"
                size="xs"
                @click="launchOrStopInstance(inst)"
              >
                <Square v-if="inst.running" :size="11" />
                <Play v-else :size="11" />
                {{ inst.running ? 'Stop' : 'Play' }}
              </SolaraButton>
            </div>
          </div>
        </article>
      </div>

      <!-- Compact List View -->
      <div v-else class="sol-row-list sol-list-table">
        <div
          v-for="inst in filteredInstances"
          :key="inst.id"
          :class="[
            'sol-row-item',
            { 'is-selected': store.selectedInstanceId === inst.id, 'is-running': inst.running },
          ]"
          @click="openInstance(inst)"
          @contextmenu.prevent="openContextMenu($event, inst)"
        >
          <div class="sol-row-item-main">
            <InstanceIcon
              :name="inst.name"
              :loader="inst.loader"
              :icon="inst.icon"
              :custom-icon="inst.customIcon"
              :size="36"
              :rounded="8"
            />
            <div class="sol-row-item-text">
              <div class="sol-row-item-title">{{ inst.name }}</div>
              <div class="sol-muted sol-text-xs">
                {{ LOADER_LABEL[inst.loader] ?? inst.loader }} {{ inst.gameVersion }}
                <template v-if="inst.group"> · {{ inst.group }}</template>
                · {{ formatDuration(inst.playtimeSeconds) }}
              </div>
            </div>
          </div>

          <div class="sol-row-item-actions" @click.stop>
            <SolaraButton variant="ghost" size="xs" @click="openInstanceSettings(inst.id)">
              <Sliders :size="13" /> Settings
            </SolaraButton>
            <SolaraButton
              :variant="inst.running ? 'danger' : 'primary'"
              size="xs"
              @click="launchOrStopInstance(inst)"
            >
              <Square v-if="inst.running" :size="11" />
              <Play v-else :size="11" />
              {{ inst.running ? 'Stop' : 'Play' }}
            </SolaraButton>
          </div>
        </div>
      </div>

      <!-- Right-Click Context Menu -->
      <div
        v-if="contextMenu"
        class="sol-menu-dropdown sol-context-fixed"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
        @click.stop
      >
        <button type="button" @click="launchOrStopInstance(contextMenu.inst); closeMenus()">
          <Square v-if="contextMenu.inst.running" :size="13" />
          <Play v-else :size="13" />
          {{ contextMenu.inst.running ? 'Stop Minecraft' : 'Play Instance' }}
        </button>
        <button type="button" @click="openInstance(contextMenu.inst); closeMenus()">
          <Package :size="13" /> Manage Content
        </button>
        <button type="button" @click="openInstanceSettings(contextMenu.inst.id); closeMenus()">
          <Sliders :size="13" /> Instance Settings
        </button>
        <button type="button" @click="togglePin(contextMenu.inst)">
          <Pin :size="13" /> {{ contextMenu.inst.pinned ? 'Unpin Sidebar' : 'Pin to Sidebar' }}
        </button>
        <button type="button" @click="native('open_instance_folder', { id: contextMenu.inst.id }).catch(() => {}); closeMenus()">
          <FolderOpen :size="13" /> Open Folder
        </button>
        <button type="button" @click="duplicateInstance(contextMenu.inst)">
          <Copy :size="13" /> Duplicate
        </button>
        <button
          type="button"
          class="is-danger"
          @click="deleteConfirm = contextMenu.inst; closeMenus()"
        >
          <Trash2 :size="13" /> Delete
        </button>
      </div>

      <!-- Delete Confirmation Modal -->
      <div v-if="deleteConfirm" class="sol-overlay" @click.self="deleteConfirm = null">
        <div class="sol-dialog-box" style="max-width: 420px">
          <h3 class="sol-modal-title">Delete "{{ deleteConfirm.name }}"?</h3>
          <p class="sol-muted sol-text-sm" style="margin: 10px 0 18px">
            This permanently removes its worlds, mods, saves, and configs inside this instance folder.
          </p>
          <div class="sol-row-gap" style="justify-content: flex-end">
            <SolaraButton variant="ghost" :disabled="deleting" @click="deleteConfirm = null">Cancel</SolaraButton>
            <SolaraButton variant="danger" :loading="deleting" @click="confirmDelete">Delete Instance</SolaraButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
