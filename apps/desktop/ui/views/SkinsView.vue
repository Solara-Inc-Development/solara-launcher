<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import {
  Upload,
  RefreshCw,
  Trash2,
  Check,
  Eye,
  EyeOff,
  Plus,
  Edit3,
  Palette,
  Sparkles,
} from '../icons';
import { store, selectedAccount, native, notify, registerUnsavedChanges, runWithDirtyGuard } from '../state';
import type { ProfileCape, ProfileSkin, SavedSkin } from '../types';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraTabs from '../components/SolaraTabs.vue';
import SkinViewer3D from '../components/SkinViewer3D.vue';
import SkinEditor from '../components/SkinEditor.vue';

interface StarterSkinSpec {
  id: string;
  name: string;
  variant: 'classic' | 'slim';
  skinTone: string;
  hair: string;
  shirt: string;
  shirtTrim: string;
  pants: string;
  eyes: string;
}

interface PreviewCapeSpec {
  id: string;
  name: string;
  base: string;
  accent: string;
  trim: string;
}

const STARTER_SKIN_SPECS: StarterSkinSpec[] = [
  {
    id: 'starter-dawn',
    name: 'Dawn Explorer',
    variant: 'classic',
    skinTone: '#e8c3a5',
    hair: '#221c18',
    shirt: '#17171c',
    shirtTrim: '#e9b86d',
    pants: '#282a34',
    eyes: '#e9b86d',
  },
  {
    id: 'starter-steve',
    name: 'Classic Steve',
    variant: 'classic',
    skinTone: '#d9a47d',
    hair: '#3e2723',
    shirt: '#2a7b8c',
    shirtTrim: '#1f5e6b',
    pants: '#34497a',
    eyes: '#38579a',
  },
  {
    id: 'starter-alex',
    name: 'Slim Alex',
    variant: 'slim',
    skinTone: '#f0cbb0',
    hair: '#b95f2c',
    shirt: '#487a3d',
    shirtTrim: '#d9b56c',
    pants: '#4e3c2c',
    eyes: '#3b7346',
  },
];

const PREVIEW_CAPE_SPECS: PreviewCapeSpec[] = [
  {
    id: 'preview-dawn-banner',
    name: 'Dawn Banner',
    base: '#121216',
    accent: '#e9b86d',
    trim: '#f5cd91',
  },
  {
    id: 'preview-obsidian-crest',
    name: 'Obsidian Crest',
    base: '#09090c',
    accent: '#c89543',
    trim: '#e9b86d',
  },
  {
    id: 'preview-solstice-wing',
    name: 'Solstice Wing',
    base: '#191512',
    accent: '#f5cd91',
    trim: '#8f6428',
  },
];

function buildStarterSkinPng(spec: StarterSkinSpec): string {
  const canvas = document.createElement('canvas');
  canvas.width = 64;
  canvas.height = 64;
  const ctx = canvas.getContext('2d');
  if (!ctx) return '';

  const fill = (x: number, y: number, w: number, h: number, color: string) => {
    ctx.fillStyle = color;
    ctx.fillRect(x, y, w, h);
  };

  // Head base (8x8x8 around (0,0)-(32,16))
  fill(8, 8, 8, 8, spec.skinTone);
  fill(0, 8, 8, 8, spec.skinTone);
  fill(16, 8, 8, 8, spec.skinTone);
  fill(24, 8, 8, 8, spec.hair);
  fill(8, 0, 8, 8, spec.hair);
  fill(16, 0, 8, 8, spec.skinTone);
  // Hair fringe + eyes
  fill(8, 8, 8, 2, spec.hair);
  fill(9, 10, 1, 1, spec.hair);
  fill(14, 10, 1, 1, spec.hair);
  fill(9, 12, 1, 1, '#ffffff');
  fill(10, 12, 1, 1, spec.eyes);
  fill(13, 12, 1, 1, spec.eyes);
  fill(14, 12, 1, 1, '#ffffff');
  fill(11, 14, 2, 1, '#a97155');

  // Torso (16,16)-(40,32)
  fill(20, 20, 8, 12, spec.shirt);
  fill(16, 20, 4, 12, spec.shirt);
  fill(28, 20, 4, 12, spec.shirt);
  fill(32, 20, 8, 12, spec.shirt);
  fill(20, 16, 8, 4, spec.shirt);
  fill(28, 16, 8, 4, spec.pants);
  // Dawn stripe emblem on chest
  fill(22, 22, 4, 4, spec.shirtTrim);
  fill(23, 23, 2, 2, '#0b0b0d');
  fill(20, 31, 8, 1, spec.shirtTrim);

  // Arms (right 40,16 + left 32,48)
  const armW = spec.variant === 'slim' ? 3 : 4;
  fill(44, 20, armW, 8, spec.shirt);
  fill(40, 20, 4, 8, spec.shirt);
  fill(44, 28, armW, 4, spec.skinTone);
  fill(40, 28, 4, 4, spec.skinTone);
  fill(36, 52, armW, 8, spec.shirt);
  fill(32, 52, 4, 8, spec.shirt);
  fill(36, 60, armW, 4, spec.skinTone);
  fill(32, 60, 4, 4, spec.skinTone);

  // Legs (right 0,16 + left 16,48)
  fill(4, 20, 4, 10, spec.pants);
  fill(0, 20, 4, 10, spec.pants);
  fill(8, 20, 4, 10, spec.pants);
  fill(12, 20, 4, 10, spec.pants);
  fill(0, 30, 16, 2, '#151518');

  fill(20, 52, 4, 10, spec.pants);
  fill(16, 52, 4, 10, spec.pants);
  fill(24, 52, 4, 10, spec.pants);
  fill(28, 52, 4, 10, spec.pants);
  fill(16, 62, 16, 2, '#151518');

  return canvas.toDataURL('image/png');
}

