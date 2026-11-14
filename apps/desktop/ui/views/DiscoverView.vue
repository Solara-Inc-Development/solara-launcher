<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import {
  Search,
  Download,
  RefreshCw,
  ArrowUpRight,
  AlertTriangle,
  Compass,
} from '../icons';
import {
  searchUrl,
  projectDetailUrl,
  versionsUrl,
  normalizeHits,
  normalizeVersions,
  normalizeProjectVersions,
  providerJson,
  projectUrl,
} from '../../../../packages/shared/content.mjs';
import {
  store,
  selectedInstance,
  refreshSnapshot,
  openCreateInstance,
  openExternal,
  LOADER_LABEL,
  native,
  notify,
} from '../state';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraModal from '../components/SolaraModal.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';

type ContentKind = 'mod' | 'modpack' | 'resourcepack' | 'shader' | 'datapack';

interface HitItem {
  project_id: string;
  project_type: ContentKind;
  slug: string;
  title: string;
  description: string;
  author: string;
  icon_url: string | null;
  downloads: number | null;
  categories: string[];
  versions: string[];
}

interface VersionOption {
  id: string;
  name: string;
  versionNumber?: string;
  type: string;
  gameVersions?: string[];
  loaders?: string[];
  games?: string;
  changelog?: string;
}

const kind = ref<ContentKind>('mod');
const query = ref('');
const sortBy = ref<'relevance' | 'downloads' | 'updated' | 'newest'>('downloads');
const targetInstanceId = ref<string>(selectedInstance.value?.id ?? '');
const filterByTarget = ref(true);
const pageOffset = ref(0);

const hits = ref<HitItem[]>([]);
const loading = ref(false);
const errorMsg = ref('');

// Detail / Version Modal
const activeProject = ref<HitItem | null>(null);
const projectBody = ref('');
const versions = ref<VersionOption[]>([]);
const selectedVersionId = ref('');
const loadingDetail = ref(false);
const installingId = ref('');

let activeController: AbortController | null = null;

const targetInstance = computed(
  () => store.instances.find((i) => i.id === targetInstanceId.value) ?? null,
);

const compatibilityWarning = computed(() => {
  if (kind.value === 'modpack') return '';
  if (!targetInstance.value) return 'Create an instance before installing content into a game folder.';
  if (kind.value === 'mod' && targetInstance.value.loader === 'vanilla') {
    return `${targetInstance.value.name} uses Vanilla, which cannot load mods. Select a Fabric, Quilt, Forge, or NeoForge instance.`;
  }
  return '';
});

