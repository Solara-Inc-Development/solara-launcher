<script setup lang="ts">
defineProps<{
  modelValue: boolean;
  disabled?: boolean;
  label?: string;
}>();

const emit = defineEmits<{ (e: 'update:modelValue', value: boolean): void }>();
</script>

<template>
  <button
    type="button"
    role="switch"
    :aria-checked="modelValue"
    :aria-label="label"
    :disabled="disabled"
    :class="['sol-toggle', { 'is-on': modelValue, 'is-disabled': disabled }]"
    @click="!disabled && emit('update:modelValue', !modelValue)"
  >
    <span class="sol-toggle-thumb" />
  </button>
</template>
