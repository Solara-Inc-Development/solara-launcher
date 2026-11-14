<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { Upload, RefreshCw, Trash2, Check, Eye, EyeOff, Plus, Shirt } from '../icons';
import { store, selectedAccount, native, notify } from '../state';
import type { ProfileCape, ProfileSkin, SavedSkin } from '../types';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraTabs from '../components/SolaraTabs.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import SkinViewer3D from '../components/SkinViewer3D.vue';
import SkinEditor from '../components/SkinEditor.vue';

const activeTab = ref<'library' | 'editor' | 'capes'>('library');
const savedSkins = ref<SavedSkin[]>([]);
const profileCapes = ref<ProfileCape[]>([]);
const profileSkins = ref<ProfileSkin[]>([]);
const activeSkinDataUrl = ref<string>('');
const activeCapeDataUrl = ref<string | null>(null);
const activeVariant = ref<'classic' | 'slim'>('classic');
const showOuter = ref(true);
const animateModel = ref(true);

const editorName = ref('Custom Solara Skin');
const editorSkinId = ref<string | null>(null);
const editorDirty = ref(false);
const saving = ref(false);
const uploading = ref(false);
const loadingProfile = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);

const tabs = [
  { id: 'library', label: 'Saved Skins' },
  { id: 'editor', label: 'Skin Editor' },
  { id: 'capes', label: 'Capes' },
];

const equippedCape = computed(
  () => profileCapes.value.find((c) => c.state === 'ACTIVE') ?? null,
);

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
      const activeCap = profileCapes.value.find((c) => c.state === 'ACTIVE');
      activeCapeDataUrl.value = activeCap?.dataUrl ?? null;
    } catch {
      // keep local skins functional even if profile API fails
    } finally {
      loadingProfile.value = false;
    }
  }
  if (!activeSkinDataUrl.value && savedSkins.value.length > 0) {
    activeSkinDataUrl.value = savedSkins.value[0].pngDataUrl;
    activeVariant.value = savedSkins.value[0].variant;
    editorName.value = savedSkins.value[0].name;
    editorSkinId.value = savedSkins.value[0].id;
  }
}

onMounted(() => {
  loadAll();
  store.dirtyChecker = () => activeTab.value === 'editor' && editorDirty.value;
});

onUnmounted(() => {
  store.dirtyChecker = null;
});

function selectSavedSkin(skin: SavedSkin) {
  activeSkinDataUrl.value = skin.pngDataUrl;
  activeVariant.value = skin.variant;
  editorName.value = skin.name;
  editorSkinId.value = skin.id;
  editorDirty.value = false;
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
      notify('error', 'Could not import skin', String(err));
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
    await loadAll();
    notify('success', 'Skin saved locally');
  } catch (err) {
    notify('error', 'Save failed', String(err));
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
    notify('error', 'Delete failed', String(err));
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
    notify('error', 'Could not apply skin', String(err));
  } finally {
    uploading.value = false;
  }
}