function buildPreviewCapePng(spec: PreviewCapeSpec): { dataUrl: string; thumbUrl: string } {
  const canvas = document.createElement('canvas');
  canvas.width = 64;
  canvas.height = 32;
  const ctx = canvas.getContext('2d');
  if (!ctx) return { dataUrl: '', thumbUrl: '' };

  // Front face of cape is (1,1) width 10, height 16; back face is (12,1) 10x16
  ctx.fillStyle = spec.base;
  ctx.fillRect(0, 0, 22, 17);

  // Border + sun crest on front cape face
  ctx.fillStyle = spec.accent;
  ctx.fillRect(1, 1, 10, 1);
  ctx.fillRect(1, 16, 10, 1);
  ctx.fillRect(1, 1, 1, 16);
  ctx.fillRect(10, 1, 1, 16);

  ctx.fillStyle = spec.trim;
  ctx.fillRect(5, 5, 2, 2);
  ctx.fillStyle = spec.accent;
  ctx.fillRect(4, 7, 4, 4);
  ctx.fillRect(5, 11, 2, 2);

  // Also generate a crisp cropped 10x16 card thumbnail
  const thumb = document.createElement('canvas');
  thumb.width = 10;
  thumb.height = 16;
  const tctx = thumb.getContext('2d');
  if (tctx) {
    tctx.drawImage(canvas, 1, 1, 10, 16, 0, 0, 10, 16);
  }
  return { dataUrl: canvas.toDataURL('image/png'), thumbUrl: thumb.toDataURL('image/png') };
}

const activeTab = ref<'library' | 'editor' | 'capes'>('library');
const savedSkins = ref<SavedSkin[]>([]);
const profileCapes = ref<ProfileCape[]>([]);
const profileSkins = ref<ProfileSkin[]>([]);
const activeSkinDataUrl = ref<string>('');
const activeCapeDataUrl = ref<string | null>(null);
const previewCapeId = ref<string | null>(null);
const activeVariant = ref<'classic' | 'slim'>('classic');
const showOuter = ref(true);
const animateModel = ref(true);

const editorName = ref('Custom Solara Skin');
const editorSkinId = ref<string | null>(null);
const editorDirty = ref(false);
let originalSkin = '';
let originalSkinName = '';
let originalVariant: 'classic' | 'slim' = 'classic';
function rememberOriginalSkin(): void {
  originalSkin = activeSkinDataUrl.value; originalSkinName = editorName.value; originalVariant = activeVariant.value;
}
const unregisterSkinGuard = registerUnsavedChanges(
  () => editorDirty.value,
  () => { activeSkinDataUrl.value = originalSkin; editorName.value = originalSkinName; activeVariant.value = originalVariant; editorDirty.value = false; },
);
function changeSkinTab(tab: string): void {
  runWithDirtyGuard(() => { activeTab.value = tab as 'library' | 'editor' | 'capes'; });
}
const saving = ref(false);
const uploading = ref(false);
const loadingProfile = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);

