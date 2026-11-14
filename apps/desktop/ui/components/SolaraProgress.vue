<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  value?: number;
  max?: number;
  indeterminate?: boolean;
  tone?: 'gold' | 'success' | 'error' | 'info';
  compact?: boolean;
}>();

const pct = computed(() => {
  if (props.indeterminate) return 40;
  const max = props.max && props.max > 0 ? props.max : 100;
  const val = Math.max(0, Math.min(max, props.value ?? 0));
  return Math.round((val / max) * 100);
});
</script>

<template>
  <div :class="['sol-progress', `sol-progress--${tone ?? 'gold'}`, { 'is-compact': compact, 'is-indeterminate': indeterminate }]" role="progressbar" :aria-valuenow="indeterminate ? undefined : pct" aria-valuemin="0" aria-valuemax="100">
    <div class="sol-progress-bar" :style="indeterminate ? undefined : { width: `${pct}%` }" />
  </div>
</template>
