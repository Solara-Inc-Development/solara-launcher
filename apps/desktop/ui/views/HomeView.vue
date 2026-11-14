<script setup lang="ts">
import { computed } from 'vue';
import {
  Play,
  Square,
  Plus,
  Settings,
  User,
  Newspaper,
  FolderOpen,
  ArrowUpRight,
  Clock,
  CheckCircle2,
  AlertTriangle,
  Info,
  RefreshCw,
} from '../icons';
import {
  store,
  selectedAccount,
  selectedInstance,
  activeJobs,
  navigate,
  selectInstance,
  openCreateInstance,
  launchOrStopInstance,
  loadAnnouncements,
  playerAvatarUrl,
  formatRelativeTime,
  formatDuration,
  formatBytes,
  LOADER_LABEL,
  native,
  notify,
} from '../state';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraProgress from '../components/SolaraProgress.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import InstanceIcon from '../components/InstanceIcon.vue';

const emit = defineEmits<{ (e: 'start-signin'): void }>();

const recentInstances = computed(() => {
  return [...store.instances]
    .sort((a, b) => {
      const ta = a.lastPlayed ? Date.parse(a.lastPlayed) : 0;
      const tb = b.lastPlayed ? Date.parse(b.lastPlayed) : 0;
      if (tb !== ta) return tb - ta;
      const ca = a.createdAt ? Date.parse(a.createdAt) : 0;
      const cb = b.createdAt ? Date.parse(b.createdAt) : 0;
      return cb - ca;
    })
    .slice(0, 6);
});

const selectedInstallJob = computed(() => {
  const inst = selectedInstance.value;
  if (!inst) return null;
  return activeJobs.value.find((j) => j.instance === inst.id) ?? null;
});

const recentActivity = computed(() => {
  const fromJobs = store.downloads.slice(-6).reverse().map((j) => ({
    id: `job-${j.id}`,
    title: j.name,
    detail: j.phase || j.status,
    tone:
      j.status === 'completed'
        ? ('success' as const)
        : j.status === 'failed'
          ? ('error' as const)
          : j.status === 'cancelled'
            ? ('warning' as const)
            : ('info' as const),
    time: j.createdAt || '',
  }));
  const combined = [
    ...store.activityFeed.slice(0, 6).map((a) => ({
      id: a.id,
      title: a.title,
      detail: formatRelativeTime(a.time),
      tone: a.tone,
      time: a.time,
    })),
    ...fromJobs,
  ];
  return combined.slice(0, 8);
});

async function retryInstall(id: string) {
  try {
    await native('retry_instance', { id });
    notify('info', 'Retrying installation', 'Download queued.');
  } catch (err) {
    notify('error', "Couldn't retry install", String(err));
  }
}

