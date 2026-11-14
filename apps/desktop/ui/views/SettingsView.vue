<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import {
  FolderOpen,
  Trash2,
  RefreshCw,
  Download,
  User,
  Shield,
  HardDrive,
} from '../icons';
import {
  store,
  savePreferencesFull,
  applyAppearance,
  navigate,
  formatBytes,
  playerAvatarUrl,
  native,
  notify,
  DEFAULT_PREFERENCES,
} from '../state';
import type { DetectedJava, LauncherPreferences, StorageSummary } from '../types';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraToggle from '../components/SolaraToggle.vue';
import SolaraSaveBar from '../components/SolaraSaveBar.vue';

const emit = defineEmits<{ (e: 'start-signin'): void }>();

type SettingsCategory =
  | 'appearance'
  | 'behaviour'
  | 'java'
  | 'window'
  | 'downloads'
  | 'storage'
  | 'accounts'
  | 'privacy'
  | 'troubleshooting';

const categories: { id: SettingsCategory; label: string }[] = [
  { id: 'appearance', label: 'Appearance' },
  { id: 'behaviour', label: 'Behaviour' },
  { id: 'java', label: 'Java & Memory' },
  { id: 'window', label: 'Game Window' },
  { id: 'downloads', label: 'Downloads' },
  { id: 'storage', label: 'Storage & Cache' },
  { id: 'accounts', label: 'Accounts' },
  { id: 'privacy', label: 'Privacy & Discord' },
  { id: 'troubleshooting', label: 'Troubleshooting' },
];

const activeCategory = ref<SettingsCategory>('appearance');
const draft = ref<LauncherPreferences>({ ...DEFAULT_PREFERENCES, ...store.preferences });
const baseline = ref(JSON.stringify(draft.value));
const saving = ref(false);

const detectedJava = ref<DetectedJava[]>([]);
const scanningJava = ref(false);
const storage = ref<StorageSummary | null>(null);
const clearingCache = ref(false);

const isDirty = computed(() => JSON.stringify(draft.value) !== baseline.value);

function syncFromStore() {
  draft.value = { ...DEFAULT_PREFERENCES, ...store.preferences, groups: [...(store.preferences.groups || [])] };
  baseline.value = JSON.stringify(draft.value);
}

function resetChanges() {
  syncFromStore();
  applyAppearance(store.preferences);
}

async function saveChanges() {
  saving.value = true;
  try {
    await savePreferencesFull(draft.value);
    syncFromStore();
    notify('success', 'Settings saved', 'All launcher preferences updated.');
  } catch (err) {
    notify('error', 'Failed to save settings', String(err));
  } finally {
    saving.value = false;
  }
}

async function scanJava() {
  if (!store.desktop) return;
  scanningJava.value = true;
  try {
    detectedJava.value = await native<DetectedJava[]>('detect_java_runtimes');
  } catch {
    detectedJava.value = [];
  } finally {
    scanningJava.value = false;
  }
}

async function loadStorage() {
  if (!store.desktop) return;
  try {
    storage.value = await native<StorageSummary>('launcher_storage_summary');
  } catch {
    storage.value = null;
  }
}

async function clearCache() {
  clearingCache.value = true;
  try {
    const res = await native<{ freedBytes: number }>('clear_launcher_cache');
    await loadStorage();
    notify('success', 'Cache cleared', `Freed ${formatBytes(res.freedBytes)} of temporary files.`);
  } catch (err) {
    notify('error', 'Could not clear cache', String(err));
  } finally {
    clearingCache.value = false;
  }
}

