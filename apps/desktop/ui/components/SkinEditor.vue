<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import {
  Palette,
  Eraser,
  Pipette,
  PaintBucket,
  Undo2,
  Redo2,
  FlipHorizontal,
  Download,
  Upload,
} from '../icons';
import SolaraButton from './SolaraButton.vue';

const props = defineProps<{
  modelValue: string;
  variant: 'classic' | 'slim';
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', dataUrl: string): void;
  (e: 'dirty', value: boolean): void;
}>();

type ToolKind = 'pencil' | 'eraser' | 'eyedropper' | 'bucket';
type RegionView = 'head-front' | 'torso-front' | 'arm-r' | 'leg-r' | 'full';

const tool = ref<ToolKind>('pencil');
const color = ref('#E9B86D');
const mirror = ref(false);
const layerMode = ref<'base' | 'overlay'>('base');
const activeRegion = ref<RegionView>('head-front');
const zoom = ref(16);

const pixelCanvas = document.createElement('canvas');
pixelCanvas.width = 64;
pixelCanvas.height = 64;
const pctx = pixelCanvas.getContext('2d', { willReadFrequently: true })!;

const viewCanvasRef = ref<HTMLCanvasElement | null>(null);
const undoStack = ref<ImageData[]>([]);
const redoStack = ref<ImageData[]>([]);
const importInput = ref<HTMLInputElement | null>(null);

const swatchPalette = [
  '#E9B86D', '#F5CD91', '#D7A258', '#17130E',
  '#F5F4F1', '#9BA3AF', '#252B36', '#090B0F',
  '#23C987', '#65A9F5', '#F06471', '#E8A64B',
  '#C89B74', '#8D5524', '#4A3020', '#FFFFFF',
];

function getRegionBox(): { x: number; y: number; w: number; h: number } {
  const isOv = layerMode.value === 'overlay';
  switch (activeRegion.value) {
    case 'head-front':
      return isOv ? { x: 40, y: 8, w: 8, h: 8 } : { x: 8, y: 8, w: 8, h: 8 };
    case 'torso-front':
      return isOv ? { x: 20, y: 36, w: 8, h: 12 } : { x: 20, y: 20, w: 8, h: 12 };
    case 'arm-r':
      return isOv
        ? { x: 44, y: 36, w: props.variant === 'slim' ? 3 : 4, h: 12 }
        : { x: 44, y: 20, w: props.variant === 'slim' ? 3 : 4, h: 12 };
    case 'leg-r':
      return isOv ? { x: 4, y: 36, w: 4, h: 12 } : { x: 4, y: 20, w: 4, h: 12 };
    default:
      return { x: 0, y: 0, w: 64, h: 64 };
  }
}

function loadFromDataUrl(url: string) {
  if (!url) {
    pctx.clearRect(0, 0, 64, 64);
    renderView();
    return;
  }
  const img = new Image();
  img.onload = () => {
    pctx.clearRect(0, 0, 64, 64);
    pctx.drawImage(img, 0, 0, 64, Math.min(64, img.height === 32 ? 32 : 64));
    undoStack.value = [];
    redoStack.value = [];
    renderView();
  };
  img.src = url;
}

watch(
  () => props.modelValue,
  (val) => {
    if (val && val !== pixelCanvas.toDataURL('image/png')) {
      loadFromDataUrl(val);
    }
  },
);

watch([activeRegion, layerMode, zoom, () => props.variant], () => renderView());

onMounted(() => {
  if (props.modelValue) {
    loadFromDataUrl(props.modelValue);
  } else {
    pctx.fillStyle = '#c89b74';
    pctx.fillRect(8, 8, 8, 8);
    pctx.fillStyle = '#252b36';
    pctx.fillRect(20, 20, 8, 12);
    renderView();
  }
});

function renderView() {
  const vc = viewCanvasRef.value;
  if (!vc) return;
  const box = getRegionBox();
  const pxScale = activeRegion.value === 'full' ? Math.max(4, Math.floor(zoom.value / 2.5)) : zoom.value;
  vc.width = box.w * pxScale;
  vc.height = box.h * pxScale;
  const vctx = vc.getContext('2d')!;
  vctx.imageSmoothingEnabled = false;

  // Checkerboard background
  for (let y = 0; y < box.h; y++) {
    for (let x = 0; x < box.w; x++) {
      vctx.fillStyle = (x + y) % 2 === 0 ? '#181B22' : '#12151B';
      vctx.fillRect(x * pxScale, y * pxScale, pxScale, pxScale);
    }
  }

  vctx.drawImage(pixelCanvas, box.x, box.y, box.w, box.h, 0, 0, vc.width, vc.height);

  // Subtle pixel grid
  if (pxScale >= 8) {
    vctx.strokeStyle = 'rgba(255,255,255,0.06)';
    vctx.lineWidth = 1;
    for (let x = 0; x <= box.w; x++) {
      vctx.beginPath();
      vctx.moveTo(x * pxScale + 0.5, 0);
      vctx.lineTo(x * pxScale + 0.5, vc.height);
      vctx.stroke();
    }
    for (let y = 0; y <= box.h; y++) {
      vctx.beginPath();
      vctx.moveTo(0, y * pxScale + 0.5);
      vctx.lineTo(vc.width, y * pxScale + 0.5);
      vctx.stroke();
    }
  }
}

