<script setup lang="ts">
import { computed, onUnmounted, reactive, ref, watch } from 'vue';
import InstanceIcon from './InstanceIcon.vue';
import SolaraButton from './SolaraButton.vue';
import SolaraModal from './SolaraModal.vue';
import SolaraSaveBar from './SolaraSaveBar.vue';
import SolaraToggle from './SolaraToggle.vue';
import {
  Archive,
  Copy,
  Cpu,
  FolderOpen,
  Maximize2,
  Package,
  RefreshCw,
  Shuffle,
  Sliders,
  Terminal,
  Trash2,
  Upload,
} from '../icons';
import {
  BUILTIN_ICON_PRESETS,
  registerUnsavedChanges,
  runWithDirtyGuard,
  LOADER_LABEL,
  native,
  navigate,
  notify,
  patchInstanceMeta,
  refreshSnapshot,
  selectInstance,
  store,
} from '../state';
import type { BackupEntry, DetectedJava, InstanceRecord, InstanceSettingsCategoryId } from '../types';

const categories: Array<{ id: InstanceSettingsCategoryId; label: string; icon: unknown }> = [
  { id: 'general', label: 'General', icon: Sliders },
  { id: 'installation', label: 'Installation', icon: Package },
  { id: 'java', label: 'Java & Memory', icon: Cpu },
  { id: 'window', label: 'Window', icon: Maximize2 },
  { id: 'hooks', label: 'Launch Hooks', icon: Terminal },
  { id: 'backups', label: 'Backups', icon: Archive },
  { id: 'advanced', label: 'Advanced', icon: RefreshCw },
];

const instance = computed<InstanceRecord | null>(
  () => store.instances.find((i) => i.id === store.instanceSettingsId) ?? null,
);

const fileInput = ref<HTMLInputElement | null>(null);
const detectedJava = ref<DetectedJava[]>([]);
const backups = ref<BackupEntry[]>([]);
const loadingBackups = ref(false);
const saving = ref(false);
const repairing = ref(false);
const backingUp = ref(false);
const deletePrompt = ref(false);
const deleting = ref(false);

const draft = reactive({
  name: '',
  group: '',
  notes: '',
  icon: '',
  customIcon: null as string | null,
  pinned: false,
  favourite: false,
  memory: 4096,
  javaPath: 'auto',
  jvmArgs: '',
  width: 854,
  height: 480,
  fullscreen: false,
  preLaunch: '',
  postExit: '',
  updateChannel: 'release' as 'release' | 'beta' | 'alpha',
});

function loadDraft(): void {
  const inst = instance.value;
  if (!inst) return;
  draft.name = inst.name;
  draft.group = inst.group ?? '';
  draft.notes = inst.notes ?? '';
  draft.icon = inst.icon ?? 'preset:dawn-crest';
  draft.customIcon = inst.customIcon ?? null;
  draft.pinned = Boolean(inst.pinned);
  draft.favourite = Boolean(inst.favourite);
  draft.memory = inst.memory ?? store.preferences.memory;
  draft.javaPath = inst.javaPath || 'auto';
  draft.jvmArgs = inst.jvmArgs ?? '';
  draft.width = inst.resolution?.[0] ?? store.preferences.defaultWidth;
  draft.height = inst.resolution?.[1] ?? store.preferences.defaultHeight;
  draft.fullscreen = Boolean(inst.fullscreen ?? store.preferences.fullscreen);
  draft.preLaunch = inst.preLaunch ?? '';
  draft.postExit = inst.postExit ?? '';
  draft.updateChannel = inst.updateChannel ?? 'release';
}

