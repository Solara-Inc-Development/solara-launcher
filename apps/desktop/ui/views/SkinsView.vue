<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import {
  Copy,
  Upload,
  RefreshCw,
  Search,
  Trash2,
  Check,
  Eye,
  EyeOff,
  Plus,
  Edit3,
  Palette,
  Sparkles,
  User,
  X,
} from '../icons';
import { store, selectedAccount, native, notify, registerUnsavedChanges, runWithDirtyGuard } from '../state';
import type { ProfileCape, ProfileSkin, SavedSkin } from '../types';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraModal from '../components/SolaraModal.vue';
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
    name: 'Cobalt Explorer',
    variant: 'classic',
    skinTone: '#e8c3a5',
    hair: '#221c18',
    shirt: '#17171c',
    shirtTrim: '#3b82f6',
    pants: '#282a34',
    eyes: '#60a5fa',
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
  {
    id: 'starter-obsidian-knight',
    name: 'Obsidian Knight',
    variant: 'classic',
    skinTone: '#e2bfa3',
    hair: '#15171e',
    shirt: '#12151f',
    shirtTrim: '#3b82f6',
    pants: '#0b0d14',
    eyes: '#60a5fa',
  },
  {
    id: 'starter-emerald-ranger',
    name: 'Emerald Ranger',
    variant: 'slim',
    skinTone: '#ecc6a8',
    hair: '#5c3822',
    shirt: '#184e3c',
    shirtTrim: '#23c987',
    pants: '#2c251d',
    eyes: '#34d399',
  },
  {
    id: 'starter-nether-alchemist',
    name: 'Crimson Alchemist',
    variant: 'classic',
    skinTone: '#dcb292',
    hair: '#2a1829',
    shirt: '#4a1936',
    shirtTrim: '#f59e0b',
    pants: '#1d1424',
    eyes: '#fbbf24',
  },
];