function snapshotUndo() {
  undoStack.value.push(pctx.getImageData(0, 0, 64, 64));
  if (undoStack.value.length > 40) undoStack.value.shift();
  redoStack.value = [];
}

function undo() {
  const prev = undoStack.value.pop();
  if (!prev) return;
  redoStack.value.push(pctx.getImageData(0, 0, 64, 64));
  pctx.putImageData(prev, 0, 0);
  commitChange();
}

function redo() {
  const next = redoStack.value.pop();
  if (!next) return;
  undoStack.value.push(pctx.getImageData(0, 0, 64, 64));
  pctx.putImageData(next, 0, 0);
  commitChange();
}

function commitChange() {
  renderView();
  emit('update:modelValue', pixelCanvas.toDataURL('image/png'));
  emit('dirty', true);
}

let painting = false;

function eventToPixel(e: PointerEvent): { gx: number; gy: number; box: { x: number; y: number; w: number; h: number } } | null {
  const vc = viewCanvasRef.value;
  if (!vc) return null;
  const rect = vc.getBoundingClientRect();
  const box = getRegionBox();
  const relX = Math.floor(((e.clientX - rect.left) / rect.width) * box.w);
  const relY = Math.floor(((e.clientY - rect.top) / rect.height) * box.h);
  if (relX < 0 || relY < 0 || relX >= box.w || relY >= box.h) return null;
  return { gx: box.x + relX, gy: box.y + relY, box };
}

function paintAt(gx: number, gy: number, box: { x: number; y: number; w: number; h: number }) {
  if (tool.value === 'pencil') {
    pctx.fillStyle = color.value;
    pctx.fillRect(gx, gy, 1, 1);
    if (mirror.value) {
      const mx = box.x + (box.w - 1 - (gx - box.x));
      pctx.fillRect(mx, gy, 1, 1);
    }
  } else if (tool.value === 'eraser') {
    pctx.clearRect(gx, gy, 1, 1);
    if (mirror.value) {
      const mx = box.x + (box.w - 1 - (gx - box.x));
      pctx.clearRect(mx, gy, 1, 1);
    }
  }
}

function floodFill(startX: number, startY: number, box: { x: number; y: number; w: number; h: number }) {
  const img = pctx.getImageData(0, 0, 64, 64);
  const data = img.data;
  const idx = (x: number, y: number) => (y * 64 + x) * 4;
  const s = idx(startX, startY);
  const tr = data[s], tg = data[s + 1], tb = data[s + 2], ta = data[s + 3];

  const hex = color.value.replace('#', '');
  const nr = parseInt(hex.slice(0, 2), 16);
  const ng = parseInt(hex.slice(2, 4), 16);
  const nb = parseInt(hex.slice(4, 6), 16);
  if (tr === nr && tg === ng && tb === nb && ta === 255) return;

  const stack: [number, number][] = [[startX, startY]];
  const visited = new Set<string>();
  while (stack.length > 0) {
    const [x, y] = stack.pop()!;
    const key = `${x},${y}`;
    if (visited.has(key)) continue;
    visited.add(key);
    if (x < box.x || y < box.y || x >= box.x + box.w || y >= box.y + box.h) continue;
    const i = idx(x, y);
    if (data[i] !== tr || data[i + 1] !== tg || data[i + 2] !== tb || data[i + 3] !== ta) continue;
    data[i] = nr;
    data[i + 1] = ng;
    data[i + 2] = nb;
    data[i + 3] = 255;
    stack.push([x + 1, y], [x - 1, y], [x, y + 1], [x, y - 1]);
  }
  pctx.putImageData(img, 0, 0);
}

function onPointerDown(e: PointerEvent) {
  const pt = eventToPixel(e);
  if (!pt) return;
  if (tool.value === 'eyedropper') {
    const px = pctx.getImageData(pt.gx, pt.gy, 1, 1).data;
    if (px[3] > 0) {
      color.value =
        '#' +
        [px[0], px[1], px[2]]
          .map((n) => n.toString(16).padStart(2, '0'))
          .join('');
    }
    tool.value = 'pencil';
    return;
  }
  snapshotUndo();
  if (tool.value === 'bucket') {
    floodFill(pt.gx, pt.gy, pt.box);
    commitChange();
    return;
  }
  painting = true;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  paintAt(pt.gx, pt.gy, pt.box);
  commitChange();
}

function onPointerMove(e: PointerEvent) {
  if (!painting) return;
  const pt = eventToPixel(e);
  if (!pt) return;
  paintAt(pt.gx, pt.gy, pt.box);
  commitChange();
}

function onPointerUp() {
  painting = false;
}

function exportPng() {
  const a = document.createElement('a');
  a.href = pixelCanvas.toDataURL('image/png');
  a.download = 'solara-skin.png';
  a.click();
}