const isDirty = computed(() => {
  const inst = instance.value;
  if (!inst) return false;
  return (
    draft.name.trim() !== inst.name ||
    draft.group.trim() !== (inst.group ?? '') ||
    draft.notes !== (inst.notes ?? '') ||
    draft.icon !== (inst.icon ?? 'preset:dawn-crest') ||
    (draft.customIcon ?? null) !== (inst.customIcon ?? null) ||
    draft.pinned !== Boolean(inst.pinned) ||
    draft.favourite !== Boolean(inst.favourite) ||
    draft.memory !== (inst.memory ?? store.preferences.memory) ||
    draft.javaPath !== (inst.javaPath || 'auto') ||
    draft.jvmArgs !== (inst.jvmArgs ?? '') ||
    draft.width !== (inst.resolution?.[0] ?? store.preferences.defaultWidth) ||
    draft.height !== (inst.resolution?.[1] ?? store.preferences.defaultHeight) ||
    draft.fullscreen !== Boolean(inst.fullscreen ?? store.preferences.fullscreen) ||
    draft.preLaunch !== (inst.preLaunch ?? '') ||
    draft.postExit !== (inst.postExit ?? '') ||
    draft.updateChannel !== (inst.updateChannel ?? 'release')
  );
});

async function loadBackups(): Promise<void> {
  if (!store.desktop || !instance.value) return;
  loadingBackups.value = true;
  try {
    backups.value = (await native<BackupEntry[]>('instance_backups', { id: instance.value.id })) ?? [];
  } catch {
    backups.value = [];
  } finally {
    loadingBackups.value = false;
  }
}

async function createBackup(): Promise<void> {
  if (!instance.value) return;
  backingUp.value = true;
  try {
    await native('instance_backup_action', {
      id: instance.value.id,
      action: 'create',
      backupId: null,
      name: `${instance.value.name} backup`,
    });
    notify('success', 'Backup created', 'World and configuration snapshot saved.');
    await loadBackups();
  } catch (err) {
    notify('error', 'Backup failed', String(err));
  } finally {
    backingUp.value = false;
  }
}

async function restoreBackup(b: BackupEntry): Promise<void> {
  if (!instance.value) return;
  try {
    await native('instance_backup_action', {
      id: instance.value.id,
      action: 'restore',
      backupId: b.id,
      name: null,
    });
    notify('success', 'Backup restored', `${b.name} restored.`);
    await refreshSnapshot();
  } catch (err) {
    notify('error', 'Restore failed', String(err));
  }
}

async function deleteBackup(b: BackupEntry): Promise<void> {
  if (!instance.value) return;
  try {
    await native('instance_backup_action', {
      id: instance.value.id,
      action: 'delete',
      backupId: b.id,
      name: null,
    });
    await loadBackups();
  } catch (err) {
    notify('error', 'Delete failed', String(err));
  }
}

function randomizeIcon(): void {
  draft.customIcon = null;
  draft.icon = BUILTIN_ICON_PRESETS[Math.floor(Math.random() * BUILTIN_ICON_PRESETS.length)];
}

function onIconFile(e: Event): void {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  if (file.size > 2 * 1024 * 1024) {
    notify('warning', 'Image too large', 'Choose an icon smaller than 2 MB.');
    input.value = '';
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    if (typeof reader.result === 'string') {
      draft.customIcon = reader.result;
    }
  };
  reader.readAsDataURL(file);
  input.value = '';
}

async function saveChanges(): Promise<void> {
  if (!instance.value) return;
  if (!draft.name.trim()) {
    notify('warning', 'Name required', 'Enter an instance name.');
    return;
  }
  saving.value = true;
  try {
    const inst = instance.value;
    const patch: Record<string, unknown> = {};
    const changed = (key: string, value: unknown, previous: unknown) => {
      if (JSON.stringify(value) !== JSON.stringify(previous)) patch[key] = value;
    };
    changed('name', draft.name.trim(), inst.name);
    changed('memory', draft.memory, inst.memory ?? store.preferences.memory);
    changed('javaPath', draft.javaPath, inst.javaPath || 'auto');
    changed('icon', draft.customIcon || draft.icon, inst.customIcon || inst.icon || 'preset:dawn-crest');
    changed('group', draft.group.trim() || null, inst.group || null);
    changed('pinned', draft.pinned, Boolean(inst.pinned));
    changed('favourite', draft.favourite, Boolean(inst.favourite));
    changed('jvmArgs', draft.jvmArgs.trim() || null, inst.jvmArgs || null);
    changed('resolution', [draft.width, draft.height], inst.resolution ?? [store.preferences.defaultWidth, store.preferences.defaultHeight]);
    changed('fullscreen', draft.fullscreen, inst.fullscreen ?? store.preferences.fullscreen);
    changed('preLaunch', draft.preLaunch.trim() || null, inst.preLaunch || null);
    changed('postExit', draft.postExit.trim() || null, inst.postExit || null);
    changed('updateChannel', draft.updateChannel, inst.updateChannel ?? 'release');
    changed('notes', draft.notes.trim() || null, inst.notes || null);
    if (Object.keys(patch).length > 0) await patchInstanceMeta(inst.id, patch);
    notify('success', 'Instance updated', `${draft.name.trim()} settings saved.`);
    loadDraft();
  } catch (err) {
    notify('error', "Couldn't save instance settings", String(err));
  } finally {
    saving.value = false;
  }
}

