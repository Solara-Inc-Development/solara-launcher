<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import SolaraButton from './SolaraButton.vue';
import SolaraEmptyState from './SolaraEmptyState.vue';
import SolaraModal from './SolaraModal.vue';
import {
  AlertTriangle,
  ArrowUpRight,
  Check,
  Compass,
  Download,
  RefreshCw,
  Search,
  Sliders,
  X,
} from '../icons';
import {
  LOADER_LABEL,
  native,
  notify,
  openExternal,
  openInstanceSettings,
  refreshSnapshot,
  store,
} from '../state';
import type { InstanceRecord, WorldEntry } from '../types';

export type BrowserContentTab = 'mod' | 'resourcepack' | 'shader' | 'datapack';

interface ModrinthHit {
  project_id: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  icon_url?: string | null;
  categories: string[];
  downloads: number | null;
  project_type: string;
}

interface ModrinthVersion {
  id: string;
  name: string;
  version_type?: string;
  game_versions?: string[];
  loaders?: string[];
}

const props = defineProps<{
  open: boolean;
  instance: InstanceRecord;
  initialTab?: BrowserContentTab;
  worlds: WorldEntry[];
  installedNames?: string[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'installed', tab: BrowserContentTab): void;
}>();

const activeTab = ref<BrowserContentTab>(props.initialTab ?? 'mod');
const searchQuery = ref('');
const sortIndex = ref<'downloads' | 'updated' | 'relevance' | 'newest'>('downloads');
const matchInstanceVersion = ref(true);
const selectedWorld = ref('');
const hits = ref<ModrinthHit[]>([]);
const pageOffset = ref(0);
const loading = ref(false);
const errorMsg = ref<string | null>(null);
const installingId = ref<string | null>(null);
const justInstalledIds = ref<Set<string>>(new Set());
const pendingInstalls = ref<Record<string, { jobId: string; title: string; tab: BrowserContentTab; world: string; instanceId: string }>>({});
let searchController: AbortController | null = null;
let detailController: AbortController | null = null;
function installationKey(projectId: string): string {
  return `${props.instance.id}:${activeTab.value}:${activeTab.value === 'datapack' ? selectedWorld.value : ''}:${projectId}`;
}
watch(() => store.downloads, (jobs) => {
  for (const [key, pending] of Object.entries(pendingInstalls.value)) {
    const job = jobs.find((item) => item.id === pending.jobId);
    if (!job || job.status === 'queued' || job.status === 'running') continue;
    delete pendingInstalls.value[key];
    if (job.status === 'completed') {
      justInstalledIds.value = new Set(justInstalledIds.value).add(key);
      notify('success', `Installed ${pending.title}`, pending.world ? `Added to world "${pending.world}".` : 'Content download completed.');
      if (pending.instanceId === props.instance.id) emit('installed', pending.tab);
    } else notify('warning', `${pending.title} ${job.status}`, job.phase || 'Open Downloads & Tasks to retry.');
  }
});

// Rich project detail view for inspecting description, gallery, dependencies & versions
const activeProject = ref<ModrinthHit | null>(null);
const projectBody = ref('');
const projectGallery = ref<Array<{ url: string; title?: string }>>([]);
const projectVersions = ref<ModrinthVersion[]>([]);
const selectedVersionId = ref('');
const loadingVersions = ref(false);

const installedSet = computed(
  () => new Set((props.installedNames ?? []).map((n) => n.toLowerCase())),
);

const isVanillaModsBlocked = computed(
  () => activeTab.value === 'mod' && props.instance.loader === 'vanilla',
);

const datapackWorldMissing = computed(
  () => activeTab.value === 'datapack' && (!selectedWorld.value || props.worlds.length === 0),
);

function syncDefaultWorld(): void {
  if (props.worlds.length > 0) {
    if (!selectedWorld.value || !props.worlds.some((w) => w.folder === selectedWorld.value)) {
      selectedWorld.value = props.worlds[0].folder;
    }
  } else {
    selectedWorld.value = '';
  }
}

