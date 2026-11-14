<script setup lang="ts">
import { computed, ref } from 'vue';
import InstanceIcon from '../components/InstanceIcon.vue';
import SolaraButton from '../components/SolaraButton.vue';
import {
  Check,
  Compass,
  Copy,
  Download,
  FolderOpen,
  Globe,
  Newspaper,
  Play,
  Plus,
  Server,
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
const copiedIp = ref(false);

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

const networkIp = computed(() => {
  const net = store.solaraBootstrap?.network;
  return String(net?.ip || net?.address || 'play.solaranetwork.in');
});

function openInstanceContent(inst: InstanceRecord): void {
  selectInstance(inst.id);
  navigate('instance', { instanceId: inst.id, tab: 'content' });
}

function openInstanceLogs(inst: InstanceRecord): void {
  selectInstance(inst.id);
  navigate('instance', { instanceId: inst.id, tab: 'logs' });
}

async function copyNetworkIp(): Promise<void> {
  try {
    await navigator.clipboard.writeText(networkIp.value);
    copiedIp.value = true;
    notify('success', 'Server address copied', networkIp.value);
    window.setTimeout(() => {
      copiedIp.value = false;
    }, 1800);
  } catch {
    notify('info', 'Solara Network', networkIp.value);
  }
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

          <button
            v-if="store.preferences.solaraHubEnabled !== false"
            type="button"
            class="sol-acct-pill sol-acct-pill--solara"
            title="Solara Inc Account & Hub"
            @click="navigate('solara')"
          >
            <Globe :size="13" />
            <span>{{ store.solaraProfile.signedIn ? (store.solaraProfile.displayName || 'Solara ID') : 'Solara Hub' }}</span>
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
              <span class="sol-badge sol-badge--gold">
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
                <span class="sol-dot sol-dot--gold" />
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

      <!-- Compact Ecosystem Strip: Modpacks, Solara Network, Announcements, Active Tasks -->
      <section class="sol-home-footer-grid">
        <!-- Quick Discover & Network -->
        <div class="sol-strip-box">
          <div class="sol-strip-head">
            <span class="sol-strip-label"><Server :size="14" /> Solara Network &amp; Hub</span>
            <button type="button" class="sol-link-btn" @click="navigate('solara')">Open Hub</button>
          </div>
          <div class="sol-strip-row">
            <div>
              <div class="sol-strip-title">{{ networkIp }}</div>
              <div class="sol-muted sol-text-xs">Official network realm &amp; local server hosting</div>
            </div>
            <div class="sol-row-gap">
              <SolaraButton variant="secondary" size="xs" @click="copyNetworkIp">
                <Check v-if="copiedIp" :size="12" />
                <Copy v-else :size="12" />
                {{ copiedIp ? 'Copied' : 'Copy IP' }}
              </SolaraButton>
              <SolaraButton variant="ghost" size="xs" @click="navigate('discover')">
                <Compass :size="12" /> Modpacks
              </SolaraButton>
            </div>
          </div>
        </div>

        <!-- Compact Announcements / Active Downloads -->
        <div v-if="store.preferences.announcementsEnabled !== false" class="sol-strip-box">
          <div class="sol-strip-head">
            <span class="sol-strip-label"><Newspaper :size="14" /> Updates &amp; Tasks</span>
            <button
              v-if="activeJobs.length > 0"
              type="button"
              class="sol-link-btn"
              @click="store.showTasksPopover = true"
            >
              <Download :size="12" /> {{ activeJobs.length }} active
            </button>
          </div>

          <div v-if="store.announcements.length > 0" class="sol-news-compact">
            <button
              v-for="item in store.announcements.slice(0, 2)"
              :key="item.id"
              type="button"
              class="sol-news-pill"
              @click="
                store.activeAnnouncement = item;
                store.showAnnouncementsModal = true;
              "
            >
              <span class="sol-news-pill-title sol-truncate">{{ item.title }}</span>
              <span class="sol-muted sol-text-xs">{{ item.date || 'Update' }}</span>
            </button>
          </div>
          <div v-else class="sol-muted sol-text-xs">
            {{ store.announcementsSource === 'loading' ? 'Checking updates from api.solara-inc.in...' : 'You are caught up on Solara updates.' }}
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