const starterSkins = ref<Array<StarterSkinSpec & { pngDataUrl: string }>>([]);
const previewCapes = ref<Array<PreviewCapeSpec & { dataUrl: string; thumbUrl: string }>>([]);

const tabs = [
  { id: 'library', label: 'Saved Skins' },
  { id: 'editor', label: 'Skin Editor' },
  { id: 'capes', label: 'Capes' },
];

const equippedCape = computed(
  () => profileCapes.value.find((c) => c.state === 'ACTIVE') ?? null,
);

const isPreviewingLocalCape = computed(() => previewCapeId.value !== null);

async function loadAll() {
  if (!store.desktop) return;
  try {
    savedSkins.value = await native<SavedSkin[]>('saved_skins_list');
  } catch {
    savedSkins.value = [];
  }
  if (selectedAccount.value) {
    loadingProfile.value = true;
    try {
      const prof = await native<{
        skins?: ProfileSkin[];
        capes?: ProfileCape[];
      }>('profile_skins_and_capes');
      profileSkins.value = prof.skins ?? [];
      profileCapes.value = prof.capes ?? [];
      const activeProfSkin = profileSkins.value.find((s) => s.state === 'ACTIVE');
      if (activeProfSkin?.dataUrl && !activeSkinDataUrl.value) {
        activeSkinDataUrl.value = activeProfSkin.dataUrl;
        activeVariant.value = activeProfSkin.variant === 'SLIM' ? 'slim' : 'classic';
      }
      if (!previewCapeId.value) {
        const activeCap = profileCapes.value.find((c) => c.state === 'ACTIVE');
        activeCapeDataUrl.value = activeCap?.dataUrl ?? null;
      }
    } catch {
      // keep local skins functional even if profile API fails
    } finally {
      loadingProfile.value = false;
    }
  }
  if (!activeSkinDataUrl.value) {
    if (savedSkins.value.length > 0) {
      activeSkinDataUrl.value = savedSkins.value[0].pngDataUrl;
      activeVariant.value = savedSkins.value[0].variant;
      editorName.value = savedSkins.value[0].name;
      editorSkinId.value = savedSkins.value[0].id;
    } else if (starterSkins.value.length > 0) {
      activeSkinDataUrl.value = starterSkins.value[0].pngDataUrl;
      activeVariant.value = starterSkins.value[0].variant;
      editorName.value = starterSkins.value[0].name;
      editorSkinId.value = null;
    }
  }
}

onMounted(() => {
  starterSkins.value = STARTER_SKIN_SPECS.map((s) => ({
    ...s,
    pngDataUrl: buildStarterSkinPng(s),
  }));
  previewCapes.value = PREVIEW_CAPE_SPECS.map((c) => {
    const built = buildPreviewCapePng(c);
    return {
      ...c,
      dataUrl: built.dataUrl,
      thumbUrl: built.thumbUrl,
    };
  });
  void loadAll().then(rememberOriginalSkin);
});

onUnmounted(() => {
  unregisterSkinGuard();
});

function selectSavedSkin(skin: SavedSkin) {
  activeSkinDataUrl.value = skin.pngDataUrl;
  activeVariant.value = skin.variant;
  editorName.value = skin.name;
  editorSkinId.value = skin.id;
  editorDirty.value = false;
  rememberOriginalSkin();
}

function selectStarterSkin(skin: StarterSkinSpec & { pngDataUrl: string }) {
  activeSkinDataUrl.value = skin.pngDataUrl;
  activeVariant.value = skin.variant;
  editorName.value = skin.name;
  editorSkinId.value = null;
  editorDirty.value = false;
  rememberOriginalSkin();
}

async function saveStarterToLibrary(skin: StarterSkinSpec & { pngDataUrl: string }) {
  try {
    const saved = await native<SavedSkin>('saved_skins_save', {
      id: null,
      name: skin.name,
      variant: skin.variant,
      pngDataUrl: skin.pngDataUrl,
    });
    await loadAll();
    selectSavedSkin(saved);
    notify('success', `${skin.name} added to Saved Skins`);
  } catch (err) {
    notify('error', "Couldn't save starter skin", String(err));
  }
}

