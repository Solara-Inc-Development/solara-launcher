<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
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
  Eye,
  EyeOff,
  Grid,
} from '../icons';
import SolaraButton from './SolaraButton.vue';
import { getUvRegionBox, SKIN_BODY_PARTS, SKIN_FACES } from '../skinUv.mjs';
import { floodFillRegion, mirrorPixelX, normalizeHexColor, pushRecentColor } from '../skinTools.mjs';

const props = defineProps<{
  modelValue: string;
  variant: 'classic' | 'slim';
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', dataUrl: string): void;
  (e: 'dirty', value: boolean): void;
}>();

type ToolKind = 'pencil' | 'eraser' | 'eyedropper' | 'bucket';
type PartKind = 'head' | 'torso' | 'arm-r' | 'arm-l' | 'leg-r' | 'leg-l' | 'full';
type FaceKind = 'front' | 'back' | 'left' | 'right' | 'top' | 'bottom';

const tool = ref<ToolKind>('pencil');
const color = ref('#3B82F6');
const recentColors = ref<string[]>(['#3B82F6', '#F5F4F1', '#252B36']);
const mirror = ref(false);
const showGrid = ref(true);
const layerMode = ref<'base' | 'overlay'>('base');
const activePart = ref<PartKind>('head');
const activeFace = ref<FaceKind>('front');
const showBaseGuide = ref(true);
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
  '#3B82F6', '#60A5FA', '#1D4ED8', '#0F2142',
  '#F5F4F1', '#9BA3AF', '#252B36', '#090B0F',
  '#23C987', '#65A9F5', '#F06471', '#E8A64B',
  '#C89B74', '#8D5524', '#4A3020', '#FFFFFF',
];

const currentBox = computed(() =>
  getUvRegionBox({
    part: activePart.value,
    face: activeFace.value,
    layer: layerMode.value,
    variant: props.variant,
  }),
);

function getRegionBox(): { x: number; y: number; w: number; h: number } {
  return currentBox.value;
}

let loadGeneration = 0;
function onEditorKeydown(e: KeyboardEvent) {
  const tag = (e.target as HTMLElement | null)?.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;
  const mod = e.ctrlKey || e.metaKey;
  if (mod && e.key.toLowerCase() === 'z' && !e.shiftKey) {
    e.preventDefault();
    undo();
  } else if ((mod && e.key.toLowerCase() === 'y') || (mod && e.shiftKey && e.key.toLowerCase() === 'z')) {
    e.preventDefault();
    redo();
  } else if (!mod && !e.altKey) {
    const k = e.key.toLowerCase();
    if (k === 'b') tool.value = 'pencil';
    else if (k === 'e') tool.value = 'eraser';
    else if (k === 'g') tool.value = 'bucket';
    else if (k === 'i') tool.value = 'eyedropper';
    else if (k === 'm') mirror.value = !mirror.value;
  }
}

