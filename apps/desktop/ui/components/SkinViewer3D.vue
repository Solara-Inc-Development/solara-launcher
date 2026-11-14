<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { RotateCw } from '../icons';

const props = withDefaults(
  defineProps<{
    skinDataUrl?: string | null;
    capeDataUrl?: string | null;
    variant?: 'classic' | 'slim';
    showOuter?: boolean;
    animate?: boolean;
    height?: number;
  }>(),
  {
    showOuter: true,
    animate: true,
  },
);

const canvasRef = ref<HTMLCanvasElement | null>(null);
const yaw = ref(-0.42);
const pitch = ref(0.18);
const zoom = ref(1.0);
const rendererMode = ref<'webgl' | '2d'>('webgl');

let rafId = 0;
let dragging = false;
let lastX = 0;
let lastY = 0;
let startTime = performance.now();

let skinImg: HTMLImageElement | null = null;
let capeImg: HTMLImageElement | null = null;
let skinReady = false;
let capeReady = false;

// Default Steve fallback skin (clean procedural 64x64 pixels)
function buildDefaultSkinCanvas(): HTMLCanvasElement {
  const c = document.createElement('canvas');
  c.width = 64;
  c.height = 64;
  const ctx = c.getContext('2d')!;
  // Head skin base (8,8 -> 16,16)
  ctx.fillStyle = '#c89b74';
  ctx.fillRect(0, 0, 32, 16);
  // Hair
  ctx.fillStyle = '#4a3020';
  ctx.fillRect(8, 0, 8, 8);
  ctx.fillRect(8, 8, 8, 2);
  ctx.fillRect(0, 8, 8, 4);
  ctx.fillRect(16, 8, 8, 4);
  ctx.fillRect(24, 8, 8, 8);
  // Eyes
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(9, 11, 2, 1);
  ctx.fillRect(13, 11, 2, 1);
  ctx.fillStyle = '#3b4f91';
  ctx.fillRect(10, 11, 1, 1);
  ctx.fillRect(13, 11, 1, 1);
  // Torso (Solara Dawn Gold shirt)
  ctx.fillStyle = '#252b36';
  ctx.fillRect(16, 16, 24, 16);
  ctx.fillStyle = '#E9B86D';
  ctx.fillRect(22, 21, 4, 6);
  // Arms
  ctx.fillStyle = '#252b36';
  ctx.fillRect(40, 16, 16, 8);
  ctx.fillRect(32, 48, 16, 8);
  ctx.fillStyle = '#c89b74';
  ctx.fillRect(40, 24, 16, 8);
  ctx.fillRect(32, 56, 16, 8);
  // Legs
  ctx.fillStyle = '#191d24';
  ctx.fillRect(0, 16, 16, 16);
  ctx.fillRect(16, 48, 16, 16);
  return c;
}

function loadImages() {
  skinReady = false;
  capeReady = false;
  if (props.skinDataUrl) {
    const img = new Image();
    img.onload = () => {
      skinImg = img;
      skinReady = true;
    };
    img.onerror = () => {
      skinImg = null;
      skinReady = false;
    };
    img.src = props.skinDataUrl;
  } else {
    skinImg = null;
  }

  if (props.capeDataUrl) {
    const cimg = new Image();
    cimg.onload = () => {
      capeImg = cimg;
      capeReady = true;
    };
    cimg.onerror = () => {
      capeImg = null;
      capeReady = false;
    };
    cimg.src = props.capeDataUrl;
  } else {
    capeImg = null;
  }
}

watch(() => [props.skinDataUrl, props.capeDataUrl], loadImages, { immediate: true });

// Minimal WebGL Minecraft Rig
interface MeshBox {
  w: number;
  h: number;
  d: number;
  texW: number;
  texH: number;
  u: number;
  v: number;
}

