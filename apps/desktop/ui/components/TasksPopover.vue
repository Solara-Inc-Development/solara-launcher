<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { X, RotateCw, Trash2, ArrowUpRight } from '../icons';
import { store, formatBytes, navigate, native, notify, refreshSnapshot } from '../state';
import SolaraProgress from './SolaraProgress.vue';
import SolaraButton from './SolaraButton.vue';

const emit = defineEmits<{ (e: 'close'): void }>();

const sortedJobs = computed(() => [...store.downloads].slice(-8).reverse());

async function cancelJob(id: string) {
  try {
    await native('download_action', { id, action: 'cancel' });
    await refreshSnapshot();
  } catch (err) {
    notify('error', "Couldn't cancel task", String(err));
  }
}

async function retryJob(id: string) {
  try {
    await native('download_action', { id, action: 'retry' });
    await refreshSnapshot();
  } catch (err) {
    notify('error', "Couldn't retry task", String(err));
  }
}

async function clearFinished() {
  try {
    await native('clear_finished_downloads');
    await refreshSnapshot();
  } catch (err) {
    notify('error', "Couldn't clear tasks", String(err));
  }
}

function handleOutside(e: PointerEvent) {
  const el = e.target as HTMLElement | null;
  if (!el?.closest('.sol-tasks-anchor')) emit('close');
}

onMounted(() => window.addEventListener('pointerdown', handleOutside));
onUnmounted(() => window.removeEventListener('pointerdown', handleOutside));
</script>

<template>
  <div class="sol-popover sol-tasks-popover" role="region" aria-label="Recent tasks">
    <div class="sol-popover-head">
      <span class="sol-popover-title">Downloads & Tasks</span>
      <div class="sol-row-gap">
        <button
          v-if="store.downloads.some((d) => d.status === 'completed' || d.status === 'failed' || d.status === 'cancelled')"
          type="button"
          class="sol-icon-btn sol-icon-btn--xs"
          title="Clear finished tasks"
          @click="clearFinished"
        >
          <Trash2 :size="13" />
        </button>
        <button
          type="button"
          class="sol-icon-btn sol-icon-btn--xs"
          title="Open full Tasks page"
          @click="
            navigate('tasks');
            emit('close');
          "
        >
          <ArrowUpRight :size="13" />
        </button>
      </div>
    </div>

    <div class="sol-popover-list sol-scroll">
      <div v-if="sortedJobs.length === 0" class="sol-popover-empty">
        No active or recent tasks.
      </div>
      <div v-for="job in sortedJobs" :key="job.id" class="sol-task-mini">
        <div class="sol-task-mini-top">
          <span class="sol-task-mini-name sol-truncate">{{ job.name }}</span>
          <span :class="['sol-badge', `sol-badge--${job.status}`]">{{ job.status }}</span>
        </div>
        <div class="sol-task-mini-meta">
          <span>{{ job.phase || job.kind }}</span>
          <span v-if="job.total > 0">{{ formatBytes(job.done) }} / {{ formatBytes(job.total) }}</span>
        </div>
        <SolaraProgress
          v-if="job.status === 'running' || job.status === 'queued'"
          :value="job.done"
          :max="job.total"
          :indeterminate="job.total <= 0"
          compact
        />
        <div v-if="job.actions.length > 0" class="sol-task-mini-actions">
          <SolaraButton
            v-if="job.actions.includes('cancel')"
            variant="ghost"
            size="xs"
            @click="cancelJob(job.id)"
          >
            <X :size="12" /> Cancel
          </SolaraButton>
          <SolaraButton
            v-else-if="job.actions.includes('retry')"
            variant="ghost"
            size="xs"
            @click="retryJob(job.id)"
          >
            <RotateCw :size="12" /> Retry
          </SolaraButton>
        </div>
      </div>
    </div>
  </div>
</template>