onUnmounted(() => {
  loadGeneration++;
  window.removeEventListener('keydown', onEditorKeydown);
});
function loadFromDataUrl(url: string, resetHistory = true) {
  const generation = ++loadGeneration;
  if (!url) {
    pctx.clearRect(0, 0, 64, 64);
    renderView();
    return;
  }
  const img = new Image();
  img.onload = () => {
    if (generation !== loadGeneration) return;
    pctx.clearRect(0, 0, 64, 64);
    pctx.drawImage(img, 0, 0, 64, Math.min(64, img.height === 32 ? 32 : 64));
    if (resetHistory) {
      undoStack.value = [];
      redoStack.value = [];
    }
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

watch([activePart, activeFace, layerMode, showBaseGuide, showGrid, mirror, zoom, () => props.variant], () => renderView());

onMounted(() => {
  window.addEventListener('keydown', onEditorKeydown);
  if (props.modelValue) {
    loadFromDataUrl(props.modelValue);
  } else {
    pctx.fillStyle = '#c89b74';
    pctx.fillRect(8, 8, 8, 8);
    pctx.fillStyle = '#252b36';
    pctx.fillRect(20, 20, 8, 12);
    renderView();
    emit('update:modelValue', pixelCanvas.toDataURL('image/png'));
  }
});

function renderView() {
  const vc = viewCanvasRef.value;
  if (!vc) return;
  const box = getRegionBox();
  const pxScale = activePart.value === 'full' ? Math.max(4, Math.floor(zoom.value / 2.5)) : zoom.value;
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

  // Translucent Base Layer reference when editing Outer Layer
  if (layerMode.value === 'overlay' && activePart.value !== 'full' && showBaseGuide.value) {
    const baseBox = getUvRegionBox({
      part: activePart.value,
      face: activeFace.value,
      layer: 'base',
      variant: props.variant,
    });
    vctx.save();
    vctx.globalAlpha = 0.28;
    vctx.drawImage(pixelCanvas, baseBox.x, baseBox.y, baseBox.w, baseBox.h, 0, 0, vc.width, vc.height);
    vctx.restore();
  }

  vctx.drawImage(pixelCanvas, box.x, box.y, box.w, box.h, 0, 0, vc.width, vc.height);

  // Subtle pixel grid
  if (showGrid.value && pxScale >= 6) {
    vctx.strokeStyle = 'rgba(255,255,255,0.07)';
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

  // Horizontal mirror symmetry axis guide
  if (mirror.value && activePart.value !== 'full') {
    const midX = (box.w / 2) * pxScale;
    vctx.save();
    vctx.strokeStyle = 'rgba(59, 130, 246, 0.78)';
    vctx.lineWidth = 1.5;
    vctx.setLineDash([4, 3]);
    vctx.beginPath();
    vctx.moveTo(midX, 0);
    vctx.lineTo(midX, vc.height);
    vctx.stroke();
    vctx.restore();
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

function rememberActiveColor() {
  const norm = normalizeHexColor(color.value);
  if (norm) {
    color.value = norm;
    recentColors.value = pushRecentColor(recentColors.value, norm, 8);
  }
}

function paintAt(gx: number, gy: number, box: { x: number; y: number; w: number; h: number }) {
  if (tool.value === 'pencil') {
    const activeHex = normalizeHexColor(color.value) ?? '#3B82F6';
    pctx.fillStyle = activeHex;
    pctx.fillRect(gx, gy, 1, 1);
    if (mirror.value) {
      pctx.fillRect(mirrorPixelX(gx, box), gy, 1, 1);
    }
  } else if (tool.value === 'eraser') {
    pctx.clearRect(gx, gy, 1, 1);
    if (mirror.value) {
      pctx.clearRect(mirrorPixelX(gx, box), gy, 1, 1);
    }
  }
}

function floodFill(startX: number, startY: number, box: { x: number; y: number; w: number; h: number }) {
  const img = pctx.getImageData(0, 0, 64, 64);
  const changed = floodFillRegion(img.data, 64, startX, startY, box, color.value);
  if (changed > 0) {
    pctx.putImageData(img, 0, 0);
    rememberActiveColor();
  }
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
          .join('')
          .toUpperCase();
      rememberActiveColor();
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
  if (tool.value === 'pencil') rememberActiveColor();
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
    const dataUrl = typeof r.result === 'string' ? r.result : '';
    if (!dataUrl.startsWith('data:image/png')) return;
    const probe = new Image();
    probe.onload = () => {
      if (probe.width !== 64 || (probe.height !== 64 && probe.height !== 32)) return;
      snapshotUndo();
      loadFromDataUrl(dataUrl, false);
      emit('update:modelValue', dataUrl);
      emit('dirty', true);
    };
    probe.src = dataUrl;
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
          title="Horizontal mirror (M)"
          @click="mirror = !mirror"
        >
          <FlipHorizontal :size="13" /> Mirror
        </button>
        <button
          type="button"
          :class="['sol-chip', { 'is-active': showGrid }]"
          title="Toggle pixel grid"
          aria-label="Toggle pixel grid"
          @click="showGrid = !showGrid"
        >
          <Grid :size="13" /> Grid
        </button>
        <button type="button" class="sol-icon-btn sol-icon-btn--xs" :disabled="undoStack.length === 0" title="Undo (Ctrl+Z)" aria-label="Undo" @click="undo">
          <Undo2 :size="13" />
        </button>
        <button type="button" class="sol-icon-btn sol-icon-btn--xs" :disabled="redoStack.length === 0" title="Redo (Ctrl+Y)" aria-label="Redo" @click="redo">
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

    <!-- Body Part, 6-Face Cube UV & Layer Controls -->
    <div class="sol-editor-subbar">
      <div class="sol-seg-group" role="group" aria-label="Body part">
        <button
          v-for="part in SKIN_BODY_PARTS"
          :key="part.id"
          type="button"
          :class="['sol-seg-btn', { 'is-active': activePart === part.id }]"
          @click="activePart = part.id as PartKind"
        >
          {{ part.label }}
        </button>
      </div>

      <div class="sol-row-gap">
        <div class="sol-seg-group" role="group" aria-label="Skin layer">
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

        <button
          v-if="layerMode === 'overlay' && activePart !== 'full'"
          type="button"
          :class="['sol-icon-btn', 'sol-icon-btn--xs', { 'is-active': showBaseGuide }]"
          title="Toggle translucent base guide"
          aria-label="Toggle base layer guide"
          @click="showBaseGuide = !showBaseGuide"
        >
          <Eye v-if="showBaseGuide" :size="13" />
          <EyeOff v-else :size="13" />
        </button>
      </div>
    </div>

    <div v-if="activePart !== 'full'" class="sol-editor-facebar">
      <div class="sol-seg-group" role="group" aria-label="6-face UV selector">
        <button
          v-for="face in SKIN_FACES"
          :key="face.id"
          type="button"
          :class="['sol-seg-btn', 'sol-face-btn', { 'is-active': activeFace === face.id }]"
          @click="activeFace = face.id as FaceKind"
        >
          {{ face.label }}
        </button>
      </div>
      <span class="sol-pill sol-uv-readout">
        UV ({{ currentBox.x }}, {{ currentBox.y }}) · {{ currentBox.w }}×{{ currentBox.h }}
      </span>
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

        <div v-if="recentColors.length > 0" class="sol-field">
          <label class="sol-label">Recent Colors</label>
          <div class="sol-swatch-grid">
            <button
              v-for="rc in recentColors"
              :key="`recent-${rc}`"
              type="button"
              :class="['sol-swatch', { 'is-active': color.toUpperCase() === rc.toUpperCase() }]"
              :style="{ background: rc }"
              :title="rc"
              @click="color = rc"
            />
          </div>
        </div>

        <div class="sol-field">
          <label class="sol-label">Studio Palette</label>
          <div class="sol-swatch-grid">
            <button
              v-for="sw in swatchPalette"
              :key="sw"
              type="button"
              :class="['sol-swatch', { 'is-active': color.toUpperCase() === sw.toUpperCase() }]"
              :style="{ background: sw }"
              :title="sw"
              @click="color = sw"
            />
          </div>
        </div>

        <div class="sol-field">
          <label class="sol-label">Canvas Zoom ({{ zoom }}x)</label>
          <input v-model.number="zoom" type="range" min="10" max="28" step="2" class="sol-range" />
        </div>

        <p class="sol-shortcut-hint" style="margin: 0">
          Shortcuts: B Draw · E Erase · G Fill · I Pick · M Mirror · Ctrl+Z/Y Undo/Redo
        </p>
      </div>
    </div>
  </div>
</template>
