<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';

export interface ContextMenuItem {
  id: string;
  label: string;
  danger?: boolean;
  disabled?: boolean;
  separator?: boolean;
}

const props = defineProps<{
  open: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'select', id: string): void;
}>();

const menuStyle = computed(() => {
  const vw = typeof window !== 'undefined' ? window.innerWidth : 1200;
  const vh = typeof window !== 'undefined' ? window.innerHeight : 800;
  const estW = 200;
  const estH = props.items.length * 34 + 16;
  const left = Math.min(props.x, Math.max(8, vw - estW - 12));
  const top = Math.min(props.y, Math.max(8, vh - estH - 12));
  return { left: `${left}px`, top: `${top}px` };
});

function handleKey(e: KeyboardEvent) {
  if (props.open && e.key === 'Escape') emit('close');
}

function handleOutside() {
  if (props.open) emit('close');
}

onMounted(() => {
  window.addEventListener('keydown', handleKey);
  window.addEventListener('pointerdown', handleOutside);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKey);
  window.removeEventListener('pointerdown', handleOutside);
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="sol-context-menu"
      :style="menuStyle"
      role="menu"
      @pointerdown.stop
    >
      <template v-for="item in items" :key="item.id">
        <div v-if="item.separator" class="sol-context-sep" />
        <button
          v-else
          type="button"
          role="menuitem"
          :disabled="item.disabled"
          :class="['sol-context-item', { 'is-danger': item.danger }]"
          @click="
            emit('select', item.id);
            emit('close');
          "
        >
          {{ item.label }}
        </button>
      </template>
    </div>
  </Teleport>
</template>
