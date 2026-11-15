/**
 * Pure pixel-editing helpers for SkinEditor.vue.
 * Tested directly in Node unit tests and bundled by Vite.
 */

/**
 * Normalizes a CSS hex color string into #RRGGBB or returns null if invalid.
 * @param {string} raw
 */
export function normalizeHexColor(raw) {
  if (typeof raw !== 'string') return null;
  const trimmed = raw.trim().replace(/^#/, '');
  if (!/^[0-9a-fA-F]{6}$/.test(trimmed)) return null;
  return '#' + trimmed.toUpperCase();
}

/**
 * Parses #RRGGBB into [r, g, b] integers or null.
 * @param {string} hex
 */
export function parseHexRgb(hex) {
  const norm = normalizeHexColor(hex);
  if (!norm) return null;
  return [
    parseInt(norm.slice(1, 3), 16),
    parseInt(norm.slice(3, 5), 16),
    parseInt(norm.slice(5, 7), 16),
  ];
}

/**
 * Computes mirrored horizontal texture X inside the active UV box.
 * Works identically for even (8px, 4px) and odd (3px slim arm) widths.
 * @param {number} gx
 * @param {{ x: number, w: number }} box
 */
export function mirrorPixelX(gx, box) {
  return box.x + (box.w - 1 - (gx - box.x));
}

/**
 * Prepends a valid hex color to a bounded recent-color palette (deduped).
 * @param {string[]} current
 * @param {string} nextHex
 * @param {number} [limit]
 */
export function pushRecentColor(current, nextHex, limit = 8) {
  const norm = normalizeHexColor(nextHex);
  if (!norm) return [...current];
  return [norm, ...current.filter((c) => c.toUpperCase() !== norm)].slice(0, limit);
}

/**
 * Performs bounded 4-way flood fill inside a 64x64 RGBA buffer restricted to `box`.
 * Mutates `data` in place and returns number of pixels recolored.
 * @param {Uint8ClampedArray} data
 * @param {number} texWidth
 * @param {number} startX
 * @param {number} startY
 * @param {{ x: number, y: number, w: number, h: number }} box
 * @param {string} hexColor
 */
export function floodFillRegion(data, texWidth, startX, startY, box, hexColor) {
  const rgb = parseHexRgb(hexColor);
  if (!rgb) return 0;
  if (
    startX < box.x ||
    startY < box.y ||
    startX >= box.x + box.w ||
    startY >= box.y + box.h
  ) {
    return 0;
  }

  const idx = (x, y) => (y * texWidth + x) * 4;
  const s = idx(startX, startY);
  const tr = data[s];
  const tg = data[s + 1];
  const tb = data[s + 2];
  const ta = data[s + 3];
  const [nr, ng, nb] = rgb;

  if (tr === nr && tg === ng && tb === nb && ta === 255) return 0;

  const stack = [[startX, startY]];
  const visited = new Uint8Array(texWidth * texWidth);
  let filled = 0;

  while (stack.length > 0) {
    const [x, y] = stack.pop();
    if (x < box.x || y < box.y || x >= box.x + box.w || y >= box.y + box.h) continue;
    const flat = y * texWidth + x;
    if (visited[flat]) continue;
    visited[flat] = 1;
    const i = flat * 4;
    if (data[i] !== tr || data[i + 1] !== tg || data[i + 2] !== tb || data[i + 3] !== ta) continue;
    data[i] = nr;
    data[i + 1] = ng;
    data[i + 2] = nb;
    data[i + 3] = 255;
    filled++;
    stack.push([x + 1, y], [x - 1, y], [x, y + 1], [x, y - 1]);
  }
  return filled;
}