async function exportDiagnosticsFile() {
  try {
    const data = await native<unknown>('export_diagnostics');
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `solara-diagnostics-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
    notify('success', 'Diagnostics exported', 'Tokens and credentials were excluded automatically.');
  } catch (err) {
    notify('error', 'Export failed', String(err));
  }
}

onMounted(() => {
  syncFromStore();
  store.dirtyChecker = () => isDirty.value;
  scanJava();
  loadStorage();
});

onUnmounted(() => {
  store.dirtyChecker = null;
});
</script>

<template>
  <div class="sol-page sol-page--column">
    <div class="sol-settings-layout">
      <!-- Fixed Left Category Navigation -->
      <aside class="sol-settings-sidebar sol-scroll">
        <div class="sol-settings-heading">Settings</div>
        <button
          v-for="cat in categories"
          :key="cat.id"
          type="button"
          :class="['sol-settings-nav-item', { 'is-active': activeCategory === cat.id }]"
          @click="activeCategory = cat.id"
        >
          {{ cat.label }}
        </button>
      </aside>

      <!-- Independent Scrollable Settings Content -->
      <main class="sol-settings-content sol-scroll">
        <div class="sol-settings-inner">
          <!-- Appearance -->
          <section v-if="activeCategory === 'appearance'" class="sol-card sol-stack">
            <h2 class="sol-card-title">Appearance</h2>
            <p class="sol-muted sol-text-xs">Solara Dawn Gold — Obsidian Edition theme variants and layout density.</p>

            <div class="sol-field">
              <label class="sol-label">Theme Variant</label>
              <div class="sol-chip-row">
                <button
                  v-for="t in [
                    { id: 'dawn-gold', label: 'Dawn Gold Obsidian' },
                    { id: 'slate', label: 'Obsidian Slate' },
                    { id: 'midnight', label: 'Midnight Gold' },
                  ] as const"
                  :key="t.id"
                  type="button"
                  :class="['sol-chip', { 'is-active': draft.theme === t.id }]"
                  @click="
                    draft.theme = t.id;
                    applyAppearance(draft);
                  "
                >
                  {{ t.label }}
                </button>
              </div>
            </div>

            <div class="sol-field">
              <label class="sol-label">UI Scale ({{ draft.uiScale }}%)</label>
              <input
                v-model.number="draft.uiScale"
                type="range"
                min="85"
                max="120"
                step="5"
                class="sol-range"
                @input="applyAppearance(draft)"
              />
            </div>

            <div class="sol-setting-row">
              <div>
                <div class="sol-setting-label">Interface Density</div>
                <div class="sol-muted sol-text-xs">Compact tightens spacing inside lists and cards.</div>
              </div>
              <div class="sol-seg-group">
                <button
                  type="button"
                  :class="['sol-seg-btn', { 'is-active': draft.density === 'comfortable' }]"
                  @click="
                    draft.density = 'comfortable';
                    applyAppearance(draft);
                  "
                >
                  Comfortable
                </button>
                <button
                  type="button"
                  :class="['sol-seg-btn', { 'is-active': draft.density === 'compact' }]"
                  @click="
                    draft.density = 'compact';
                    applyAppearance(draft);
                  "
                >
                  Compact
                </button>
              </div>
            </div>

            <div class="sol-setting-row">
              <div>
                <div class="sol-setting-label">Expand Sidebar Labels</div>
                <div class="sol-muted sol-text-xs">Show text labels beside icons in the left navigation rail.</div>
              </div>
              <SolaraToggle v-model="draft.sidebarExpanded" label="Expand sidebar labels" />
            </div>

            <div class="sol-setting-row">
              <div>
                <div class="sol-setting-label">Reduced Motion</div>
                <div class="sol-muted sol-text-xs">Disable transition animations across views and dialogs.</div>
              </div>
              <SolaraToggle
                :model-value="draft.reducedMotion"
                label="Reduced motion"
                @update:model-value="
                  draft.reducedMotion = $event;
                  applyAppearance(draft);
                "
              />
            </div>
          </section>

          <!-- Behaviour -->
          <section v-else-if="activeCategory === 'behaviour'" class="sol-card sol-stack">
            <h2 class="sol-card-title">Launcher Behaviour</h2>
            <div class="sol-setting-row">
              <div>
                <div class="sol-setting-label">When Minecraft Launches</div>
                <div class="sol-muted sol-text-xs">Choose what happens to the Solara window after starting a game.</div>
              </div>
              <select v-model="draft.onLaunch" class="sol-select">
                <option value="keep">Keep window open</option>
                <option value="minimize">Minimize to taskbar</option>
                <option value="hide">Hide window</option>
              </select>
            </div>

            <div class="sol-setting-row">
              <div>
                <div class="sol-setting-label">Confirm Before Launching</div>
                <div class="sol-muted sol-text-xs">Ask for confirmation before starting an instance.</div>
              </div>
              <SolaraToggle v-model="draft.confirmLaunch" label="Confirm launch" />
            </div>
          </section>

          <!-- Java & Memory -->
          <section v-else-if="activeCategory === 'java'" class="sol-card sol-stack">
            <div class="sol-side-head">
              <h2 class="sol-card-title">Default Java & Memory</h2>
              <SolaraButton variant="secondary" size="xs" :loading="scanningJava" @click="scanJava">
                <RefreshCw :size="12" /> Scan Java
              </SolaraButton>
            </div>

            <div class="sol-field">
              <label class="sol-label">Default Memory Allocation ({{ draft.memory }} MB)</label>
              <input v-model.number="draft.memory" type="range" min="512" max="16384" step="512" class="sol-range" />
              <div class="sol-range-hints">
                <span>512 MB</span>
                <span>4096 MB (Recommended)</span>
                <span>16384 MB</span>
              </div>
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
              <label class="sol-label">Default Global JVM Arguments</label>
              <input
                v-model="draft.jvmArgs"
                type="text"
                class="sol-input"
                placeholder="Optional extra JVM flags applied unless overridden per instance"
              />
            </div>

            <div v-if="detectedJava.length > 0" class="sol-stack">
              <div class="sol-label">Detected Java Installations ({{ detectedJava.length }})</div>
              <div v-for="j in detectedJava" :key="j.path" class="sol-mini-row">
                <div class="sol-truncate">
                  <strong>Java {{ j.major ?? '?' }}</strong> ({{ j.vendor }}) — <span class="sol-muted sol-text-xs">{{ j.path }}</span>
                </div>
                <SolaraButton variant="ghost" size="xs" @click="draft.javaPath = j.path">Use</SolaraButton>
              </div>
            </div>
          </section>

          <!-- Window -->
          <section v-else-if="activeCategory === 'window'" class="sol-card sol-stack">
            <h2 class="sol-card-title">Default Game Window</h2>
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
                <div class="sol-muted sol-text-xs">Start Minecraft fullscreen for instances without custom window overrides.</div>
              </div>
              <SolaraToggle v-model="draft.fullscreen" label="Default fullscreen" />
            </div>
          </section>

          <!-- Downloads -->
          <section v-else-if="activeCategory === 'downloads'" class="sol-card sol-stack">
            <h2 class="sol-card-title">Downloads & Integrity</h2>
            <div class="sol-field">
              <label class="sol-label">Concurrent Downloads ({{ draft.maxConcurrentDownloads }})</label>
              <input
                v-model.number="draft.maxConcurrentDownloads"
                type="range"
                min="1"
                max="8"
                step="1"
                class="sol-range"
              />
            </div>
            <p class="sol-muted sol-text-xs">
              All game jars, libraries, mods, and packs are SHA-1 / SHA-512 verified before moving into instance folders.
            </p>
          </section>

          <!-- Storage & Cache -->
          <section v-else-if="activeCategory === 'storage'" class="sol-card sol-stack">
            <div class="sol-side-head">
              <h2 class="sol-card-title sol-inline-flex"><HardDrive :size="16" /> Storage & Resources</h2>
              <SolaraButton variant="secondary" size="xs" @click="native('open_data_folder').catch(() => {})">
                <FolderOpen :size="13" /> Open Data Folder
              </SolaraButton>
            </div>

            <div v-if="storage" class="sol-kv-grid">
              <div class="sol-kv"><span class="sol-muted">Total Launcher Data</span><strong>{{ formatBytes(storage.totalBytes) }}</strong></div>
              <div class="sol-kv"><span class="sol-muted">Instances & Worlds</span><strong>{{ formatBytes(storage.instancesBytes) }}</strong></div>
              <div class="sol-kv"><span class="sol-muted">Verified File Cache</span><strong>{{ formatBytes(storage.cacheBytes) }}</strong></div>
              <div class="sol-kv"><span class="sol-muted">Java Runtimes</span><strong>{{ formatBytes(storage.javaBytes) }}</strong></div>
            </div>

            <div>
              <SolaraButton variant="secondary" size="sm" :loading="clearingCache" @click="clearCache">
                <Trash2 :size="14" /> Clear Unused Temp & Cache Files
              </SolaraButton>
            </div>
          </section>

          <!-- Accounts -->
          <section v-else-if="activeCategory === 'accounts'" class="sol-card sol-stack">
            <div class="sol-side-head">
              <h2 class="sol-card-title">Minecraft Accounts</h2>
              <SolaraButton variant="primary" size="xs" @click="emit('start-signin')">
                <User :size="13" /> Add Microsoft Account
              </SolaraButton>
            </div>

            <div v-if="store.accounts.length === 0" class="sol-muted sol-text-xs">
              No Microsoft Minecraft accounts connected yet.
            </div>
            <div v-for="acc in store.accounts" :key="acc.id" class="sol-mini-row">
              <div class="sol-row-gap">
                <img :src="playerAvatarUrl(acc.username)" :alt="acc.username" class="sol-avatar-24" />
                <strong>{{ acc.username }}</strong>
                <span v-if="acc.selected" class="sol-badge sol-badge--success">Active</span>
              </div>
              <SolaraButton variant="ghost" size="xs" @click="navigate('account')">Manage</SolaraButton>
            </div>
          </section>

          <!-- Privacy -->
          <section v-else-if="activeCategory === 'privacy'" class="sol-card sol-stack">
            <h2 class="sol-card-title sol-inline-flex"><Shield :size="16" /> Privacy & Credentials</h2>
            <p class="sol-muted sol-text-xs">
              Microsoft and Minecraft access tokens are stored strictly inside your operating system's credential vault and never enter launcher logs or web storage.
            </p>
            <div class="sol-setting-row">
              <div>
                <div class="sol-setting-label">Discord Rich Presence</div>
                <div class="sol-muted sol-text-xs">Share active Minecraft status with Discord when running.</div>
              </div>
              <SolaraToggle v-model="draft.discord" label="Discord presence" />
            </div>
            <div class="sol-setting-row">
              <div>
                <div class="sol-setting-label">Local Diagnostic Collection</div>
                <div class="sol-muted sol-text-xs">Allow generating redacted diagnostic JSON bundles locally.</div>
              </div>
              <SolaraToggle v-model="draft.diagnosticsEnabled" label="Local diagnostics" />
            </div>
          </section>

          <!-- Troubleshooting -->
          <section v-else-if="activeCategory === 'troubleshooting'" class="sol-card sol-stack">
            <h2 class="sol-card-title">Troubleshooting & Diagnostics</h2>
            <p class="sol-muted sol-text-xs">
              Export a safe, redacted diagnostic report containing your OS architecture, instance summary, Java installations, and storage stats.
            </p>
            <div class="sol-row-gap">
              <SolaraButton variant="secondary" size="sm" @click="exportDiagnosticsFile">
                <Download :size="14" /> Export Redacted Diagnostics (.json)
              </SolaraButton>
              <SolaraButton variant="ghost" size="sm" @click="native('open_data_folder').catch(() => {})">
                <FolderOpen :size="14" /> Open Launcher Folder
              </SolaraButton>
            </div>
          </section>
        </div>
      </main>
    </div>

    <SolaraSaveBar
      :dirty="isDirty"
      :saving="saving"
      @reset="resetChanges"
      @save="saveChanges"
    />
  </div>
</template>
