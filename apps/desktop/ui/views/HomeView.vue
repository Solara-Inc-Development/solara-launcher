<script setup lang="ts">
import { computed } from 'vue';
import InstanceIcon from '../components/InstanceIcon.vue';
import SolaraButton from '../components/SolaraButton.vue';
import {
  Camera,
  Compass,
  Download,
  FolderOpen,
  Play,
  Plus,
  Shirt,
  Sliders,
  Square,
  User,
} from '../icons';
import {
  activeJobs,
  formatDuration,
  formatRelativeTime,
  launchOrStopInstance,
  LOADER_LABEL,
  navigate,
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

function openInstanceContent(inst: InstanceRecord): void {
  selectInstance(inst.id);
  navigate('instance', { instanceId: inst.id, tab: 'content' });
}

function openInstanceLogs(inst: InstanceRecord): void {
  selectInstance(inst.id);
  navigate('instance', { instanceId: inst.id, tab: 'logs' });
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-home-wrap">
      <!-- Compact Context Header -->
      <header class="sol-home-topbar">
        <div class="sol-row-gap">
          <h1 class="sol-home-heading">
            {{ selectedAccount ? `Welcome back, ${selectedAccount.username}` : 'Welcome to Solara' }}
          </h1>
          <span v-if="store.instances.length > 0" class="sol-badge">
            {{ store.instances.length }} {{ store.instances.length === 1 ? 'instance' : 'instances' }}
          </span>
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
      <section v-if="heroInstance" class="sol-launch-hero">
        <div class="sol-launch-hero-main">
          <InstanceIcon
            :name="heroInstance.name"
            :loader="heroInstance.loader"
            :icon="heroInstance.icon"
            :custom-icon="heroInstance.customIcon"
            :size="68"
            :rounded="15"
            class="sol-clickable"
            @click="openInstanceContent(heroInstance)"
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
        <div class="sol-home-empty-text">
          <h2>Create your first Minecraft profile</h2>
          <p class="sol-muted">
            Start a clean Vanilla or Fabric/NeoForge instance in seconds, or install a curated modpack.
          </p>
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

      <!-- Jump back in: Modrinth-style compact horizontal instance rows -->
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
                {{ LOADER_LABEL[inst.loader] ?? inst.loader }} {{ inst.gameVersion }}
                <template v-if="inst.lastPlayed"> · {{ formatRelativeTime(inst.lastPlayed) }}</template>
              </div>
            </div>

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
      </section>

      <!-- Quick Tools Strip: Modpacks, Skins Studio, Screenshots & Tasks -->
      <section class="sol-home-footer-grid">
        <div class="sol-strip-box">
          <div class="sol-strip-head">
            <span class="sol-strip-label"><Compass :size="14" /> Modpacks &amp; Content</span>
            <button type="button" class="sol-link-btn" @click="navigate('discover')">Browse</button>
          </div>
          <div class="sol-strip-row">
            <div>
              <div class="sol-strip-title">Discover Modpacks &amp; Mods</div>
              <div class="sol-muted sol-text-xs">Verified Modrinth packs, Fabric, Forge, and NeoForge</div>
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
            <span class="sol-strip-label"><Shirt :size="14" /> Studio &amp; Captures</span>
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
              <div class="sol-muted sol-text-xs">Edit skins pixel-by-pixel or browse captures</div>
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
  </div>
</template>
