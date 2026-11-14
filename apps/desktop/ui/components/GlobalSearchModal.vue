<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { Search, Play, ArrowUpRight, Plus } from '../icons';
import { store, navigate, launchOrStopInstance, openCreateInstance, LOADER_LABEL } from '../state';
import InstanceIcon from './InstanceIcon.vue';
import SolaraModal from './SolaraModal.vue';

const query = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

watch(
  () => store.showGlobalSearch,
  async (open) => {
    if (open) {
      query.value = '';
      await nextTick();
      inputRef.value?.focus();
    }
  },
);

const pageEntries = [
  { id: 'home', label: 'Home Dashboard', hint: 'Launcher overview & active instance' },
  { id: 'library', label: 'Library', hint: 'Manage all Minecraft instances & groups' },
  { id: 'discover', label: 'Discover Content', hint: 'Browse mods, modpacks, shaders, resource packs' },
  { id: 'skins', label: 'Skin & Capes', hint: '3D viewer, pixel skin editor, capes' },
  { id: 'screenshots', label: 'Screenshots', hint: 'Browse captures across instances' },
  { id: 'tasks', label: 'Downloads & Tasks', hint: 'Installation and update queue' },
  { id: 'settings', label: 'Global Settings', hint: 'Appearance, Java, memory, downloads, storage' },
  { id: 'account', label: 'Microsoft Accounts', hint: 'Switch or sign in Minecraft Java accounts' },
] as const;

const matchedInstances = computed(() => {
  const q = query.value.trim().toLowerCase();
  const list = store.instances;
  if (!q) return list.slice(0, 5);
  return list
    .filter(
      (i) =>
        i.name.toLowerCase().includes(q) ||
        i.gameVersion.toLowerCase().includes(q) ||
        i.loader.toLowerCase().includes(q) ||
        (i.group ?? '').toLowerCase().includes(q),
    )
    .slice(0, 8);
});

const matchedPages = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return pageEntries.slice(0, 5);
  return pageEntries.filter(
    (p) => p.label.toLowerCase().includes(q) || p.hint.toLowerCase().includes(q),
  );
});

function pickInstance(id: string) {
  store.showGlobalSearch = false;
  navigate('instance', { instanceId: id, tab: 'overview' });
}

function pickPage(id: (typeof pageEntries)[number]['id']) {
  store.showGlobalSearch = false;
  navigate(id);
}
</script>

<template>
  <SolaraModal
    :open="store.showGlobalSearch"
    width="md"
    title="Quick Jump"
    @close="store.showGlobalSearch = false"
  >
    <div class="sol-search-input-wrap">
      <Search :size="16" class="sol-muted" />
      <input
        ref="inputRef"
        v-model="query"
        type="text"
        class="sol-search-raw"
        placeholder="Jump to an instance, page, or action..."
        autocomplete="off"
      />
    </div>

    <div class="sol-palette-section">
      <div class="sol-palette-heading">Instances</div>
      <div v-if="matchedInstances.length === 0" class="sol-palette-empty">
        No matching instances.
        <button
          type="button"
          class="sol-link-btn"
          @click="
            store.showGlobalSearch = false;
            openCreateInstance();
          "
        >
          <Plus :size="13" /> Create new instance
        </button>
      </div>
      <div v-for="inst in matchedInstances" :key="inst.id" class="sol-palette-row" @click="pickInstance(inst.id)">
        <InstanceIcon
          :name="inst.name"
          :loader="inst.loader"
          :icon="inst.icon"
          :custom-icon="inst.customIcon"
          :size="28"
          :rounded="7"
        />
        <div class="sol-palette-info">
          <div class="sol-palette-title">{{ inst.name }}</div>
          <div class="sol-palette-sub">
            {{ inst.gameVersion }} • {{ LOADER_LABEL[inst.loader] || inst.loader }}
            <span v-if="inst.group"> • {{ inst.group }}</span>
          </div>
        </div>
        <button
          v-if="inst.state === 'installed'"
          type="button"
          class="sol-icon-btn sol-icon-btn--xs"
          :title="inst.running ? 'Stop' : 'Play'"
          @click.stop="
            store.showGlobalSearch = false;
            launchOrStopInstance(inst);
          "
        >
          <Play :size="13" />
        </button>
      </div>
    </div>

    <div v-if="matchedPages.length > 0" class="sol-palette-section">
      <div class="sol-palette-heading">Pages & Actions</div>
      <button
        v-for="page in matchedPages"
        :key="page.id"
        type="button"
        class="sol-palette-row sol-palette-row--btn"
        @click="pickPage(page.id)"
      >
        <div class="sol-palette-info">
          <div class="sol-palette-title">{{ page.label }}</div>
          <div class="sol-palette-sub">{{ page.hint }}</div>
        </div>
        <ArrowUpRight :size="14" class="sol-muted" />
      </button>
    </div>
  </SolaraModal>
</template>