async function runSearch(resetOffset = true) {
  if (resetOffset) pageOffset.value = 0;
  activeController?.abort();
  activeController = new AbortController();
  loading.value = true;
  errorMsg.value = '';
  try {
    const inst = filterByTarget.value && kind.value !== 'modpack' ? targetInstance.value : null;
    const loaderArg =
      inst && kind.value === 'mod' && inst.loader !== 'vanilla' ? inst.loader : 'all';
    const verArg = inst ? inst.gameVersion : 'all';
    const url = searchUrl(query.value, kind.value, {
      loader: loaderArg,
      gameVersion: verArg,
      index: sortBy.value,
      offset: pageOffset.value,
    });
    const data = await providerJson(url, activeController.signal);
    hits.value = normalizeHits(data, kind.value) as HitItem[];
  } catch (err) {
    if ((err as Error)?.name === 'AbortError') return;
    errorMsg.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}

watch([kind, sortBy, filterByTarget, targetInstanceId], () => {
  runSearch(true);
});

onMounted(() => {
  if (!targetInstanceId.value && store.instances.length > 0) {
    targetInstanceId.value = store.instances[0].id;
  }
  runSearch(true);
});

async function openProject(hit: HitItem) {
  activeProject.value = hit;
  projectBody.value = '';
  versions.value = [];
  selectedVersionId.value = '';
  loadingDetail.value = true;
  const ctrl = new AbortController();
  try {
    const [detailData, versData] = await Promise.all([
      providerJson(projectDetailUrl(hit.project_id), ctrl.signal).catch(() => null),
      providerJson(versionsUrl(hit.project_id), ctrl.signal),
    ]);
    if (detailData && typeof detailData.body === 'string') {
      projectBody.value = detailData.body.slice(0, 6000);
    }
    if (hit.project_type === 'modpack') {
      const normalized = normalizeVersions(versData, hit.project_id) as VersionOption[];
      versions.value = normalized;
      selectedVersionId.value = normalized[0]?.id ?? '';
    } else {
      let normalized = normalizeProjectVersions(versData, hit.project_id) as VersionOption[];
      if (targetInstance.value) {
        const gv = targetInstance.value.gameVersion;
        const ldr = targetInstance.value.loader;
        const matched = normalized.filter((v) => {
          const okVer = !v.gameVersions?.length || v.gameVersions.includes(gv);
          const okLdr =
            hit.project_type !== 'mod' || !v.loaders?.length || v.loaders.includes(ldr);
          return okVer && okLdr;
        });
        if (matched.length > 0) normalized = matched;
      }
      versions.value = normalized;
      selectedVersionId.value = normalized[0]?.id ?? '';
    }
  } catch (err) {
    notify('error', 'Failed loading project versions', String(err));
  } finally {
    loadingDetail.value = false;
  }
}

async function installProject(project: HitItem, versionId?: string) {
  if (project.project_type !== 'modpack' && !targetInstance.value) {
    notify('warning', 'No instance selected', 'Create or pick a target instance first.');
    return;
  }
  installingId.value = project.project_id;
  try {
    await native('install_content', {
      instanceId: project.project_type === 'modpack' ? null : targetInstance.value?.id,
      projectId: project.project_id,
      contentType: project.project_type,
      versionId: versionId || null,
    });
    await refreshSnapshot();
    notify(
      'success',
      project.project_type === 'modpack' ? `Installing ${project.title}` : `Queued ${project.title}`,
      project.project_type === 'modpack'
        ? 'Creating new instance from modpack...'
        : `Installing into ${targetInstance.value?.name}`,
    );
    activeProject.value = null;
  } catch (err) {
    notify('error', `Install failed: ${project.title}`, String(err));
  } finally {
    installingId.value = '';
  }
}

function formatDownloadsCount(n: number | null): string {
  if (n === null) return '';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-page-inner">
      <header class="sol-page-header">
        <div>
          <h1 class="sol-page-title">Discover Content</h1>
          <p class="sol-page-sub">Search verified mods, modpacks, resource packs, shaders, and datapacks</p>
        </div>

        <div class="sol-row-gap">
          <div v-if="kind !== 'modpack'" class="sol-row-gap">
            <span class="sol-muted sol-text-xs">Target:</span>
            <select v-model="targetInstanceId" class="sol-select">
              <option value="" disabled>Select instance...</option>
              <option v-for="inst in store.instances" :key="inst.id" :value="inst.id">
                {{ inst.name }} ({{ inst.gameVersion }} {{ LOADER_LABEL[inst.loader] }})
              </option>
            </select>
            <label class="sol-checkbox-label">
              <input v-model="filterByTarget" type="checkbox" />
              <span>Compatible only</span>
            </label>
          </div>
          <SolaraButton v-if="store.instances.length === 0" variant="primary" size="sm" @click="openCreateInstance()">
            Create Instance
          </SolaraButton>
        </div>
      </header>

      <!-- Kind & Search Toolbar -->
      <div class="sol-toolbar">
        <div class="sol-seg-group">
          <button
            v-for="tab in [
              { id: 'mod', label: 'Mods' },
              { id: 'modpack', label: 'Modpacks' },
              { id: 'resourcepack', label: 'Resource Packs' },
              { id: 'shader', label: 'Shaders' },
              { id: 'datapack', label: 'Datapacks' },
            ] as const"
            :key="tab.id"
            type="button"
            :class="['sol-seg-btn', { 'is-active': kind === tab.id }]"
            @click="kind = tab.id"
          >
            {{ tab.label }}
          </button>
        </div>

        <form class="sol-search-box" style="flex: 1; min-width: 220px" @submit.prevent="runSearch(true)">
          <Search :size="14" class="sol-muted" />
          <input
            v-model="query"
            type="text"
            class="sol-search-input"
            :placeholder="`Search ${kind}s...`"
          />
          <SolaraButton type="submit" variant="primary" size="xs" :loading="loading">Search</SolaraButton>
        </form>

        <select v-model="sortBy" class="sol-select">
          <option value="downloads">Sort: Downloads</option>
          <option value="relevance">Sort: Relevance</option>
          <option value="updated">Sort: Recently Updated</option>
          <option value="newest">Sort: Newest</option>
        </select>
      </div>

      <div v-if="compatibilityWarning" class="sol-banner sol-banner--warning" style="margin-bottom: 14px">
        <AlertTriangle :size="15" />
        <span>{{ compatibilityWarning }}</span>
      </div>

      <div v-if="errorMsg" class="sol-banner sol-banner--error" style="margin-bottom: 14px">
        <AlertTriangle :size="15" />
        <span>{{ errorMsg }}</span>
      </div>

      <div v-if="loading && hits.length === 0" class="sol-center-state">
        <RefreshCw :size="20" class="is-spinning sol-accent" />
        <span>Searching Modrinth catalog...</span>
      </div>

      <SolaraEmptyState
        v-else-if="!loading && hits.length === 0"
        title="No results found"
        description="Try broadening your search terms or toggling 'Compatible only'."
      >
        <template #icon><Compass :size="26" /></template>
      </SolaraEmptyState>

      <!-- Results Grid -->
      <div v-else class="sol-discover-grid">
        <article
          v-for="hit in hits"
          :key="hit.project_id"
          class="sol-card sol-discover-card"
          @click="openProject(hit)"
        >
          <div class="sol-discover-top">
            <img
              v-if="hit.icon_url"
              :src="hit.icon_url"
              :alt="hit.title"
              class="sol-discover-icon"
              loading="lazy"
            />
            <div v-else class="sol-discover-icon sol-discover-icon--empty">
              {{ hit.title.slice(0, 2).toUpperCase() }}
            </div>
            <div class="sol-discover-meta">
              <h3 class="sol-discover-title sol-truncate">{{ hit.title }}</h3>
              <div class="sol-muted sol-text-xs">
                by {{ hit.author || 'Unknown' }}
                <template v-if="hit.downloads !== null"> • {{ formatDownloadsCount(hit.downloads) }} downloads</template>
              </div>
            </div>
          </div>

          <p class="sol-discover-desc">{{ hit.description }}</p>

          <div class="sol-discover-foot">
            <div class="sol-tag-row">
              <span v-for="cat in hit.categories.slice(0, 3)" :key="cat" class="sol-badge">{{ cat }}</span>
            </div>
            <SolaraButton
              v-if="hit.project_type !== 'modpack'"
              variant="primary"
              size="xs"
              :loading="installingId === hit.project_id"
              :disabled="Boolean(compatibilityWarning)"
              @click.stop="installProject(hit)"
            >
              <Download :size="12" /> Install
            </SolaraButton>
            <SolaraButton v-else variant="secondary" size="xs" @click.stop="openProject(hit)">
              Select Version
            </SolaraButton>
          </div>
        </article>
      </div>

      <!-- Pagination -->
      <div v-if="hits.length > 0" class="sol-pagination">
        <SolaraButton
          variant="secondary"
          size="sm"
          :disabled="pageOffset === 0 || loading"
          @click="
            pageOffset = Math.max(0, pageOffset - 18);
            runSearch(false);
          "
        >
          Previous
        </SolaraButton>
        <span class="sol-muted sol-text-xs">Page {{ Math.floor(pageOffset / 18) + 1 }}</span>
        <SolaraButton
          variant="secondary"
          size="sm"
          :disabled="hits.length < 18 || loading"
          @click="
            pageOffset += 18;
            runSearch(false);
          "
        >
          Next
        </SolaraButton>
      </div>
    </div>

    <!-- Project Detail & Version Picker Modal -->
    <SolaraModal
      :open="Boolean(activeProject)"
      :title="activeProject?.title"
      :subtitle="activeProject ? `by ${activeProject.author || 'Unknown'}` : ''"
      width="lg"
      @close="activeProject = null"
    >
      <template v-if="activeProject">
        <div class="sol-stack">
          <div class="sol-row-gap" style="justify-content: space-between">
            <div class="sol-tag-row">
              <span class="sol-badge">{{ activeProject.project_type }}</span>
              <span v-for="c in activeProject.categories.slice(0, 5)" :key="c" class="sol-badge">{{ c }}</span>
            </div>
            <button
              type="button"
              class="sol-link-btn"
              @click="openExternal(projectUrl(activeProject))"
            >
              View on Modrinth <ArrowUpRight :size="13" />
            </button>
          </div>

          <p class="sol-muted">{{ activeProject.description }}</p>

          <div class="sol-field">
            <label class="sol-label">Choose Version</label>
            <div v-if="loadingDetail" class="sol-muted sol-text-xs">Loading available releases...</div>
            <select v-else v-model="selectedVersionId" class="sol-input">
              <option v-if="versions.length === 0" value="">No compatible versions returned</option>
              <option v-for="v in versions" :key="v.id" :value="v.id">
                {{ v.name }} ({{ v.type }})
                <template v-if="v.games"> — {{ v.games }}</template>
              </option>
            </select>
          </div>

          <div v-if="projectBody" class="sol-project-body sol-scroll selectable">
            {{ projectBody }}
          </div>
        </div>
      </template>

      <template #footer>
        <SolaraButton variant="ghost" @click="activeProject = null">Close</SolaraButton>
        <SolaraButton
          v-if="activeProject"
          variant="primary"
          :loading="installingId === activeProject.project_id"
          :disabled="!selectedVersionId || Boolean(compatibilityWarning)"
          @click="installProject(activeProject, selectedVersionId)"
        >
          <Download :size="14" />
          {{ activeProject.project_type === 'modpack' ? 'Create Instance from Modpack' : 'Install Version' }}
        </SolaraButton>
      </template>
    </SolaraModal>
  </div>
</template>
