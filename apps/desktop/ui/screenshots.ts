import { onBeforeUnmount, reactive } from 'vue';
import { native, notify, store } from './state';
import type { ScreenshotEntry } from './types';

const MAX_PREVIEW_CACHE = 72;
const MAX_CONCURRENT_THUMBNAILS = 3;
const THUMB_MAX_WIDTH = 440;
const THUMB_MAX_HEIGHT = 280;

interface CachedThumb {
  url: string;
  sig: string;
}

const cache = reactive<Record<string, CachedThumb>>({});
const lruOrder: string[] = [];
const inFlight = new Map<string, Promise<string | null>>();
const pendingQueue: Array<() => void> = [];
let activeCount = 0;

export function screenshotKey(shot: Pick<ScreenshotEntry, 'instanceId' | 'path'>): string {
  return `${shot.instanceId}::${shot.path}`;
}

function screenshotSig(shot: Pick<ScreenshotEntry, 'size' | 'modifiedMs'>): string {
  return `${shot.size}:${shot.modifiedMs}`;
}

export function peekCachedPreview(
  shot: Pick<ScreenshotEntry, 'instanceId' | 'path' | 'size' | 'modifiedMs'>,
): string | undefined {
  const entry = cache[screenshotKey(shot)];
  if (!entry || entry.sig !== screenshotSig(shot)) return undefined;
  return entry.url;
}

export function getCachedPreview(
  shot: Pick<ScreenshotEntry, 'instanceId' | 'path' | 'size' | 'modifiedMs'>,
): string | undefined {
  const key = screenshotKey(shot);
  const hit = cache[key];
  if (!hit || hit.sig !== screenshotSig(shot)) return undefined;
  const idx = lruOrder.indexOf(key);
  if (idx !== -1) {
    lruOrder.splice(idx, 1);
    lruOrder.push(key);
  }
  return hit.url;
}

function storeCachedPreview(key: string, sig: string, url: string) {
  const idx = lruOrder.indexOf(key);
  if (idx !== -1) lruOrder.splice(idx, 1);
  cache[key] = { url, sig };
  lruOrder.push(key);
  while (lruOrder.length > MAX_PREVIEW_CACHE) {
    const oldest = lruOrder.shift();
    if (oldest) delete cache[oldest];
  }
}

export function invalidateScreenshotPreview(shot: Pick<ScreenshotEntry, 'instanceId' | 'path'>) {
  const key = screenshotKey(shot);
  delete cache[key];
  const idx = lruOrder.indexOf(key);
  if (idx !== -1) lruOrder.splice(idx, 1);
}

function withSlot<T>(task: () => Promise<T>): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const run = () => {
      activeCount++;
      task()
        .then(resolve, reject)
        .finally(() => {
          activeCount = Math.max(0, activeCount - 1);
          const next = pendingQueue.shift();
          if (next) next();
        });
    };
    if (activeCount < MAX_CONCURRENT_THUMBNAILS) run();
    else pendingQueue.push(run);
  });
}

async function downscaleDataUrl(rawUrl: string): Promise<string> {
  if (typeof document === 'undefined' || typeof Image === 'undefined') {
    return rawUrl;
  }
  return new Promise<string>((resolve) => {
    const img = new Image();
    img.onload = () => {
      try {
        const w = img.naturalWidth || 0;
        const h = img.naturalHeight || 0;
        if (!w || !h) {
          resolve(rawUrl);
          return;
        }
        const scale = Math.min(1, THUMB_MAX_WIDTH / w, THUMB_MAX_HEIGHT / h);
        const targetW = Math.max(1, Math.round(w * scale));
        const targetH = Math.max(1, Math.round(h * scale));
        const canvas = document.createElement('canvas');
        canvas.width = targetW;
        canvas.height = targetH;
        const ctx = canvas.getContext('2d');
        if (!ctx) {
          resolve(rawUrl);
          return;
        }
        ctx.drawImage(img, 0, 0, targetW, targetH);
        const compressed = canvas.toDataURL('image/webp', 0.82);
        resolve(compressed && compressed.startsWith('data:image/') ? compressed : rawUrl);
      } catch {
        resolve(rawUrl);
      }
    };
    img.onerror = () => resolve(rawUrl);
    img.src = rawUrl;
  });
}

export async function loadScreenshotPreview(shot: ScreenshotEntry): Promise<string | null> {
  if (!store.desktop) return null;
  const cached = getCachedPreview(shot);
  if (cached) return cached;

  const key = screenshotKey(shot);
  const existing = inFlight.get(key);
  if (existing) return existing;

  const promise = withSlot(async () => {
    try {
      const rawUrl = await native<string>('launcher_screenshot_image', {
        instanceId: shot.instanceId,
        path: shot.path,
      });
      if (!rawUrl) return null;
      const thumbUrl = await downscaleDataUrl(rawUrl);
      storeCachedPreview(key, screenshotSig(shot), thumbUrl);
      return thumbUrl;
    } catch {
      return null;
    } finally {
      inFlight.delete(key);
    }
  });

  inFlight.set(key, promise);
  return promise;
}

export async function loadFullScreenshot(shot: ScreenshotEntry): Promise<string> {
  return native<string>('launcher_screenshot_image', {
    instanceId: shot.instanceId,
    path: shot.path,
  });
}

export async function copyScreenshotPath(shot: ScreenshotEntry): Promise<void> {
  const pathText = shot.fullPath || shot.path;
  if (!pathText) return;
  try {
    await navigator.clipboard.writeText(pathText);
    notify('success', 'File path copied', pathText);
  } catch (err) {
    notify('error', "Couldn't copy file path", String(err));
  }
}

// Shared viewport loading for both the global gallery and an instance gallery.
export function useScreenshotObserver(): (shot: ScreenshotEntry, element: unknown) => void {
  const entries = new WeakMap<Element, ScreenshotEntry>();
  const observed = new WeakSet<Element>();
  let observer: IntersectionObserver | null = null;
  onBeforeUnmount(() => { observer?.disconnect(); observer = null; });
  return (shot, element) => {
    if (!(element instanceof Element)) return;
    if (typeof IntersectionObserver === 'undefined') {
      if (!peekCachedPreview(shot)) void loadScreenshotPreview(shot);
      return;
    }
    if (!observer) observer = new IntersectionObserver((changes) => {
      for (const change of changes) {
        if (!change.isIntersecting) continue;
        const item = entries.get(change.target);
        if (item && !peekCachedPreview(item)) void loadScreenshotPreview(item);
      }
    }, { rootMargin: '160px' });
    entries.set(element, shot);
    if (!observed.has(element)) { observed.add(element); observer.observe(element); }
  };
}
