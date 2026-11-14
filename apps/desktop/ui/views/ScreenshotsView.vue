<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { Camera, Search, FolderOpen, Trash2, RefreshCw, Copy } from '../icons';
import { store, formatBytes, formatRelativeTime, native, notify } from '../state';
import {
  copyScreenshotPath,
  getCachedPreview,
  invalidateScreenshotPreview,
  loadFullScreenshot,
  loadScreenshotPreview,
  peekCachedPreview,
  screenshotKey,
} from '../screenshots';
import type { ScreenshotEntry } from '../types';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import SolaraModal from '../components/SolaraModal.vue';

const screenshots = ref<ScreenshotEntry[]>([]);
const loading = ref(false);
const instanceFilter = ref('all');
const search = ref('');
const lightbox = ref<ScreenshotEntry | null>(null);
const lightboxSrc = ref<string | null>(null);
const lightboxLoading = ref(false);
const lightboxError = ref<string | null>(null);

const observedMap = new WeakMap<Element, ScreenshotEntry>();
const observedElements = new WeakSet<Element>();
let observer: IntersectionObserver | null = null;

function ensureObserver() {
  if (observer || typeof IntersectionObserver === 'undefined') return;
  observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        const shot = observedMap.get(entry.target);
        if (shot && !peekCachedPreview(shot)) {
          void loadScreenshotPreview(shot);
        }
      }
    },
    { rootMargin: '160px' },
  );
}

function bindCardRef(shot: ScreenshotEntry, el: unknown) {
  if (!(el instanceof Element)) return;
  if (typeof IntersectionObserver === 'undefined') {
    if (!peekCachedPreview(shot)) void loadScreenshotPreview(shot);
    return;
  }
  ensureObserver();
  observedMap.set(el, shot);
  if (!observedElements.has(el)) {
    observedElements.add(el);
    observer?.observe(el);
  }
}

async function loadAll() {
  if (!store.desktop) return;
  loading.value = true;
  try {
    screenshots.value = await native<ScreenshotEntry[]>('launcher_screenshots', {
      instanceId: null,
    });
  } catch (e) {
    notify('error', "Couldn't load screenshots", String(e));
  } finally {
    loading.value = false;
  }
}

async function openLightbox(shot: ScreenshotEntry) {
  lightbox.value = shot;
  lightboxSrc.value = getCachedPreview(shot) ?? null;
  lightboxError.value = null;
  lightboxLoading.value = true;
  try {
    const full = await loadFullScreenshot(shot);
    if (lightbox.value && screenshotKey(lightbox.value) === screenshotKey(shot)) {
      lightboxSrc.value = full;
    }
  } catch (e) {
    if (lightbox.value && screenshotKey(lightbox.value) === screenshotKey(shot)) {
      lightboxError.value = String(e);
    }
  } finally {
    if (lightbox.value && screenshotKey(lightbox.value) === screenshotKey(shot)) {
      lightboxLoading.value = false;
    }
  }
}

function closeLightbox() {
  lightbox.value = null;
  lightboxSrc.value = null;
  lightboxError.value = null;
  lightboxLoading.value = false;
}

onMounted(loadAll);
onBeforeUnmount(() => {
  observer?.disconnect();
  observer = null;
});

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
    invalidateScreenshotPreview(s);
    closeLightbox();
    await loadAll();
    notify('info', 'Moved screenshot to trash');
  } catch (e) {
    notify('error', "Couldn't move screenshot to trash", String(e));
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
          :key="screenshotKey(shot)"
          :ref="(el) => bindCardRef(shot, el)"
          class="sol-shot-card"
          @click="openLightbox(shot)"
        >
          <img
            v-if="peekCachedPreview(shot)"
            :src="peekCachedPreview(shot)"
            :alt="shot.fileName"
            class="sol-shot-img"
          />
          <div v-else class="sol-shot-placeholder">
            <Camera :size="20" />
          </div>
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
      @close="closeLightbox"
    >
      <img
        v-if="lightbox && lightboxSrc"
        :src="lightboxSrc"
        :alt="lightbox.fileName"
        class="sol-lightbox-img"
      />
      <div v-else-if="lightbox" class="sol-lightbox-placeholder">
        <Camera :size="26" />
        <span v-if="lightboxLoading">Loading full-resolution image...</span>
        <span v-else-if="lightboxError" class="sol-muted sol-text-xs">{{ lightboxError }}</span>
      </div>
      <template v-if="lightbox" #footer>
        <SolaraButton variant="secondary" size="sm" @click="copyScreenshotPath(lightbox)">
          <Copy :size="13" /> Copy file path
        </SolaraButton>
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
