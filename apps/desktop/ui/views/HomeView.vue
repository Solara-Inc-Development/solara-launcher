<script setup lang="ts">
import { computed, ref } from 'vue';
import InstanceIcon from '../components/InstanceIcon.vue';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraLogo from '../components/SolaraLogo.vue';
import SolaraModal from '../components/SolaraModal.vue';
import {
  Camera,
  Clock,
  Compass,
  Download,
  FolderOpen,
  Info,
  Layers,
  Play,
  Plus,
  Shirt,
  Sliders,
  Square,
  Terminal,
  User,
} from '../icons';
import {
  activeJobs,
  formatDuration,
  formatRelativeTime,
  launchOrStopInstance,
  LOADER_LABEL,
  native,
  navigate,
  notify,
  openCreateInstance,
  openInstanceSettings,
  playerAvatarUrl,
  selectedAccount,
  selectedInstance,
  selectInstance,
  store,
} from '../state';
import type { InstanceRecord } from '../types';

const emit = defineEmits<{ (e: 'start-signin'): void }>();

const quickSheetInstanceId = ref<string | null>(null);

const greetingLabel = computed(() => {
  const hour = new Date().getHours();
  if (hour < 12) return 'Good morning';
  if (hour < 18) return 'Good afternoon';
  return 'Good evening';
});

const runningInstancesCount = computed(() => store.instances.filter((i) => i.running).length);

const greetingSubtitle = computed(() => {
  if (runningInstancesCount.value > 0) {
    return `${runningInstancesCount.value} Minecraft ${runningInstancesCount.value === 1 ? 'session' : 'sessions'} currently running`;
  }
  if (activeJobs.value.length > 0) {
    return `${activeJobs.value.length} background ${activeJobs.value.length === 1 ? 'download' : 'downloads'} in progress`;
  }
  if (store.instances.length === 0) {
    return 'Clean OLED Minecraft workspace — create an instance or browse modpacks to begin.';
  }
  return 'Pick up right where you left off or inspect any profile in place.';
});

const jumpBackIn = computed(() => {
  return [...store.instances]
    .sort((a, b) => {
      const aTime = a.lastPlayed ? new Date(a.lastPlayed).getTime() : 0;
      const bTime = b.lastPlayed ? new Date(b.lastPlayed).getTime() : 0;
      if (bTime !== aTime) return bTime - aTime;
      return (b.playtimeSeconds ?? 0) - (a.playtimeSeconds ?? 0);
    })
    .slice(0, 8);
});

const heroInstance = computed<InstanceRecord | null>(() => {
  return selectedInstance.value ?? jumpBackIn.value[0] ?? null;
});

const heroLaunchState = computed(() => {
  if (!heroInstance.value) return null;
  return store.launchStates[heroInstance.value.id] ?? null;
});

const isHeroLaunching = computed(() => {
  const st = heroLaunchState.value;
  return Boolean(st && st.stage !== 'running' && st.stage !== 'error');
});

const quickSheetInstance = computed<InstanceRecord | null>(() => {
  if (!quickSheetInstanceId.value) return null;
  return store.instances.find((i) => i.id === quickSheetInstanceId.value) ?? null;
});

const quickSheetLaunchState = computed(() => {
  if (!quickSheetInstance.value) return null;
  return store.launchStates[quickSheetInstance.value.id] ?? null;
});

function openInstanceContent(inst: InstanceRecord): void {
  quickSheetInstanceId.value = null;
  selectInstance(inst.id);
  navigate('instance', { instanceId: inst.id, tab: 'content' });
}

function openInstanceLogs(inst: InstanceRecord): void {
  quickSheetInstanceId.value = null;
  selectInstance(inst.id);
  navigate('instance', { instanceId: inst.id, tab: 'logs' });
}

function openQuickSheet(inst: InstanceRecord): void {
  selectInstance(inst.id);
  quickSheetInstanceId.value = inst.id;
}

