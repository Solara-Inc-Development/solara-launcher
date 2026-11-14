<script setup lang="ts">
import SolaraLogo from './SolaraLogo.vue';
import SolaraButton from './SolaraButton.vue';
import { RefreshCw } from '../icons';
import { store, initializeEngine } from '../state';
</script>

<template>
  <Transition name="sol-splash-fade">
    <div v-if="!store.splashDone" class="sol-splash" role="status" aria-live="polite">
      <div class="sol-splash-inner">
        <div class="sol-splash-logo">
          <SolaraLogo :size="68" />
        </div>
        <div class="sol-splash-brand">
          <span class="sol-splash-title">Solara</span>
          <span class="sol-splash-sub">Minecraft Java Launcher</span>
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
          <div class="sol-splash-progress">
            <div class="sol-splash-progress-fill" :style="{ width: `${Math.max(10, store.initProgress)}%` }" />
          </div>
          <div class="sol-splash-status">{{ store.initPhase }}</div>
        </template>
      </div>
    </div>
  </Transition>
</template>
