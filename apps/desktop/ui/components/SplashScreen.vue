<script setup lang="ts">
import { computed } from 'vue';
import SolaraLogo from './SolaraLogo.vue';
import SolaraButton from './SolaraButton.vue';
import { Check, RefreshCw } from '../icons';
import { store, initializeEngine } from '../state';

const pct = computed(() => Math.min(100, Math.max(8, Math.round(store.initProgress))));
</script>

<template>
  <Transition name="sol-splash-fade">
    <div v-if="!store.splashDone" class="sol-splash" role="status" aria-live="polite">
      <div class="sol-splash-aura" aria-hidden="true" />
      <div class="sol-splash-card">
        <div class="sol-splash-logo-ring">
          <div class="sol-splash-logo">
            <SolaraLogo :size="88" />
          </div>
        </div>

        <div class="sol-splash-brand">
          <span class="sol-splash-title">Solara</span>
          <span class="sol-splash-sub">Minecraft: Java Edition Launcher</span>
        </div>

        <template v-if="store.initError">
          <div class="sol-splash-error">
            <strong>Startup failed</strong>
            <span>{{ store.initError }}</span>
            <SolaraButton variant="primary" size="sm" @click="initializeEngine()">
              <RefreshCw :size="14" /> Retry Startup
            </SolaraButton>
          </div>
        </template>
        <template v-else>
          <div class="sol-splash-meter">
            <div class="sol-splash-meter-head">
              <span class="sol-splash-status">{{ store.initPhase }}</span>
              <span class="sol-splash-pct">{{ pct }}%</span>
            </div>
            <div
              class="sol-splash-progress"
              role="progressbar"
              :aria-valuenow="pct"
              aria-valuemin="0"
              aria-valuemax="100"
              :aria-label="store.initPhase"
            >
              <div class="sol-splash-progress-fill" :style="{ width: `${pct}%` }" />
            </div>
          </div>

          <div class="sol-splash-stages" aria-label="Startup stages">
            <div
              v-for="stage in store.initStages"
              :key="stage.id"
              :class="['sol-splash-stage', `is-${stage.state}`]"
            >
              <span class="sol-splash-stage-dot">
                <Check v-if="stage.state === 'done'" :size="10" />
              </span>
              <span>{{ stage.label }}</span>
            </div>
          </div>
        </template>
      </div>
    </div>
  </Transition>
</template>