async function repairInstance(): Promise<void> {
  if (!instance.value) return;
  repairing.value = true;
  try {
    await native('repair_instance', { id: instance.value.id });
    notify('success', 'Repair queued', 'Re-verifying libraries and game assets.');
    await refreshSnapshot();
  } catch (err) {
    notify('error', 'Repair failed', String(err));
  } finally {
    repairing.value = false;
  }
}

async function duplicateInstance(): Promise<void> {
  if (!instance.value) return;
  try {
    await native('duplicate_instance', { id: instance.value.id, name: `${instance.value.name} (Copy)` });
    notify('success', 'Instance duplicated', 'Copy added to your Library.');
    await refreshSnapshot();
  } catch (err) {
    notify('error', 'Duplicate failed', String(err));
  }
}

async function confirmDeleteInstance(): Promise<void> {
  if (!instance.value) return;
  deleting.value = true;
  try {
    const victimId = instance.value.id;
    const victimName = instance.value.name;
    await native('delete_instance', { id: victimId });
    deletePrompt.value = false;
    store.showInstanceSettingsModal = false;
    selectInstance(null);
    await refreshSnapshot();
    navigate('library');
    notify('success', 'Instance deleted', `${victimName} removed.`);
  } catch (err) {
    notify('error', 'Delete failed', String(err));
  } finally {
    deleting.value = false;
  }
}

const unregisterGuard = registerUnsavedChanges(
  () => store.showInstanceSettingsModal && isDirty.value,
  loadDraft,
);
onUnmounted(unregisterGuard);
function handleClose(): void {
  if (saving.value || deleting.value) return;
  runWithDirtyGuard(() => { store.showInstanceSettingsModal = false; });
}

watch(
  () => [store.showInstanceSettingsModal, store.instanceSettingsId],
  ([open]) => {
    if (open) {
      loadDraft();
      deletePrompt.value = false;
      if (store.desktop) {
        native<DetectedJava[]>('detect_java_runtimes')
          .then((list) => {
            detectedJava.value = Array.isArray(list) ? list : [];
          })
          .catch(() => {});
        void loadBackups();
      }
    }
  },
);
</script>