function formatDownloads(n: number | null | undefined): string {
  if (!n) return '0';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

async function runSearch(resetOffset = true): Promise<void> {
  if (!props.open) return;
  if (resetOffset) pageOffset.value = 0;
  searchController?.abort();
  const controller = searchController = new AbortController();
  loading.value = true;
  errorMsg.value = null;
  try {
    const facets: string[][] = [[`project_type:${activeTab.value}`]];
    if (matchInstanceVersion.value && props.instance.gameVersion) {
      facets.push([`versions:${props.instance.gameVersion}`]);
    }
    if (activeTab.value === 'mod' && props.instance.loader !== 'vanilla') {
      facets.push([`categories:${props.instance.loader}`]);
    }

    const params = new URLSearchParams({
      query: searchQuery.value.trim(),
      facets: JSON.stringify(facets),
      index: sortIndex.value,
      limit: '15',
      offset: String(pageOffset.value),
    });
    const res = await fetch(`https://api.modrinth.com/v2/search?${params.toString()}`, {
      headers: { Accept: 'application/json' }, signal: controller.signal,
    });
    if (!res.ok) throw new Error(`Modrinth returned HTTP ${res.status}`);
    const data = (await res.json()) as { hits?: ModrinthHit[] };
    if (!controller.signal.aborted) hits.value = Array.isArray(data.hits) ? data.hits : [];
  } catch (err) {
    if (!controller.signal.aborted) errorMsg.value = err instanceof Error ? err.message : String(err);
  } finally {
    if (searchController === controller) loading.value = false;
  }
}

async function openVersions(hit: ModrinthHit): Promise<void> {
  detailController?.abort();
  const controller = detailController = new AbortController();
  const tab = activeTab.value;
  activeProject.value = hit;
  projectBody.value = hit.description || '';
  projectGallery.value = [];
  projectVersions.value = [];
  selectedVersionId.value = '';
  loadingVersions.value = true;
  try {
    const params = new URLSearchParams();
    if (props.instance.gameVersion) {
      params.set('game_versions', JSON.stringify([props.instance.gameVersion]));
    }
    if (activeTab.value === 'mod' && props.instance.loader !== 'vanilla') {
      params.set('loaders', JSON.stringify([props.instance.loader]));
    }
    const [projRes, verRes] = await Promise.all([
      fetch(`https://api.modrinth.com/v2/project/${encodeURIComponent(hit.project_id)}`, { signal: controller.signal }),
      fetch(
        `https://api.modrinth.com/v2/project/${encodeURIComponent(hit.project_id)}/version?${params.toString()}`, { signal: controller.signal },
      ),
    ]);
    if (projRes.ok) {
      const proj = (await projRes.json()) as {
        body?: string;
        gallery?: Array<{ url: string; title?: string }>;
      };
      if (controller.signal.aborted) return;
      projectBody.value = (proj.body || hit.description || '').slice(0, 1200);
      projectGallery.value = Array.isArray(proj.gallery) ? proj.gallery.slice(0, 3) : [];
    }
    if (verRes.ok) {
      const raw = (await verRes.json()) as ModrinthVersion[];
      if (controller.signal.aborted) return;
      // Never offer an incompatible fallback release as compatible.
      const list = Array.isArray(raw) ? raw.filter((v) =>
        v.game_versions?.includes(props.instance.gameVersion) &&
        (tab !== 'mod' || v.loaders?.includes(props.instance.loader))
      ) : [];
      projectVersions.value = list;
      selectedVersionId.value = list[0]?.id ?? '';
    }
  } catch (err) {
    if (!controller.signal.aborted) notify('warning', 'Could not load project details', String(err));
  } finally {
    if (detailController === controller) loadingVersions.value = false;
  }
}

async function installItem(hit: ModrinthHit, explicitVersionId?: string): Promise<void> {
  if (installingId.value || pendingInstalls.value[installationKey(hit.project_id)]) return;
  const tab = activeTab.value;
  const world = tab === 'datapack' ? selectedWorld.value : '';
  const instanceId = props.instance.id;
  const key = installationKey(hit.project_id);
  if (isVanillaModsBlocked.value) {
    notify('warning', 'Mod loader required', 'Vanilla instances cannot load mods.');
    return;
  }
  if (datapackWorldMissing.value) {
    notify('warning', 'Select a target world', 'Datapacks must be installed into a specific saved world.');
    return;
  }
  if (!store.desktop) {
    notify('info', 'Desktop required', 'Content installation runs inside the Solara desktop app.');
    return;
  }
  installingId.value = hit.project_id;
  try {
    let versionId = explicitVersionId;
    if (!versionId) {
      const params = new URLSearchParams();
      if (props.instance.gameVersion) {
        params.set('game_versions', JSON.stringify([props.instance.gameVersion]));
      }
      if (activeTab.value === 'mod' && props.instance.loader !== 'vanilla') {
        params.set('loaders', JSON.stringify([props.instance.loader]));
      }
      const res = await fetch(
        `https://api.modrinth.com/v2/project/${encodeURIComponent(hit.project_id)}/version?${params.toString()}`,
      );
      if (res.ok) {
        const list = (await res.json()) as ModrinthVersion[];
        versionId = list[0]?.id;
      }
    }
    if (!versionId) {
      throw new Error(
        `No release found matching ${props.instance.loader} ${props.instance.gameVersion}. Open Versions to pick manually.`,
      );
    }
    const receipt = await native<{ instanceId: string; jobId: string }>('install_content', {
      instanceId,
      projectId: hit.project_id,
      contentType: tab,
      versionId,
      world: world || null,
    });
    pendingInstalls.value[key] = { jobId: receipt.jobId, title: hit.title, tab, world, instanceId };
    activeProject.value = null;
    notify('info', `Queued ${hit.title}`, 'Follow the download in Downloads & Tasks.');
    await refreshSnapshot();
  } catch (err) {
    notify('error', `Couldn't install ${hit.title}`, String(err));
  } finally {
    installingId.value = null;
  }
}

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      activeTab.value = props.initialTab ?? 'mod';
      syncDefaultWorld();
      void runSearch(true);
    } else {
      searchController?.abort();
      detailController?.abort();
      activeProject.value = null;
    }
  },
);