function createBoxGeometry(box: MeshBox) {
  const { w, h, d, texW, texH, u, v } = box;
  const hx = w / 2;
  const hy = h / 2;
  const hz = d / 2;
  const positions: number[] = [];
  const uvs: number[] = [];
  const indices: number[] = [];

  function addFace(
    verts: [number, number, number][],
    rx: number,
    ry: number,
    rw: number,
    rh: number,
  ) {
    const base = positions.length / 3;
    for (const [x, y, z] of verts) positions.push(x, y, z);
    const u0 = rx / texW;
    const v0 = ry / texH;
    const u1 = (rx + rw) / texW;
    const v1 = (ry + rh) / texH;
    uvs.push(u0, v0, u1, v0, u1, v1, u0, v1);
    indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
  }

  // Right (+x), Left (-x), Top (+y), Bottom (-y), Front (+z), Back (-z)
  addFace(
    [
      [hx, hy, -hz],
      [hx, hy, hz],
      [hx, -hy, hz],
      [hx, -hy, -hz],
    ],
    u,
    v + d,
    d,
    h,
  );
  addFace(
    [
      [-hx, hy, hz],
      [-hx, hy, -hz],
      [-hx, -hy, -hz],
      [-hx, -hy, hz],
    ],
    u + d + w,
    v + d,
    d,
    h,
  );
  addFace(
    [
      [-hx, hy, -hz],
      [hx, hy, -hz],
      [hx, hy, hz],
      [-hx, hy, hz],
    ],
    u + d,
    v,
    w,
    d,
  );
  addFace(
    [
      [-hx, -hy, hz],
      [hx, -hy, hz],
      [hx, -hy, -hz],
      [-hx, -hy, -hz],
    ],
    u + d + w,
    v,
    w,
    d,
  );
  addFace(
    [
      [-hx, hy, hz],
      [hx, hy, hz],
      [hx, -hy, hz],
      [-hx, -hy, hz],
    ],
    u + d,
    v + d,
    w,
    h,
  );
  addFace(
    [
      [hx, hy, -hz],
      [-hx, hy, -hz],
      [-hx, -hy, -hz],
      [hx, -hy, -hz],
    ],
    u + d + w + d,
    v + d,
    w,
    h,
  );

  return {
    positions: new Float32Array(positions),
    uvs: new Float32Array(uvs),
    indices: new Uint16Array(indices),
  };
}

function mat4Mul(a: Float32Array, b: Float32Array): Float32Array {
  const out = new Float32Array(16);
  for (let c = 0; c < 4; c++) {
    for (let r = 0; r < 4; r++) {
      out[c * 4 + r] =
        a[0 * 4 + r] * b[c * 4 + 0] +
        a[1 * 4 + r] * b[c * 4 + 1] +
        a[2 * 4 + r] * b[c * 4 + 2] +
        a[3 * 4 + r] * b[c * 4 + 3];
    }
  }
  return out;
}

function mat4Persp(fov: number, aspect: number, near: number, far: number): Float32Array {
  const f = 1 / Math.tan(fov / 2);
  const nf = 1 / (near - far);
  return new Float32Array([
    f / aspect, 0, 0, 0,
    0, f, 0, 0,
    0, 0, (far + near) * nf, -1,
    0, 0, 2 * far * near * nf, 0,
  ]);
}

function mat4Trans(x: number, y: number, z: number): Float32Array {
  return new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, x, y, z, 1]);
}

function mat4RotX(rad: number): Float32Array {
  const c = Math.cos(rad);
  const s = Math.sin(rad);
  return new Float32Array([1, 0, 0, 0, 0, c, s, 0, 0, -s, c, 0, 0, 0, 0, 1]);
}

function mat4RotY(rad: number): Float32Array {
  const c = Math.cos(rad);
  const s = Math.sin(rad);
  return new Float32Array([c, 0, -s, 0, 0, 1, 0, 0, s, 0, c, 0, 0, 0, 0, 1]);
}

let gl: WebGLRenderingContext | null = null;
let fallbackSkin: HTMLCanvasElement | null = null;

function draw2DFallback(canvas: HTMLCanvasElement, t: number) {
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  const w = canvas.width;
  const h = canvas.height;
  ctx.clearRect(0, 0, w, h);
  ctx.imageSmoothingEnabled = false;

  const src: CanvasImageSource =
    skinReady && skinImg ? skinImg : (fallbackSkin ??= buildDefaultSkinCanvas());
  const sway = props.animate !== false ? Math.sin(t * 2.2) * 3 : 0;
  const cx = w / 2;
  const cy = h * 0.5;
  const s = Math.min(w, h) * 0.022 * zoom.value;
  const armW = props.variant === 'slim' ? 3 : 4;
  const showBack = Math.cos(yaw.value) < 0;

  ctx.save();
  ctx.translate(cx, cy - 2 * s);

  if (capeReady && capeImg && showBack) {
    ctx.drawImage(capeImg, 1, 1, 10, 16, -5 * s, -4 * s, 10 * s, 16 * s);
  }

  // Head
  const headU = showBack ? 24 : 8;
  ctx.drawImage(src, headU, 8, 8, 8, -4 * s, -12 * s, 8 * s, 8 * s);
  if (props.showOuter !== false) {
    ctx.drawImage(src, showBack ? 56 : 40, 8, 8, 8, -4.3 * s, -12.3 * s, 8.6 * s, 8.6 * s);
  }
  // Body
  ctx.drawImage(src, showBack ? 32 : 20, 20, 8, 12, -4 * s, -4 * s, 8 * s, 12 * s);
  // Arms
  ctx.drawImage(src, showBack ? 52 : 44, 20, armW, 12, (-4 - armW) * s, (-4 + sway * 0.15) * s, armW * s, 12 * s);
  ctx.drawImage(src, showBack ? 44 : 36, 52, armW, 12, 4 * s, (-4 - sway * 0.15) * s, armW * s, 12 * s);
  // Legs
  ctx.drawImage(src, showBack ? 12 : 4, 20, 4, 12, -4 * s, (8 - sway * 0.15) * s, 4 * s, 12 * s);
  ctx.drawImage(src, showBack ? 28 : 20, 52, 4, 12, 0, (8 + sway * 0.15) * s, 4 * s, 12 * s);

  ctx.restore();
}