function onImportFile(e: Event) {
  const f = (e.target as HTMLInputElement).files?.[0];
  (e.target as HTMLInputElement).value = '';
  if (!f) return;
  const r = new FileReader();
  r.onload = () => {
    if (typeof r.result === 'string' && r.result.startsWith('data:image/')) {
      snapshotUndo();
      loadFromDataUrl(r.result);
      emit('update:modelValue', r.result);
      emit('dirty', true);
    }
  };
  r.readAsDataURL(f);
}
</script>

<template>
  <div class="sol-skin-editor">
    <!-- Top Tool Bar -->
    <div class="sol-editor-toolbar">
      <div class="sol-seg-group">
        <button
          type="button"
          :class="['sol-seg-btn', { 'is-active': tool === 'pencil' }]"
          title="Pencil"
          @click="tool = 'pencil'"
        >
          <Palette :size="13" /> Draw
        </button>
        <button
          type="button"
          :class="['sol-seg-btn', { 'is-active': tool === 'eraser' }]"
          title="Eraser"
          @click="tool = 'eraser'"
        >
          <Eraser :size="13" /> Erase
        </button>
        <button
          type="button"
          :class="['sol-seg-btn', { 'is-active': tool === 'bucket' }]"
          title="Bucket Fill"
          @click="tool = 'bucket'"
        >
          <PaintBucket :size="13" /> Fill
        </button>
        <button
          type="button"
          :class="['sol-seg-btn', { 'is-active': tool === 'eyedropper' }]"
          title="Color Picker"
          @click="tool = 'eyedropper'"
        >
          <Pipette :size="13" /> Pick
        </button>
      </div>

      <div class="sol-row-gap">
        <button
          type="button"
          :class="['sol-chip', { 'is-active': mirror }]"
          title="Horizontal mirror"
          @click="mirror = !mirror"
        >
          <FlipHorizontal :size="13" /> Mirror
        </button>
        <button type="button" class="sol-icon-btn sol-icon-btn--xs" :disabled="undoStack.length === 0" title="Undo" @click="undo">
          <Undo2 :size="13" />
        </button>
        <button type="button" class="sol-icon-btn sol-icon-btn--xs" :disabled="redoStack.length === 0" title="Redo" @click="redo">
          <Redo2 :size="13" />
        </button>
      </div>

      <div class="sol-row-gap">
        <input ref="importInput" type="file" accept="image/png" hidden @change="onImportFile" />
        <SolaraButton variant="secondary" size="xs" @click="importInput?.click()">
          <Upload :size="12" /> Import PNG
        </SolaraButton>
        <SolaraButton variant="secondary" size="xs" @click="exportPng">
          <Download :size="12" /> Export PNG
        </SolaraButton>
      </div>
    </div>

    <!-- Region & Layer Controls -->
    <div class="sol-editor-subbar">
      <div class="sol-seg-group">
        <button
          v-for="r in [
            { id: 'head-front', label: 'Head' },
            { id: 'torso-front', label: 'Torso' },
            { id: 'arm-r', label: 'Arm' },
            { id: 'leg-r', label: 'Leg' },
            { id: 'full', label: 'Full UV (64×64)' },
          ] as const"
          :key="r.id"
          type="button"
          :class="['sol-seg-btn', { 'is-active': activeRegion === r.id }]"
          @click="activeRegion = r.id"
        >
          {{ r.label }}
        </button>
      </div>

      <div class="sol-seg-group">
        <button
          type="button"
          :class="['sol-seg-btn', { 'is-active': layerMode === 'base' }]"
          @click="layerMode = 'base'"
        >
          Base Layer
        </button>
        <button
          type="button"
          :class="['sol-seg-btn', { 'is-active': layerMode === 'overlay' }]"
          @click="layerMode = 'overlay'"
        >
          Outer Layer
        </button>
      </div>
    </div>

    <!-- Main Pixel Canvas & Color Palette -->
    <div class="sol-editor-body">
      <div class="sol-editor-canvas-wrap">
        <canvas
          ref="viewCanvasRef"
          class="sol-editor-canvas"
          @pointerdown="onPointerDown"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
        />
      </div>

      <div class="sol-editor-palette">
        <div class="sol-field">
          <label class="sol-label">Active Color</label>
          <div class="sol-row-gap">
            <input v-model="color" type="color" class="sol-color-picker" />
            <input v-model="color" type="text" class="sol-input sol-input--sm" maxlength="7" />
          </div>
        </div>

        <div class="sol-swatch-grid">
          <button
            v-for="sw in swatchPalette"
            :key="sw"
            type="button"
            class="sol-swatch"
            :style="{ background: sw }"
            :title="sw"
            @click="color = sw"
          />
        </div>

        <div class="sol-field">
          <label class="sol-label">Canvas Zoom ({{ zoom }}x)</label>
          <input v-model.number="zoom" type="range" min="10" max="28" step="2" class="sol-range" />
        </div>
      </div>
    </div>
  </div>
</template>
