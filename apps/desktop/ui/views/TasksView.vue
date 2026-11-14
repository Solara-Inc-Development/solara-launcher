<script setup lang="ts">
import { computed, ref } from 'vue';
import { Download, Trash2, RotateCw, X } from '../icons';
import { store, formatBytes, navigate, native, notify, refreshSnapshot } from '../state';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraProgress from '../components/SolaraProgress.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';

const filter = ref<'all' | 'active' | 'completed' | 'failed'>('all');

const filteredJobs = computed(() => {
  const list = [...store.downloads].reverse();
  return list.filter((j) => {
    if (filter.value === 'active') return j.status === 'running' || j.status === 'queued';
    if (filter.value === 'completed') return j.status === 'completed';
    if (filter.value === 'failed') return j.status === 'failed' || j.status === 'cancelled';
    return true;
  });
});

async function action(id: string, kind: 'cancel' | 'retry') {
  try {
    await native('download_action', { id, action: kind });
    await refreshSnapshot();
  } catch (e) {
    notify('error', `Could not ${kind} task`, String(e));
  }
}

async function clearFinished() {
  try {
    await native('clear_finished_downloads');
    await refreshSnapshot();
    notify('info', 'Cleared finished tasks');
  } catch (e) {
    notify('error', 'Failed clearing tasks', String(e));
  }
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-page-inner">
      <header class="sol-page-header">
        <div>
          <h1 class="sol-page-title">Downloads & Tasks</h1>
          <p class="sol-page-sub">Real-time queue for instance installations, Java downloads, and content updates</p>
        </div>
        <SolaraButton
          v-if="store.downloads.some((d) => ['completed', 'failed', 'cancelled'].includes(d.status))"
          variant="secondary"
          size="sm"
          @click="clearFinished"
        >
          <Trash2 :size="13" /> Clear Finished
        </SolaraButton>
      </header>

      <div class="sol-toolbar">
        <div class="sol-seg-group">
          <button
            v-for="opt in [
              { id: 'all', label: 'All' },
              { id: 'active', label: 'Active & Queued' },
              { id: 'completed', label: 'Completed' },
              { id: 'failed', label: 'Failed / Cancelled' },
            ] as const"
            :key="opt.id"
            type="button"
            :class="['sol-seg-btn', { 'is-active': filter === opt.id }]"
            @click="filter = opt.id"
          >
            {{ opt.label }}
          </button>
        </div>
      </div>

      <SolaraEmptyState
        v-if="filteredJobs.length === 0"
        title="No tasks in this queue"
        description="Instance installations, Java runtime downloads, and content installs appear here."
      >
        <template #icon><Download :size="26" /></template>
      </SolaraEmptyState>

      <div v-else class="sol-stack">
        <article v-for="job in filteredJobs" :key="job.id" class="sol-card sol-task-card">
          <div class="sol-task-top">
            <div>
              <div class="sol-row-gap">
                <strong class="sol-task-name">{{ job.name }}</strong>
                <span :class="['sol-badge', `sol-badge--${job.status}`]">{{ job.status }}</span>
                <span class="sol-badge">{{ job.kind }}</span>
              </div>
              <div class="sol-muted sol-text-xs" style="margin-top: 4px">
                {{ job.phase || 'Queued' }}
                <template v-if="job.total > 0">
                  — {{ formatBytes(job.done) }} / {{ formatBytes(job.total) }}
                </template>
              </div>
            </div>

            <div class="sol-row-gap">
              <SolaraButton
                v-if="job.instance"
                variant="ghost"
                size="xs"
                @click="navigate('instance', { instanceId: job.instance, tab: 'overview' })"
              >
                Open Instance
              </SolaraButton>
              <SolaraButton
                v-if="job.status === 'running' || job.status === 'queued'"
                variant="danger"
                size="xs"
                @click="action(job.id, 'cancel')"
              >
                <X :size="12" /> Cancel
              </SolaraButton>
              <SolaraButton
                v-else-if="job.status === 'failed' || job.status === 'cancelled'"
                variant="secondary"
                size="xs"
                @click="action(job.id, 'retry')"
              >
                <RotateCw :size="12" /> Retry
              </SolaraButton>
            </div>
          </div>

          <SolaraProgress
            v-if="job.status === 'running' || job.status === 'queued'"
            :value="job.done"
            :max="job.total"
            :indeterminate="job.total <= 0"
            style="margin-top: 10px"
          />
        </article>
      </div>
    </div>
  </div>
</template>
