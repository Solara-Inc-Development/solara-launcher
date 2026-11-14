<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { RefreshCw, CheckCircle2, ArrowRight } from '../icons';
import { native, notify } from '../state';
import type { ContentUpdateItem } from '../types';
import SolaraModal from './SolaraModal.vue';
import SolaraButton from './SolaraButton.vue';

const props = defineProps<{
  open: boolean;
  instanceId: string;
  instanceName: string;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'updated'): void;
}>();

const checking = ref(false);
const applying = ref(false);
const errorMsg = ref('');
const updates = ref<ContentUpdateItem[]>([]);
const selected = ref<Set<string>>(new Set());
const activePreview = ref<ContentUpdateItem | null>(null);
let checkSequence = 0;

watch(
  () => [props.open, props.instanceId] as const,
  ([open]) => { if (open) void runCheck(); else { checkSequence++; checking.value = false; } },
);

async function runCheck() {
  if (applying.value) return;
  const sequence = ++checkSequence;
  const id = props.instanceId;
  checking.value = true;
  errorMsg.value = '';
  updates.value = [];
  selected.value = new Set();
  activePreview.value = null;
  try {
    const res = await native<ContentUpdateItem[]>('check_instance_content_updates', {
      id,
    });
    if (sequence !== checkSequence || props.instanceId !== id || !props.open) return;
    updates.value = Array.isArray(res) ? res : [];
    for (const item of updates.value) selected.value.add(item.currentPath);
    activePreview.value = updates.value[0] ?? null;
  } catch (err) {
    if (sequence === checkSequence) errorMsg.value = String(err);
  } finally {
    if (sequence === checkSequence) checking.value = false;
  }
}

const selectedList = computed(() =>
  updates.value.filter((u) => selected.value.has(u.currentPath)),
);

function toggleAll(checked: boolean) {
  selected.value = new Set(checked ? updates.value.map((u) => u.currentPath) : []);
}

function toggleOne(path: string) {
  const next = new Set(selected.value);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  selected.value = next;
}

async function applySelected() {
  if (selectedList.value.length === 0 || applying.value || checking.value) return;
  const items = [...selectedList.value];
  const name = props.instanceName;
  applying.value = true;
  errorMsg.value = '';
  try {
    await native('apply_instance_content_updates', {
      id: props.instanceId,
      items,
    });
    notify(
      'success',
      'Content updated',
      `Updated ${items.length} item(s) in ${name}.`,
    );
    emit('updated');
    emit('close');
  } catch (err) {
    errorMsg.value = String(err);
  } finally {
    applying.value = false;
  }
}
</script>

<template>
  <SolaraModal
    :open="open"
    title="Smart Content Updates"
    :subtitle="`Checking Modrinth hashes & compatible releases for ${instanceName}`"
    width="lg"
    :closable="!applying"
    @close="!applying && emit('close')"
  >
    <div v-if="errorMsg" class="sol-banner-warn">
      <span>{{ errorMsg }} Re-check installed files before trying again.</span>
      <SolaraButton variant="secondary" size="xs" :disabled="applying || checking" @click="runCheck">Check Again</SolaraButton>
    </div>
    <div v-if="checking" class="sol-center-state">
      <RefreshCw :size="20" class="is-spinning sol-accent" />
      <span>Verifying installed mod hashes against Modrinth...</span>
    </div>

    <div v-else-if="updates.length === 0 && !errorMsg" class="sol-center-state">
      <CheckCircle2 :size="24" class="sol-success" />
      <div class="sol-center-title">Everything checked is up to date</div>
      <div class="sol-muted sol-text-xs">
        All tracked Modrinth mods matching this Minecraft version and loader are on their latest release.
      </div>
    </div>

    <div v-else-if="updates.length > 0" class="sol-updates-grid">
      <div class="sol-updates-list sol-scroll">
        <div class="sol-updates-bar">
          <label class="sol-checkbox-label">
            <input
              type="checkbox"
              :disabled="applying"
              :checked="selectedList.length === updates.length"
              @change="toggleAll(($event.target as HTMLInputElement).checked)"
            />
            <span>Select all ({{ selectedList.length }}/{{ updates.length }})</span>
          </label>
          <button type="button" class="sol-link-btn" @click="runCheck">Re-check</button>
        </div>

        <div
          v-for="item in updates"
          :key="item.currentPath"
          :class="['sol-update-row', { 'is-active': activePreview?.currentPath === item.currentPath }]"
          @click="activePreview = item"
        >
          <input
            type="checkbox"
            :disabled="applying"
            :checked="selected.has(item.currentPath)"
            @click.stop
            @change="toggleOne(item.currentPath)"
          />
          <div class="sol-update-row-copy">
            <div class="sol-update-name">{{ item.projectName }}</div>
            <div class="sol-update-vers">
              <span class="sol-muted">{{ item.currentVersionNumber }}</span>
              <ArrowRight :size="11" />
              <span class="sol-accent">{{ item.latestVersionNumber }}</span>
            </div>
          </div>
          <span class="sol-badge">{{ item.releaseChannel }}</span>
        </div>
      </div>

      <div class="sol-update-detail sol-scroll">
        <template v-if="activePreview">
          <h3 class="sol-update-detail-title">{{ activePreview.projectName }}</h3>
          <div class="sol-update-detail-ver">
            {{ activePreview.latestVersionName || activePreview.latestVersionNumber }}
          </div>
          <div class="sol-update-changelog selectable">
            {{ activePreview.changelog || 'No changelog provided for this release.' }}
          </div>
        </template>
      </div>
    </div>

    <template #footer>
      <SolaraButton variant="ghost" :disabled="applying" @click="emit('close')">
        Close
      </SolaraButton>
      <SolaraButton
        v-if="updates.length > 0"
        variant="primary"
        :loading="applying"
        :disabled="selectedList.length === 0"
        @click="applySelected"
      >
        Update {{ selectedList.length }} Selected
      </SolaraButton>
    </template>
  </SolaraModal>
</template>
