<script setup lang="ts">
import { computed } from 'vue';
import SolaraModal from './SolaraModal.vue';
import SolaraButton from './SolaraButton.vue';
import SolaraLogo from './SolaraLogo.vue';
import { Download, RefreshCw } from '../icons';
import {
  dismissUpdateModalForLater,
  formatBytes,
  formatPublicVersion,
  installAppUpdate,
  store,
} from '../state';

const stage = computed(() => store.updateStatus?.stage ?? 'idle');
const isDownloading = computed(() => stage.value === 'downloading');
const isInstalling = computed(() => stage.value === 'installing');
const isChecking = computed(() => store.updateInstallPending && stage.value === 'checking');
const isBusyUpdating = computed(
  () => store.updateInstallPending || isDownloading.value || isInstalling.value || isChecking.value,
);

const newPublicVersion = computed(() => {
  const raw = store.updateStatus?.publicVersion || store.updateStatus?.version;
  return raw ? formatPublicVersion(raw) : 'latest release';
});
const currentPublicVersion = computed(() =>
  store.updateStatus?.currentVersion ? formatPublicVersion(store.updateStatus.currentVersion) : null,
);

const downloadPct = computed(() => {
  const done = Number(store.updateStatus?.downloaded ?? 0);
  const total = Number(store.updateStatus?.total ?? 0);
  if (total > 0 && done >= 0) {
    return Math.min(100, Math.max(2, Math.round((done / total) * 100)));
  }
  return isDownloading.value ? 35 : isInstalling.value ? 100 : 12;
});

const statusHeadline = computed(() => {
  if (isInstalling.value) return `Installing ${newPublicVersion.value} and restarting Solara...`;
  if (isDownloading.value) {
    const done = Number(store.updateStatus?.downloaded ?? 0);
    const total = store.updateStatus?.total;
    if (total && total > 0) {
      return `Downloading ${newPublicVersion.value} · ${downloadPct.value}% (${formatBytes(done)} / ${formatBytes(total)})`;
    }
    if (done > 0) {
      return `Downloading ${newPublicVersion.value} (${formatBytes(done)})...`;
    }
    return `Downloading ${newPublicVersion.value}...`;
  }
  if (isChecking.value) return 'Preparing signed update package...';
  if (stage.value === 'waiting-for-idle')
    return `Update ${newPublicVersion.value} is staged and waiting for your running game or downloads to finish.`;
  if (stage.value === 'ready') return `Update ${newPublicVersion.value} is verified and ready to install.`;
  return 'A signed Solara Launcher update is available for your device.';
});

function handleClose(): void {
  if (isInstalling.value) return;
  dismissUpdateModalForLater();
}
</script>

<template>
  <SolaraModal
    :open="store.showUpdateModal"
    title="New Version Available"
    :subtitle="
      currentPublicVersion
        ? `Installed: ${currentPublicVersion}  →  Latest: ${newPublicVersion}`
        : `Latest release: ${newPublicVersion}`
    "
    width="sm"
    :closable="!isInstalling"
    @close="handleClose"
  >
    <div class="sol-update-modal">
      <div class="sol-update-hero">
        <SolaraLogo :size="52" />
        <div class="sol-update-hero-copy">
          <div class="sol-update-title-row">
            <strong>New Version Available</strong>
            <span class="sol-badge sol-badge--accent">{{ newPublicVersion }}</span>
          </div>
          <p class="sol-muted sol-text-xs" style="margin: 4px 0 0">
            {{ statusHeadline }}
          </p>
        </div>
      </div>

      <div v-if="isBusyUpdating" class="sol-update-progress-wrap">
        <div class="sol-update-progress-track">
          <div class="sol-update-progress-fill" :style="{ width: `${downloadPct}%` }" />
        </div>
      </div>

      <div v-if="store.updateStatus?.notes" class="sol-update-notes sol-scroll selectable">
        {{ store.updateStatus.notes }}
      </div>
      <div v-else class="sol-update-notes sol-muted">
        Updates verify cryptographically before replacing the launcher app. Your worlds, instances, mods, and accounts remain untouched.
      </div>

      <p v-if="store.updateStatus?.error" class="sol-inline-error" style="margin: 0">
        {{ store.updateStatus.error }}
      </p>
    </div>

    <template #footer>
      <SolaraButton variant="ghost" :disabled="isInstalling" @click="dismissUpdateModalForLater">
        I'll do it later
      </SolaraButton>
      <SolaraButton variant="primary" :loading="isBusyUpdating" @click="installAppUpdate">
        <RefreshCw v-if="stage === 'ready'" :size="14" />
        <Download v-else :size="14" />
        <span>{{ isInstalling ? 'Restarting...' : isDownloading ? 'Downloading...' : 'Update Now' }}</span>
      </SolaraButton>
    </template>
  </SolaraModal>
</template>
