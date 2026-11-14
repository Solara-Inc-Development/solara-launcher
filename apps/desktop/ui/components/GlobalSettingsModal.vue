<script setup lang="ts">
import { computed, onUnmounted, reactive, ref, watch } from 'vue';
import SolaraButton from './SolaraButton.vue';
import SolaraLogo from './SolaraLogo.vue';
import SolaraModal from './SolaraModal.vue';
import SolaraSaveBar from './SolaraSaveBar.vue';
import SolaraToggle from './SolaraToggle.vue';
import {
  Cpu,
  Download,
  FolderOpen,
  Globe,
  HardDrive,
  Info,
  Maximize2,
  Palette,
  RefreshCw,
  Shield,
  Sliders,
  Sparkles,
  Trash2,
  User,
} from '../icons';
import {
  applyAppearance,
  registerUnsavedChanges,
  runWithDirtyGuard,
  DEFAULT_PREFERENCES,
  formatBytes,
  native,
  navigate,
  notify,
  playerAvatarUrl,
  refreshUpdateStatus,
  savePreferencesFull,
  store,
} from '../state';
import build from '../../../../packages/shared/build.json';
import type {
  DetectedJava,
  LauncherPreferences,
  SettingsCategoryId,
  StorageSummary,
} from '../types';

const emit = defineEmits<{ (e: 'start-signin'): void }>();

const categories: Array<{ id: SettingsCategoryId; label: string; icon: unknown }> = [
  { id: 'appearance', label: 'Appearance', icon: Palette },
  { id: 'features', label: 'Features', icon: Sparkles },
  { id: 'behaviour', label: 'Behaviour', icon: Sliders },
  { id: 'accounts', label: 'Accounts', icon: User },
  { id: 'java', label: 'Java & Memory', icon: Cpu },
  { id: 'window', label: 'Window', icon: Maximize2 },
  { id: 'downloads', label: 'Downloads', icon: Download },
  { id: 'storage', label: 'Storage', icon: HardDrive },
  { id: 'privacy', label: 'Privacy', icon: Shield },
  { id: 'resources', label: 'Resources', icon: Globe },
  { id: 'troubleshooting', label: 'Troubleshooting', icon: Shield },
  { id: 'about', label: 'About Solara', icon: Info },
];

const draft = reactive<LauncherPreferences>({ ...DEFAULT_PREFERENCES });
const detectedJava = ref<DetectedJava[]>([]);
const storage = ref<StorageSummary | null>(null);
const saving = ref(false);
const clearingCache = ref(false);
const checkingUpdate = ref(false);
const newGroup = ref('');

function syncFromStore(): void {
  Object.assign(draft, { ...DEFAULT_PREFERENCES, ...store.preferences });
  if (draft.onLaunch !== 'minimize') draft.onLaunch = 'keep';
  applyAppearance(draft);
}

const isDirty = computed(() => {
  const baseline = {
    ...DEFAULT_PREFERENCES,
    ...store.preferences,
    onLaunch: store.preferences.onLaunch === 'minimize' ? 'minimize' : 'keep',
  };
  return JSON.stringify(draft) !== JSON.stringify(baseline);
});

async function loadSystemInfo(): Promise<void> {
  if (!store.desktop) return;
  try {
    const [javaList, storageInfo] = await Promise.all([
      native<DetectedJava[]>('detect_java_runtimes').catch(() => []),
      native<StorageSummary>('launcher_storage_summary').catch(() => null),
    ]);
    detectedJava.value = Array.isArray(javaList) ? javaList : [];
    storage.value = storageInfo;
  } catch {
    // ignore
  }
}

async function saveChanges(): Promise<void> {
  saving.value = true;
  try {
    await savePreferencesFull({ ...draft, groups: [...draft.groups], prefsVersion: 2 });
    syncFromStore();
    notify('success', 'Settings saved', 'Your launcher preferences have been updated.');
  } catch (err) {
    notify('error', "Couldn't save settings", String(err));
  } finally {
    saving.value = false;
  }
}

