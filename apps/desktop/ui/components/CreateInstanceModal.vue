<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Upload, Shuffle, ChevronDown, Search, X } from '../icons';
import {
  store,
  BUILTIN_ICON_PRESETS,
  loadMinecraftVersions,
  refreshSnapshot,
  selectInstance,
  navigate,
  native,
  notify,
  LOADER_LABEL,
} from '../state';
import type { DetectedJava, LoaderKind } from '../types';
import SolaraModal from '../components/SolaraModal.vue';
import SolaraButton from '../components/SolaraButton.vue';
import InstanceIcon from '../components/InstanceIcon.vue';

interface LoaderVersionItem {
  version: string;
  stable?: boolean;
}

const name = ref('');
const loader = ref<LoaderKind>('fabric');
const gameVersion = ref('');
const versionFilter = ref<'release' | 'snapshot' | 'old'>('release');
const versionSearch = ref('');
const loaderVersions = ref<LoaderVersionItem[]>([]);
const loaderMode = ref<'recommended' | 'latest' | 'custom'>('recommended');
const selectedLoaderVersion = ref('');
const loadingLoaderVersions = ref(false);
const loaderVersionError = ref('');

const iconSelection = ref<string>('preset:dawn-crest');
const customIconPreview = ref<string | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);

const showAdvanced = ref(false);
const group = ref('');
const memory = ref(4096);
const javaPath = ref('auto');
const width = ref(854);
const height = ref(480);
const fullscreen = ref(false);
const updateChannel = ref<'release' | 'beta' | 'alpha'>('release');
const detectedJava = ref<DetectedJava[]>([]);

const creating = ref(false);
const errorMsg = ref('');

watch(
  () => store.showCreateModal,
  async (open) => {
    if (!open) return;
    errorMsg.value = '';
    name.value = '';
    gameVersion.value = '';
    showAdvanced.value = false;
    updateChannel.value = 'release';
    loader.value = store.createModalDefaults.loader || 'fabric';
    group.value = store.createModalDefaults.group || '';
    memory.value = store.preferences.memory || 4096;
    javaPath.value = store.preferences.javaPath || 'auto';
    width.value = store.preferences.defaultWidth || 854;
    height.value = store.preferences.defaultHeight || 480;
    fullscreen.value = Boolean(store.preferences.fullscreen);
    iconSelection.value = 'preset:dawn-crest';
    customIconPreview.value = null;
    versionSearch.value = '';
    versionFilter.value = 'release';
    try {
      const list = await loadMinecraftVersions();
      const firstRelease = list.find((v) => v.type === 'release') ?? list[0];
      if (firstRelease) gameVersion.value = firstRelease.id;
    } catch (err) {
      errorMsg.value = `Couldn't load official Minecraft versions: ${String(err)}`;
    }
    if (store.desktop) {
      native<DetectedJava[]>('detect_java_runtimes')
        .then((j) => (detectedJava.value = j))
        .catch(() => {});
    }
  },
);

const filteredVersions = computed(() => {
  const q = versionSearch.value.trim().toLowerCase();
  return store.mcVersions
    .filter((v) => {
      if (versionFilter.value === 'release' && v.type !== 'release') return false;
      if (versionFilter.value === 'snapshot' && v.type !== 'snapshot') return false;
      if (
        versionFilter.value === 'old' &&
        v.type !== 'old_beta' &&
        v.type !== 'old_alpha'
      ) {
        return false;
      }
      if (q && !v.id.toLowerCase().includes(q)) return false;
      return true;
    })
    .slice(0, 120);
});

