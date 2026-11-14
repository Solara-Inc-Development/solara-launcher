import { h, type FunctionalComponent } from 'vue';
export * from '../../../packages/ui/icons';

type IconProps = { size?: number; strokeWidth?: number; class?: string };

function makeIcon(name: string, paths: string[]): FunctionalComponent<IconProps> {
  const fn: FunctionalComponent<IconProps> = (props) =>
    h(
      'svg',
      {
        width: props.size ?? 18,
        height: props.size ?? 18,
        viewBox: '0 0 24 24',
        fill: 'none',
        stroke: 'currentColor',
        'stroke-width': props.strokeWidth ?? 1.75,
        'stroke-linecap': 'round',
        'stroke-linejoin': 'round',
        'aria-hidden': 'true',
        class: props.class,
      },
      paths.map((d) =>
        d.startsWith('circle:')
          ? (() => {
              const [cx, cy, r] = d.slice(7).split(',');
              return h('circle', { cx, cy, r });
            })()
          : d.startsWith('rect:')
            ? (() => {
                const [x, y, width, height, rx] = d.slice(5).split(',');
                return h('rect', { x, y, width, height, rx });
              })()
            : h('path', { d }),
      ),
    );
  fn.displayName = name;
  return fn;
}

export const Shirt = makeIcon('Shirt', [
  'M20.38 3.46 16 2a4 4 0 0 1-8 0L3.62 3.46a2 2 0 0 0-1.34 2.23l.58 3.47a1 1 0 0 0 .99.84H6v10c0 1.1.9 2 2 2h8a2 2 0 0 0 2-2V10h2.15a1 1 0 0 0 .99-.84l.58-3.47a2 2 0 0 0-1.34-2.23z',
]);

export const Camera = makeIcon('Camera', [
  'M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z',
  'circle:12,13,3',
]);

export const Pin = makeIcon('Pin', [
  'M12 17v5',
  'M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z',
]);

export const Star = makeIcon('Star', [
  'M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z',
]);

export const Palette = makeIcon('Palette', [
  'M12 22a1 1 0 0 1 0-20 10 9 0 0 1 10 9 5 5 0 0 1-5 5h-2.25a1.75 1.75 0 0 0-1.4 2.8l.3.4a1.75 1.75 0 0 1-1.4 2.8z',
  'circle:13.5,6.5,0.5',
  'circle:17.5,10.5,0.5',
  'circle:6.5,12.5,0.5',
  'circle:8.5,7.5,0.5',
]);

export const Eraser = makeIcon('Eraser', [
  'm7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21',
  'M22 21H7',
  'm5 11 9 9',
]);

export const Pipette = makeIcon('Pipette', [
  'm2 22 1-1h3l9-9',
  'M3 21v-3l9-9',
  'm15 6 3.4-3.4a2.1 2.1 0 1 1 3 3L18 9l.4.4a2.1 2.1 0 1 1-3 3l-3.8-3.8a2.1 2.1 0 1 1 3-3l.4.4Z',
]);

export const PaintBucket = makeIcon('PaintBucket', [
  'm19 11-8-8-8.6 8.6a2 2 0 0 0 0 2.8l5.2 5.2c.8.8 2 .8 2.8 0L19 11Z',
  'm5 2 5 5',
  'M2 13h15',
  'M22 20a2 2 0 1 1-4 0c0-1.6 1.7-2.4 2-4 .3 1.6 2 2.4 2 4Z',
]);

export const Undo2 = makeIcon('Undo2', [
  'M9 14 4 9l5-5',
  'M4 9h10.5a5.5 5.5 0 0 1 5.5 5.5a5.5 5.5 0 0 1-5.5 5.5H11',
]);

export const Redo2 = makeIcon('Redo2', [
  'm15 14 5-5-5-5',
  'M20 9H9.5A5.5 5.5 0 0 0 4 14.5A5.5 5.5 0 0 0 9.5 20H13',
]);

export const Grid = makeIcon('Grid', [
  'rect:3,3,7,7,1.5',
  'rect:14,3,7,7,1.5',
  'rect:14,14,7,7,1.5',
  'rect:3,14,7,7,1.5',
]);

export const List = makeIcon('List', [
  'M8 6h13',
  'M8 12h13',
  'M8 18h13',
  'M3 6h.01',
  'M3 12h.01',
  'M3 18h.01',
]);

export const AlertTriangle = makeIcon('AlertTriangle', [
  'm21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z',
  'M12 9v4',
  'M12 17h.01',
]);

export const Info = makeIcon('Info', [
  'circle:12,12,10',
  'M12 16v-4',
  'M12 8h.01',
]);

export const CheckCircle2 = makeIcon('CheckCircle2', [
  'circle:12,12,10',
  'm9 12 2 2 4-4',
]);

