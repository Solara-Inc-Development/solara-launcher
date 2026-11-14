<script setup lang="ts">
import { ref, watch } from 'vue';
import SolaraModal from './SolaraModal.vue';
import SolaraButton from './SolaraButton.vue';

const props = defineProps<{
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
  requireText?: string;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: 'confirm'): void;
  (e: 'cancel'): void;
}>();

const typed = ref('');
watch(
  () => props.open,
  (v) => {
    if (v) typed.value = '';
  },
);
</script>

<template>
  <SolaraModal :open="open" :title="title" width="sm" @close="emit('cancel')">
    <p class="sol-confirm-msg">{{ message }}</p>
    <div v-if="requireText" class="sol-field" style="margin-top: 12px">
      <label class="sol-label">Type <strong>{{ requireText }}</strong> to confirm</label>
      <input
        v-model="typed"
        type="text"
        class="sol-input"
        :placeholder="requireText"
        autocomplete="off"
      />
    </div>
    <template #footer>
      <SolaraButton variant="ghost" :disabled="loading" @click="emit('cancel')">
        {{ cancelLabel ?? 'Cancel' }}
      </SolaraButton>
      <SolaraButton
        :variant="danger ? 'danger' : 'primary'"
        :loading="loading"
        :disabled="Boolean(requireText && typed.trim() !== requireText)"
        @click="emit('confirm')"
      >
        {{ confirmLabel ?? 'Confirm' }}
      </SolaraButton>
    </template>
  </SolaraModal>
</template>
