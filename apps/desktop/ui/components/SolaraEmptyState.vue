<script setup lang="ts">
defineProps<{
  title: string;
  description?: string;
}>();
</script>

<template>
  <div class="sol-empty">
    <div v-if="$slots.icon" class="sol-empty-icon">
      <slot name="icon" />
    </div>
    <h3 class="sol-empty-title">{{ title }}</h3>
    <p v-if="description" class="sol-empty-desc">{{ description }}</p>
    <div v-if="$slots.actions" class="sol-empty-actions">
      <slot name="actions" />
    </div>
  </div>
</template>