<template>
  <SolaraModal
    :open="store.showInstanceSettingsModal && Boolean(instance)"
    :title="instance ? `${instance.name} — Settings` : 'Instance Settings'"
    :subtitle="instance ? `${LOADER_LABEL[instance.loader] ?? instance.loader} · Minecraft ${instance.gameVersion}` : ''"
    width="xl"
    :closable="!saving && !deleting"
    @close="handleClose"
  >
    <div v-if="instance" class="sol-settings-dialog">
      <!-- Left Category Rail -->
      <aside class="sol-settings-dialog-nav">
        <button
          v-for="cat in categories"
          :key="cat.id"
          type="button"
          :class="['sol-settings-nav-btn', { 'is-active': store.instanceSettingsCategory === cat.id }]"
          @click="store.instanceSettingsCategory = cat.id"
        >
          <component :is="cat.icon" :size="15" />
          <span>{{ cat.label }}</span>
        </button>
      </aside>

      <!-- Right Scrollable Panel -->
      <div class="sol-settings-dialog-body sol-scroll">
        <!-- General -->
        <section v-if="store.instanceSettingsCategory === 'general'" class="sol-stack">
          <div class="sol-create-hero">
            <div class="sol-create-icon-panel">
              <InstanceIcon
                :name="draft.name"
                :loader="instance.loader"
                :icon="draft.icon"
                :custom-icon="draft.customIcon"
                :size="64"
                :rounded="14"
              />
              <div class="sol-row-gap">
                <input ref="fileInput" type="file" accept="image/png,image/jpeg,image/webp" hidden @change="onIconFile" />
                <SolaraButton variant="secondary" size="xs" @click="fileInput?.click()">
                  <Upload :size="12" /> Upload
                </SolaraButton>
                <SolaraButton variant="ghost" size="xs" @click="randomizeIcon">
                  <Shuffle :size="12" />
                </SolaraButton>
              </div>
            </div>

            <div class="sol-create-identity">
              <div class="sol-field">
                <label class="sol-label">Instance Name</label>
                <input v-model="draft.name" type="text" class="sol-input" maxlength="80" />
              </div>
              <div class="sol-field">
                <label class="sol-label">Library Group</label>
                <input v-model="draft.group" type="text" class="sol-input" placeholder="e.g. SMP, Modded" maxlength="60" />
              </div>
            </div>
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Pin to Sidebar</div>
              <div class="sol-muted sol-text-xs">Keep a one-click icon in the left navigation rail.</div>
            </div>
            <SolaraToggle v-model="draft.pinned" label="Pin to sidebar" />
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Favourite Instance</div>
              <div class="sol-muted sol-text-xs">Show under Favourites in your Library filters.</div>
            </div>
            <SolaraToggle v-model="draft.favourite" label="Favourite instance" />
          </div>

          <div class="sol-field">
            <label class="sol-label">Instance Notes</label>
            <textarea
              v-model="draft.notes"
              rows="3"
              class="sol-input"
              placeholder="Server IP, shader settings, or mod notes for this profile..."
            />
          </div>
        </section>

        <!-- Installation -->
        <section v-else-if="store.instanceSettingsCategory === 'installation'" class="sol-stack">
          <div class="sol-kv-grid">
            <div class="sol-kv"><span class="sol-muted">Minecraft Version</span><strong>{{ instance.gameVersion }}</strong></div>
            <div class="sol-kv"><span class="sol-muted">Mod Loader</span><strong>{{ LOADER_LABEL[instance.loader] ?? instance.loader }}</strong></div>
            <div class="sol-kv"><span class="sol-muted">Loader Build</span><strong>{{ instance.loaderVersion || 'Standard' }}</strong></div>
            <div class="sol-kv"><span class="sol-muted">Status</span><strong>{{ instance.state }}</strong></div>
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Repair / Re-verify Files</div>
              <div class="sol-muted sol-text-xs">Re-download missing libraries, natives, or assets with SHA verification.</div>
            </div>
            <SolaraButton variant="secondary" size="sm" :loading="repairing" @click="repairInstance">
              <RefreshCw :size="13" /> Repair Files
            </SolaraButton>
          </div>
        </section>

        <!-- Java & Memory -->
        <section v-else-if="store.instanceSettingsCategory === 'java'" class="sol-stack">
          <div class="sol-field">
            <label class="sol-label">Memory Allocation ({{ draft.memory }} MB)</label>
            <input v-model.number="draft.memory" type="range" min="512" max="16384" step="512" class="sol-range" />
          </div>

          <div class="sol-field">
            <label class="sol-label">Java Executable Override</label>
            <select v-model="draft.javaPath" class="sol-input">
              <option value="auto">Global Default / Automatic</option>
              <option v-for="j in detectedJava" :key="j.path" :value="j.path">
                Java {{ j.major ?? '?' }} ({{ j.vendor }}) — {{ j.path }}
              </option>
            </select>
          </div>

          <div class="sol-field">
            <label class="sol-label">Custom JVM Arguments</label>
            <input
              v-model="draft.jvmArgs"
              type="text"
              class="sol-input"
              placeholder="-XX:+UseG1GC (leave blank to use global defaults)"
            />
          </div>
        </section>

        <!-- Window -->
        <section v-else-if="store.instanceSettingsCategory === 'window'" class="sol-stack">
          <div class="sol-split-2">
            <div class="sol-field">
              <label class="sol-label">Game Width (px)</label>
              <input v-model.number="draft.width" type="number" min="320" max="7680" class="sol-input" />
            </div>
            <div class="sol-field">
              <label class="sol-label">Game Height (px)</label>
              <input v-model.number="draft.height" type="number" min="240" max="4320" class="sol-input" />
            </div>
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Start Minecraft Fullscreen</div>
              <div class="sol-muted sol-text-xs">Overrides global fullscreen behaviour for this instance only.</div>
            </div>
            <SolaraToggle v-model="draft.fullscreen" label="Start fullscreen" />
          </div>
        </section>

        <!-- Launch Hooks -->
        <section v-else-if="store.instanceSettingsCategory === 'hooks'" class="sol-stack">
          <div class="sol-field">
            <label class="sol-label">Pre-Launch Command</label>
            <input v-model="draft.preLaunch" type="text" class="sol-input" placeholder="Optional command run before Java starts" />
          </div>
          <div class="sol-field">
            <label class="sol-label">Post-Exit Command</label>
            <input v-model="draft.postExit" type="text" class="sol-input" placeholder="Optional command run after Minecraft closes" />
          </div>
        </section>

        <!-- Backups -->
        <section v-else-if="store.instanceSettingsCategory === 'backups'" class="sol-stack">
          <div class="sol-row-between">
            <span class="sol-muted sol-text-xs">Create full zip snapshots of this instance's saves and configs.</span>
            <SolaraButton variant="primary" size="xs" :loading="backingUp" @click="createBackup">
              <Archive :size="12" /> Create Backup
            </SolaraButton>
          </div>

          <div v-if="loadingBackups" class="sol-muted sol-text-xs">Loading backups...</div>
          <div v-else-if="backups.length === 0" class="sol-muted sol-text-xs">No backups created for this instance yet.</div>
          <div v-else class="sol-stack">
            <div v-for="b in backups" :key="b.id" class="sol-mini-row">
              <div>
                <strong>{{ b.name }}</strong>
                <div class="sol-muted sol-text-xs">{{ b.createdAt }}</div>
              </div>
              <div class="sol-row-gap">
                <SolaraButton variant="secondary" size="xs" @click="restoreBackup(b)">Restore</SolaraButton>
                <SolaraButton variant="ghost" size="xs" @click="deleteBackup(b)"><Trash2 :size="12" /></SolaraButton>
              </div>
            </div>
          </div>
        </section>

        <!-- Advanced -->
        <section v-else-if="store.instanceSettingsCategory === 'advanced'" class="sol-stack">
          <div class="sol-field">
            <label class="sol-label">Preferred Content Update Channel</label>
            <select v-model="draft.updateChannel" class="sol-input">
              <option value="release">Release only</option>
              <option value="beta">Beta &amp; Release</option>
              <option value="alpha">All (Alpha / Beta / Release)</option>
            </select>
          </div>

          <div class="sol-row-gap">
            <SolaraButton variant="secondary" size="sm" @click="native('open_instance_folder', { id: instance.id }).catch(() => {})">
              <FolderOpen :size="14" /> Open Folder
            </SolaraButton>
            <SolaraButton variant="secondary" size="sm" @click="duplicateInstance">
              <Copy :size="14" /> Duplicate Instance
            </SolaraButton>
          </div>

          <div class="sol-danger-box">
            <div>
              <strong>Delete Instance</strong>
              <p class="sol-muted sol-text-xs">Removes all worlds, mods, configs, and logs inside this instance.</p>
            </div>
            <SolaraButton v-if="!deletePrompt" variant="danger" size="sm" @click="deletePrompt = true">
              <Trash2 :size="13" /> Delete
            </SolaraButton>
            <div v-else class="sol-row-gap">
              <SolaraButton variant="ghost" size="xs" @click="deletePrompt = false">Cancel</SolaraButton>
              <SolaraButton variant="danger" size="xs" :loading="deleting" @click="confirmDeleteInstance">
                Confirm Delete
              </SolaraButton>
            </div>
          </div>
        </section>
      </div>
    </div>

    <template #footer>
      <SolaraSaveBar
        :dirty="isDirty"
        :saving="saving"
        @reset="loadDraft"
        @save="saveChanges"
      />
      <SolaraButton v-if="!isDirty" variant="secondary" @click="handleClose">Close</SolaraButton>
    </template>
  </SolaraModal>
</template>