function openSheetSettings(inst: InstanceRecord): void {
  quickSheetInstanceId.value = null;
  openInstanceSettings(inst.id);
}

async function openSheetFolder(inst: InstanceRecord): Promise<void> {
  if (!store.desktop) {
    notify('info', 'Desktop required', 'Opening instance folders is available in the Solara desktop app.');
    return;
  }
  await native('open_instance_folder', { id: inst.id }).catch((err) => {
    notify('error', 'Could not open folder', String(err));
  });
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-home-wrap">
      <!-- Welcoming Context Header -->
      <header class="sol-home-topbar">
        <div class="sol-home-greeting-col">
          <div class="sol-home-heading-row">
            <h1 class="sol-home-heading">
              {{ selectedAccount ? `${greetingLabel}, ${selectedAccount.username}` : `${greetingLabel} — Welcome to Solara` }}
            </h1>
            <span v-if="store.instances.length > 0" class="sol-badge">
              {{ store.instances.length }} {{ store.instances.length === 1 ? 'profile' : 'profiles' }}
            </span>
            <span v-if="runningInstancesCount > 0" class="sol-badge sol-badge--success">
              {{ runningInstancesCount }} running
            </span>
          </div>
          <p class="sol-home-subtitle">{{ greetingSubtitle }}</p>
        </div>

        <div class="sol-row-gap">
          <button
            type="button"
            class="sol-acct-pill"
            :title="selectedAccount ? 'Manage Minecraft accounts' : 'Sign in with Microsoft'"
            @click="selectedAccount ? navigate('account') : emit('start-signin')"
          >
            <img
              v-if="selectedAccount"
              :src="playerAvatarUrl(selectedAccount.username)"
              :alt="selectedAccount.username"
              class="sol-avatar-18"
            />
            <User v-else :size="13" />
            <span>{{ selectedAccount ? selectedAccount.username : 'Sign in (Microsoft)' }}</span>
          </button>

          <SolaraButton variant="primary" size="sm" @click="openCreateInstance()">
            <Plus :size="14" /> New Instance
          </SolaraButton>
        </div>
      </header>

      <!-- Selected Instance Launch Hero -->
      <section v-if="heroInstance" :class="['sol-launch-hero', `sol-launch-hero--${heroInstance.loader}`]">
        <div class="sol-launch-hero-main">
          <InstanceIcon
            :name="heroInstance.name"
            :loader="heroInstance.loader"
            :icon="heroInstance.icon"
            :custom-icon="heroInstance.customIcon"
            :size="72"
            :rounded="16"
            class="sol-clickable"
            @click="openQuickSheet(heroInstance)"
          />

          <div class="sol-launch-hero-info">
            <div class="sol-row-gap">
              <button
                type="button"
                class="sol-hero-title-btn"
                @click="openInstanceContent(heroInstance)"
              >
                {{ heroInstance.name }}
              </button>
              <span :class="['sol-badge', `sol-badge--${heroInstance.loader}`]">
                {{ LOADER_LABEL[heroInstance.loader] ?? heroInstance.loader }} {{ heroInstance.gameVersion }}
              </span>
              <span v-if="heroInstance.running" class="sol-badge sol-badge--success">Running</span>
            </div>

            <div class="sol-launch-hero-meta">
              <span>{{ formatDuration(heroInstance.playtimeSeconds) }}</span>
              <span>·</span>
              <span>{{ heroInstance.lastPlayed ? `Played ${formatRelativeTime(heroInstance.lastPlayed)}` : 'Ready to launch' }}</span>
              <span>·</span>
              <span>Account: {{ selectedAccount?.username ?? 'Not signed in' }}</span>
            </div>

            <!-- Real-time launch stage bar -->
            <div v-if="heroLaunchState && (isHeroLaunching || heroLaunchState.stage === 'error')" class="sol-launch-stage">
              <template v-if="heroLaunchState.stage === 'error'">
                <span class="sol-launch-stage-err">{{ heroLaunchState.error || heroLaunchState.detail }}</span>
                <button type="button" class="sol-link-btn" @click="openInstanceLogs(heroInstance)">
                  View Logs
                </button>
              </template>
              <template v-else>
                <span class="sol-dot sol-dot--accent" />
                <span>{{ heroLaunchState.detail }}</span>
              </template>
            </div>
          </div>
        </div>

        <div class="sol-launch-hero-actions">
          <SolaraButton
            variant="ghost"
            size="md"
            title="Quick Profile Overview"
            @click="openQuickSheet(heroInstance)"
          >
            <Info :size="16" /> Quick Info
          </SolaraButton>
          <SolaraButton
            variant="secondary"
            size="md"
            @click="openInstanceContent(heroInstance)"
          >
            <FolderOpen :size="16" /> Content
          </SolaraButton>
          <SolaraButton
            variant="ghost"
            size="md"
            title="Instance Settings"
            @click="openInstanceSettings(heroInstance.id)"
          >
            <Sliders :size="16" />
          </SolaraButton>
          <SolaraButton
            :variant="heroInstance.running ? 'danger' : 'primary'"
            size="lg"
            :loading="isHeroLaunching"
            @click="launchOrStopInstance(heroInstance)"
          >
            <Square v-if="heroInstance.running" :size="15" />
            <Play v-else :size="17" />
            {{
              heroInstance.running
                ? 'Stop'
                : isHeroLaunching
                  ? heroLaunchState?.detail || 'Launching...'
                  : 'Play'
            }}
          </SolaraButton>
        </div>
      </section>

      <!-- Empty Instance Onboarding CTA -->
      <section v-else class="sol-home-empty">
        <div class="sol-row-gap">
          <SolaraLogo :size="48" />
          <div class="sol-home-empty-text">
            <h2>Create your first Minecraft profile</h2>
            <p class="sol-muted">
              Start a clean Vanilla, Fabric, Forge, or NeoForge instance in seconds, or install a curated modpack.
            </p>
          </div>
        </div>
        <div class="sol-row-gap">
          <SolaraButton variant="primary" size="md" @click="openCreateInstance()">
            <Plus :size="15" /> Create Instance
          </SolaraButton>
          <SolaraButton variant="secondary" size="md" @click="navigate('discover')">
            <Compass :size="15" /> Discover Modpacks
          </SolaraButton>
        </div>
      </section>

      <!-- Jump back in: compact frosted horizontal instance cards -->
      <section v-if="jumpBackIn.length > 0" class="sol-home-section">
        <div class="sol-section-head">
          <h2 class="sol-section-title">Jump back in</h2>
          <button
            v-if="store.instances.length > 1"
            type="button"
            class="sol-link-btn"
            @click="navigate('library')"
          >
            View all {{ store.instances.length }}
          </button>
        </div>

        <div class="sol-jump-grid">
          <div
            v-for="inst in jumpBackIn"
            :key="inst.id"
            :class="[
              'sol-jump-item',
              { 'is-selected': heroInstance?.id === inst.id, 'is-running': inst.running },
            ]"
            role="button"
            tabindex="0"
            @click="openInstanceContent(inst)"
            @keydown.enter="openInstanceContent(inst)"
          >
            <InstanceIcon
              :name="inst.name"
              :loader="inst.loader"
              :icon="inst.icon"
              :custom-icon="inst.customIcon"
              :size="46"
              :rounded="11"
            />

            <div class="sol-jump-info">
              <div class="sol-jump-name sol-truncate">{{ inst.name }}</div>
              <div class="sol-jump-sub sol-truncate">
                <span :class="['sol-loader-dot', `sol-loader-dot--${inst.loader}`]" />
                {{ LOADER_LABEL[inst.loader] ?? inst.loader }} {{ inst.gameVersion }}
                <template v-if="inst.lastPlayed"> · {{ formatRelativeTime(inst.lastPlayed) }}</template>
              </div>
            </div>

            <div class="sol-jump-actions">
              <button
                type="button"
                class="sol-jump-info-btn"
                :title="`Quick info for ${inst.name}`"
                :aria-label="`Quick info for ${inst.name}`"
                @click.stop="openQuickSheet(inst)"
              >
                <Info :size="14" />
              </button>
              <button
                type="button"
                :class="['sol-jump-play', { 'is-running': inst.running }]"
                :title="inst.running ? 'Stop Minecraft' : `Play ${inst.name}`"
                :aria-label="inst.running ? `Stop ${inst.name}` : `Play ${inst.name}`"
                @click.stop="launchOrStopInstance(inst)"
              >
                <Square v-if="inst.running" :size="15" />
                <Play v-else :size="16" />
              </button>
            </div>
          </div>
        </div>
      </section>

      <!-- Quick Tools Strip: Modpacks, Skins Studio, Screenshots & Tasks -->
      <section class="sol-home-footer-grid">
        <div class="sol-strip-box">
          <div class="sol-strip-head">
            <span class="sol-strip-label">
              <span class="sol-strip-icon-pill"><Compass :size="13" /></span>
              Modpacks &amp; Content
            </span>
            <button type="button" class="sol-link-btn" @click="navigate('discover')">Browse</button>
          </div>
          <div class="sol-strip-row">
            <div>
              <div class="sol-strip-title">Discover Modpacks &amp; Mods</div>
              <div class="sol-muted sol-text-xs">Curated modpacks &amp; mods for Fabric, Forge, NeoForge, and Quilt</div>
            </div>
            <div class="sol-row-gap">
              <SolaraButton variant="secondary" size="xs" @click="navigate('discover')">
                <Compass :size="12" /> Explore
              </SolaraButton>
            </div>
          </div>
        </div>

        <div class="sol-strip-box">
          <div class="sol-strip-head">
            <span class="sol-strip-label">
              <span class="sol-strip-icon-pill sol-strip-icon-pill--amber"><Shirt :size="13" /></span>
              Studio &amp; Captures
            </span>
            <button
              v-if="activeJobs.length > 0"
              type="button"
              class="sol-link-btn"
              @click="store.showTasksPopover = true"
            >
              <Download :size="12" /> {{ activeJobs.length }} active
            </button>
          </div>
          <div class="sol-strip-row">
            <div>
              <div class="sol-strip-title">3D Skins, Capes &amp; Screenshots</div>
              <div class="sol-muted sol-text-xs">Edit skins pixel-by-pixel or browse in-game captures</div>
            </div>
            <div class="sol-row-gap">
              <SolaraButton variant="secondary" size="xs" @click="navigate('skins')">
                <Shirt :size="12" /> Skins
              </SolaraButton>
              <SolaraButton variant="ghost" size="xs" @click="navigate('screenshots')">
                <Camera :size="12" /> Shots
              </SolaraButton>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- In-Place Instance Quick Profile Sheet -->
    <SolaraModal
      :open="Boolean(quickSheetInstance)"
      variant="sheet"
      width="md"
      :title="quickSheetInstance?.name ?? 'Instance Overview'"
      :subtitle="quickSheetInstance ? `${LOADER_LABEL[quickSheetInstance.loader] ?? quickSheetInstance.loader} ${quickSheetInstance.gameVersion}` : ''"
      @close="quickSheetInstanceId = null"
    >
      <template v-if="quickSheetInstance">
        <div class="sol-stack">
          <div class="sol-row-between">
            <div class="sol-row-gap">
              <InstanceIcon
                :name="quickSheetInstance.name"
                :loader="quickSheetInstance.loader"
                :icon="quickSheetInstance.icon"
                :custom-icon="quickSheetInstance.customIcon"
                :size="54"
                :rounded="13"
              />
              <div>
                <div class="sol-row-gap">
                  <span :class="['sol-badge', `sol-badge--${quickSheetInstance.loader}`]">
                    {{ LOADER_LABEL[quickSheetInstance.loader] ?? quickSheetInstance.loader }}
                  </span>
                  <span class="sol-badge">MC {{ quickSheetInstance.gameVersion }}</span>
                  <span v-if="quickSheetInstance.running" class="sol-badge sol-badge--success">Running</span>
                </div>
                <p class="sol-muted sol-text-xs" style="margin: 6px 0 0">
                  {{ quickSheetInstance.loaderVersion ? `Loader ${quickSheetInstance.loaderVersion}` : 'Managed vanilla runtime' }}
                </p>
              </div>
            </div>
          </div>

          <div class="sol-sheet-stats">
            <div class="sol-sheet-stat">
              <span class="sol-sheet-stat-label">Playtime</span>
              <span class="sol-sheet-stat-val">{{ formatDuration(quickSheetInstance.playtimeSeconds) }}</span>
            </div>
            <div class="sol-sheet-stat">
              <span class="sol-sheet-stat-label">Last Played</span>
              <span class="sol-sheet-stat-val">
                {{ quickSheetInstance.lastPlayed ? formatRelativeTime(quickSheetInstance.lastPlayed) : 'Never yet' }}
              </span>
            </div>
            <div class="sol-sheet-stat">
              <span class="sol-sheet-stat-label">Account</span>
              <span class="sol-sheet-stat-val sol-truncate">{{ selectedAccount?.username ?? 'Not signed in' }}</span>
            </div>
          </div>

          <div v-if="quickSheetLaunchState?.stage === 'error'" class="sol-banner-warn">
            <span>{{ quickSheetLaunchState.error || quickSheetLaunchState.detail }}</span>
            <SolaraButton variant="secondary" size="xs" @click="openInstanceLogs(quickSheetInstance)">
              <Terminal :size="12" /> Logs
            </SolaraButton>
          </div>

          <div>
            <div class="sol-label" style="margin-bottom: 8px">Quick Actions</div>
            <div class="sol-sheet-action-grid">
              <button type="button" class="sol-sheet-action-tile" @click="openInstanceContent(quickSheetInstance)">
                <span class="sol-sheet-action-icon"><Layers :size="16" /></span>
                <span>
                  <strong>Browse Content</strong>
                  <span>Mods, resource packs &amp; shaders</span>
                </span>
              </button>
              <button type="button" class="sol-sheet-action-tile" @click="openInstanceLogs(quickSheetInstance)">
                <span class="sol-sheet-action-icon"><Terminal :size="16" /></span>
                <span>
                  <strong>Logs &amp; Console</strong>
                  <span>Live output &amp; crash assistant</span>
                </span>
              </button>
              <button type="button" class="sol-sheet-action-tile" @click="openSheetSettings(quickSheetInstance)">
                <span class="sol-sheet-action-icon"><Sliders :size="16" /></span>
                <span>
                  <strong>Instance Settings</strong>
                  <span>Memory, Java, JVM &amp; window</span>
                </span>
              </button>
              <button type="button" class="sol-sheet-action-tile" @click="openSheetFolder(quickSheetInstance)">
                <span class="sol-sheet-action-icon"><Clock :size="16" /></span>
                <span>
                  <strong>Open Folder</strong>
                  <span>Saves, config &amp; resourcepacks</span>
                </span>
              </button>
            </div>
          </div>
        </div>
      </template>

      <template #footer>
        <SolaraButton variant="ghost" @click="quickSheetInstanceId = null">Close</SolaraButton>
        <SolaraButton
          v-if="quickSheetInstance"
          :variant="quickSheetInstance.running ? 'danger' : 'primary'"
          @click="launchOrStopInstance(quickSheetInstance); quickSheetInstanceId = null"
        >
          <Square v-if="quickSheetInstance.running" :size="14" />
          <Play v-else :size="14" />
          {{ quickSheetInstance.running ? 'Stop Minecraft' : 'Play Instance' }}
        </SolaraButton>
      </template>
    </SolaraModal>
  </div>
</template>
