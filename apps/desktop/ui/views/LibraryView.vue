<script setup lang="ts">
import { computed, ref } from 'vue';
import {
  Search,
  Plus,
  FolderPlus,
  Grid,
  List,
  Play,
  Square,
  Star,
  Pin,
  MoreHorizontal,
  FolderOpen,
  Settings,
  Clock,
  Library,
} from '../icons';
import {
  store,
  selectedInstance,
  selectInstance,
  navigate,
  openCreateInstance,
  launchOrStopInstance,
  patchInstanceMeta,
  savePreferencesFull,
  refreshSnapshot,
  formatRelativeTime,
  formatDuration,
  LOADER_LABEL,
  native,
  notify,
} from '../state';
import type { InstanceRecord } from '../types';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import SolaraModal from '../components/SolaraModal.vue';
import SolaraContextMenu, { type ContextMenuItem } from '../components/SolaraContextMenu.vue';
import ConfirmDialog from '../components/ConfirmDialog.vue';
import InstanceIcon from '../components/InstanceIcon.vue';

const search = ref('');
const sortBy = ref<'lastPlayed' | 'name' | 'created' | 'playtime'>('lastPlayed');
const groupFilter = ref<string>('all');
const loaderFilter = ref<string>('all');
const versionFilter = ref<string>('all');
const viewMode = ref<'grid' | 'list'>('grid');

// New Group Modal
const showGroupModal = ref(false);
const newGroupName = ref('');
const savingGroup = ref(false);

// Context Menu & Rename / Group / Delete Modals
const ctx = ref<{ open: boolean; x: number; y: number; inst: InstanceRecord | null }>({
  open: false,
  x: 0,
  y: 0,
  inst: null,
});
const deleteTarget = ref<InstanceRecord | null>(null);
const deleting = ref(false);
const renameTarget = ref<InstanceRecord | null>(null);
const renameText = ref('');
const assignGroupTarget = ref<InstanceRecord | null>(null);
const assignGroupValue = ref('');

const availableGroups = computed(() => {
  const set = new Set<string>(store.preferences.groups || []);
  for (const inst of store.instances) {
    if (inst.group) set.add(inst.group);
  }
  return [...set].sort((a, b) => a.localeCompare(b));
});

const availableVersions = computed(() => {
  const set = new Set<string>();
  for (const inst of store.instances) set.add(inst.gameVersion);
  return [...set];
});

