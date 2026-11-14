<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  name?: string;
  loader?: string;
  icon?: string | null;
  customIcon?: string | null;
  size?: number;
  rounded?: number;
}>();

const initials = computed(() => {
  const words = (props.name || 'S').trim().split(/\s+/);
  if (words.length >= 2) return (words[0][0] + words[1][0]).toUpperCase();
  return (props.name || 'S').slice(0, 2).toUpperCase();
});

const presetPalette = computed(() => {
  const key = props.icon || '';
  switch (key) {
    case 'preset:dawn-crest':
      return { bg: '#2B2216', fg: '#E9B86D', shape: 'crest' };
    case 'preset:obsidian-cube':
      return { bg: '#1B1F27', fg: '#F5F4F1', shape: 'cube' };
    case 'preset:amber-emerald':
      return { bg: '#152720', fg: '#23C987', shape: 'gem' };
    case 'preset:solar-compass':
      return { bg: '#1B2332', fg: '#65A9F5', shape: 'compass' };
    case 'preset:forge-anvil':
      return { bg: '#281F19', fg: '#E8A64B', shape: 'anvil' };
    case 'preset:nether-portal':
      return { bg: '#261829', fg: '#C97BF5', shape: 'portal' };
    case 'preset:redstone-core':
      return { bg: '#2B171B', fg: '#F06471', shape: 'core' };
    case 'preset:end-crystal':
      return { bg: '#1E1A2C', fg: '#B998FF', shape: 'crystal' };
    default:
      return null;
  }
});

const loaderColor = computed(() => {
  switch (props.loader) {
    case 'fabric':
      return { bg: '#26211A', fg: '#E9B86D' };
    case 'quilt':
      return { bg: '#221B2B', fg: '#B998FF' };
    case 'forge':
      return { bg: '#1A222D', fg: '#65A9F5' };
    case 'neoforge':
      return { bg: '#2B1F18', fg: '#E8A64B' };
    default:
      return { bg: '#17261E', fg: '#23C987' };
  }
});

const boxStyle = computed(() => {
  const px = props.size ?? 40;
  return {
    width: `${px}px`,
    height: `${px}px`,
    borderRadius: `${props.rounded ?? Math.max(8, Math.round(px * 0.24))}px`,
    background: presetPalette.value?.bg ?? loaderColor.value.bg,
    color: presetPalette.value?.fg ?? loaderColor.value.fg,
    fontSize: `${Math.max(11, Math.round(px * 0.34))}px`,
  };
});
</script>

<template>
  <div class="sol-inst-icon" :style="boxStyle">
    <img
      v-if="customIcon"
      :src="customIcon"
      :alt="name || 'Instance icon'"
      class="sol-inst-icon-img"
    />
    <svg
      v-else-if="presetPalette"
      viewBox="0 0 24 24"
      :width="Math.round((size ?? 40) * 0.56)"
      :height="Math.round((size ?? 40) * 0.56)"
      fill="none"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path v-if="presetPalette.shape === 'crest'" d="M12 3l7 4v5c0 4.5-3 7.5-7 9-4-1.5-7-4.5-7-9V7l7-4z" />
      <path v-else-if="presetPalette.shape === 'cube'" d="M12 2l8.5 5v10L12 22l-8.5-5V7L12 2zM12 22V12M20.5 7L12 12 3.5 7" />
      <path v-else-if="presetPalette.shape === 'gem'" d="M12 2l7 7-7 13L5 9l7-7zM5 9h14" />
      <path v-else-if="presetPalette.shape === 'compass'" d="M12 2a10 10 0 1 0 10 10A10 10 0 0 0 12 2zm3.5 6.5-2 5.5-5.5 2 2-5.5z" />
      <path v-else-if="presetPalette.shape === 'anvil'" d="M4 7h16v3c0 2-2 3-4 3v4h2v2H6v-2h2v-4c-2 0-4-1-4-3V7z" />
      <path v-else-if="presetPalette.shape === 'portal'" d="M6 3h12v18H6zM9 6h6v12H9z" />
      <path v-else-if="presetPalette.shape === 'core'" d="M12 3v3m0 12v3M3 12h3m12 0h3M6.5 6.5l2 2m7 7 2 2m0-11-2 2m-7 7-2 2M12 9a3 3 0 1 0 3 3 3 3 0 0 0-3-3z" />
      <path v-else d="M12 2l6 10-6 10-6-10 6-10zm0 5l-2.5 5L12 17l2.5-5L12 7z" />
    </svg>
    <span v-else class="sol-inst-icon-text">{{ initials }}</span>
  </div>
</template>