async function clearCache(): Promise<void> {
  clearingCache.value = true;
  try {
    const result = await native<{ freedBytes: number }>('clear_launcher_cache');
    storage.value = await native<StorageSummary>('launcher_storage_summary');
    notify('success', 'Cache cleaned', `${formatBytes(result.freedBytes)} of temporary download files removed.`);
  } catch (err) {
    notify('error', 'Cache clean failed', String(err));
  } finally {
    clearingCache.value = false;
  }
}

async function exportDiagnosticsFile(): Promise<void> {
  try {
    const report = await native<Record<string, unknown>>('export_diagnostics');
    const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `solara-diagnostics-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
    notify('success', 'Diagnostics exported', 'Redacted JSON report saved.');
  } catch (err) {
    notify('error', 'Export failed', String(err));
  }
}

async function runUpdateCheck(): Promise<void> {
  checkingUpdate.value = true;
  try {
    if (store.desktop) {
      await native('check_update');
      await refreshUpdateStatus();
    }
    notify('info', 'Update check complete', `Status: ${store.updateStatus.stage}`);
  } catch (err) {
    notify('error', "Couldn't check for updates", String(err));
  } finally {
    checkingUpdate.value = false;
  }
}

function addGroup(): void {
  const name = newGroup.value.trim();
  if (!name) return;
  if (!draft.groups.includes(name)) {
    draft.groups = [...draft.groups, name];
  }
  newGroup.value = '';
}

function removeGroup(name: string): void {
  draft.groups = draft.groups.filter((g) => g !== name);
}

const unregisterGuard = registerUnsavedChanges(
  () => store.showSettingsModal && isDirty.value,
  syncFromStore,
);
onUnmounted(unregisterGuard);
function handleClose(): void {
  if (saving.value) return;
  runWithDirtyGuard(() => { store.showSettingsModal = false; });
}

watch(
  () => ({ ...draft }),
  (val) => {
    applyAppearance(val as LauncherPreferences);
  },
  { deep: true },
);

watch(
  () => store.showSettingsModal,
  (isOpen) => {
    if (isOpen) {
      syncFromStore();
      void loadSystemInfo();
    }
  },
);
</script>

<template>
  <SolaraModal
    :open="store.showSettingsModal"
    title="Launcher Settings"
    subtitle="Solara Dawn Gold — Obsidian Edition"
    width="xl"
    :closable="!saving"
    @close="handleClose"
  >
    <div class="sol-settings-dialog">
      <!-- Left Category Sidebar -->
      <aside class="sol-settings-dialog-nav sol-scroll">
        <button
          v-for="cat in categories"
          :key="cat.id"
          type="button"
          :class="['sol-settings-nav-btn', { 'is-active': store.settingsModalCategory === cat.id }]"
          @click="store.settingsModalCategory = cat.id"
        >
          <component :is="cat.icon" :size="15" />
          <span>{{ cat.label }}</span>
        </button>
      </aside>

      <!-- Right Content Area -->
      <div class="sol-settings-dialog-body sol-scroll">
        <!-- Appearance -->
        <section v-if="store.settingsModalCategory === 'appearance'" class="sol-stack">
          <div class="sol-field">
            <label class="sol-label">Theme Identity</label>
            <div class="sol-chip-row">
              <button
                v-for="t in [
                  { id: 'dawn-gold', label: 'OLED Black + Dawn Gold' },
                  { id: 'midnight', label: 'Deep Obsidian' },
                  { id: 'slate', label: 'Obsidian Slate' },
                ] as const"
                :key="t.id"
                type="button"
                :class="['sol-chip', { 'is-active': draft.theme === t.id }]"
                @click="draft.theme = t.id"
              >
                {{ t.label }}
              </button>
            </div>
          </div>

          <div class="sol-split-2">
            <div class="sol-field">
              <label class="sol-label">Interface Density</label>
              <div class="sol-chip-row">
                <button
                  type="button"
                  :class="['sol-chip', { 'is-active': draft.density === 'comfortable' }]"
                  @click="draft.density = 'comfortable'"
                >
                  Comfortable
                </button>
                <button
                  type="button"
                  :class="['sol-chip', { 'is-active': draft.density === 'compact' }]"
                  @click="draft.density = 'compact'"
                >
                  Compact
                </button>
              </div>
            </div>

            <div class="sol-field">
              <label class="sol-label">UI Scale ({{ draft.uiScale }}%)</label>
              <input v-model.number="draft.uiScale" type="range" min="90" max="115" step="5" class="sol-range" />
            </div>
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Reduced Motion</div>
              <div class="sol-muted sol-text-xs">Disable subtle hover transitions and startup slide effects.</div>
            </div>
            <SolaraToggle v-model="draft.reducedMotion" label="Reduced motion" />
          </div>
        </section>

        <!-- Features -->
        <section v-else-if="store.settingsModalCategory === 'features'" class="sol-stack">
          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Solara Hub &amp; Local Server Tab</div>
              <div class="sol-muted sol-text-xs">Show Solara Inc hosting tools, services, and local servers in the sidebar rail.</div>
            </div>
            <SolaraToggle v-model="draft.solaraHubEnabled" label="Enable Solara Hub" />
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Home News &amp; Announcements Strip</div>
              <div class="sol-muted sol-text-xs">Show release notes and network news from api.solara-inc.in on Home.</div>
            </div>
            <SolaraToggle v-model="draft.announcementsEnabled" label="Enable announcements" />
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Automatic Background Update Checks</div>
              <div class="sol-muted sol-text-xs">Safely check for new Solara Launcher releases while idle.</div>
            </div>
            <SolaraToggle v-model="draft.autoUpdateEnabled" label="Auto updates" />
          </div>
        </section>

        <!-- Behaviour -->
        <section v-else-if="store.settingsModalCategory === 'behaviour'" class="sol-stack">
          <div class="sol-field">
            <label class="sol-label">After Launching Minecraft</label>
            <div class="sol-chip-row">
              <button
                type="button"
                :class="['sol-chip', { 'is-active': draft.onLaunch === 'keep' }]"
                @click="draft.onLaunch = 'keep'"
              >
                Keep Launcher Open (Recommended)
              </button>
              <button
                type="button"
                :class="['sol-chip', { 'is-active': draft.onLaunch === 'minimize' }]"
                @click="draft.onLaunch = 'minimize'"
              >
                Minimize Launcher
              </button>
            </div>
            <p class="sol-muted sol-text-xs">
              Solara stays open and interactive by default so you can inspect live logs or manage instances while playing.
            </p>
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Confirm Before Launching</div>
              <div class="sol-muted sol-text-xs">Prompt before starting Minecraft from a Play button.</div>
            </div>
            <SolaraToggle v-model="draft.confirmLaunch" label="Confirm launch" />
          </div>
        </section>

        <!-- Accounts -->
        <section v-else-if="store.settingsModalCategory === 'accounts'" class="sol-stack">
          <div class="sol-row-between">
            <span class="sol-muted sol-text-xs">Microsoft Minecraft accounts stay strictly in your OS credential vault.</span>
            <SolaraButton variant="primary" size="xs" @click="store.showSettingsModal = false; emit('start-signin')">
              Add Microsoft Account
            </SolaraButton>
          </div>

          <div v-if="store.accounts.length === 0" class="sol-muted sol-text-xs">No Microsoft accounts connected yet.</div>
          <div v-for="acc in store.accounts" :key="acc.id" class="sol-mini-row">
            <div class="sol-row-gap">
              <img :src="playerAvatarUrl(acc.username)" :alt="acc.username" class="sol-avatar-24" />
              <strong>{{ acc.username }}</strong>
              <span v-if="acc.selected" class="sol-badge sol-badge--success">Active</span>
            </div>
            <SolaraButton variant="ghost" size="xs" @click="store.showSettingsModal = false; navigate('account')">
              Manage
            </SolaraButton>
          </div>

          <div class="sol-mini-row">
            <div>
              <strong>Solara Inc Identity</strong>
              <div class="sol-muted sol-text-xs">
                {{ store.solaraProfile.signedIn ? `Signed in as ${store.solaraProfile.displayName || store.solaraProfile.email}` : 'Not connected (optional for Solara Hub services)' }}
              </div>
            </div>
            <SolaraButton variant="secondary" size="xs" @click="store.showSettingsModal = false; navigate('solara')">
              Open Solara Hub
            </SolaraButton>
          </div>
        </section>

        <!-- Java & Memory -->
        <section v-else-if="store.settingsModalCategory === 'java'" class="sol-stack">
          <div class="sol-field">
            <label class="sol-label">Default Memory Allocation ({{ draft.memory }} MB)</label>
            <input v-model.number="draft.memory" type="range" min="512" max="16384" step="512" class="sol-range" />
          </div>

          <div class="sol-field">
            <label class="sol-label">Default Java Executable</label>
            <select v-model="draft.javaPath" class="sol-input">
              <option value="auto">Automatic (Solara Managed Runtime)</option>
              <option v-for="j in detectedJava" :key="j.path" :value="j.path">
                Java {{ j.major ?? '?' }} ({{ j.vendor }}) — {{ j.path }}
              </option>
            </select>
          </div>

          <div class="sol-field">
            <label class="sol-label">Default JVM Arguments</label>
            <input v-model="draft.jvmArgs" type="text" class="sol-input" placeholder="Optional extra flags applied to instances" />
          </div>
        </section>

        <!-- Window -->
        <section v-else-if="store.settingsModalCategory === 'window'" class="sol-stack">
          <div class="sol-split-2">
            <div class="sol-field">
              <label class="sol-label">Default Width (px)</label>
              <input v-model.number="draft.defaultWidth" type="number" min="320" max="7680" class="sol-input" />
            </div>
            <div class="sol-field">
              <label class="sol-label">Default Height (px)</label>
              <input v-model.number="draft.defaultHeight" type="number" min="240" max="4320" class="sol-input" />
            </div>
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Default Fullscreen</div>
              <div class="sol-muted sol-text-xs">Start Minecraft fullscreen unless overridden per instance.</div>
            </div>
            <SolaraToggle v-model="draft.fullscreen" label="Default fullscreen" />
          </div>
        </section>

        <!-- Downloads -->
        <section v-else-if="store.settingsModalCategory === 'downloads'" class="sol-stack">
          <div class="sol-field">
            <label class="sol-label">Max Concurrent Downloads ({{ draft.maxConcurrentDownloads }})</label>
            <input v-model.number="draft.maxConcurrentDownloads" type="range" min="1" max="8" step="1" class="sol-range" />
          </div>
          <p class="sol-muted sol-text-xs">
            Every Mojang jar, Modrinth release, asset, and Java runtime is verified against SHA-1/SHA-512 checksums before activation.
          </p>
        </section>

        <!-- Storage -->
        <section v-else-if="store.settingsModalCategory === 'storage'" class="sol-stack">
          <div class="sol-row-between">
            <span class="sol-muted sol-text-xs">Disk usage breakdown across instances, cached artifacts, and runtimes.</span>
            <SolaraButton variant="secondary" size="xs" @click="native('open_data_folder').catch(() => {})">
              <FolderOpen :size="13" /> Open Data Folder
            </SolaraButton>
          </div>

          <div v-if="storage" class="sol-kv-grid">
            <div class="sol-kv"><span class="sol-muted">Total Launcher Data</span><strong>{{ formatBytes(storage.totalBytes) }}</strong></div>
            <div class="sol-kv"><span class="sol-muted">Instances &amp; Worlds</span><strong>{{ formatBytes(storage.instancesBytes) }}</strong></div>
            <div class="sol-kv"><span class="sol-muted">Verified Cache</span><strong>{{ formatBytes(storage.cacheBytes) }}</strong></div>
            <div class="sol-kv"><span class="sol-muted">Java Runtimes</span><strong>{{ formatBytes(storage.javaBytes) }}</strong></div>
          </div>

          <div>
            <SolaraButton variant="secondary" size="sm" :loading="clearingCache" @click="clearCache">
              <Trash2 :size="14" /> Clean Unused Cache Files
            </SolaraButton>
          </div>
        </section>

        <!-- Privacy -->
        <section v-else-if="store.settingsModalCategory === 'privacy'" class="sol-stack">
          <div class="sol-banner-info">
            <Shield :size="15" />
            <span>
              Microsoft refresh tokens stay in your OS credential vault. Sign-in tokens are used only with official Microsoft, Xbox, and Minecraft services. They are never sent to Solara.
            </span>
          </div>

          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Local Redacted Diagnostics Only</div>
              <div class="sol-muted sol-text-xs">Automatically strip tokens, local usernames, and secrets before writing support diagnostic bundles.</div>
            </div>
            <SolaraToggle v-model="draft.diagnosticsEnabled" label="Local diagnostics" />
          </div>
        </section>

        <!-- Resources & Groups -->
        <section v-else-if="store.settingsModalCategory === 'resources'" class="sol-stack">
          <div class="sol-field">
            <label class="sol-label">Library Instance Groups</label>
            <form class="sol-row-gap" @submit.prevent="addGroup">
              <input v-model="newGroup" type="text" class="sol-input" placeholder="Create group (e.g. Optimization, SMP)..." maxlength="40" />
              <SolaraButton variant="secondary" size="sm" type="submit">Add</SolaraButton>
            </form>
            <div class="sol-chip-row" style="margin-top: 8px">
              <span v-for="g in draft.groups" :key="g" class="sol-badge sol-badge--gold">
                {{ g }}
                <button type="button" class="sol-tag-x" @click="removeGroup(g)">×</button>
              </span>
            </div>
          </div>
        </section>

        <!-- Troubleshooting -->
        <section v-else-if="store.settingsModalCategory === 'troubleshooting'" class="sol-stack">
          <div class="sol-setting-row">
            <div>
              <div class="sol-setting-label">Local Redacted Diagnostics</div>
              <div class="sol-muted sol-text-xs">Allow exporting sanitized hardware, instance, and Java reports without secrets or tokens.</div>
            </div>
            <SolaraToggle v-model="draft.diagnosticsEnabled" label="Local diagnostics" />
          </div>

          <div class="sol-row-gap">
            <SolaraButton variant="secondary" size="sm" @click="exportDiagnosticsFile">
              <Download :size="14" /> Export Diagnostics (.json)
            </SolaraButton>
            <SolaraButton variant="ghost" size="sm" @click="native('open_data_folder').catch(() => {})">
              <FolderOpen :size="14" /> Open Data Folder
            </SolaraButton>
          </div>
        </section>

        <!-- About -->
        <section v-else-if="store.settingsModalCategory === 'about'" class="sol-stack">
          <div class="sol-about-hero">
            <SolaraLogo :size="48" />
            <div>
              <h3 class="sol-about-title">Solara Launcher</h3>
              <p class="sol-muted sol-text-xs">
                Independent Minecraft: Java Edition Launcher · Dawn Gold OLED Edition
              </p>
            </div>
          </div>

          <div class="sol-kv-grid">
            <div class="sol-kv"><span class="sol-muted">Release Track</span><strong>{{ build.publicVersion }} ({{ build.internalVersion }})</strong></div>
            <div class="sol-kv"><span class="sol-muted">Update Status</span><strong>{{ store.updateStatus?.stage ?? 'idle' }}</strong></div>
            <div class="sol-kv"><span class="sol-muted">API Origin</span><strong>api.solara-inc.in</strong></div>
            <div class="sol-kv"><span class="sol-muted">Website</span><strong>app.solara-inc.in</strong></div>
          </div>

          <div class="sol-row-gap">
            <SolaraButton variant="primary" size="sm" :loading="checkingUpdate" @click="runUpdateCheck">
              <RefreshCw :size="13" /> Check for Updates
            </SolaraButton>
          </div>
        </section>
      </div>
    </div>

    <template #footer>
      <SolaraSaveBar :dirty="isDirty" :saving="saving" @reset="syncFromStore" @save="saveChanges" />
      <SolaraButton v-if="!isDirty" variant="secondary" @click="handleClose">Close</SolaraButton>
    </template>
  </SolaraModal>
</template>