watch([gameVersion, loader], async ([gv, ldr], _prior, onCleanup) => {
  let active = true;
  onCleanup(() => { active = false; });
  loadingLoaderVersions.value = false;
  loaderVersionError.value = '';
  loaderVersions.value = [];
  selectedLoaderVersion.value = '';
  loaderMode.value = 'recommended';
  if (!gv || ldr === 'vanilla' || !store.desktop) return;
  loadingLoaderVersions.value = true;
  try {
    const raw = await native<LoaderVersionItem[] | string[]>('loader_versions', {
      gameVersion: gv,
      loader: ldr,
    });
    const list: LoaderVersionItem[] = Array.isArray(raw)
      ? raw.map((item) => (typeof item === 'string' ? { version: item, stable: true } : item))
      : [];
    if (!active) return;
    loaderVersions.value = list;
    const stable = list.find((x) => x.stable) ?? list[0];
    selectedLoaderVersion.value = stable?.version ?? '';
  } catch (err) {
    if (!active) return;
    loaderVersionError.value = `No ${LOADER_LABEL[ldr]} build found for ${gv}: ${String(err)}`;
  } finally {
    if (active) loadingLoaderVersions.value = false;
  }
});

const effectiveLoaderVersion = computed(() => {
  if (loader.value === 'vanilla') return '';
  if (loaderMode.value === 'latest') return loaderVersions.value[0]?.version ?? '';
  if (loaderMode.value === 'recommended') {
    const stable = loaderVersions.value.find((v) => v.stable) ?? loaderVersions.value[0];
    return stable?.version ?? '';
  }
  return selectedLoaderVersion.value;
});

const resolvedName = computed(() => {
  const trimmed = name.value.trim();
  if (trimmed) return trimmed;
  if (!gameVersion.value) return '';
  return loader.value === 'vanilla'
    ? `Minecraft ${gameVersion.value}`
    : `${LOADER_LABEL[loader.value]} ${gameVersion.value}`;
});

function randomizeIcon() {
  customIconPreview.value = null;
  const idx = Math.floor(Math.random() * BUILTIN_ICON_PRESETS.length);
  iconSelection.value = BUILTIN_ICON_PRESETS[idx];
}

function handleUploadClick() {
  fileInput.value?.click();
}

function onFilePicked(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = '';
  if (!file) return;
  if (file.size > 2 * 1024 * 1024) {
    errorMsg.value = 'Custom icon image must be smaller than 2 MB.';
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    const res = typeof reader.result === 'string' ? reader.result : '';
    if (res.startsWith('data:image/')) {
      customIconPreview.value = res;
      iconSelection.value = res;
    }
  };
  reader.readAsDataURL(file);
}

async function handleCreate() {
  if (creating.value || loadingLoaderVersions.value) return;
  errorMsg.value = '';
  const finalName = resolvedName.value;
  if (!finalName) {
    errorMsg.value = 'Enter an instance name.';
    return;
  }
  if (!gameVersion.value) {
    errorMsg.value = 'Select a Minecraft version.';
    return;
  }
  creating.value = true;
  try {
    const res = await native<{ jobId: string; instanceId: string }>('create_instance', {
      name: finalName,
      gameVersion: gameVersion.value,
      loader: loader.value,
      loaderVersion: effectiveLoaderVersion.value,
      options: {
        icon: customIconPreview.value ? customIconPreview.value : iconSelection.value,
        group: group.value.trim() || null,
        memory: memory.value,
        javaPath: javaPath.value,
        resolution: [width.value, height.value],
        fullscreen: fullscreen.value,
        updateChannel: updateChannel.value,
      },
    });
    store.showCreateModal = false;
    await refreshSnapshot();
    selectInstance(res.instanceId);
    navigate('instance', { instanceId: res.instanceId, tab: 'content' });
    notify('info', `Creating ${finalName}`, 'Installing Minecraft & loader files...');
  } catch (err) {
    errorMsg.value = String(err);
  } finally {
    creating.value = false;
  }
}
</script>

