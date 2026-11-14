<script setup lang="ts">
import { computed, useId } from 'vue';

const props = withDefaults(
  defineProps<{
    size?: number;
    framed?: boolean;
  }>(),
  {
    size: 24,
    framed: true,
  },
);

const uid = useId();
const gradId = computed(() => `sol-gold-${uid}`);
const rimId = computed(() => `sol-rim-${uid}`);
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 64 64"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    aria-hidden="true"
  >
    <defs>
      <linearGradient :id="gradId" x1="12" y1="10" x2="54" y2="54" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stop-color="#F5CD91" />
        <stop offset="52%" stop-color="#E9B86D" />
        <stop offset="100%" stop-color="#C89543" />
      </linearGradient>
      <linearGradient :id="rimId" x1="0" y1="0" x2="64" y2="64" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stop-color="#392B18" />
        <stop offset="50%" stop-color="#29292F" />
        <stop offset="100%" stop-color="#1C1C21" />
      </linearGradient>
    </defs>

    <rect
      v-if="framed"
      x="2"
      y="2"
      width="60"
      height="60"
      rx="16"
      fill="#080809"
      :stroke="`url(#${rimId})`"
      stroke-width="2"
    />

    <!-- Outer solar meridian arc with eclipse break -->
    <path
      d="M44.8 18.2A17.5 17.5 0 1 0 47.6 39.8"
      :stroke="`url(#${gradId})`"
      stroke-width="4.4"
      stroke-linecap="round"
    />
    <!-- Dawn ray dot on upper right -->
    <circle cx="48.8" cy="24.6" r="2.6" fill="#F5CD91" />
    <!-- Precision geometric Solara sunbeam bolt -->
    <path
      d="M35.2 18.5L23.5 33.4H31.8L28.8 45.5L40.8 30.6H32.2L35.2 18.5Z"
      :fill="`url(#${gradId})`"
    />
  </svg>
</template>
