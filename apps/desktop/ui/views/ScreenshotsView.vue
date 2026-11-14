<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { Camera, Search, FolderOpen, Trash2, RefreshCw } from '../icons';
import { store, formatBytes, formatRelativeTime, native, notify } from '../state';
import type { ScreenshotEntry } from '../types';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import SolaraModal from '../components/SolaraModal.vue';

const screenshots = ref<ScreenshotEntry[]>([]);
const loading = ref(false);
const instanceFilter = ref('all');
const search = ref('');
const lightbox = ref<ScreenshotEntry | null>(null);

async function loadAll() {
  if (!store.desktop) return;
  loading.value = true;
  try {
    screenshots.value = await native<ScreenshotEntry[]>('launcher_screenshots', {
      instanceId: null,
    });
  } catch (e) {
    notify('error', 'Failed loading screenshots', String(e));
  } finally {
    loading.value = false;
  }
}

onMounted(loadAll);

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  return screenshots.value.filter((s) => {
    if (instanceFilter.value !== 'all' && s.instanceId !== instanceFilter.value) return false;
    if (q && !`${s.fileName} ${s.instanceName}`.toLowerCase().includes(q)) return false;
    return true;
  });
});

async function trashShot(s: ScreenshotEntry) {
  try {
    await native('instance_file_action', {
      id: s.instanceId,
      action: 'trash',
      path: s.path,
    });
    lightbox.value = null;
    await loadAll();
    notify('info', 'Screenshot moved to instance trash');
  } catch (e) {
    notify('error', 'Failed', String(e));
  }
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-page-inner">
      <header class="sol-page-header">
        <div>
          <h1 class="sol-page-title">Screenshots</h1>
          <p class="sol-page-sub">{{ screenshots.length }} capture(s) across all instances</p>
        </div>
        <SolaraButton variant="secondary" size="sm" :loading="loading" @click="loadAll">
          <RefreshCw :size="13" /> Refresh
        </SolaraButton>
      </header>

      <div class="sol-toolbar">
        <div class="sol-search-box" style="flex: 1">
          <Search :size="13" class="sol-muted" />
          <input v-model="search" type="text" class="sol-search-input" placeholder="Search by filename or instance..." />
        </div>
        <select v-model="instanceFilter" class="sol-select">
          <option value="all">All Instances</option>
          <option v-for="inst in store.instances" :key="inst.id" :value="inst.id">{{ inst.name }}</option>
        </select>
      </div>

      <SolaraEmptyState
        v-if="!loading && filtered.length === 0"
        title="No screenshots found"
        description="Press F2 while playing Minecraft to capture screenshots. They are read directly from each instance's screenshots/ folder."
      >
        <template #icon><Camera :size="26" /></template>
      </SolaraEmptyState>

      <div v-else class="sol-shots-grid">
        <div
          v-for="shot in filtered"
          :key="`${shot.instanceId}-${shot.path}`"
          class="sol-shot-card"
          @click="lightbox = shot"
        >
          <img :src="shot.dataUrl" :alt="shot.fileName" class="sol-shot-img" loading="lazy" />
          <div class="sol-shot-meta">
            <div class="sol-truncate sol-shot-name">{{ shot.fileName }}</div>
            <div class="sol-muted sol-text-xs sol-truncate">
              {{ shot.instanceName }} • {{ formatRelativeTime(shot.modifiedMs) }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <SolaraModal
      :open="Boolean(lightbox)"
      :title="lightbox?.fileName"
      :subtitle="lightbox ? `${lightbox.instanceName} • ${formatBytes(lightbox.size)}` : ''"
      width="lg"
      @close="lightbox = null"
    >
      <img v-if="lightbox" :src="lightbox.dataUrl" :alt="lightbox.fileName" class="sol-lightbox-img" />
      <template v-if="lightbox" #footer>
        <SolaraButton
          variant="secondary"
          size="sm"
          @click="native('open_instance_folder', { id: lightbox.instanceId, path: 'screenshots' }).catch(() => {})"
        >
          <FolderOpen :size="13" /> Open Folder
        </SolaraButton>
        <SolaraButton variant="danger" size="sm" @click="trashShot(lightbox)">
          <Trash2 :size="13" /> Move to Trash
        </SolaraButton>
      </template>
    </SolaraModal>
  </div>
</template>
