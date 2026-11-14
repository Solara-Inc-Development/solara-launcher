<script setup lang="ts">
import { store, dismissToast } from '../state';
import { CheckCircle2, AlertTriangle, Info, X } from '../icons';
</script>

<template>
  <Teleport to="body">
    <div class="sol-toast-stack" aria-live="polite">
      <TransitionGroup name="sol-toast">
        <div
          v-for="toast in store.toasts"
          :key="toast.id"
          :class="['sol-toast', `sol-toast--${toast.tone}`]"
          role="status"
        >
          <span class="sol-toast-icon" aria-hidden="true">
            <CheckCircle2 v-if="toast.tone === 'success'" :size="16" />
            <AlertTriangle v-else-if="toast.tone === 'error' || toast.tone === 'warning'" :size="16" />
            <Info v-else :size="16" />
          </span>
          <div class="sol-toast-body">
            <div class="sol-toast-title">{{ toast.title }}</div>
            <div v-if="toast.message" class="sol-toast-msg">{{ toast.message }}</div>
          </div>
          <button
            type="button"
            class="sol-icon-btn sol-icon-btn--xs"
            aria-label="Dismiss notification"
            @click="dismissToast(toast.id)"
          >
            <X :size="13" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>