async function equipCape(cape: ProfileCape | null) {
  if (!selectedAccount.value) return;
  try {
    await native('set_minecraft_cape', { capeId: cape?.id ?? null });
    activeCapeDataUrl.value = cape?.dataUrl ?? null;
    await loadAll();
    notify('success', cape ? `Equipped ${cape.alias} cape` : 'Cape hidden');
  } catch (err) {
    notify('error', 'Could not update cape', String(err));
  }
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-page-inner">
      <header class="sol-page-header">
        <div>
          <h1 class="sol-page-title">Skin & Capes</h1>
          <p class="sol-page-sub">Preview in 3D, edit pixel layers, manage saved skins, and equip owned capes</p>
        </div>

        <div class="sol-row-gap">
          <input ref="fileInput" type="file" accept="image/png" hidden @change="onUploadPng" />
          <SolaraButton variant="secondary" size="sm" @click="fileInput?.click()">
            <Upload :size="14" /> Import Skin PNG
          </SolaraButton>
          <SolaraButton
            variant="primary"
            size="sm"
            :loading="uploading"
            :disabled="!activeSkinDataUrl || !selectedAccount"
            @click="applySkinToAccount"
          >
            <Check :size="14" /> Apply to Minecraft Account
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
            :animate="animateModel"
            :height="320"
          />

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

            <div class="sol-muted sol-text-xs">
              Active Cape: <strong>{{ equippedCape?.alias ?? 'None' }}</strong>
            </div>
          </div>
        </aside>

        <!-- Right Workspace: Library / Pixel Editor / Capes -->
        <section class="sol-skin-workspace">
          <div class="sol-side-head" style="margin-bottom: 12px">
            <SolaraTabs v-model="activeTab" :tabs="tabs" />
            <SolaraButton
              v-if="activeTab === 'editor'"
              variant="primary"
              size="xs"
              :loading="saving"
              @click="saveEditorSkin"
            >
              Save to Library
            </SolaraButton>
          </div>

          <!-- Saved Skins Library -->
          <div v-if="activeTab === 'library'" class="sol-stack">
            <SolaraEmptyState
              v-if="savedSkins.length === 0 && profileSkins.length === 0"
              title="No saved skins yet"
              description="Import a 64×64 Minecraft skin PNG or create one inside the built-in Skin Editor."
            >
              <template #icon><Shirt :size="26" /></template>
              <template #actions>
                <SolaraButton variant="primary" size="sm" @click="fileInput?.click()">
                  <Upload :size="14" /> Import PNG
                </SolaraButton>
                <SolaraButton variant="secondary" size="sm" @click="activeTab = 'editor'">
                  <Plus :size="14" /> Open Skin Editor
                </SolaraButton>
              </template>
            </SolaraEmptyState>

            <div v-else class="sol-skins-grid">
              <article
                v-for="skin in savedSkins"
                :key="skin.id"
                :class="['sol-card', 'sol-skin-card', { 'is-selected': activeSkinDataUrl === skin.pngDataUrl }]"
                @click="selectSavedSkin(skin)"
              >
                <img :src="skin.pngDataUrl" :alt="skin.name" class="sol-skin-thumb" />
                <div class="sol-skin-card-info">
                  <div class="sol-skin-name sol-truncate">{{ skin.name }}</div>
                  <div class="sol-muted sol-text-xs">{{ skin.variant === 'slim' ? 'Slim (Alex)' : 'Classic (Steve)' }}</div>
                </div>
                <div class="sol-row-gap">
                  <SolaraButton
                    variant="ghost"
                    size="xs"
                    @click.stop="
                      selectSavedSkin(skin);
                      activeTab = 'editor';
                    "
                  >
                    Edit
                  </SolaraButton>
                  <button
                    type="button"
                    class="sol-icon-btn sol-icon-btn--xs"
                    title="Delete saved skin"
                    @click.stop="deleteSkin(skin)"
                  >
                    <Trash2 :size="13" />
                  </button>
                </div>
              </article>
            </div>
          </div>

          <!-- Built-in Skin Editor -->
          <div v-else-if="activeTab === 'editor'" class="sol-card sol-stack">
            <div class="sol-split-2">
              <div class="sol-field">
                <label class="sol-label">Skin Name</label>
                <input v-model="editorName" type="text" class="sol-input" maxlength="60" />
              </div>
              <div class="sol-field" style="justify-content: flex-end; display: flex; align-items: flex-end">
                <span v-if="editorDirty" class="sol-badge sol-badge--warning">Unsaved edits</span>
              </div>
            </div>
            <SkinEditor
              v-model="activeSkinDataUrl"
              :variant="activeVariant"
              @dirty="editorDirty = $event"
            />
          </div>

          <!-- Cape Selector -->
          <div v-else-if="activeTab === 'capes'" class="sol-stack">
            <div class="sol-toolbar">
              <span class="sol-muted sol-text-xs">
                Capes come from your authenticated Minecraft: Java Edition account.
              </span>
              <SolaraButton variant="secondary" size="xs" :loading="loadingProfile" @click="loadAll">
                <RefreshCw :size="12" /> Refresh Capes
              </SolaraButton>
            </div>

            <SolaraEmptyState
              v-if="!selectedAccount"
              title="Sign in to view account capes"
              description="Connect your Microsoft Minecraft account to view and equip capes you own."
            />
            <SolaraEmptyState
              v-else-if="profileCapes.length === 0"
              title="No capes found on this account"
              description="Any Mojang, Migrator, Vanilla, or event capes unlocked on your Minecraft account will appear here."
            />
            <div v-else class="sol-capes-grid">
              <article
                :class="['sol-card', 'sol-cape-card', { 'is-selected': !equippedCape }]"
                @click="equipCape(null)"
              >
                <div class="sol-cape-preview sol-cape-preview--none">None</div>
                <div class="sol-cape-name">No Cape</div>
                <span v-if="!equippedCape" class="sol-badge sol-badge--success">Equipped</span>
              </article>

              <article
                v-for="cape in profileCapes"
                :key="cape.id"
                :class="['sol-card', 'sol-cape-card', { 'is-selected': cape.state === 'ACTIVE' }]"
                @click="equipCape(cape)"
              >
                <img v-if="cape.dataUrl" :src="cape.dataUrl" :alt="cape.alias" class="sol-cape-preview" />
                <div v-else class="sol-cape-preview sol-cape-preview--none">{{ cape.alias }}</div>
                <div class="sol-cape-name">{{ cape.alias }}</div>
                <span v-if="cape.state === 'ACTIVE'" class="sol-badge sol-badge--success">Equipped</span>
              </article>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>