function initWebGL(canvas: HTMLCanvasElement): boolean {
  const ctx = canvas.getContext('webgl', { alpha: true, antialias: true });
  if (!ctx) return false;
  gl = ctx;

  const vsSrc = `
    attribute vec3 aPos;
    attribute vec2 aUv;
    uniform mat4 uMvp;
    varying vec2 vUv;
    void main() {
      gl_Position = uMvp * vec4(aPos, 1.0);
      vUv = aUv;
    }
  `;
  const fsSrc = `
    precision mediump float;
    uniform sampler2D uTex;
    varying vec2 vUv;
    void main() {
      vec4 col = texture2D(uTex, vUv);
      if (col.a < 0.05) discard;
      gl_FragColor = col;
    }
  `;
  const compile = (type: number, src: string) => {
    const s = ctx.createShader(type)!;
    ctx.shaderSource(s, src);
    ctx.compileShader(s);
    return s;
  };
  const prog = ctx.createProgram()!;
  ctx.attachShader(prog, compile(ctx.VERTEX_SHADER, vsSrc));
  ctx.attachShader(prog, compile(ctx.FRAGMENT_SHADER, fsSrc));
  ctx.linkProgram(prog);
  ctx.useProgram(prog);

  const aPos = ctx.getAttribLocation(prog, 'aPos');
  const aUv = ctx.getAttribLocation(prog, 'aUv');
  const uMvp = ctx.getUniformLocation(prog, 'uMvp');
  const uTex = ctx.getUniformLocation(prog, 'uTex');

  const createTex = () => {
    const tex = ctx.createTexture()!;
    ctx.bindTexture(ctx.TEXTURE_2D, tex);
    ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_MIN_FILTER, ctx.NEAREST);
    ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_MAG_FILTER, ctx.NEAREST);
    ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_S, ctx.CLAMP_TO_EDGE);
    ctx.texParameteri(ctx.TEXTURE_2D, ctx.TEXTURE_WRAP_T, ctx.CLAMP_TO_EDGE);
    return tex;
  };
  const skinTex = createTex();
  const capeTex = createTex();
  let uploadedSkin: unknown = null;
  let uploadedCape: unknown = null;

  const drawBox = (
    geom: ReturnType<typeof createBoxGeometry>,
    mvp: Float32Array,
    tex: WebGLTexture,
  ) => {
    const pBuf = ctx.createBuffer();
    ctx.bindBuffer(ctx.ARRAY_BUFFER, pBuf);
    ctx.bufferData(ctx.ARRAY_BUFFER, geom.positions, ctx.STATIC_DRAW);
    ctx.enableVertexAttribArray(aPos);
    ctx.vertexAttribPointer(aPos, 3, ctx.FLOAT, false, 0, 0);

    const uBuf = ctx.createBuffer();
    ctx.bindBuffer(ctx.ARRAY_BUFFER, uBuf);
    ctx.bufferData(ctx.ARRAY_BUFFER, geom.uvs, ctx.STATIC_DRAW);
    ctx.enableVertexAttribArray(aUv);
    ctx.vertexAttribPointer(aUv, 2, ctx.FLOAT, false, 0, 0);

    const iBuf = ctx.createBuffer();
    ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, iBuf);
    ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, geom.indices, ctx.STATIC_DRAW);

    ctx.activeTexture(ctx.TEXTURE0);
    ctx.bindTexture(ctx.TEXTURE_2D, tex);
    ctx.uniform1i(uTex, 0);
    ctx.uniformMatrix4fv(uMvp, false, mvp);
    ctx.drawElements(ctx.TRIANGLES, geom.indices.length, ctx.UNSIGNED_SHORT, 0);

    ctx.deleteBuffer(pBuf);
    ctx.deleteBuffer(uBuf);
    ctx.deleteBuffer(iBuf);
  };

  const renderFrame = () => {
    if (!canvasRef.value) return;
    if (document.hidden) {
      rafId = requestAnimationFrame(renderFrame);
      return;
    }
    const cw = canvas.clientWidth || 280;
    const ch = canvas.clientHeight || 340;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    if (canvas.width !== Math.round(cw * dpr) || canvas.height !== Math.round(ch * dpr)) {
      canvas.width = Math.round(cw * dpr);
      canvas.height = Math.round(ch * dpr);
    }

    const srcSkin = skinReady && skinImg ? skinImg : (fallbackSkin ??= buildDefaultSkinCanvas());
    if (uploadedSkin !== srcSkin) {
      ctx.bindTexture(ctx.TEXTURE_2D, skinTex);
      ctx.texImage2D(ctx.TEXTURE_2D, 0, ctx.RGBA, ctx.RGBA, ctx.UNSIGNED_BYTE, srcSkin);
      uploadedSkin = srcSkin;
    }
    if (capeReady && capeImg && uploadedCape !== capeImg) {
      ctx.bindTexture(ctx.TEXTURE_2D, capeTex);
      ctx.texImage2D(ctx.TEXTURE_2D, 0, ctx.RGBA, ctx.RGBA, ctx.UNSIGNED_BYTE, capeImg);
      uploadedCape = capeImg;
    }

    ctx.viewport(0, 0, canvas.width, canvas.height);
    ctx.clearColor(0, 0, 0, 0);
    ctx.clear(ctx.COLOR_BUFFER_BIT | ctx.DEPTH_BUFFER_BIT);
    ctx.enable(ctx.DEPTH_TEST);
    ctx.enable(ctx.BLEND);
    ctx.blendFunc(ctx.SRC_ALPHA, ctx.ONE_MINUS_SRC_ALPHA);

    const t = (performance.now() - startTime) / 1000;
    const swing = props.animate !== false ? Math.sin(t * 2.4) * 0.35 : 0;

    const proj = mat4Persp(0.6, canvas.width / Math.max(1, canvas.height), 0.1, 200);
    const cam = mat4Mul(
      mat4Trans(0, -1, -48 / zoom.value),
      mat4Mul(mat4RotX(pitch.value), mat4RotY(yaw.value)),
    );
    const vp = mat4Mul(proj, cam);

    const armW = props.variant === 'slim' ? 3 : 4;
    const armOffX = props.variant === 'slim' ? 5.5 : 6;
    const legacy = Boolean(skinReady && skinImg && skinImg.height === 32);
    const texH = legacy ? 32 : 64;

    const parts: { box: MeshBox; m: Float32Array; tex: WebGLTexture }[] = [
      // Head
      { box: { w: 8, h: 8, d: 8, texW: 64, texH, u: 0, v: 0 }, m: mat4Trans(0, 12, 0), tex: skinTex },
      // Torso
      { box: { w: 8, h: 12, d: 4, texW: 64, texH, u: 16, v: 16 }, m: mat4Trans(0, 2, 0), tex: skinTex },
      // Right Arm
      {
        box: { w: armW, h: 12, d: 4, texW: 64, texH, u: 40, v: 16 },
        m: mat4Mul(mat4Trans(-armOffX, 8, 0), mat4Mul(mat4RotX(swing), mat4Trans(0, -6, 0))),
        tex: skinTex,
      },
      // Left Arm
      {
        box: { w: armW, h: 12, d: 4, texW: 64, texH, u: legacy ? 40 : 32, v: legacy ? 16 : 48 },
        m: mat4Mul(mat4Trans(armOffX, 8, 0), mat4Mul(mat4RotX(-swing), mat4Trans(0, -6, 0))),
        tex: skinTex,
      },
      // Right Leg
      {
        box: { w: 4, h: 12, d: 4, texW: 64, texH, u: 0, v: 16 },
        m: mat4Mul(mat4Trans(-2, -4, 0), mat4Mul(mat4RotX(-swing), mat4Trans(0, -6, 0))),
        tex: skinTex,
      },
      // Left Leg
      {
        box: { w: 4, h: 12, d: 4, texW: 64, texH, u: legacy ? 0 : 16, v: legacy ? 16 : 48 },
        m: mat4Mul(mat4Trans(2, -4, 0), mat4Mul(mat4RotX(swing), mat4Trans(0, -6, 0))),
        tex: skinTex,
      },
    ];

    // Outer overlay layer
    if (props.showOuter !== false && !legacy) {
      parts.push(
        { box: { w: 8.6, h: 8.6, d: 8.6, texW: 64, texH: 64, u: 32, v: 0 }, m: mat4Trans(0, 12, 0), tex: skinTex },
        { box: { w: 8.5, h: 12.5, d: 4.5, texW: 64, texH: 64, u: 16, v: 32 }, m: mat4Trans(0, 2, 0), tex: skinTex },
        {
          box: { w: armW + 0.5, h: 12.5, d: 4.5, texW: 64, texH: 64, u: 40, v: 32 },
          m: mat4Mul(mat4Trans(-armOffX, 8, 0), mat4Mul(mat4RotX(swing), mat4Trans(0, -6, 0))),
          tex: skinTex,
        },
        {
          box: { w: armW + 0.5, h: 12.5, d: 4.5, texW: 64, texH: 64, u: 48, v: 48 },
          m: mat4Mul(mat4Trans(armOffX, 8, 0), mat4Mul(mat4RotX(-swing), mat4Trans(0, -6, 0))),
          tex: skinTex,
        },
        {
          box: { w: 4.5, h: 12.5, d: 4.5, texW: 64, texH: 64, u: 0, v: 32 },
          m: mat4Mul(mat4Trans(-2, -4, 0), mat4Mul(mat4RotX(-swing), mat4Trans(0, -6, 0))),
          tex: skinTex,
        },
        {
          box: { w: 4.5, h: 12.5, d: 4.5, texW: 64, texH: 64, u: 0, v: 48 },
          m: mat4Mul(mat4Trans(2, -4, 0), mat4Mul(mat4RotX(swing), mat4Trans(0, -6, 0))),
          tex: skinTex,
        },
      );
    }

    // Optional Cape on back
    if (capeReady && capeImg) {
      parts.push({
        box: { w: 10, h: 16, d: 1, texW: 64, texH: 32, u: 0, v: 0 },
        m: mat4Mul(mat4Trans(0, 8, -2.5), mat4Mul(mat4RotX(-0.14 - Math.abs(swing) * 0.2), mat4Trans(0, -8, 0))),
        tex: capeTex,
      });
    }

    for (const p of parts) {
      drawBox(createBoxGeometry(p.box), mat4Mul(vp, p.m), p.tex);
    }
    rafId = requestAnimationFrame(renderFrame);
  };

  rafId = requestAnimationFrame(renderFrame);
  return true;
}

