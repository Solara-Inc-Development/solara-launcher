<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import SolaraModal from '../components/SolaraModal.vue';
import {
  ArrowUpRight,
  Compass,
  Download,
  RefreshCw,
  Search,
  X,
} from '../icons';
import {
  loadMinecraftVersions,
  native,
  navigate,
  notify,
  openExternal,
  refreshSnapshot,
  selectInstance,
  store,
} from '../state';
import type { LoaderKind } from '../types';

interface ModrinthHit {
  project_id: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  icon_url?: string | null;
  categories: string[];
  versions: string[];
  downloads: number | null;
  follows?: number | null;
  project_type: string;
}

interface ModrinthVersion {
  id: string;
  name: string;
  version_number?: string;
  version_type?: string;
  game_versions?: string[];
  loaders?: string[];
  dependencies?: Array<{ project_id?: string; dependency_type?: string }>;
}

const searchQuery = ref('');
const quickTag = ref<'featured' | 'optimization' | 'adventure' | 'technology' | 'magic'>('featured');
const loaderFilter = ref<'all' | Exclude<LoaderKind, 'vanilla'>>('all');
const versionFilter = ref('');
const sortIndex = ref<'relevance' | 'downloads' | 'updated' | 'newest'>('downloads');
const pageOffset = ref(0);
const hits = ref<ModrinthHit[]>([]);
const totalHits = ref(0);
const loading = ref(false);
const errorMsg = ref<string | null>(null);

// Modpack detail dialog state
const activePack = ref<ModrinthHit | null>(null);
const loadingDetail = ref(false);
const versions = ref<ModrinthVersion[]>([]);
const selectedVersionId = ref('');
const packBody = ref('');
const packGallery = ref<Array<{ url: string; title?: string }>>([]);
const customInstanceName = ref('');
const installing = ref(false);

const releaseVersions = computed(() =>
  store.mcVersions.filter((v) => v.type === 'release').slice(0, 24),
);

const selectedVersionObj = computed(
  () => versions.value.find((v) => v.id === selectedVersionId.value) ?? versions.value[0] ?? null,
);

function formatDownloads(n: number | null | undefined): string {
  if (!n) return '0';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(n);
}

