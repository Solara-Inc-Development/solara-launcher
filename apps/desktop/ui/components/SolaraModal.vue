<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { X } from '../icons';

const props = withDefaults(
  defineProps<{
    open: boolean;
    title?: string;
    subtitle?: string;
    width?: 'sm' | 'md' | 'lg' | 'xl';
    closable?: boolean;
  }>(),
  {
    closable: true,
  },
);

const emit = defineEmits<{ (e: 'close'): void }>();

function onKeydown(e: KeyboardEvent) {
  if (props.open && e.key === 'Escape' && props.closable !== false) {
    emit('close');
  }
}

onMounted(() => window.addEventListener('keydown', onKeydown));
onUnmounted(() => window.removeEventListener('keydown', onKeydown));
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="sol-modal-backdrop"
      role="dialog"
      aria-modal="true"
      :aria-label="title"
      @mousedown.self="closable !== false && emit('close')"
    >
      <div :class="['sol-modal', `sol-modal--${width ?? 'md'}`]">
        <header v-if="title || $slots.header" class="sol-modal-header">
          <slot name="header">
            <div class="sol-modal-title-wrap">
              <h2 class="sol-modal-title">{{ title }}</h2>
              <p v-if="subtitle" class="sol-modal-subtitle">{{ subtitle }}</p>
            </div>
          </slot>
          <button
            v-if="closable !== false"
            type="button"
            class="sol-icon-btn"
            aria-label="Close dialog"
            @click="emit('close')"
          >
            <X :size="16" />
          </button>
        </header>
        <div class="sol-modal-body sol-scroll">
          <slot />
        </div>
        <footer v-if="$slots.footer" class="sol-modal-footer">
          <slot name="footer" />
        </footer>
      </div>
    </div>
  </Teleport>
</template>