onMounted(() => {
  const canvas = canvasRef.value;
  if (!canvas) return;
  if (!initWebGL(canvas)) {
    rendererMode.value = '2d';
    const loop2d = () => {
      if (!canvasRef.value) return;
      const cw = canvas.clientWidth || 280;
      const ch = canvas.clientHeight || 340;
      if (canvas.width !== cw || canvas.height !== ch) {
        canvas.width = cw;
        canvas.height = ch;
      }
      draw2DFallback(canvas, (performance.now() - startTime) / 1000);
      rafId = requestAnimationFrame(loop2d);
    };
    rafId = requestAnimationFrame(loop2d);
  }
});

onUnmounted(() => {
  cancelAnimationFrame(rafId);
  gl = null;
});

function onPointerDown(e: PointerEvent) {
  dragging = true;
  lastX = e.clientX;
  lastY = e.clientY;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
}

function onPointerMove(e: PointerEvent) {
  if (!dragging) return;
  const dx = e.clientX - lastX;
  const dy = e.clientY - lastY;
  lastX = e.clientX;
  lastY = e.clientY;
  yaw.value += dx * 0.012;
  pitch.value = Math.max(-0.9, Math.min(0.9, pitch.value + dy * 0.01));
}

function onPointerUp() {
  dragging = false;
}

function onWheel(e: WheelEvent) {
  e.preventDefault();
  zoom.value = Math.max(0.6, Math.min(1.8, zoom.value - e.deltaY * 0.001));
}

function resetCamera() {
  yaw.value = -0.42;
  pitch.value = 0.18;
  zoom.value = 1.0;
}
</script>

<template>
  <div class="sol-skin-viewer" :style="{ height: `${height ?? 320}px` }">
    <canvas
      ref="canvasRef"
      class="sol-skin-canvas"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @wheel="onWheel"
    />
    <button
      type="button"
      class="sol-icon-btn sol-icon-btn--xs sol-viewer-reset"
      title="Reset camera angle"
      @click="resetCamera"
    >
      <RotateCw :size="12" />
    </button>
    <span class="sol-viewer-hint">Drag to rotate • Scroll to zoom</span>
  </div>
</template>