async function runSearch(resetOffset = true): Promise<void> {
  if (resetOffset) pageOffset.value = 0;
  loading.value = true;
  errorMsg.value = null;
  try {
    const facets: string[][] = [['project_type:modpack']];
    if (loaderFilter.value !== 'all') {
      facets.push([`categories:${loaderFilter.value}`]);
    }
    if (quickTag.value !== 'featured') {
      facets.push([`categories:${quickTag.value}`]);
    }
    if (versionFilter.value) {
      facets.push([`versions:${versionFilter.value}`]);
    }

    const params = new URLSearchParams({
      query: searchQuery.value.trim(),
      facets: JSON.stringify(facets),
      index: sortIndex.value,
      limit: '18',
      offset: String(pageOffset.value),
    });
    const res = await fetch(`https://api.modrinth.com/v2/search?${params.toString()}`, {
      headers: { Accept: 'application/json' },
    });
    if (!res.ok) throw new Error(`Modrinth returned HTTP ${res.status}`);
    const data = (await res.json()) as { hits?: ModrinthHit[]; total_hits?: number };
    hits.value = Array.isArray(data.hits) ? data.hits : [];
    totalHits.value = data.total_hits ?? hits.value.length;
  } catch (err) {
    errorMsg.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
}

async function openModpack(hit: ModrinthHit): Promise<void> {
  activePack.value = hit;
  customInstanceName.value = hit.title;
  versions.value = [];
  selectedVersionId.value = '';
  packBody.value = '';
  packGallery.value = [];
  loadingDetail.value = true;
  try {
    const [projRes, verRes] = await Promise.all([
      fetch(`https://api.modrinth.com/v2/project/${encodeURIComponent(hit.project_id)}`),
      fetch(`https://api.modrinth.com/v2/project/${encodeURIComponent(hit.project_id)}/version`),
    ]);
    if (projRes.ok) {
      const proj = (await projRes.json()) as {
        body?: string;
        gallery?: Array<{ url: string; title?: string }>;
      };
      packBody.value = (proj.body || hit.description || '').slice(0, 1600);
      packGallery.value = Array.isArray(proj.gallery) ? proj.gallery.slice(0, 4) : [];
    }
    if (verRes.ok) {
      const list = (await verRes.json()) as ModrinthVersion[];
      versions.value = list;
      selectedVersionId.value = list[0]?.id ?? '';
    }
  } catch (err) {
    notify('warning', 'Could not load modpack details', String(err));
  } finally {
    loadingDetail.value = false;
  }
}

async function installModpack(): Promise<void> {
  if (!activePack.value) return;
  const ver = selectedVersionObj.value;
  if (!ver) {
    notify('warning', 'Select a modpack version', 'Choose a release before installing.');
    return;
  }
  if (!store.desktop) {
    notify('info', 'Desktop required', 'Modpack installation runs in the Solara desktop app.');
    return;
  }
  installing.value = true;
  try {
    const receipt = await native<{ instanceId: string; jobId: string }>('install_content', {
      instanceId: null,
      projectId: activePack.value.project_id,
      contentType: 'modpack',
      versionId: ver.id,
      world: null,
    });
    const desiredName = customInstanceName.value.trim();
    const packTitle = activePack.value.title;
    activePack.value = null;
    await refreshSnapshot();
    const created = store.instances.find((i) => i.id === receipt.instanceId);
    if (created) {
      if (desiredName && desiredName !== created.name) {
        await native('update_instance_meta', { id: created.id, patch: { name: desiredName } }).catch(() => {});
        await refreshSnapshot();
      }
      selectInstance(created.id);
      navigate('instance', { instanceId: created.id, tab: 'content' });
    } else {
      navigate('library');
    }
    notify('info', `Installing ${desiredName || packTitle}`, 'Creating a new instance from modpack...');
  } catch (err) {
    notify('error', 'Modpack install failed', String(err));
  } finally {
    installing.value = false;
  }
}

onMounted(() => {
  void loadMinecraftVersions().catch(() => {});
  void runSearch(true);
});
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-view-wrap">
      <!-- Dedicated Modpacks-Only Discover Toolbar -->
      <header class="sol-discover-head">
        <div>
          <h1 class="sol-page-title">Discover Modpacks</h1>
          <p class="sol-muted sol-text-xs">
            Install complete modpacks as standalone instances. To add mods, resource packs, shaders, or datapacks, open an instance's Content tab.
          </p>
        </div>

        <form class="sol-discover-search" @submit.prevent="runSearch(true)">
          <div class="sol-search-box sol-search-box--grow">
            <Search :size="14" class="sol-muted" />
            <input
              v-model="searchQuery"
              type="text"
              class="sol-search-input"
              placeholder="Search Modpacks (e.g. Fabulously Optimized, Better MC, Cobblemon)..."
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

          <select v-model="versionFilter" class="sol-select" aria-label="Minecraft version" @change="runSearch(true)">
            <option value="">All MC versions</option>
            <option v-for="v in releaseVersions" :key="v.id" :value="v.id">{{ v.id }}</option>
          </select>

          <select v-model="loaderFilter" class="sol-select" aria-label="Mod loader" @change="runSearch(true)">
            <option value="all">All Loaders</option>
            <option value="fabric">Fabric</option>
            <option value="neoforge">NeoForge</option>
            <option value="forge">Forge</option>
            <option value="quilt">Quilt</option>
          </select>

          <select v-model="sortIndex" class="sol-select" aria-label="Sort modpacks" @change="runSearch(true)">
            <option value="downloads">Popular</option>
            <option value="updated">Updated</option>
            <option value="newest">Newest</option>
            <option value="relevance">Relevance</option>
          </select>

          <SolaraButton variant="primary" size="sm" type="submit" :loading="loading">
            Search
          </SolaraButton>
        </form>

        <div class="sol-chip-row">
          <button
            v-for="tag in [
              { id: 'featured', label: 'Popular Packs' },
              { id: 'optimization', label: 'Optimization' },
              { id: 'adventure', label: 'Adventure' },
              { id: 'technology', label: 'Technology' },
              { id: 'magic', label: 'Magic' },
            ] as const"
            :key="tag.id"
            type="button"
            :class="['sol-chip', { 'is-active': quickTag === tag.id }]"
            @click="quickTag = tag.id; runSearch(true)"
          >
            {{ tag.label }}
          </button>
          <span v-if="totalHits > 0" class="sol-muted sol-text-xs" style="margin-left: auto">
            {{ totalHits.toLocaleString() }} modpacks
          </span>
        </div>
      </header>

      <div v-if="errorMsg" class="sol-banner-warn">
        <span>Couldn't load modpacks: {{ errorMsg }}</span>
        <SolaraButton variant="secondary" size="xs" @click="runSearch(false)">
          <RefreshCw :size="12" /> Retry
        </SolaraButton>
      </div>

      <SolaraEmptyState
        v-else-if="!loading && hits.length === 0"
        title="No modpacks matched"
        description="Try clearing the version or category filter."
      >
        <template #icon><Compass :size="28" /></template>
      </SolaraEmptyState>

      <!-- Clean Compact Modpack List/Grid -->
      <div v-else class="sol-pack-grid">
        <article
          v-for="hit in hits"
          :key="hit.project_id"
          class="sol-pack-row"
          @click="openModpack(hit)"
        >
          <img
            v-if="hit.icon_url"
            :src="hit.icon_url"
            :alt="hit.title"
            class="sol-pack-icon"
            loading="lazy"
          />
          <div v-else class="sol-pack-icon sol-pack-icon--empty">{{ hit.title.slice(0, 2).toUpperCase() }}</div>

          <div class="sol-pack-body">
            <div class="sol-pack-top">
              <strong class="sol-pack-title sol-truncate">{{ hit.title }}</strong>
              <span class="sol-muted sol-text-xs">by {{ hit.author || 'Unknown' }}</span>
            </div>
            <p class="sol-pack-desc">{{ hit.description }}</p>
            <div class="sol-pack-foot">
              <div class="sol-tag-row">
                <span class="sol-badge sol-badge--gold">Modpack</span>
                <span v-for="c in hit.categories.slice(0, 2)" :key="c" class="sol-badge">{{ c }}</span>
              </div>
              <span class="sol-muted sol-text-xs">{{ formatDownloads(hit.downloads) }} downloads</span>
            </div>
          </div>

          <div class="sol-pack-cta" @click.stop="openModpack(hit)">
            <SolaraButton variant="primary" size="xs">
              <Download :size="12" /> Install
            </SolaraButton>
          </div>
        </article>
      </div>

      <!-- Pagination -->
      <div v-if="hits.length > 0" class="sol-pagination">
        <SolaraButton
          variant="secondary"
          size="xs"
          :disabled="pageOffset === 0 || loading"
          @click="pageOffset = Math.max(0, pageOffset - 18); runSearch(false)"
        >
          Previous
        </SolaraButton>
        <span class="sol-muted sol-text-xs">Page {{ Math.floor(pageOffset / 18) + 1 }}</span>
        <SolaraButton
          variant="secondary"
          size="xs"
          :disabled="hits.length < 18 || loading"
          @click="pageOffset += 18; runSearch(false)"
        >
          Next
        </SolaraButton>
      </div>
    </div>

    <!-- Rich Modpack Detail & Create-Instance Modal -->
    <SolaraModal
      :open="Boolean(activePack)"
      :title="activePack?.title"
      :subtitle="activePack ? `Modpack by ${activePack.author || 'Unknown'}` : ''"
      width="lg"
      @close="activePack = null"
    >
      <template v-if="activePack">
        <div class="sol-stack">
          <div class="sol-row-between">
            <div class="sol-tag-row">
              <span class="sol-badge sol-badge--gold">Modpack</span>
              <span v-for="c in activePack.categories.slice(0, 5)" :key="c" class="sol-badge">{{ c }}</span>
            </div>
            <button
              type="button"
              class="sol-link-btn"
              @click="openExternal(`https://modrinth.com/modpack/${activePack.slug || activePack.project_id}`)"
            >
              View on Modrinth <ArrowUpRight :size="13" />
            </button>
          </div>

          <div v-if="packGallery.length > 0" class="sol-gallery-strip">
            <img
              v-for="img in packGallery"
              :key="img.url"
              :src="img.url"
              :alt="img.title || activePack.title"
              class="sol-gallery-thumb"
              loading="lazy"
            />
          </div>

          <div class="sol-split-2">
            <div class="sol-field">
              <label class="sol-label">New Instance Name</label>
              <input v-model="customInstanceName" type="text" class="sol-input" maxlength="80" />
            </div>

            <div class="sol-field">
              <label class="sol-label">Modpack Release Version</label>
              <div v-if="loadingDetail" class="sol-muted sol-text-xs">Loading releases...</div>
              <select v-else v-model="selectedVersionId" class="sol-input">
                <option v-if="versions.length === 0" value="">No versions returned</option>
                <option v-for="v in versions" :key="v.id" :value="v.id">
                  {{ v.name }} ({{ (v.loaders || []).join(', ') || 'modpack' }} · {{ (v.game_versions || []).slice(0, 2).join(', ') }})
                </option>
              </select>
            </div>
          </div>

          <div v-if="selectedVersionObj?.dependencies?.length" class="sol-muted sol-text-xs">
            Bundled dependencies: {{ selectedVersionObj.dependencies.length }} referenced modules/packs.
          </div>

          <div v-if="packBody" class="sol-project-body sol-scroll selectable">
            {{ packBody }}
          </div>
        </div>
      </template>

      <template #footer>
        <SolaraButton variant="ghost" @click="activePack = null">Cancel</SolaraButton>
        <SolaraButton
          v-if="activePack"
          variant="primary"
          :loading="installing"
          :disabled="!selectedVersionObj"
          @click="installModpack"
        >
          <Download :size="14" /> Create Instance from Modpack
        </SolaraButton>
      </template>
    </SolaraModal>
  </div>
</template>
