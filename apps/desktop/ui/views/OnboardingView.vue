<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import {
  CheckCircle2,
  Play,
  ArrowRight,
  ArrowLeft,
  User,
  Cpu,
  Shield,
  Sparkles,
} from '../icons';
import {
  store,
  selectedAccount,
  selectedInstance,
  loadMinecraftVersions,
  savePreferencesFull,
  refreshSnapshot,
  launchOrStopInstance,
  playerAvatarUrl,
  native,
  notify,
  navigate,
} from '../state';
import type { DetectedJava, LoaderKind } from '../types';
import SolaraButton from '../components/SolaraButton.vue';
import InstanceIcon from '../components/InstanceIcon.vue';

const emit = defineEmits<{ (e: 'start-signin'): void }>();

const step = ref(1);
const memory = ref(store.preferences.memory || 4096);
const javaPath = ref(store.preferences.javaPath || 'auto');
const theme = ref(store.preferences.theme || 'dawn-gold');
const detectedJava = ref<DetectedJava[]>([]);
const scanningJava = ref(false);

const firstInstanceName = ref('Survival 1.21');
const firstVersion = ref('');
const firstLoader = ref<LoaderKind>('fabric');
const creating = ref(false);
const createError = ref('');

const releaseVersions = computed(() =>
  store.mcVersions.filter((v) => v.type === 'release').slice(0, 60),
);

onMounted(async () => {
  try {
    await loadMinecraftVersions();
    if (!firstVersion.value && releaseVersions.value.length > 0) {
      firstVersion.value = releaseVersions.value[0].id;
      firstInstanceName.value = `Minecraft ${firstVersion.value}`;
    }
  } catch {
    // offline fallback handled gracefully
  }
  scanJava();
});

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

async function finishSetupAndCreate() {
  createError.value = '';
  creating.value = true;
  try {
    await savePreferencesFull({
      ...store.preferences,
      memory: memory.value,
      javaPath: javaPath.value || 'auto',
      theme: theme.value,
    });
    if (store.desktop && firstVersion.value && store.instances.length === 0) {
      const created = await native<{ instanceId: string }>('create_instance', {
        name: firstInstanceName.value.trim() || `Minecraft ${firstVersion.value}`,
        gameVersion: firstVersion.value,
        loader: firstLoader.value,
        loaderVersion: '',
        options: {
          icon: 'preset:dawn-crest',
          memory: memory.value,
          javaPath: javaPath.value || 'auto',
        },
      });
      await refreshSnapshot();
      store.selectedInstanceId = created.instanceId;
    }
    step.value = 4;
  } catch (err) {
    createError.value = String(err);
  } finally {
    creating.value = false;
  }
}

async function completeOnboarding(launchNow = false) {
  try {
    await savePreferencesFull({
      ...store.preferences,
      memory: memory.value,
      javaPath: javaPath.value || 'auto',
      theme: theme.value,
      onboardingCompleted: true,
    });
  } catch {
    store.preferences.onboardingCompleted = true;
  }
  if (launchNow && selectedInstance.value && selectedInstance.value.state === 'installed') {
    await launchOrStopInstance(selectedInstance.value);
  }
  navigate('home');
}
</script>