const PREVIEW_CAPE_SPECS: PreviewCapeSpec[] = [
  {
    id: 'preview-dawn-banner',
    name: 'Solara Banner',
    base: '#0d1322',
    accent: '#3b82f6',
    trim: '#60a5fa',
  },
  {
    id: 'preview-obsidian-crest',
    name: 'Cobalt Crest',
    base: '#09090c',
    accent: '#2563eb',
    trim: '#3b82f6',
  },
  {
    id: 'preview-solstice-wing',
    name: 'Glacier Wing',
    base: '#0c1629',
    accent: '#7dd3fc',
    trim: '#1d4ed8',
  },
  {
    id: 'preview-emerald-ward',
    name: 'Emerald Ward',
    base: '#0b1e18',
    accent: '#10b981',
    trim: '#34d399',
  },
  {
    id: 'preview-nether-ember',
    name: 'Nether Ember',
    base: '#210e16',
    accent: '#f43f5e',
    trim: '#fbbf24',
  },
  {
    id: 'preview-amethyst-prism',
    name: 'Amethyst Prism',
    base: '#171026',
    accent: '#a855f7',
    trim: '#c084fc',
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
const viewerPose = ref<'idle' | 'walk' | 'run' | 'cape'>('walk');

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
const skinSearch = ref('');
const skinVariantFilter = ref<'all' | 'classic' | 'slim'>('all');
const deleteConfirmSkin = ref<SavedSkin | null>(null);
const deletingSkin = ref(false);
const skinHeadThumbs = ref<Record<string, string>>({});

function extractHeadThumbnail(key: string, pngDataUrl: string): void {
  if (!pngDataUrl || skinHeadThumbs.value[key]) return;
  const img = new Image();
  img.onload = () => {
    const c = document.createElement('canvas');
    c.width = 8;
    c.height = 8;
    const ctx = c.getContext('2d');
    if (!ctx) return;
    ctx.drawImage(img, 8, 8, 8, 8, 0, 0, 8, 8);
    if (img.height >= 64) {
      ctx.drawImage(img, 40, 8, 8, 8, 0, 0, 8, 8);
    }
    skinHeadThumbs.value = { ...skinHeadThumbs.value, [key]: c.toDataURL('image/png') };
  };
  img.src = pngDataUrl;
}

const activeAccountSkin = computed(
  () => profileSkins.value.find((s) => s.state === 'ACTIVE') ?? null,
);

const filteredSavedSkins = computed(() => {
  const q = skinSearch.value.trim().toLowerCase();
  return savedSkins.value.filter((skin) => {
    if (skinVariantFilter.value !== 'all' && skin.variant !== skinVariantFilter.value) return false;
    if (q && !skin.name.toLowerCase().includes(q)) return false;
    return true;
  });
});

const filteredStarterSkins = computed(() => {
  const q = skinSearch.value.trim().toLowerCase();
  return starterSkins.value.filter((skin) => {
    if (skinVariantFilter.value !== 'all' && skin.variant !== skinVariantFilter.value) return false;
    if (q && !skin.name.toLowerCase().includes(q)) return false;
    return true;
  });
});

const saving = ref(false);
const uploading = ref(false);
const loadingProfile = ref(false);
const equippingCapeId = ref<string | null>(null);
const capeFileInput = ref<HTMLInputElement | null>(null);
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
    for (const s of savedSkins.value) extractHeadThumbnail(s.id, s.pngDataUrl);
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
      if (activeProfSkin?.dataUrl) {
        extractHeadThumbnail('account-active', activeProfSkin.dataUrl);
      }
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
  starterSkins.value = STARTER_SKIN_SPECS.map((s) => {
    const pngDataUrl = buildStarterSkinPng(s);
    extractHeadThumbnail(s.id, pngDataUrl);
    return { ...s, pngDataUrl };
  });
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
  r.onload = () => {
    const dataUrl = typeof r.result === 'string' ? r.result : '';
    if (!dataUrl.startsWith('data:image/png')) {
      notify('error', 'Invalid file', 'Minecraft skins must be a 64×64 or 64×32 PNG.');
      return;
    }
    const probe = new Image();
    probe.onload = async () => {
      const validSize = (probe.width === 64 && (probe.height === 64 || probe.height === 32));
      if (!validSize) {
        notify('error', 'Invalid skin dimensions', `Expected 64×64 or 64×32 PNG, got ${probe.width}×${probe.height}.`);
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
    probe.onerror = () => {
      notify('error', 'Corrupted image', 'Could not decode the selected PNG file.');
    };
    probe.src = dataUrl;
  };
  r.readAsDataURL(file);
}

function createBlankSkin() {
  const starter = starterSkins.value[0];
  if (starter) {
    activeSkinDataUrl.value = starter.pngDataUrl;
    activeVariant.value = 'classic';
  }
  editorSkinId.value = null;
  editorName.value = 'New Custom Skin';
  editorDirty.value = false;
  rememberOriginalSkin();
  activeTab.value = 'editor';
}

async function duplicateSkin(skin: SavedSkin) {
  try {
    const saved = await native<SavedSkin>('saved_skins_save', {
      id: null,
      name: `${skin.name} Copy`,
      variant: skin.variant,
      pngDataUrl: skin.pngDataUrl,
    });
    await loadAll();
    selectSavedSkin(saved);
    notify('success', `Duplicated ${skin.name}`);
  } catch (err) {
    notify('error', "Couldn't duplicate skin", String(err));
  }
}

async function quickEquipSkin(pngDataUrl: string, variant: 'classic' | 'slim', label: string) {
  activeSkinDataUrl.value = pngDataUrl;
  activeVariant.value = variant;
  await applySkinToAccount();
  if (selectedAccount.value) {
    notify('info', `Selected ${label}`, `Applied to ${selectedAccount.value.username}.`);
  }
}

async function syncAccountSkinToLibrary() {
  if (!activeAccountSkin.value?.dataUrl) return;
  try {
    const variant = activeAccountSkin.value.variant === 'SLIM' ? 'slim' : 'classic';
    const saved = await native<SavedSkin>('saved_skins_save', {
      id: null,
      name: `${selectedAccount.value?.username || 'Account'} Skin`,
      variant,
      pngDataUrl: activeAccountSkin.value.dataUrl,
    });
    await loadAll();
    selectSavedSkin(saved);
    notify('success', 'Saved active account skin to library');
  } catch (err) {
    notify('error', "Couldn't save account skin", String(err));
  }
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

async function confirmDeleteSkin() {
  const skin = deleteConfirmSkin.value;
  if (!skin || deletingSkin.value) return;
  deletingSkin.value = true;
  try {
    await native('saved_skins_delete', { id: skin.id });
    if (editorSkinId.value === skin.id) editorSkinId.value = null;
    deleteConfirmSkin.value = null;
    await loadAll();
    notify('info', 'Skin deleted from local library');
  } catch (err) {
    notify('error', "Couldn't delete skin", String(err));
  } finally {
    deletingSkin.value = false;
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
  if (!selectedAccount.value || equippingCapeId.value !== null) return;
  previewCapeId.value = null;
  equippingCapeId.value = cape?.id ?? 'none';
  try {
    await native('set_minecraft_cape', { capeId: cape?.id ?? null });
    activeCapeDataUrl.value = cape?.dataUrl ?? null;
    viewerPose.value = cape ? 'cape' : 'walk';
    animateModel.value = true;
    await loadAll();
    notify('success', cape ? `Equipped ${cape.alias} cape` : 'Cape hidden');
  } catch (err) {
    notify('error', "Couldn't update cape", String(err));
  } finally {
    equippingCapeId.value = null;
  }
}

function previewLocalCape(cape: { id: string; dataUrl: string }) {
  previewCapeId.value = cape.id;
  activeCapeDataUrl.value = cape.dataUrl;
  viewerPose.value = 'cape';
  animateModel.value = true;
}

function clearLocalCapePreview() {
  previewCapeId.value = null;
  activeCapeDataUrl.value = equippedCape.value?.dataUrl ?? null;
  if (!equippedCape.value) viewerPose.value = 'walk';
}

function onUploadPreviewCape(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  (e.target as HTMLInputElement).value = '';
  if (!file) return;
  const r = new FileReader();
  r.onload = () => {
    const dataUrl = typeof r.result === 'string' ? r.result : '';
    if (!dataUrl.startsWith('data:image/png')) {
      notify('error', 'Invalid cape file', 'Minecraft cape previews must be a 64×32 PNG.');
      return;
    }
    const probe = new Image();
    probe.onload = () => {
      if (probe.width !== 64 || probe.height !== 32) {
        notify('error', 'Invalid cape dimensions', `Expected 64×32 PNG, got ${probe.width}×${probe.height}.`);
        return;
      }
      const thumb = document.createElement('canvas');
      thumb.width = 10;
      thumb.height = 16;
      const tctx = thumb.getContext('2d');
      tctx?.drawImage(probe, 1, 1, 10, 16, 0, 0, 10, 16);
      const customId = `preview-custom-${Date.now()}`;
      previewCapes.value = [
        {
          id: customId,
          name: file.name.replace(/\.png$/i, '') || 'Custom Preview Cape',
          base: '#0d1322',
          accent: '#3b82f6',
          trim: '#60a5fa',
          dataUrl,
          thumbUrl: thumb.toDataURL('image/png'),
        },
        ...previewCapes.value,
      ];
      previewLocalCape({ id: customId, dataUrl });
      notify('info', 'Loaded 3D cape preview', 'Preview only — local capes do not equip onto your Minecraft account.');
    };
    probe.onerror = () => {
      notify('error', 'Corrupted image', 'Could not decode the selected cape PNG.');
    };
    probe.src = dataUrl;
  };
  r.readAsDataURL(file);
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
            :pose="viewerPose"
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

            <div class="sol-pose-pills" role="group" aria-label="3D viewer pose">
              <button
                v-for="p in (['idle', 'walk', 'run', 'cape'] as const)"
                :key="p"
                type="button"
                :class="['sol-pill', 'sol-pose-btn', { 'is-active': viewerPose === p }]"
                @click="viewerPose = p; animateModel = true"
              >
                {{ p[0].toUpperCase() + p.slice(1) }}
              </button>
            </div>

            <div class="sol-row-gap" style="justify-content: space-between; width: 100%">
              <SolaraButton variant="ghost" size="xs" @click="showOuter = !showOuter">
                <Eye v-if="showOuter" :size="13" />
                <EyeOff v-else :size="13" />
                Outer Layer
              </SolaraButton>
              <SolaraButton variant="ghost" size="xs" @click="animateModel = !animateModel">
                {{ animateModel ? 'Pause Walk' : 'Resume Pose' }}
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

          <!-- Saved Skins + Account Sync + Filter Toolbar + Default Starter Skins -->
          <div v-if="activeTab === 'library'" class="sol-section-stack">
            <div v-if="selectedAccount && activeAccountSkin?.dataUrl" class="sol-skin-account-strip">
              <div class="sol-row-gap">
                <img
                  :src="skinHeadThumbs['account-active'] || activeAccountSkin.dataUrl"
                  :alt="selectedAccount.username"
                  class="sol-skin-head-chip"
                />
                <div>
                  <div class="sol-row-gap">
                    <strong>{{ selectedAccount.username }}</strong>
                    <span class="sol-pill is-active">{{ activeAccountSkin.variant === 'SLIM' ? 'Slim' : 'Classic' }}</span>
                  </div>
                  <div class="sol-muted" style="font-size: 12px">Active Minecraft profile skin</div>
                </div>
              </div>
              <SolaraButton variant="secondary" size="xs" @click="syncAccountSkinToLibrary">
                <User :size="12" /> Save Active to Library
              </SolaraButton>
            </div>

            <div class="sol-skin-toolbar">
              <label class="sol-skin-search">
                <Search :size="14" />
                <input
                  v-model="skinSearch"
                  type="text"
                  placeholder="Search saved or starter skins..."
                  aria-label="Search skins"
                />
                <button
                  v-if="skinSearch"
                  type="button"
                  class="sol-skin-search-clear"
                  aria-label="Clear skin search"
                  @click="skinSearch = ''"
                >
                  <X :size="12" />
                </button>
              </label>
              <div class="sol-skin-filter-pills" role="group" aria-label="Filter skin model">
                <button
                  v-for="opt in (['all', 'classic', 'slim'] as const)"
                  :key="opt"
                  type="button"
                  :class="['sol-pill', 'sol-skin-filter-btn', { 'is-active': skinVariantFilter === opt }]"
                  @click="skinVariantFilter = opt"
                >
                  {{ opt === 'all' ? 'All Models' : opt === 'classic' ? 'Classic (4px)' : 'Slim (3px)' }}
                </button>
              </div>
              <SolaraButton variant="secondary" size="xs" @click="createBlankSkin">
                <Sparkles :size="12" /> New Blank Skin
              </SolaraButton>
            </div>

            <div class="sol-skins-grid">
              <button
                type="button"
                class="sol-skin-add-tile"
                @click="fileInput?.click()"
              >
                <Plus :size="20" />
                <strong>Add Skin</strong>
                <span>Import 64×64 / 64×32 PNG</span>
              </button>

              <article
                v-for="skin in filteredSavedSkins"
                :key="skin.id"
                :class="['sol-card', 'sol-skin-card', { 'is-selected': activeSkinDataUrl === skin.pngDataUrl }]"
                @click="selectSavedSkin(skin)"
              >
                <div class="sol-skin-thumb-wrap">
                  <img :src="skin.pngDataUrl" :alt="skin.name" class="sol-skin-thumb" />
                  <img
                    v-if="skinHeadThumbs[skin.id]"
                    :src="skinHeadThumbs[skin.id]"
                    :alt="`${skin.name} face`"
                    class="sol-skin-head-chip"
                  />
                </div>
                <div class="sol-skin-meta">
                  <strong class="sol-ellipsis">{{ skin.name }}</strong>
                  <span class="sol-pill">{{ skin.variant === 'slim' ? 'Slim' : 'Classic' }}</span>
                </div>
                <div class="sol-skin-actions">
                  <SolaraButton
                    v-if="selectedAccount"
                    variant="primary"
                    size="xs"
                    @click.stop="quickEquipSkin(skin.pngDataUrl, skin.variant, skin.name)"
                  >
                    Equip
                  </SolaraButton>
                  <SolaraButton
                    variant="ghost"
                    size="xs"
                    @click.stop="selectSavedSkin(skin); activeTab = 'editor'"
                  >
                    <Palette :size="12" /> Edit
                  </SolaraButton>
                  <SolaraButton
                    variant="ghost"
                    size="xs"
                    title="Duplicate skin"
                    aria-label="Duplicate skin"
                    @click.stop="duplicateSkin(skin)"
                  >
                    <Copy :size="12" />
                  </SolaraButton>
                  <SolaraButton
                    variant="danger"
                    size="xs"
                    title="Delete skin"
                    aria-label="Delete skin"
                    @click.stop="deleteConfirmSkin = skin"
                  >
                    <Trash2 :size="12" />
                  </SolaraButton>
                </div>
              </article>
            </div>

            <div v-if="savedSkins.length > 0 && filteredSavedSkins.length === 0" class="sol-muted" style="font-size: 12.5px">
              No saved skins match "{{ skinSearch }}" ({{ skinVariantFilter }}).
            </div>

            <div class="sol-starter-block">
              <div class="sol-side-head">
                <h2 class="sol-section-title">Default Starter Skins</h2>
                <span class="sol-muted">{{ filteredStarterSkins.length }} built-in 64×64 presets</span>
              </div>
              <div class="sol-skins-grid">
                <article
                  v-for="starter in filteredStarterSkins"
                  :key="starter.id"
                  :class="['sol-card', 'sol-skin-card', { 'is-selected': activeSkinDataUrl === starter.pngDataUrl }]"
                  @click="selectStarterSkin(starter)"
                >
                  <div class="sol-skin-thumb-wrap">
                    <img :src="starter.pngDataUrl" :alt="starter.name" class="sol-skin-thumb" />
                    <img
                      v-if="skinHeadThumbs[starter.id]"
                      :src="skinHeadThumbs[starter.id]"
                      :alt="`${starter.name} face`"
                      class="sol-skin-head-chip"
                    />
                  </div>
                  <div class="sol-skin-meta">
                    <strong class="sol-ellipsis">{{ starter.name }}</strong>
                    <span class="sol-pill">{{ starter.variant === 'slim' ? 'Slim' : 'Classic' }}</span>
                  </div>
                  <div class="sol-skin-actions">
                    <SolaraButton
                      v-if="selectedAccount"
                      variant="primary"
                      size="xs"
                      @click.stop="quickEquipSkin(starter.pngDataUrl, starter.variant, starter.name)"
                    >
                      Equip
                    </SolaraButton>
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
            <div class="sol-cape-status-strip">
              <div>
                <div class="sol-row-gap">
                  <strong>
                    {{
                      isPreviewingLocalCape
                        ? `3D Preview: ${previewCapes.find((c) => c.id === previewCapeId)?.name ?? 'Custom Cape'}`
                        : equippedCape
                          ? `Equipped on Account: ${equippedCape.alias}`
                          : 'No Account Cape Active'
                    }}
                  </strong>
                  <span :class="['sol-pill', { 'is-active': Boolean(equippedCape) && !isPreviewingLocalCape }]">
                    {{ isPreviewingLocalCape ? 'Preview only' : equippedCape ? 'Account Active' : 'Hidden' }}
                  </span>
                </div>
                <div class="sol-muted" style="font-size: 12px; margin-top: 2px">
                  {{
                    selectedAccount
                      ? `Selecting an owned cape equips it for ${selectedAccount.username} and turns the 3D stage to inspect the back.`
                      : 'Sign in with Microsoft to equip owned capes, or test local 3D preview designs below.'
                  }}
                </div>
              </div>
              <div class="sol-row-gap">
                <input ref="capeFileInput" type="file" accept="image/png" hidden @change="onUploadPreviewCape" />
                <SolaraButton variant="secondary" size="xs" @click="capeFileInput?.click()">
                  <Upload :size="12" /> Preview 64×32 PNG
                </SolaraButton>
                <SolaraButton variant="ghost" size="xs" @click="viewerPose = 'cape'; animateModel = true">
                  Inspect Back in 3D
                </SolaraButton>
                <SolaraButton v-if="isPreviewingLocalCape" variant="ghost" size="xs" @click="clearLocalCapePreview">
                  Clear Preview
                </SolaraButton>
              </div>
            </div>

            <div>
              <div class="sol-side-head" style="margin-bottom: 10px">
                <h2 class="sol-section-title">Account-Owned Capes</h2>
                <span v-if="selectedAccount" class="sol-muted">{{ profileCapes.length }} owned on {{ selectedAccount.username }}</span>
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
                    <span class="sol-pill" :class="{ 'is-active': cape.state === 'ACTIVE' && !isPreviewingLocalCape }">
                      {{ cape.state === 'ACTIVE' ? 'Equipped' : 'Owned' }}
                    </span>
                  </div>
                  <div class="sol-skin-actions">
                    <SolaraButton
                      variant="primary"
                      size="xs"
                      :loading="equippingCapeId === cape.id"
                      :disabled="cape.state === 'ACTIVE' && !isPreviewingLocalCape"
                      @click.stop="equipCape(cape)"
                    >
                      {{ cape.state === 'ACTIVE' ? 'Equipped' : 'Equip' }}
                    </SolaraButton>
                    <SolaraButton
                      v-if="cape.dataUrl"
                      variant="ghost"
                      size="xs"
                      @click.stop="previewLocalCape({ id: cape.id, dataUrl: cape.dataUrl })"
                    >
                      3D View
                    </SolaraButton>
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
                  <div class="sol-skin-actions">
                    <SolaraButton
                      :variant="previewCapeId === cape.id ? 'primary' : 'secondary'"
                      size="xs"
                      @click.stop="previewCapeId === cape.id ? clearLocalCapePreview() : previewLocalCape(cape)"
                    >
                      {{ previewCapeId === cape.id ? 'Previewing' : 'Preview 3D' }}
                    </SolaraButton>
                  </div>
                </article>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>

    <SolaraModal
      :open="Boolean(deleteConfirmSkin)"
      title="Delete Saved Skin?"
      size="sm"
      @close="deleteConfirmSkin = null"
    >
      <p class="sol-muted" style="margin: 0 0 14px; font-size: 13px; line-height: 1.5">
        Remove <strong>{{ deleteConfirmSkin?.name }}</strong> from your local skin library? This cannot be undone.
      </p>
      <template #footer>
        <SolaraButton variant="ghost" size="sm" @click="deleteConfirmSkin = null">Cancel</SolaraButton>
        <SolaraButton variant="danger" size="sm" :loading="deletingSkin" @click="confirmDeleteSkin">
          <Trash2 :size="13" /> Delete Skin
        </SolaraButton>
      </template>
    </SolaraModal>
  </div>
</template>