<template>
  <SolaraModal
    :open="store.showCreateModal"
    title="Create Instance"
    subtitle="Configure an isolated Minecraft installation"
    width="lg"
    :closable="!creating"
    @close="store.showCreateModal = false"
  >
    <div class="sol-create-layout">
      <!-- Icon + Identity -->
      <div class="sol-create-hero">
        <div class="sol-create-icon-panel">
          <InstanceIcon
            :name="resolvedName || 'New'"
            :loader="loader"
            :icon="customIconPreview ? 'custom' : iconSelection"
            :custom-icon="customIconPreview"
            :size="72"
            :rounded="16"
          />
          <div class="sol-create-icon-btns">
            <input
              ref="fileInput"
              type="file"
              accept="image/png,image/jpeg,image/webp"
              hidden
              @change="onFilePicked"
            />
            <SolaraButton variant="secondary" size="xs" @click="handleUploadClick">
              <Upload :size="12" /> Upload
            </SolaraButton>
            <SolaraButton variant="ghost" size="xs" @click="randomizeIcon">
              <Shuffle :size="12" /> Randomize
            </SolaraButton>
          </div>
          <div class="sol-preset-strip">
            <button
              v-for="preset in BUILTIN_ICON_PRESETS"
              :key="preset"
              type="button"
              :class="['sol-preset-btn', { 'is-active': !customIconPreview && iconSelection === preset }]"
              :title="preset.replace('preset:', '')"
              @click="
                customIconPreview = null;
                iconSelection = preset;
              "
            >
              <InstanceIcon :icon="preset" :size="24" :rounded="6" />
            </button>
          </div>
        </div>

        <div class="sol-create-identity">
          <div class="sol-field">
            <label class="sol-label">Instance Name</label>
            <input
              v-model="name"
              type="text"
              class="sol-input"
              :placeholder="resolvedName || 'e.g. Survival 1.21.4'"
              maxlength="80"
            />
          </div>

          <div class="sol-field">
            <label class="sol-label">Mod Loader</label>
            <div class="sol-chip-row">
              <button
                v-for="ldr in (['vanilla', 'fabric', 'neoforge', 'forge', 'quilt'] as LoaderKind[])"
                :key="ldr"
                type="button"
                :class="['sol-chip', { 'is-active': loader === ldr }]"
                @click="loader = ldr"
              >
                {{ LOADER_LABEL[ldr] }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Minecraft Version Picker -->
      <div class="sol-field">
        <div class="sol-field-head">
          <label class="sol-label">Game Version</label>
          <div class="sol-row-gap">
            <div class="sol-seg-group">
              <button
                v-for="cat in [
                  { id: 'release', label: 'Releases' },
                  { id: 'snapshot', label: 'Snapshots' },
                  { id: 'old', label: 'Older' },
                ] as const"
                :key="cat.id"
                type="button"
                :class="['sol-seg-btn', { 'is-active': versionFilter === cat.id }]"
                @click="versionFilter = cat.id"
              >
                {{ cat.label }}
              </button>
            </div>
            <div class="sol-search-box sol-search-box--sm">
              <Search :size="13" class="sol-muted" />
              <input v-model="versionSearch" type="text" class="sol-search-input" placeholder="Filter..." />
              <button v-if="versionSearch" type="button" class="sol-icon-btn sol-icon-btn--xs" @click="versionSearch = ''">
                <X :size="11" />
              </button>
            </div>
          </div>
        </div>

        <div class="sol-version-list sol-scroll">
          <button
            v-for="ver in filteredVersions"
            :key="ver.id"
            type="button"
            :class="['sol-version-row', { 'is-active': gameVersion === ver.id }]"
            @click="gameVersion = ver.id"
          >
            <span class="sol-version-id">{{ ver.id }}</span>
            <span class="sol-badge">{{ ver.type }}</span>
          </button>
          <div v-if="filteredVersions.length === 0" class="sol-muted sol-text-xs" style="padding: 12px">
            No matching Minecraft versions found.
          </div>
        </div>
      </div>

      <!-- Loader Version Picker -->
      <div v-if="loader !== 'vanilla'" class="sol-field">
        <label class="sol-label">{{ LOADER_LABEL[loader] }} Version</label>
        <div class="sol-split-2">
          <div class="sol-chip-row">
            <button
              v-for="mode in (['recommended', 'latest', 'custom'] as const)"
              :key="mode"
              type="button"
              :class="['sol-chip', { 'is-active': loaderMode === mode }]"
              @click="loaderMode = mode"
            >
              {{ mode === 'recommended' ? 'Recommended' : mode === 'latest' ? 'Latest' : 'Select version' }}
            </button>
          </div>
          <select
            v-model="selectedLoaderVersion"
            class="sol-input"
            :disabled="loaderMode !== 'custom' || loadingLoaderVersions || loaderVersions.length === 0"
          >
            <option v-if="loadingLoaderVersions" value="">Loading builds...</option>
            <option v-else-if="loaderVersions.length === 0" value="">Unavailable for {{ gameVersion }}</option>
            <option v-for="lv in loaderVersions" :key="lv.version" :value="lv.version">
              {{ lv.version }}{{ lv.stable ? ' (Recommended)' : '' }}
            </option>
          </select>
        </div>
        <p v-if="loaderVersionError" class="sol-inline-warn">{{ loaderVersionError }}</p>
      </div>

      <!-- Expandable Advanced Settings -->
      <div class="sol-advanced-box">
        <button type="button" class="sol-advanced-toggle" @click="showAdvanced = !showAdvanced">
          <span>Advanced Settings (Group, Memory, Java, Window)</span>
          <ChevronDown :size="15" :class="{ 'is-rotated': showAdvanced }" />
        </button>

        <div v-if="showAdvanced" class="sol-advanced-body">
          <div class="sol-split-2">
            <div class="sol-field">
              <label class="sol-label">Group (Optional)</label>
              <input
                v-model="group"
                type="text"
                class="sol-input"
                list="solara-create-groups"
                placeholder="e.g. Modded"
                maxlength="60"
              />
              <datalist id="solara-create-groups">
                <option v-for="g in store.preferences.groups" :key="g" :value="g" />
              </datalist>
            </div>

            <div class="sol-field">
              <label class="sol-label">Content Update Channel</label>
              <select v-model="updateChannel" class="sol-input">
                <option value="release">Release only</option>
                <option value="beta">Beta & Release</option>
                <option value="alpha">All (Alpha / Beta / Release)</option>
              </select>
            </div>
          </div>

          <div class="sol-split-2">
            <div class="sol-field">
              <label class="sol-label">Memory Allocation ({{ memory }} MB)</label>
              <input v-model.number="memory" type="range" min="512" max="16384" step="512" class="sol-range" />
            </div>

            <div class="sol-field">
              <label class="sol-label">Java Runtime</label>
              <select v-model="javaPath" class="sol-input">
                <option value="auto">Automatic (Solara Managed)</option>
                <option v-for="j in detectedJava" :key="j.path" :value="j.path">
                  Java {{ j.major ?? '?' }} — {{ j.path }}
                </option>
              </select>
            </div>
          </div>

          <div class="sol-split-3">
            <div class="sol-field">
              <label class="sol-label">Window Width</label>
              <input v-model.number="width" type="number" min="320" max="7680" class="sol-input" />
            </div>
            <div class="sol-field">
              <label class="sol-label">Window Height</label>
              <input v-model.number="height" type="number" min="240" max="4320" class="sol-input" />
            </div>
            <div class="sol-field sol-field--center">
              <label class="sol-checkbox-label">
                <input v-model="fullscreen" type="checkbox" />
                <span>Start Fullscreen</span>
              </label>
            </div>
          </div>
        </div>
      </div>

      <p v-if="errorMsg" class="sol-inline-error">{{ errorMsg }}</p>
    </div>

    <template #footer>
      <SolaraButton variant="ghost" :disabled="creating" @click="store.showCreateModal = false">
        Cancel
      </SolaraButton>
      <SolaraButton
        variant="primary"
        :loading="creating"
        :disabled="!gameVersion || loadingLoaderVersions || (loader !== 'vanilla' && !effectiveLoaderVersion)"
        @click="handleCreate"
      >
        Create Instance
      </SolaraButton>
    </template>
  </SolaraModal>
</template>