async function onUploadPng(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  (e.target as HTMLInputElement).value = '';
  if (!file) return;
  const r = new FileReader();
  r.onload = async () => {
    const dataUrl = typeof r.result === 'string' ? r.result : '';
    if (!dataUrl.startsWith('data:image/png')) {
      notify('error', 'Invalid file', 'Minecraft skins must be a 64×64 or 64×32 PNG.');
      return;
    }
    try {
      const saved = await native<SavedSkin>('saved_skins_save', {
        id: null,
        name: file.name.replace(/\.png$/i, '') || 'Imported Skin',
        variant: activeVariant.value,
        pngDataUrl: dataUrl,
      });
      await loadAll();
      selectSavedSkin(saved);
      notify('success', 'Skin imported to library');
    } catch (err) {
      notify('error', "Couldn't import skin", String(err));
    }
  };
  r.readAsDataURL(file);
}

async function saveEditorSkin() {
  if (!activeSkinDataUrl.value) return;
  saving.value = true;
  try {
    const saved = await native<SavedSkin>('saved_skins_save', {
      id: editorSkinId.value,
      name: editorName.value.trim() || 'Custom Skin',
      variant: activeVariant.value,
      pngDataUrl: activeSkinDataUrl.value,
    });
    editorSkinId.value = saved.id;
    editorDirty.value = false;
    rememberOriginalSkin();
    await loadAll();
    notify('success', 'Skin saved locally');
  } catch (err) {
    notify('error', "Couldn't save skin", String(err));
  } finally {
    saving.value = false;
  }
}

async function deleteSkin(skin: SavedSkin) {
  try {
    await native('saved_skins_delete', { id: skin.id });
    if (editorSkinId.value === skin.id) editorSkinId.value = null;
    await loadAll();
    notify('info', 'Skin deleted from local library');
  } catch (err) {
    notify('error', "Couldn't delete skin", String(err));
  }
}

async function applySkinToAccount() {
  if (!selectedAccount.value) {
    notify('warning', 'Sign in required', 'Sign in with Microsoft first to apply skins to your account.');
    return;
  }
  if (!activeSkinDataUrl.value) return;
  uploading.value = true;
  try {
    await native('upload_minecraft_skin', {
      variant: activeVariant.value,
      pngDataUrl: activeSkinDataUrl.value,
    });
    notify('success', 'Skin applied to Minecraft account', `Updated active skin for ${selectedAccount.value.username}.`);
    await loadAll();
  } catch (err) {
    notify('error', "Couldn't apply skin", String(err));
  } finally {
    uploading.value = false;
  }
}

async function equipCape(cape: ProfileCape | null) {
  if (!selectedAccount.value) return;
  previewCapeId.value = null;
  try {
    await native('set_minecraft_cape', { capeId: cape?.id ?? null });
    activeCapeDataUrl.value = cape?.dataUrl ?? null;
    await loadAll();
    notify('success', cape ? `Equipped ${cape.alias} cape` : 'Cape hidden');
  } catch (err) {
    notify('error', "Couldn't update cape", String(err));
  }
}

function previewLocalCape(cape: PreviewCapeSpec & { dataUrl: string }) {
  previewCapeId.value = cape.id;
  activeCapeDataUrl.value = cape.dataUrl;
}