const filteredInstances = computed(() => {
  const q = search.value.trim().toLowerCase();
  const list = store.instances.filter((inst) => {
    if (q) {
      const hay = `${inst.name} ${inst.gameVersion} ${inst.loader} ${inst.group || ''}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    if (loaderFilter.value !== 'all' && inst.loader !== loaderFilter.value) return false;
    if (versionFilter.value !== 'all' && inst.gameVersion !== versionFilter.value) return false;
    if (groupFilter.value === 'favourites' && !inst.favourite) return false;
    if (groupFilter.value === 'pinned' && !inst.pinned) return false;
    if (groupFilter.value === 'ungrouped' && inst.group) return false;
    if (
      groupFilter.value !== 'all' &&
      groupFilter.value !== 'favourites' &&
      groupFilter.value !== 'pinned' &&
      groupFilter.value !== 'ungrouped' &&
      inst.group !== groupFilter.value
    ) {
      return false;
    }
    return true;
  });

  list.sort((a, b) => {
    if (sortBy.value === 'name') return a.name.localeCompare(b.name);
    if (sortBy.value === 'playtime') return (b.playtimeSeconds ?? 0) - (a.playtimeSeconds ?? 0);
    if (sortBy.value === 'created') {
      return Date.parse(b.createdAt || '') - Date.parse(a.createdAt || '');
    }
    const ta = a.lastPlayed ? Date.parse(a.lastPlayed) : 0;
    const tb = b.lastPlayed ? Date.parse(b.lastPlayed) : 0;
    if (tb !== ta) return tb - ta;
    return a.name.localeCompare(b.name);
  });
  return list;
});

const ctxItems = computed<ContextMenuItem[]>(() => {
  const inst = ctx.value.inst;
  if (!inst) return [];
  return [
    { id: 'play', label: inst.running ? 'Stop Game' : 'Play', disabled: inst.state !== 'installed' },
    { id: 'open', label: 'Open Instance' },
    { id: 'edit', label: 'Rename' },
    { id: 'settings', label: 'Instance Settings' },
    { id: 'sep1', label: '', separator: true },
    { id: 'pin', label: inst.pinned ? 'Unpin from Sidebar' : 'Pin to Sidebar' },
    { id: 'fav', label: inst.favourite ? 'Remove Favourite' : 'Mark Favourite' },
    { id: 'group', label: 'Set Group...' },
    { id: 'sep2', label: '', separator: true },
    { id: 'duplicate', label: 'Duplicate Instance' },
    { id: 'folder', label: 'Open Folder' },
    { id: 'delete', label: 'Delete Instance', danger: true, disabled: Boolean(inst.running) },
  ];
});

function openContextMenu(e: MouseEvent, inst: InstanceRecord) {
  e.preventDefault();
  e.stopPropagation();
  ctx.value = { open: true, x: e.clientX, y: e.clientY, inst };
}

async function handleContextAction(action: string) {
  const inst = ctx.value.inst;
  if (!inst) return;
  if (action === 'play') await launchOrStopInstance(inst);
  else if (action === 'open') navigate('instance', { instanceId: inst.id, tab: 'overview' });
  else if (action === 'settings') navigate('instance', { instanceId: inst.id, tab: 'settings' });
  else if (action === 'edit') {
    renameTarget.value = inst;
    renameText.value = inst.name;
  } else if (action === 'pin') {
    await patchInstanceMeta(inst.id, { pinned: !inst.pinned });
  } else if (action === 'fav') {
    await patchInstanceMeta(inst.id, { favourite: !inst.favourite });
  } else if (action === 'group') {
    assignGroupTarget.value = inst;
    assignGroupValue.value = inst.group || '';
  } else if (action === 'duplicate') {
    try {
      await native('duplicate_instance', { id: inst.id });
      await refreshSnapshot();
      notify('success', 'Instance duplicated', `Created a copy of ${inst.name}.`);
    } catch (err) {
      notify('error', "Couldn't duplicate instance", String(err));
    }
  } else if (action === 'folder') {
    if (store.desktop) {
      native('open_instance_folder', { id: inst.id }).catch((err) =>
        notify('error', "Couldn't open folder", String(err)),
      );
    }
  } else if (action === 'delete') {
    deleteTarget.value = inst;
  }
}

async function confirmCreateGroup() {
  const clean = newGroupName.value.trim();
  if (!clean) return;
  savingGroup.value = true;
  try {
    const groups = [...new Set([...(store.preferences.groups || []), clean])];
    await savePreferencesFull({ ...store.preferences, groups });
    newGroupName.value = '';
    showGroupModal.value = false;
    groupFilter.value = clean;
    notify('success', 'Group created', `Added group "${clean}".`);
  } catch (err) {
    notify('error', "Couldn't create group", String(err));
  } finally {
    savingGroup.value = false;
  }
}

async function confirmRename() {
  if (!renameTarget.value) return;
  const trimmed = renameText.value.trim();
  if (!trimmed) return;
  try {
    await patchInstanceMeta(renameTarget.value.id, { name: trimmed });
    renameTarget.value = null;
    notify('success', 'Instance renamed');
  } catch (err) {
    notify('error', "Couldn't rename instance", String(err));
  }
}

async function confirmAssignGroup() {
  if (!assignGroupTarget.value) return;
  const clean = assignGroupValue.value.trim();
  try {
    if (clean && !(store.preferences.groups || []).includes(clean)) {
      await savePreferencesFull({
        ...store.preferences,
        groups: [...(store.preferences.groups || []), clean],
      });
    }
    await patchInstanceMeta(assignGroupTarget.value.id, { group: clean || null });
    assignGroupTarget.value = null;
    notify('success', 'Group updated');
  } catch (err) {
    notify('error', "Couldn't update group", String(err));
  }
}

async function confirmDeleteInstance() {
  if (!deleteTarget.value) return;
  deleting.value = true;
  try {
    await native('delete_instance', { id: deleteTarget.value.id });
    const name = deleteTarget.value.name;
    deleteTarget.value = null;
    await refreshSnapshot();
    notify('success', 'Instance deleted', `${name} was removed from disk.`);
  } catch (err) {
    notify('error', "Couldn't delete instance", String(err));
  } finally {
    deleting.value = false;
  }
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-page-inner">
      <!-- Header -->
      <header class="sol-page-header">
        <div>
          <h1 class="sol-page-title">Library</h1>
          <p class="sol-page-sub">{{ store.instances.length }} instance{{ store.instances.length === 1 ? '' : 's' }} configured</p>
        </div>
        <div class="sol-row-gap">
          <SolaraButton variant="secondary" size="sm" @click="showGroupModal = true">
            <FolderPlus :size="14" /> New Group
          </SolaraButton>
          <SolaraButton variant="primary" size="sm" @click="openCreateInstance()">
            <Plus :size="15" /> New Instance
          </SolaraButton>
        </div>
      </header>

      <!-- Toolbar -->
      <div class="sol-toolbar">
        <div class="sol-search-box">
          <Search :size="14" class="sol-muted" />
          <input
            v-model="search"
            type="text"
            class="sol-search-input"
            placeholder="Search instances..."
          />
        </div>

        <select v-model="groupFilter" class="sol-select" aria-label="Filter by group">
          <option value="all">All Groups</option>
          <option value="favourites">Favourites</option>
          <option value="pinned">Pinned to Sidebar</option>
          <option value="ungrouped">Ungrouped</option>
          <option v-for="g in availableGroups" :key="g" :value="g">{{ g }}</option>
        </select>

        <select v-model="loaderFilter" class="sol-select" aria-label="Filter by loader">
          <option value="all">All Loaders</option>
          <option value="vanilla">Vanilla</option>
          <option value="fabric">Fabric</option>
          <option value="neoforge">NeoForge</option>
          <option value="forge">Forge</option>
          <option value="quilt">Quilt</option>
        </select>

        <select v-model="versionFilter" class="sol-select" aria-label="Filter by version">
          <option value="all">All Versions</option>
          <option v-for="v in availableVersions" :key="v" :value="v">{{ v }}</option>
        </select>

        <select v-model="sortBy" class="sol-select" aria-label="Sort instances">
          <option value="lastPlayed">Sort: Last Played</option>
          <option value="name">Sort: Name (A–Z)</option>
          <option value="created">Sort: Created Date</option>
          <option value="playtime">Sort: Playtime</option>
        </select>

        <div class="sol-seg-group" role="group" aria-label="View layout">
          <button
            type="button"
            :class="['sol-seg-btn', { 'is-active': viewMode === 'grid' }]"
            title="Grid view"
            aria-label="Grid view"
            @click="viewMode = 'grid'"
          >
            <Grid :size="14" />
          </button>
          <button
            type="button"
            :class="['sol-seg-btn', { 'is-active': viewMode === 'list' }]"
            title="List view"
            aria-label="List view"
            @click="viewMode = 'list'"
          >
            <List :size="14" />
          </button>
        </div>
      </div>

      <!-- Empty State -->
      <SolaraEmptyState
        v-if="store.instances.length === 0"
        title="Your Library is Empty"
        description="Create a Minecraft instance or install a modpack from Discover to get started."
      >
        <template #icon><Library :size="26" /></template>
        <template #actions>
          <SolaraButton variant="primary" @click="openCreateInstance()">
            <Plus :size="15" /> Create Instance
          </SolaraButton>
          <SolaraButton variant="secondary" @click="navigate('discover')">
            Browse Modpacks
          </SolaraButton>
        </template>
      </SolaraEmptyState>

      <SolaraEmptyState
        v-else-if="filteredInstances.length === 0"
        title="No matching instances"
        description="Try clearing your search or changing the active group and loader filters."
      >
        <template #actions>
          <SolaraButton
            variant="secondary"
            size="sm"
            @click="
              search = '';
              groupFilter = 'all';
              loaderFilter = 'all';
              versionFilter = 'all';
            "
          >
            Clear Filters
          </SolaraButton>
        </template>
      </SolaraEmptyState>

      <!-- Grid View -->
      <div v-else-if="viewMode === 'grid'" class="sol-inst-grid">
        <article
          v-for="inst in filteredInstances"
          :key="inst.id"
          :class="['sol-card', 'sol-inst-card', { 'is-selected': selectedInstance?.id === inst.id }]"
          @click="selectInstance(inst.id)"
          @dblclick="navigate('instance', { instanceId: inst.id, tab: 'overview' })"
          @contextmenu="openContextMenu($event, inst)"
        >
          <div class="sol-inst-card-top">
            <InstanceIcon
              :name="inst.name"
              :loader="inst.loader"
              :icon="inst.icon"
              :custom-icon="inst.customIcon"
              :size="44"
              :rounded="11"
            />
            <div class="sol-inst-card-actions">
              <button
                type="button"
                :class="['sol-icon-btn', 'sol-icon-btn--xs', { 'is-gold': inst.favourite }]"
                :title="inst.favourite ? 'Remove favourite' : 'Favourite'"
                @click.stop="patchInstanceMeta(inst.id, { favourite: !inst.favourite })"
              >
                <Star :size="13" />
              </button>
              <button
                type="button"
                :class="['sol-icon-btn', 'sol-icon-btn--xs', { 'is-gold': inst.pinned }]"
                :title="inst.pinned ? 'Unpin from sidebar' : 'Pin to sidebar'"
                @click.stop="patchInstanceMeta(inst.id, { pinned: !inst.pinned })"
              >
                <Pin :size="13" />
              </button>
              <button
                type="button"
                class="sol-icon-btn sol-icon-btn--xs"
                title="More actions"
                @click.stop="openContextMenu($event, inst)"
              >
                <MoreHorizontal :size="14" />
              </button>
            </div>
          </div>

          <div class="sol-inst-card-body">
            <div class="sol-inst-card-title-row">
              <h3 class="sol-inst-card-title sol-truncate">{{ inst.name }}</h3>
              <span v-if="inst.running" class="sol-badge sol-badge--success">Running</span>
              <span v-else-if="inst.state !== 'installed'" :class="['sol-badge', `sol-badge--${inst.state}`]">{{ inst.state }}</span>
            </div>
            <div class="sol-inst-card-meta">
              <span>{{ inst.gameVersion }}</span>
              <span>•</span>
              <span>{{ LOADER_LABEL[inst.loader] || inst.loader }}</span>
              <span v-if="inst.group" class="sol-group-chip">{{ inst.group }}</span>
            </div>
          </div>

          <div class="sol-inst-card-foot">
            <span class="sol-muted sol-text-xs">
              <template v-if="inst.playtimeSeconds">
                <Clock :size="11" /> {{ formatDuration(inst.playtimeSeconds) }} •
              </template>
              {{ formatRelativeTime(inst.lastPlayed) }}
            </span>
            <div class="sol-row-gap">
              <SolaraButton
                variant="ghost"
                size="xs"
                @click.stop="navigate('instance', { instanceId: inst.id, tab: 'overview' })"
              >
                Open
              </SolaraButton>
              <SolaraButton
                v-if="inst.state === 'installed'"
                :variant="inst.running ? 'danger' : 'primary'"
                size="xs"
                @click.stop="launchOrStopInstance(inst)"
              >
                <Square v-if="inst.running" :size="11" />
                <Play v-else :size="11" />
                {{ inst.running ? 'Stop' : 'Play' }}
              </SolaraButton>
            </div>
          </div>
        </article>
      </div>

      <!-- List View -->
      <div v-else class="sol-card sol-list-table">
        <div
          v-for="inst in filteredInstances"
          :key="inst.id"
          :class="['sol-list-row', { 'is-selected': selectedInstance?.id === inst.id }]"
          @click="selectInstance(inst.id)"
          @dblclick="navigate('instance', { instanceId: inst.id, tab: 'overview' })"
          @contextmenu="openContextMenu($event, inst)"
        >
          <div class="sol-list-cell-main">
            <InstanceIcon
              :name="inst.name"
              :loader="inst.loader"
              :icon="inst.icon"
              :custom-icon="inst.customIcon"
              :size="36"
              :rounded="9"
            />
            <div class="sol-list-info">
              <div class="sol-row-gap">
                <span class="sol-list-name">{{ inst.name }}</span>
                <span v-if="inst.group" class="sol-group-chip">{{ inst.group }}</span>
                <span v-if="inst.running" class="sol-badge sol-badge--success">Running</span>
                <span v-else-if="inst.state !== 'installed'" :class="['sol-badge', `sol-badge--${inst.state}`]">{{ inst.state }}</span>
              </div>
              <div class="sol-muted sol-text-xs">
                Minecraft {{ inst.gameVersion }} • {{ LOADER_LABEL[inst.loader] || inst.loader }} • {{ inst.memory }} MB
              </div>
            </div>
          </div>

          <div class="sol-list-cell-meta sol-muted sol-text-xs">
            <span>Last played: {{ formatRelativeTime(inst.lastPlayed) }}</span>
            <span v-if="inst.playtimeSeconds">Playtime: {{ formatDuration(inst.playtimeSeconds) }}</span>
          </div>

          <div class="sol-row-gap">
            <button
              type="button"
              class="sol-icon-btn sol-icon-btn--xs"
              title="Open Folder"
              @click.stop="native('open_instance_folder', { id: inst.id }).catch(() => {})"
            >
              <FolderOpen :size="13" />
            </button>
            <button
              type="button"
              class="sol-icon-btn sol-icon-btn--xs"
              title="Settings"
              @click.stop="navigate('instance', { instanceId: inst.id, tab: 'settings' })"
            >
              <Settings :size="13" />
            </button>
            <SolaraButton
              v-if="inst.state === 'installed'"
              :variant="inst.running ? 'danger' : 'primary'"
              size="xs"
              @click.stop="launchOrStopInstance(inst)"
            >
              <Square v-if="inst.running" :size="11" />
              <Play v-else :size="11" />
              {{ inst.running ? 'Stop' : 'Play' }}
            </SolaraButton>
            <button
              type="button"
              class="sol-icon-btn sol-icon-btn--xs"
              title="More"
              @click.stop="openContextMenu($event, inst)"
            >
              <MoreHorizontal :size="14" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Context Menu -->
    <SolaraContextMenu
      :open="ctx.open"
      :x="ctx.x"
      :y="ctx.y"
      :items="ctxItems"
      @close="ctx.open = false"
      @select="handleContextAction"
    />

    <!-- New Group Modal -->
    <SolaraModal :open="showGroupModal" title="Create Instance Group" width="sm" @close="showGroupModal = false">
      <div class="sol-field">
        <label class="sol-label">Group Name</label>
        <input
          v-model="newGroupName"
          type="text"
          class="sol-input"
          placeholder="e.g. Survival, Modded SMP, Technical"
          maxlength="60"
          @keydown.enter="confirmCreateGroup"
        />
      </div>
      <template #footer>
        <SolaraButton variant="ghost" @click="showGroupModal = false">Cancel</SolaraButton>
        <SolaraButton variant="primary" :loading="savingGroup" :disabled="!newGroupName.trim()" @click="confirmCreateGroup">
          Create Group
        </SolaraButton>
      </template>
    </SolaraModal>

    <!-- Rename Modal -->
    <SolaraModal :open="Boolean(renameTarget)" title="Rename Instance" width="sm" @close="renameTarget = null">
      <div class="sol-field">
        <label class="sol-label">Instance Name</label>
        <input
          v-model="renameText"
          type="text"
          class="sol-input"
          maxlength="80"
          @keydown.enter="confirmRename"
        />
      </div>
      <template #footer>
        <SolaraButton variant="ghost" @click="renameTarget = null">Cancel</SolaraButton>
        <SolaraButton variant="primary" :disabled="!renameText.trim()" @click="confirmRename">Save</SolaraButton>
      </template>
    </SolaraModal>

    <!-- Assign Group Modal -->
    <SolaraModal :open="Boolean(assignGroupTarget)" title="Set Instance Group" width="sm" @close="assignGroupTarget = null">
      <div class="sol-field">
        <label class="sol-label">Group</label>
        <input
          v-model="assignGroupValue"
          type="text"
          class="sol-input"
          list="solara-groups-list"
          placeholder="Leave blank to ungroup"
          maxlength="60"
        />
        <datalist id="solara-groups-list">
          <option v-for="g in availableGroups" :key="g" :value="g" />
        </datalist>
      </div>
      <template #footer>
        <SolaraButton variant="ghost" @click="assignGroupTarget = null">Cancel</SolaraButton>
        <SolaraButton variant="primary" @click="confirmAssignGroup">Apply</SolaraButton>
      </template>
    </SolaraModal>

    <!-- Delete Confirmation -->
    <ConfirmDialog
      :open="Boolean(deleteTarget)"
      title="Delete Instance"
      :message="`This will permanently remove '${deleteTarget?.name ?? ''}', including all mods, configurations, and worlds inside it.`"
      confirm-label="Delete Permanently"
      :danger="true"
      :require-text="deleteTarget?.name"
      :loading="deleting"
      @cancel="deleteTarget = null"
      @confirm="confirmDeleteInstance"
    />
  </div>
</template>
