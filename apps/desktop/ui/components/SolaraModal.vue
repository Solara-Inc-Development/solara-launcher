<script lang="ts">
const openDialogs: symbol[] = [];
let dialogOrder = 0;
</script>
<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
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

const dialog = ref<HTMLElement | null>(null);
const identity = Symbol('dialog');
const zIndex = ref(60);
let returnFocus: HTMLElement | null = null;
function removeFromStack(): void {
  const index = openDialogs.indexOf(identity);
  if (index >= 0) openDialogs.splice(index, 1);
}
watch(() => props.open, async (open) => {
  if (open) {
    returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    removeFromStack();
    openDialogs.push(identity);
    zIndex.value = 60 + ++dialogOrder;
    await nextTick();
    if (props.open && openDialogs.at(-1) === identity) dialog.value?.focus();
  } else {
    removeFromStack();
    if (returnFocus?.isConnected) returnFocus.focus();
    returnFocus = null;
  }
}, { immediate: true, flush: 'sync' });
function onKeydown(e: KeyboardEvent): void {
  if (!props.open || openDialogs.at(-1) !== identity) return;
  if (e.key === 'Escape') {
    e.preventDefault();
    e.stopImmediatePropagation();
    if (props.closable !== false) emit('close');
  } else if (e.key === 'Tab') {
    const focusable = Array.from(dialog.value?.querySelectorAll<HTMLElement>(
      'button:not(:disabled), input:not(:disabled), textarea:not(:disabled), select:not(:disabled), a[href], [tabindex="0"]',
    ) ?? []).filter((el) => el.getClientRects().length > 0);
    const first = focusable[0];
    const last = focusable.at(-1);
    if (!first) { e.preventDefault(); dialog.value?.focus(); }
    else if (e.shiftKey && (document.activeElement === first || document.activeElement === dialog.value)) {
      e.preventDefault(); last?.focus();
    } else if (!e.shiftKey && (document.activeElement === last || document.activeElement === dialog.value)) {
      e.preventDefault(); first.focus();
    }
  }
}
onMounted(() => window.addEventListener('keydown', onKeydown, true));
onUnmounted(() => { removeFromStack(); window.removeEventListener('keydown', onKeydown, true); });
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="sol-modal-backdrop"
      :style="{ zIndex }"
      role="dialog"
      aria-modal="true"
      :aria-label="title"
      @mousedown.self="closable !== false && emit('close')"
    >
      <div ref="dialog" tabindex="-1" :class="['sol-modal', `sol-modal--${width ?? 'md'}`]">
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