function clearLocalCapePreview() {
  previewCapeId.value = null;
  activeCapeDataUrl.value = equippedCape.value?.dataUrl ?? null;
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-page-inner">
      <header class="sol-page-header">
        <div>
          <h1 class="sol-page-title">Skin Studio</h1>
          <p class="sol-page-sub">Inspect in 3D, edit pixels, manage saved skins, and equip owned capes</p>
        </div>

        <div class="sol-row-gap">
          <input ref="fileInput" type="file" accept="image/png" hidden @change="onUploadPng" />
          <SolaraButton variant="secondary" size="sm" @click="fileInput?.click()">
            <Upload :size="14" /> Import PNG
          </SolaraButton>
          <SolaraButton
            variant="primary"
            size="sm"
            :loading="uploading"
            :disabled="!activeSkinDataUrl || !selectedAccount"
            @click="applySkinToAccount"
          >
            <Check :size="14" /> Apply to Account
          </SolaraButton>
        </div>
      </header>

      <div class="sol-skins-layout">
        <!-- Left Sticky 3D Character Preview -->
        <aside class="sol-card sol-skin-stage">
          <SkinViewer3D
            :skin-data-url="activeSkinDataUrl"
            :cape-data-url="activeCapeDataUrl"
            :variant="activeVariant"
            :show-outer="showOuter"
            :animate="animateModel && !store.preferences.reducedMotion"
            :height="320"
          />

          <div v-if="isPreviewingLocalCape" class="sol-cape-preview-note">
            <span>Previewing local cape (not owned on account)</span>
            <button type="button" class="sol-link-btn" @click="clearLocalCapePreview">Clear</button>
          </div>

          <div class="sol-stage-controls">
            <div class="sol-seg-group" style="width: 100%">
              <button
                type="button"
                :class="['sol-seg-btn', { 'is-active': activeVariant === 'classic' }]"
                style="flex: 1"
                @click="activeVariant = 'classic'"
              >
                Classic (Steve)
              </button>
              <button
                type="button"
                :class="['sol-seg-btn', { 'is-active': activeVariant === 'slim' }]"
                style="flex: 1"
                @click="activeVariant = 'slim'"
              >
                Slim (Alex)
              </button>
            </div>

            <div class="sol-row-gap" style="justify-content: space-between; width: 100%">
              <SolaraButton variant="ghost" size="xs" @click="showOuter = !showOuter">
                <Eye v-if="showOuter" :size="13" />
                <EyeOff v-else :size="13" />
                Outer Layer
              </SolaraButton>
              <SolaraButton variant="ghost" size="xs" @click="animateModel = !animateModel">
                {{ animateModel ? 'Pause Walk' : 'Walk Pose' }}
              </SolaraButton>
            </div>

            <SolaraButton
              variant="primary"
              size="sm"
              style="width: 100%"
              @click="activeTab = 'editor'"
            >
              <Edit3 :size="14" /> Edit Skin
            </SolaraButton>
          </div>
        </aside>

        <!-- Right Workspace: Library / Pixel Editor / Capes -->
        <section class="sol-skin-workspace">
          <div class="sol-side-head" style="margin-bottom: 12px">
            <SolaraTabs :model-value="activeTab" :tabs="tabs" @update:model-value="changeSkinTab" />
            <SolaraButton
              v-if="activeTab === 'editor'"
              variant="primary"
              size="xs"
              :loading="saving"
              @click="saveEditorSkin"
            >
              Save to Library
            </SolaraButton>
            <SolaraButton
              v-else-if="activeTab === 'capes' && selectedAccount"
              variant="ghost"
              size="xs"
              :loading="loadingProfile"
              @click="loadAll"
            >
              <RefreshCw :size="13" /> Refresh Capes
            </SolaraButton>
          </div>

          <!-- Saved Skins + Add Skin Tile + Default Starter Skins -->
          <div v-if="activeTab === 'library'" class="sol-section-stack">
            <div class="sol-skins-grid">
              <button
                type="button"
                class="sol-skin-add-tile"
                @click="fileInput?.click()"
              >
                <Plus :size="20" />
                <strong>Add Skin</strong>
                <span>Import 64×64 PNG</span>
              </button>

              <article
                v-for="skin in savedSkins"
                :key="skin.id"
                :class="['sol-card', 'sol-skin-card', { 'is-selected': activeSkinDataUrl === skin.pngDataUrl }]"
                @click="selectSavedSkin(skin)"
              >
                <img :src="skin.pngDataUrl" :alt="skin.name" class="sol-skin-thumb" />
                <div class="sol-skin-meta">
                  <strong class="sol-ellipsis">{{ skin.name }}</strong>
                  <span class="sol-pill">{{ skin.variant === 'slim' ? 'Slim' : 'Classic' }}</span>
                </div>
                <div class="sol-row-gap" style="justify-content: flex-end">
                  <SolaraButton
                    variant="ghost"
                    size="xs"
                    @click.stop="selectSavedSkin(skin); activeTab = 'editor'"
                  >
                    <Palette :size="12" /> Edit
                  </SolaraButton>
                  <SolaraButton variant="danger" size="xs" @click.stop="deleteSkin(skin)">
                    <Trash2 :size="12" />
                  </SolaraButton>
                </div>
              </article>
            </div>

            <div class="sol-starter-block">
              <div class="sol-side-head">
                <h2 class="sol-section-title">Default Starter Skins</h2>
                <span class="sol-muted">Built-in 64×64 presets</span>
              </div>
              <div class="sol-skins-grid">
                <article
                  v-for="starter in starterSkins"
                  :key="starter.id"
                  :class="['sol-card', 'sol-skin-card', { 'is-selected': activeSkinDataUrl === starter.pngDataUrl }]"
                  @click="selectStarterSkin(starter)"
                >
                  <img :src="starter.pngDataUrl" :alt="starter.name" class="sol-skin-thumb" />
                  <div class="sol-skin-meta">
                    <strong class="sol-ellipsis">{{ starter.name }}</strong>
                    <span class="sol-pill">{{ starter.variant === 'slim' ? 'Slim' : 'Classic' }}</span>
                  </div>
                  <div class="sol-row-gap" style="justify-content: flex-end">
                    <SolaraButton variant="secondary" size="xs" @click.stop="saveStarterToLibrary(starter)">
                      <Plus :size="12" /> Save
                    </SolaraButton>
                  </div>
                </article>
              </div>
            </div>
          </div>

          <!-- Integrated Pixel Skin Editor -->
          <div v-else-if="activeTab === 'editor'" class="sol-card sol-skin-editor">
            <div class="sol-row-gap" style="margin-bottom: 6px">
              <input
                v-model="editorName"
                type="text"
                class="sol-input"
                placeholder="Skin name"
                style="max-width: 240px"
              />
              <span class="sol-pill">{{ activeVariant === 'slim' ? 'Slim (3px arms)' : 'Classic (4px arms)' }}</span>
              <span v-if="editorDirty" class="sol-muted" style="font-size: 11.5px">Unsaved pixels</span>
            </div>

            <SkinEditor
              v-model="activeSkinDataUrl"
              :variant="activeVariant"
              @dirty="editorDirty = $event"
            />
          </div>

          <!-- Account-Owned Capes + Local 3D Preview Capes -->
          <div v-else class="sol-section-stack">
            <div>
              <div class="sol-side-head" style="margin-bottom: 10px">
                <h2 class="sol-section-title">Account-Owned Capes</h2>
                <span v-if="selectedAccount" class="sol-muted">Equips directly on {{ selectedAccount.username }}</span>
                <span v-else class="sol-muted">Sign in with Microsoft to equip owned capes</span>
              </div>

              <div class="sol-capes-grid">
                <article
                  :class="['sol-card', 'sol-cape-card', { 'is-selected': !equippedCape && !isPreviewingLocalCape }]"
                  @click="selectedAccount ? equipCape(null) : clearLocalCapePreview()"
                >
                  <div class="sol-cape-preview sol-cape-preview--none">No Cape</div>
                  <div class="sol-skin-meta">
                    <strong>Hide Cape</strong>
                    <span class="sol-pill">{{ !equippedCape && !isPreviewingLocalCape ? 'Active' : 'Account' }}</span>
                  </div>
                </article>

                <article
                  v-for="cape in profileCapes"
                  :key="cape.id"
                  :class="['sol-card', 'sol-cape-card', { 'is-selected': cape.state === 'ACTIVE' && !isPreviewingLocalCape }]"
                  @click="equipCape(cape)"
                >
                  <img v-if="cape.dataUrl" :src="cape.dataUrl" :alt="cape.alias" class="sol-cape-preview" />
                  <div v-else class="sol-cape-preview sol-cape-preview--none">{{ cape.alias }}</div>
                  <div class="sol-skin-meta">
                    <strong class="sol-ellipsis">{{ cape.alias }}</strong>
                    <span class="sol-pill" :class="{ 'is-active': cape.state === 'ACTIVE' }">
                      {{ cape.state === 'ACTIVE' ? 'Equipped' : 'Owned' }}
                    </span>
                  </div>
                </article>
              </div>
            </div>

            <div class="sol-starter-block">
              <div class="sol-side-head" style="margin-bottom: 10px">
                <div class="sol-row-gap">
                  <h2 class="sol-section-title">Local 3D Preview Capes</h2>
                  <span class="sol-cape-preview-badge">Preview only — never claims to equip on Minecraft</span>
                </div>
                <SolaraButton
                  v-if="isPreviewingLocalCape"
                  variant="ghost"
                  size="xs"
                  @click="clearLocalCapePreview"
                >
                  Restore Account Cape
                </SolaraButton>
              </div>

              <div class="sol-capes-grid">
                <article
                  v-for="cape in previewCapes"
                  :key="cape.id"
                  :class="['sol-card', 'sol-cape-card', { 'is-selected': previewCapeId === cape.id }]"
                  @click="previewLocalCape(cape)"
                >
                  <img :src="cape.thumbUrl || cape.dataUrl" :alt="cape.name" class="sol-cape-preview" />
                  <div class="sol-skin-meta">
                    <strong class="sol-ellipsis">{{ cape.name }}</strong>
                    <span class="sol-pill">
                      <Sparkles :size="10" /> Preview only
                    </span>
                  </div>
                </article>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
