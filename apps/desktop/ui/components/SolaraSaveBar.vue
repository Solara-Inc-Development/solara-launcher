<script setup lang="ts">
import SolaraButton from './SolaraButton.vue';

defineProps<{
  dirty: boolean;
  saving?: boolean;
  message?: string;
}>();

const emit = defineEmits<{
  (e: 'reset'): void;
  (e: 'save'): void;
}>();
</script>

<template>
  <Transition name="sol-slide-up">
    <div v-if="dirty" class="sol-savebar" role="status" aria-live="polite">
      <div class="sol-savebar-text">
        <span class="sol-savebar-dot" aria-hidden="true" />
        <span>{{ message ?? 'You have unsaved changes.' }}</span>
      </div>
      <div class="sol-savebar-actions">
        <SolaraButton variant="ghost" size="sm" :disabled="saving" @click="emit('reset')">
          Reset
        </SolaraButton>
        <SolaraButton variant="primary" size="sm" :loading="saving" @click="emit('save')">
          Save Changes
        </SolaraButton>
      </div>
    </div>
  </Transition>
</template>
