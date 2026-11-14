<script setup lang="ts">
export interface TabItem {
  id: string;
  label: string;
  badge?: string | number | null;
}

defineProps<{
  tabs: TabItem[];
  modelValue: string;
  variant?: 'underline' | 'pills';
}>();

const emit = defineEmits<{ (e: 'update:modelValue', value: string): void }>();
</script>

<template>
  <div :class="['sol-tabs', `sol-tabs--${variant ?? 'underline'}`]" role="tablist">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      type="button"
      role="tab"
      :aria-selected="modelValue === tab.id"
      :class="['sol-tab', { 'is-active': modelValue === tab.id }]"
      @click="emit('update:modelValue', tab.id)"
    >
      <span>{{ tab.label }}</span>
      <span v-if="tab.badge !== undefined && tab.badge !== null && tab.badge !== ''" class="sol-tab-badge">
        {{ tab.badge }}
      </span>
    </button>
  </div>
</template>