function openAnnouncement(item: (typeof store.announcements)[number]) {
  store.activeAnnouncement = item;
  store.showAnnouncementsModal = true;
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-page-inner sol-home-layout">
      <!-- Left Main Column -->
      <div class="sol-home-main">
        <!-- Primary Launch Section -->
        <section class="sol-card sol-launch-card">
          <template v-if="selectedInstance">
            <div class="sol-launch-top">
              <div class="sol-launch-identity">
                <InstanceIcon
                  :name="selectedInstance.name"
                  :loader="selectedInstance.loader"
                  :icon="selectedInstance.icon"
                  :custom-icon="selectedInstance.customIcon"
                  :size="56"
                  :rounded="14"
                />
                <div class="sol-launch-meta">
                  <div class="sol-launch-title-row">
                    <h1 class="sol-launch-title">{{ selectedInstance.name }}</h1>
                    <span
                      v-if="selectedInstance.running"
                      class="sol-badge sol-badge--success"
                    >Running</span>
                    <span
                      v-else-if="selectedInstance.state !== 'installed'"
                      :class="['sol-badge', `sol-badge--${selectedInstance.state}`]"
                    >{{ selectedInstance.state }}</span>
                  </div>
                  <div class="sol-launch-specs">
                    <span>Minecraft {{ selectedInstance.gameVersion }}</span>
                    <span>•</span>
                    <span>{{ LOADER_LABEL[selectedInstance.loader] || selectedInstance.loader }}</span>
                    <template v-if="selectedInstance.loaderVersion">
                      <span>{{ selectedInstance.loaderVersion }}</span>
                    </template>
                    <span>•</span>
                    <span>{{ selectedInstance.memory }} MB RAM</span>
                    <template v-if="selectedInstance.playtimeSeconds">
                      <span>•</span>
                      <span class="sol-inline-flex"><Clock :size="12" /> {{ formatDuration(selectedInstance.playtimeSeconds) }}</span>
                    </template>
                  </div>
                </div>
              </div>

              <div class="sol-launch-cta">
                <SolaraButton
                  v-if="selectedInstance.state === 'installed'"
                  :variant="selectedInstance.running ? 'danger' : 'primary'"
                  size="lg"
                  @click="launchOrStopInstance(selectedInstance)"
                >
                  <Square v-if="selectedInstance.running" :size="15" />
                  <Play v-else :size="15" />
                  <span>{{ selectedInstance.running ? 'Stop' : 'Play' }}</span>
                </SolaraButton>
                <SolaraButton
                  v-else-if="selectedInstance.state === 'failed' || selectedInstance.state === 'cancelled'"
                  variant="primary"
                  size="lg"
                  @click="retryInstall(selectedInstance.id)"
                >
                  <RefreshCw :size="15" />
                  <span>Retry Install</span>
                </SolaraButton>
                <SolaraButton
                  v-else
                  variant="secondary"
                  size="lg"
                  disabled
                >
                  Installing...
                </SolaraButton>

                <button
                  type="button"
                  class="sol-icon-btn"
                  title="Instance settings"
                  aria-label="Open instance settings"
                  @click="navigate('instance', { instanceId: selectedInstance.id, tab: 'settings' })"
                >
                  <Settings :size="16" />
                </button>
                <button
                  type="button"
                  class="sol-icon-btn"
                  title="Open instance details"
                  aria-label="Open instance details"
                  @click="navigate('instance', { instanceId: selectedInstance.id, tab: 'overview' })"
                >
                  <ArrowUpRight :size="16" />
                </button>
              </div>
            </div>

            <div v-if="selectedInstallJob" class="sol-launch-progress">
              <div class="sol-launch-progress-row">
                <span>{{ selectedInstallJob.phase || 'Installing...' }}</span>
                <span v-if="selectedInstallJob.total > 0">
                  {{ formatBytes(selectedInstallJob.done) }} / {{ formatBytes(selectedInstallJob.total) }}
                </span>
              </div>
              <SolaraProgress
                :value="selectedInstallJob.done"
                :max="selectedInstallJob.total"
                :indeterminate="selectedInstallJob.total <= 0"
              />
            </div>

            <div class="sol-launch-footer">
              <div class="sol-launch-account-hint">
                <template v-if="selectedAccount">
                  <img
                    :src="playerAvatarUrl(selectedAccount.username)"
                    :alt="selectedAccount.username"
                    class="sol-avatar-20"
                  />
                  <span>Playing as <strong>{{ selectedAccount.username }}</strong></span>
                </template>
                <template v-else>
                  <User :size="14" class="sol-warning" />
                  <span>No Microsoft account selected —</span>
                  <button type="button" class="sol-link-btn" @click="emit('start-signin')">
                    Sign in now
                  </button>
                </template>
              </div>
              <span class="sol-muted">
                Last played: {{ formatRelativeTime(selectedInstance.lastPlayed) }}
              </span>
            </div>
          </template>

          <SolaraEmptyState
            v-else
            title="No Minecraft instances yet"
            description="Create your first Vanilla, Fabric, Quilt, Forge, or NeoForge instance to start playing."
          >
            <template #actions>
              <SolaraButton variant="primary" @click="openCreateInstance()">
                <Plus :size="15" /> Create Instance
              </SolaraButton>
              <SolaraButton variant="secondary" @click="navigate('discover')">
                Browse Modpacks
              </SolaraButton>
            </template>
          </SolaraEmptyState>
        </section>

        <!-- Recent Instances -->
        <section class="sol-section">
          <div class="sol-section-header">
            <h2 class="sol-section-heading">Instances</h2>
            <div class="sol-row-gap">
              <SolaraButton variant="ghost" size="xs" @click="openCreateInstance()">
                <Plus :size="13" /> New Instance
              </SolaraButton>
              <SolaraButton variant="ghost" size="xs" @click="navigate('library')">
                Open Library
              </SolaraButton>
            </div>
          </div>

          <div v-if="recentInstances.length === 0" class="sol-card sol-muted-box">
            Instances you create or import will appear here for quick access.
          </div>

          <div v-else class="sol-recent-grid">
            <div
              v-for="inst in recentInstances"
              :key="inst.id"
              :class="['sol-card', 'sol-recent-card', { 'is-selected': selectedInstance?.id === inst.id }]"
              @click="selectInstance(inst.id)"
              @dblclick="navigate('instance', { instanceId: inst.id, tab: 'overview' })"
            >
              <div class="sol-recent-top">
                <InstanceIcon
                  :name="inst.name"
                  :loader="inst.loader"
                  :icon="inst.icon"
                  :custom-icon="inst.customIcon"
                  :size="36"
                  :rounded="9"
                />
                <div class="sol-recent-info">
                  <div class="sol-recent-name sol-truncate">{{ inst.name }}</div>
                  <div class="sol-recent-sub">
                    {{ inst.gameVersion }} • {{ LOADER_LABEL[inst.loader] || inst.loader }}
                  </div>
                </div>
              </div>
              <div class="sol-recent-bottom">
                <span class="sol-muted sol-text-xs">
                  {{ inst.lastPlayed ? `Played ${formatRelativeTime(inst.lastPlayed)}` : 'Ready' }}
                </span>
                <div class="sol-row-gap">
                  <button
                    type="button"
                    class="sol-icon-btn sol-icon-btn--xs"
                    title="Open instance"
                    @click.stop="navigate('instance', { instanceId: inst.id, tab: 'overview' })"
                  >
                    <FolderOpen :size="13" />
                  </button>
                  <button
                    v-if="inst.state === 'installed'"
                    type="button"
                    class="sol-icon-btn sol-icon-btn--xs sol-icon-btn--gold"
                    :title="inst.running ? 'Stop' : 'Play'"
                    @click.stop="launchOrStopInstance(inst)"
                  >
                    <Square v-if="inst.running" :size="12" />
                    <Play v-else :size="12" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>

      <!-- Right Secondary Column -->
      <aside class="sol-home-side">
        <!-- Account Card -->
        <section class="sol-card sol-side-card">
          <div class="sol-side-head">
            <h2 class="sol-side-title">Account</h2>
            <button type="button" class="sol-link-btn" @click="navigate('account')">
              Manage
            </button>
          </div>
          <div v-if="selectedAccount" class="sol-home-account">
            <img
              :src="playerAvatarUrl(selectedAccount.username)"
              :alt="selectedAccount.username"
              class="sol-avatar-40"
            />
            <div class="sol-home-account-info">
              <div class="sol-home-account-name">{{ selectedAccount.username }}</div>
              <div class="sol-badge sol-badge--success">Java Ownership Verified</div>
            </div>
          </div>
          <div v-else class="sol-home-account-empty">
            <p class="sol-muted">Sign in with Microsoft to play Java Edition and customize skins or capes.</p>
            <SolaraButton variant="primary" size="sm" block @click="emit('start-signin')">
              Sign in with Microsoft
            </SolaraButton>
          </div>
        </section>

        <!-- Recent Activity -->
        <section class="sol-card sol-side-card">
          <div class="sol-side-head">
            <h2 class="sol-side-title">Recent Activity</h2>
            <button type="button" class="sol-link-btn" @click="navigate('tasks')">
              Tasks
            </button>
          </div>
          <div v-if="recentActivity.length === 0" class="sol-muted sol-text-xs">
            No launcher activity recorded in this session yet.
          </div>
          <ul v-else class="sol-activity-list">
            <li v-for="act in recentActivity" :key="act.id" class="sol-activity-item">
              <CheckCircle2 v-if="act.tone === 'success'" :size="14" class="sol-success" />
              <AlertTriangle v-else-if="act.tone === 'error' || act.tone === 'warning'" :size="14" class="sol-warning" />
              <Info v-else :size="14" class="sol-info" />
              <div class="sol-activity-copy">
                <div class="sol-activity-title sol-truncate">{{ act.title }}</div>
                <div class="sol-activity-sub">{{ act.detail }}</div>
              </div>
            </li>
          </ul>
        </section>

        <!-- Compact Announcements -->
        <section class="sol-card sol-side-card">
          <div class="sol-side-head">
            <h2 class="sol-side-title sol-inline-flex"><Newspaper :size="14" /> Announcements</h2>
            <button
              type="button"
              class="sol-icon-btn sol-icon-btn--xs"
              title="Refresh announcements"
              @click="loadAnnouncements()"
            >
              <RefreshCw :size="12" :class="{ 'is-spinning': store.announcementsSource === 'loading' }" />
            </button>
          </div>

          <div v-if="store.announcementsSource === 'loading'" class="sol-muted sol-text-xs">
            Checking Solara release notices...
          </div>
          <div v-else-if="store.announcements.length === 0" class="sol-muted sol-text-xs">
            <template v-if="store.announcementsSource === 'unavailable'">
              Announcements feed offline right now. Your local launcher and instances work normally.
            </template>
            <template v-else>
              No new announcements at the moment.
            </template>
          </div>
          <div v-else class="sol-announce-list">
            <button
              v-for="item in store.announcements.slice(0, 3)"
              :key="item.id"
              type="button"
              class="sol-announce-item"
              @click="openAnnouncement(item)"
            >
              <div class="sol-announce-title sol-truncate">{{ item.title }}</div>
              <div class="sol-announce-meta">{{ item.category || 'Notice' }}</div>
            </button>
          </div>
        </section>
      </aside>
    </div>
  </div>
</template>
