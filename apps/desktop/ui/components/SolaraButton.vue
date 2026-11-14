<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'success';
  size?: 'xs' | 'sm' | 'md' | 'lg';
  disabled?: boolean;
  loading?: boolean;
  type?: 'button' | 'submit' | 'reset';
  block?: boolean;
}>();
</script>

<template>
  <button
    :type="type ?? 'button'"
    :disabled="disabled || loading"
    :class="[
      'sol-btn',
      `sol-btn--${variant ?? 'secondary'}`,
      `sol-btn--${size ?? 'md'}`,
      { 'sol-btn--block': block, 'sol-btn--loading': loading },
    ]"
  >
    <span v-if="loading" class="sol-btn-spin" aria-hidden="true"></span>
    <slot />
  </button>
</template>