<template>
  <div class="sol-page sol-onboarding-wrap sol-scroll">
    <div class="sol-onboarding-card">
      <!-- Step Indicator -->
      <div class="sol-stepper" aria-label="Setup steps">
        <div
          v-for="s in [
            { n: 1, label: 'Welcome' },
            { n: 2, label: 'Account' },
            { n: 3, label: 'Setup' },
            { n: 4, label: 'Ready' },
          ]"
          :key="s.n"
          :class="['sol-step', { 'is-active': step === s.n, 'is-done': step > s.n }]"
        >
          <span class="sol-step-num">
            <CheckCircle2 v-if="step > s.n" :size="13" />
            <template v-else>{{ s.n }}</template>
          </span>
          <span class="sol-step-label">{{ s.label }}</span>
        </div>
      </div>

      <!-- Step 1: Welcome -->
      <section v-if="step === 1" class="sol-onboarding-pane">
        <div class="sol-onboarding-brand">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M12 2L20.5 7V17L12 22L3.5 17V7L12 2Z" stroke="#E9B86D" stroke-width="1.8" />
            <circle cx="12" cy="12" r="3.5" fill="#E9B86D" />
          </svg>
          <div>
            <h1 class="sol-onboarding-title">Welcome to Solara Launcher</h1>
            <p class="sol-onboarding-sub">
              A fast, native Minecraft Java Edition launcher built around clean instance isolation,
              mod management, and verified account security.
            </p>
          </div>
        </div>

        <div class="sol-onboarding-highlights">
          <div class="sol-mini-feature">
            <Shield :size="16" class="sol-accent" />
            <div>
              <div class="sol-mini-feature-title">Direct Microsoft Auth</div>
              <div class="sol-mini-feature-desc">Credentials stay on your device via your system browser and Windows Credential Vault.</div>
            </div>
          </div>
          <div class="sol-mini-feature">
            <Cpu :size="16" class="sol-accent" />
            <div>
              <div class="sol-mini-feature-title">Automatic Java & Loaders</div>
              <div class="sol-mini-feature-desc">First-class Vanilla, Fabric, Quilt, Forge, and NeoForge runtime management.</div>
            </div>
          </div>
          <div class="sol-mini-feature">
            <Sparkles :size="16" class="sol-accent" />
            <div>
              <div class="sol-mini-feature-title">Content, Worlds, Skins & Logs</div>
              <div class="sol-mini-feature-desc">Browse Modrinth content, inspect worlds, edit files, and customize 3D skins.</div>
            </div>
          </div>
        </div>

        <div class="sol-onboarding-footer">
          <SolaraButton variant="ghost" @click="completeOnboarding(false)">
            Skip setup
          </SolaraButton>
          <SolaraButton variant="primary" @click="step = 2">
            Continue <ArrowRight :size="15" />
          </SolaraButton>
        </div>
      </section>

      <!-- Step 2: Microsoft Account -->
      <section v-else-if="step === 2" class="sol-onboarding-pane">
        <h2 class="sol-section-title">Microsoft Account</h2>
        <p class="sol-section-desc">
          Sign in with your Microsoft account owning Minecraft: Java Edition. Solara opens your system
          browser using OAuth PKCE — your password never enters the launcher.
        </p>

        <div v-if="selectedAccount" class="sol-account-verified-card">
          <img
            :src="playerAvatarUrl(selectedAccount.username)"
            :alt="selectedAccount.username"
            class="sol-avatar-44"
          />
          <div class="sol-account-verified-info">
            <div class="sol-account-name">{{ selectedAccount.username }}</div>
            <div class="sol-badge sol-badge--completed">Minecraft Java Ownership Verified</div>
          </div>
        </div>

        <div v-else class="sol-signin-box">
          <User :size="20" class="sol-accent" />
          <div class="sol-signin-copy">
            <div class="sol-signin-heading">No account connected yet</div>
            <div class="sol-signin-sub">Official Microsoft sign-in is required to launch Minecraft.</div>
          </div>
          <SolaraButton variant="primary" @click="emit('start-signin')">
            Sign in with Microsoft
          </SolaraButton>
        </div>

        <div class="sol-onboarding-footer">
          <SolaraButton variant="ghost" @click="step = 1">
            <ArrowLeft :size="15" /> Back
          </SolaraButton>
          <div class="sol-row-gap">
            <SolaraButton v-if="!selectedAccount" variant="secondary" @click="step = 3">
              Sign in later
            </SolaraButton>
            <SolaraButton variant="primary" @click="step = 3">
              Continue <ArrowRight :size="15" />
            </SolaraButton>
          </div>
        </div>
      </section>

      <!-- Step 3: Initial Setup -->
      <section v-else-if="step === 3" class="sol-onboarding-pane">
        <h2 class="sol-section-title">Initial Setup</h2>
        <p class="sol-section-desc">
          Choose default Java, memory allocation, theme, and create your first Minecraft instance.
        </p>

        <div class="sol-form-grid">
          <div class="sol-field">
            <label class="sol-label">Default Memory Allocation ({{ memory }} MB)</label>
            <input
              v-model.number="memory"
              type="range"
              min="1024"
              max="16384"
              step="512"
              class="sol-range"
            />
            <div class="sol-range-hints">
              <span>1 GB</span>
              <span>4 GB (Recommended)</span>
              <span>16 GB</span>
            </div>
          </div>

          <div class="sol-field">
            <label class="sol-label">Java Runtime</label>
            <select v-model="javaPath" class="sol-input">
              <option value="auto">Automatic (Solara Managed Java)</option>
              <option v-for="j in detectedJava" :key="j.path" :value="j.path">
                Java {{ j.major ?? '?' }} ({{ j.vendor }}) — {{ j.path }}
              </option>
            </select>
          </div>

          <div class="sol-field">
            <label class="sol-label">Appearance Variant</label>
            <div class="sol-chip-row">
              <button
                v-for="opt in [
                  { id: 'dawn-gold', label: 'Dawn Gold Obsidian' },
                  { id: 'slate', label: 'Obsidian Slate' },
                  { id: 'midnight', label: 'Midnight Gold' },
                ] as const"
                :key="opt.id"
                type="button"
                :class="['sol-chip', { 'is-active': theme === opt.id }]"
                @click="theme = opt.id"
              >
                {{ opt.label }}
              </button>
            </div>
          </div>

          <div class="sol-field">
            <label class="sol-label">First Instance</label>
            <div class="sol-split-2">
              <input
                v-model="firstInstanceName"
                type="text"
                class="sol-input"
                placeholder="Instance name"
                maxlength="80"
              />
              <select v-model="firstVersion" class="sol-input">
                <option value="" disabled>Select version...</option>
                <option v-for="v in releaseVersions" :key="v.id" :value="v.id">
                  Minecraft {{ v.id }}
                </option>
              </select>
            </div>
            <div class="sol-chip-row" style="margin-top: 8px">
              <button
                v-for="ldr in (['vanilla', 'fabric', 'neoforge', 'forge', 'quilt'] as LoaderKind[])"
                :key="ldr"
                type="button"
                :class="['sol-chip', { 'is-active': firstLoader === ldr }]"
                @click="firstLoader = ldr"
              >
                {{ ldr }}
              </button>
            </div>
          </div>
        </div>

        <p v-if="createError" class="sol-inline-error">{{ createError }}</p>

        <div class="sol-onboarding-footer">
          <SolaraButton variant="ghost" @click="step = 2">
            <ArrowLeft :size="15" /> Back
          </SolaraButton>
          <SolaraButton variant="primary" :loading="creating" @click="finishSetupAndCreate">
            Create & Finish Setup <ArrowRight :size="15" />
          </SolaraButton>
        </div>
      </section>

      <!-- Step 4: Ready -->
      <section v-else class="sol-onboarding-pane">
        <h2 class="sol-section-title">Everything is Ready</h2>
        <p class="sol-section-desc">
          Solara is configured and ready. You can enter the launcher dashboard now while any instance files finish installing.
        </p>

        <div class="sol-ready-summary">
          <div class="sol-ready-row">
            <span class="sol-muted">Account</span>
            <span v-if="selectedAccount" class="sol-row-gap">
              <img :src="playerAvatarUrl(selectedAccount.username)" :alt="selectedAccount.username" class="sol-avatar-20" />
              <strong>{{ selectedAccount.username }}</strong>
            </span>
            <span v-else class="sol-muted">Sign in when ready</span>
          </div>

          <div v-if="selectedInstance" class="sol-ready-row">
            <span class="sol-muted">Instance</span>
            <span class="sol-row-gap">
              <InstanceIcon
                :name="selectedInstance.name"
                :loader="selectedInstance.loader"
                :icon="selectedInstance.icon"
                :size="24"
                :rounded="6"
              />
              <strong>{{ selectedInstance.name }}</strong>
              <span class="sol-badge">{{ selectedInstance.gameVersion }} {{ selectedInstance.loader }}</span>
            </span>
          </div>
        </div>

        <div class="sol-onboarding-footer">
          <SolaraButton variant="ghost" @click="step = 3">
            <ArrowLeft :size="15" /> Back
          </SolaraButton>
          <div class="sol-row-gap">
            <SolaraButton
              v-if="selectedInstance?.state === 'installed'"
              variant="secondary"
              @click="completeOnboarding(true)"
            >
              <Play :size="14" /> Launch Game
            </SolaraButton>
            <SolaraButton variant="primary" @click="completeOnboarding(false)">
              Enter Solara Launcher
            </SolaraButton>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
