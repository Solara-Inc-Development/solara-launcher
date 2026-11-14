<script setup lang="ts">
import { ref } from 'vue';

withDefaults(
  defineProps<{
    size?: number;
    framed?: boolean;
  }>(),
  {
    size: 24,
    framed: true,
  },
);

const src = ref('/logo.svg');

function onError(): void {
  if (src.value !== '/logo-192.png') {
    src.value = '/logo-192.png';
  }
}
</script>

<template>
  <img
    :src="src"
    :width="size"
    :height="size"
    :class="['sol-brand-mark', { 'is-framed': framed }]"
    alt=""
    aria-hidden="true"
    draggable="false"
    @error="onError"
  />
</template>
