/**
 * Official Minecraft 64x64 Skin UV Net Layout Helpers.
 * Shared by SkinEditor.vue and Node unit tests.
 */

export const SKIN_BODY_PARTS = [
  { id: 'head', label: 'Head' },
  { id: 'torso', label: 'Torso' },
  { id: 'arm-r', label: 'R. Arm' },
  { id: 'arm-l', label: 'L. Arm' },
  { id: 'leg-r', label: 'R. Leg' },
  { id: 'leg-l', label: 'L. Leg' },
  { id: 'full', label: 'Full UV (64×64)' },
];

export const SKIN_FACES = [
  { id: 'front', label: 'Front' },
  { id: 'back', label: 'Back' },
  { id: 'right', label: 'Right' },
  { id: 'left', label: 'Left' },
  { id: 'top', label: 'Top' },
  { id: 'bottom', label: 'Bottom' },
];

/**
 * Returns the 64x64 UV cube spec { u, v, w, h, d } for a given part, layer, and arm model.
 * @param {'head'|'torso'|'arm-r'|'arm-l'|'leg-r'|'leg-l'} part
 * @param {'base'|'overlay'} layer
 * @param {'classic'|'slim'} variant
 */
export function getPartBoxSpec(part, layer, variant) {
  const armW = variant === 'slim' ? 3 : 4;
  const ov = layer === 'overlay';
  switch (part) {
    case 'head':
      return { u: ov ? 32 : 0, v: 0, w: 8, h: 8, d: 8 };
    case 'torso':
      return { u: 16, v: ov ? 32 : 16, w: 8, h: 12, d: 4 };
    case 'arm-r':
      return { u: 40, v: ov ? 32 : 16, w: armW, h: 12, d: 4 };
    case 'arm-l':
      return { u: ov ? 48 : 32, v: 48, w: armW, h: 12, d: 4 };
    case 'leg-r':
      return { u: 0, v: ov ? 32 : 16, w: 4, h: 12, d: 4 };
    case 'leg-l':
      return { u: ov ? 0 : 16, v: 48, w: 4, h: 12, d: 4 };
    default:
      throw new RangeError(`Unknown skin part: ${String(part)}`);
  }
}

/**
 * Resolves the exact 2D pixel rectangle { x, y, w, h } on the 64x64 skin texture.
 * @param {{ part: 'head'|'torso'|'arm-r'|'arm-l'|'leg-r'|'leg-l'|'full', face: 'front'|'back'|'left'|'right'|'top'|'bottom', layer: 'base'|'overlay', variant: 'classic'|'slim' }} opts
 */
export function getUvRegionBox({ part, face, layer, variant }) {
  if (part === 'full') {
    return { x: 0, y: 0, w: 64, h: 64 };
  }
  const { u, v, w, h, d } = getPartBoxSpec(part, layer, variant);
  switch (face) {
    case 'top':
      return { x: u + d, y: v, w, h: d };
    case 'bottom':
      return { x: u + d + w, y: v, w, h: d };
    case 'right':
      return { x: u, y: v + d, w: d, h };
    case 'front':
      return { x: u + d, y: v + d, w, h };
    case 'left':
      return { x: u + d + w, y: v + d, w: d, h };
    case 'back':
      return { x: u + d + w + d, y: v + d, w, h };
    default:
      throw new RangeError(`Unknown cube face: ${String(face)}`);
  }
}