export const FolderPlus = makeIcon('FolderPlus', [
  'M12 10v6',
  'M9 13h6',
  'M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z',
]);

export const Upload = makeIcon('Upload', [
  'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4',
  'M17 8l-5-5-5 5',
  'M12 3v12',
]);

export const Shuffle = makeIcon('Shuffle', [
  'M2 18h1.4c1.3 0 2.5-.6 3.3-1.7l6.1-8.6c.7-1.1 2-1.7 3.3-1.7H22',
  'm18 2 4 4-4 4',
  'M2 6h1.9c1.5 0 2.9.9 3.6 2.2',
  'M22 18h-5.9c-1.3 0-2.6-.7-3.3-1.8l-.5-.8',
  'm18 14 4 4-4 4',
]);

export const Wrench = makeIcon('Wrench', [
  'M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z',
]);

export const HardDrive = makeIcon('HardDrive', [
  'M22 12H2',
  'M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z',
  'M6 16h.01',
  'M10 16h.01',
]);

export const Library = makeIcon('Library', [
  'm16 6 4 14',
  'M12 6v14',
  'M8 8v12',
  'M4 4v16',
]);

export const Newspaper = makeIcon('Newspaper', [
  'M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2',
  'M18 14h-8',
  'M15 18h-5',
  'M10 6h8v4h-8V6Z',
]);

export const File = makeIcon('File', [
  'M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z',
  'M14 2v4a2 2 0 0 0 2 2h4',
]);

export const Folder = makeIcon('Folder', [
  'M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z',
]);

export const Archive = makeIcon('Archive', [
  'rect:2,3,20,5,1',
  'M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8',
  'M10 12h4',
]);

export const Edit3 = makeIcon('Edit3', [
  'M12 20h9',
  'M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z',
]);

export const Shield = makeIcon('Shield', [
  'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z',
]);

export const Terminal = makeIcon('Terminal', [
  'M4 17l6-6-6-6',
  'M12 19h8',
]);

export const Maximize2 = makeIcon('Maximize2', [
  'M15 3h6v6',
  'M9 21H3v-6',
  'M21 3l-7 7',
  'M3 21l7-7',
]);

export const RotateCw = makeIcon('RotateCw', [
  'M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8',
  'M21 3v5h-5',
]);

export const Eye = makeIcon('Eye', [
  'M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z',
  'circle:12,12,3',
]);

export const EyeOff = makeIcon('EyeOff', [
  'M9.88 9.88a3 3 0 1 0 4.24 4.24',
  'M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68',
  'M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61',
  'M2 2l20 20',
]);

export const Bell = makeIcon('Bell', [
  'M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9',
  'M10.3 21a1.94 1.94 0 0 0 3.4 0',
]);

export const CommandIcon = makeIcon('CommandIcon', [
  'M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3',
]);

export const Layers = makeIcon('Layers', [
  'm12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z',
  'm22 17.65-9.17 4.16a2 2 0 0 1-1.66 0L2 17.65',
  'm22 12.65-9.17 4.16a2 2 0 0 1-1.66 0L2 12.65',
]);

export const MoreHorizontal = makeIcon('MoreHorizontal', [
  'circle:12,12,1',
  'circle:19,12,1',
  'circle:5,12,1',
]);

export const Globe = makeIcon('Globe', [
  'circle:12,12,10',
  'M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20',
  'M2 12h20',
]);

export const Sparkles = makeIcon('Sparkles', [
  'm12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z',
  'M5 3v4',
  'M19 17v4',
  'M3 5h4',
  'M17 19h4',
]);

export const Lock = makeIcon('Lock', [
  'rect:3,11,18,11,2',
  'M7 11V7a5 5 0 0 1 10 0v4',
]);

export const Cpu = makeIcon('Cpu', [
  'rect:4,4,16,16,2',
  'rect:9,9,6,6,1',
  'M15 2v2',
  'M15 20v2',
  'M2 15h2',
  'M2 9h2',
  'M20 15h2',
  'M20 9h2',
  'M9 2v2',
  'M9 20v2',
]);

export const Clock = makeIcon('Clock', [
  'circle:12,12,10',
  'M12 6v6l4 2',
]);

export const Filter = makeIcon('Filter', [
  'M22 3H2l8 9.46V19l4 2v-8.54L22 3z',
]);

export const PanelLeft = makeIcon('PanelLeft', [
  'rect:3,3,18,18,2',
  'M9 3v18',
]);

export const FlipHorizontal = makeIcon('FlipHorizontal', [
  'M8 3H5a2 2 0 0 0-2 2v14c0 1.1.9 2 2 2h3',
  'M16 3h3a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-3',
  'M12 20v2',
  'M12 14v2',
  'M12 8v2',
  'M12 2v2',
]);