watch(
  () => props.worlds,
  () => syncDefaultWorld(),
  { deep: true },
);

onUnmounted(() => { searchController?.abort(); detailController?.abort(); });

onMounted(() => {
  if (props.open) {
    syncDefaultWorld();
    void runSearch(true);
  }
});
</script>

<template>
  <SolaraModal
    :open="open"
    :title="`Browse Content for ${instance.name}`"
    :subtitle="`Auto-filtered for ${LOADER_LABEL[instance.loader] ?? instance.loader} · Minecraft ${instance.gameVersion}`"
    width="lg"
    @close="emit('close')"
  >
    <div class="sol-browser-wrap">
      <!-- Content Type Tabs inside Instance Browser -->
      <div class="sol-browser-tabs">
        <button
          v-for="tab in [
            { id: 'mod', label: 'Mods' },
            { id: 'resourcepack', label: 'Resource Packs' },
            { id: 'shader', label: 'Shaders' },
            { id: 'datapack', label: 'Datapacks' },
          ] as const"
          :key="tab.id"
          type="button"
          :class="['sol-browser-tab', { 'is-active': activeTab === tab.id }]"
          @click="detailController?.abort(); activeTab = tab.id; activeProject = null; runSearch(true)"
        >
          {{ tab.label }}
        </button>

        <label class="sol-checkbox-label sol-browser-match">
          <input v-model="matchInstanceVersion" type="checkbox" @change="runSearch(true)" />
          <span>Match MC {{ instance.gameVersion }}</span>
        </label>
      </div>

      <!-- Contextual Compatibility Guidance -->
      <div v-if="isVanillaModsBlocked" class="sol-banner-warn">
        <div class="sol-inline-flex">
          <AlertTriangle :size="15" />
          <span>
            <strong>{{ instance.name }}</strong> uses Vanilla Minecraft and cannot run mods. Create a Fabric, NeoForge, Forge, or Quilt instance to install mods.
          </span>
        </div>
        <SolaraButton
          variant="secondary"
          size="xs"
          @click="emit('close'); openInstanceSettings(instance.id, 'installation')"
        >
          <Sliders :size="12" /> Installation
        </SolaraButton>
      </div>

      <div v-else-if="activeTab === 'shader'" class="sol-banner-info">
        <span>
          Shaders require an installed shader mod such as <strong>Iris</strong> (Fabric/NeoForge) or <strong>Oculus</strong> (Forge) on this instance.
        </span>
      </div>

      <div v-else-if="activeTab === 'datapack'" class="sol-datapack-target">
        <div class="sol-datapack-target-label">
          <strong>Target World</strong>
          <span class="sol-muted sol-text-xs">Datapacks install directly into saves/&lt;world&gt;/datapacks</span>
        </div>
        <select v-if="worlds.length > 0" v-model="selectedWorld" class="sol-select" aria-label="Target world">
          <option v-for="w in worlds" :key="w.folder" :value="w.folder">
            {{ w.name }} ({{ w.folder }}) · {{ w.datapackCount }} packs
          </option>
        </select>
        <span v-else class="sol-inline-warn">
          No saved worlds found yet — create or play a world in Minecraft first before installing datapacks.
        </span>
      </div>

      <!-- Search + Sort -->
      <form class="sol-discover-search" @submit.prevent="runSearch(true)">
        <div class="sol-search-box sol-search-box--grow">
          <Search :size="14" class="sol-muted" />
          <input
            v-model="searchQuery"
            type="text"
            class="sol-search-input"
            :placeholder="`Search compatible ${activeTab === 'resourcepack' ? 'resource packs' : `${activeTab}s`}...`"
          />
          <button
            v-if="searchQuery"
            type="button"
            class="sol-icon-btn sol-icon-btn--xs"
            @click="searchQuery = ''; runSearch(true)"
          >
            <X :size="12" />
          </button>
        </div>

        <select v-model="sortIndex" class="sol-select" @change="runSearch(true)">
          <option value="downloads">Popular</option>
          <option value="updated">Updated</option>
          <option value="relevance">Relevance</option>
          <option value="newest">Newest</option>
        </select>

        <SolaraButton variant="primary" size="sm" type="submit" :loading="loading">
          Search
        </SolaraButton>
      </form>

      <!-- Rich Project Detail panel when user clicked Details / Versions -->
      <div v-if="activeProject" class="sol-browser-version-box">
        <div class="sol-row-between">
          <div>
            <strong>{{ activeProject.title }}</strong>
            <span class="sol-muted sol-text-xs"> by {{ activeProject.author }}</span>
          </div>
          <div class="sol-row-gap">
            <button
              type="button"
              class="sol-link-btn"
              @click="openExternal(`https://modrinth.com/${activeTab}/${activeProject.slug || activeProject.project_id}`)"
            >
              Modrinth <ArrowUpRight :size="12" />
            </button>
            <SolaraButton variant="ghost" size="xs" @click="activeProject = null">Back to results</SolaraButton>
          </div>
        </div>
        <div v-if="projectGallery.length > 0" class="sol-gallery-strip">
          <img
            v-for="img in projectGallery"
            :key="img.url"
            :src="img.url"
            :alt="img.title || activeProject.title"
            class="sol-gallery-thumb"
            loading="lazy"
          />
        </div>
        <div v-if="projectBody" class="sol-project-body sol-scroll selectable" style="max-height: 150px">
          {{ projectBody }}
        </div>
        <div class="sol-row-gap">
          <select v-model="selectedVersionId" class="sol-input" :disabled="loadingVersions">
            <option v-if="loadingVersions" value="">Loading compatible releases...</option>
            <option v-else-if="projectVersions.length === 0" value="">No versions returned</option>
            <option v-for="v in projectVersions" :key="v.id" :value="v.id">
              {{ v.name }} ({{ (v.loaders || []).join(', ') || activeTab }} · {{ (v.game_versions || []).slice(0, 3).join(', ') }})
            </option>
          </select>
          <SolaraButton
            variant="primary"
            size="sm"
            :loading="installingId === activeProject.project_id"
            :disabled="!selectedVersionId || isVanillaModsBlocked || datapackWorldMissing"
            @click="installItem(activeProject, selectedVersionId)"
          >
            <Download :size="13" /> Install Selected
          </SolaraButton>
        </div>
      </div>

      <div v-else-if="errorMsg" class="sol-banner-warn">
        <span>Couldn't search Modrinth: {{ errorMsg }}</span>
        <SolaraButton variant="secondary" size="xs" @click="runSearch(false)">
          <RefreshCw :size="12" /> Retry
        </SolaraButton>
      </div>

      <SolaraEmptyState
        v-else-if="!loading && hits.length === 0"
        title="No matching content found"
        description="Try broadening your query or unchecking 'Match MC version' to see all releases."
      >
        <template #icon><Compass :size="24" /></template>
      </SolaraEmptyState>

      <!-- Results List -->
      <div v-else class="sol-browser-list sol-scroll">
        <div v-for="hit in hits" :key="hit.project_id" class="sol-browser-row">
          <img v-if="hit.icon_url" :src="hit.icon_url" :alt="hit.title" class="sol-browser-icon" loading="lazy" />
          <div v-else class="sol-browser-icon sol-browser-icon--empty">{{ hit.title.slice(0, 2).toUpperCase() }}</div>

          <div class="sol-browser-meta">
            <div class="sol-row-gap">
              <strong class="sol-truncate">{{ hit.title }}</strong>
              <span class="sol-muted sol-text-xs">by {{ hit.author }}</span>
              <span class="sol-muted sol-text-xs">· {{ formatDownloads(hit.downloads) }}</span>
            </div>
            <div class="sol-muted sol-text-xs sol-truncate">{{ hit.description }}</div>
          </div>

          <div class="sol-row-gap">
            <SolaraButton variant="ghost" size="xs" @click="openVersions(hit)">Versions</SolaraButton>
            <SolaraButton
              v-if="justInstalledIds.has(installationKey(hit.project_id)) || (activeTab !== 'datapack' && installedSet.has(hit.title.toLowerCase()))"
              variant="secondary"
              size="xs"
              disabled
            >
              <Check :size="12" /> Installed
            </SolaraButton>
            <SolaraButton v-else-if="pendingInstalls[installationKey(hit.project_id)]" variant="secondary" size="xs" disabled>
              <Download :size="12" /> Queued
            </SolaraButton>
            <SolaraButton
              v-else
              variant="primary"
              size="xs"
              :loading="installingId === hit.project_id"
              :disabled="Boolean(installingId) || isVanillaModsBlocked || datapackWorldMissing"
              @click="installItem(hit)"
            >
              <Download :size="12" /> Install
            </SolaraButton>
          </div>
        </div>
      </div>

      <div v-if="!activeProject && hits.length > 0" class="sol-pagination">
        <SolaraButton
          variant="secondary"
          size="xs"
          :disabled="pageOffset === 0 || loading"
          @click="pageOffset = Math.max(0, pageOffset - 15); runSearch(false)"
        >
          Previous
        </SolaraButton>
        <span class="sol-muted sol-text-xs">Page {{ Math.floor(pageOffset / 15) + 1 }}</span>
        <SolaraButton
          variant="secondary"
          size="xs"
          :disabled="hits.length < 15 || loading"
          @click="pageOffset += 15; runSearch(false)"
        >
          Next
        </SolaraButton>
      </div>
    </div>

    <template #footer>
      <SolaraButton variant="secondary" @click="emit('close')">Done</SolaraButton>
    </template>
  </SolaraModal>
</template>
