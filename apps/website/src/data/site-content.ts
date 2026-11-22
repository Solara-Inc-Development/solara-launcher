export type SiteRoute = '/' | '/features' | '/downloads' | '/changelog' | '/security' | '/docs';

export interface ShowcaseTab {
  id: 'modpacks' | 'dashboard' | 'icon-studio' | 'white' | 'settings';
  label: string;
  badge: string;
  src: string;
  caption: string;
}

export const SHOWCASE_TABS: ShowcaseTab[] = [
  {
    id: 'modpacks',
    label: 'Modpacks',
    badge: 'One-Click',
    src: '/screenshots/modpacks.png',
    caption: 'Browse thousands of modpacks, pick your Minecraft version, and install with one click.',
  },
  {
    id: 'dashboard',
    label: 'Home',
    badge: 'OLED Black',
    src: '/screenshots/dashboard.png',
    caption: 'Pure black OLED home screen with your worlds ready to launch and zero ads.',
  },
  {
    id: 'icon-studio',
    label: 'Icon Studio',
    badge: '3D + Badges',
    src: '/screenshots/icon-studio.png',
    caption: 'Make custom icons for every world using 30 3D icons, text badges, colors, and frames.',
  },
  {
    id: 'settings',
    label: 'Storage',
    badge: 'Clean & Fast',
    src: '/screenshots/settings-resources.png',
    caption: 'See how much space your packs use, clear extra files, and keep everything tidy.',
  },
  {
    id: 'white',
    label: 'White Theme',
    badge: 'Light Mode',
    src: '/screenshots/downloads-white.png',
    caption: 'Prefer a bright screen? Switch between pure OLED black and crisp White mode anytime.',
  },
];

export const LOADER_MATRIX = [
  {
    name: 'Vanilla',
    focus: 'Classic unmodded Minecraft',
    mcRange: 'All releases & snapshots',
    javaAuto: 'Set up automatically',
    bestFor: 'Pure survival, Realms, and trying new snapshots',
  },
  {
    name: 'Fabric',
    focus: 'Fast, lightweight mod loader',
    mcRange: '1.14 → Latest',
    javaAuto: 'Set up automatically',
    bestFor: 'FPS boost packs, shaders, and modern SMP servers',
  },
  {
    name: 'NeoForge',
    focus: 'Modern loader for big modpacks',
    mcRange: '1.20.1 → Latest',
    javaAuto: 'Set up automatically',
    bestFor: 'Huge tech, magic, and quest modpacks',
  },
  {
    name: 'Forge',
    focus: 'Classic loader with tons of mods',
    mcRange: '1.12.2 → Latest',
    javaAuto: 'Set up automatically',
    bestFor: 'Classic adventure, RPG, and older favorite packs',
  },
  {
    name: 'Quilt',
    focus: 'Community loader for Fabric-style mods',
    mcRange: '1.18.2 → Latest',
    javaAuto: 'Set up automatically',
    bestFor: 'Custom modded setups and Quilt packs',
  },
] as const;

export const SYSTEM_REQUIREMENTS = [
  { label: 'Windows Version', minimum: 'Windows 10 (64-bit)', recommended: 'Windows 11 (64-bit)' },
  { label: 'Processor', minimum: '64-bit CPU', recommended: '4+ core modern CPU' },
  { label: 'WebView2', minimum: 'Microsoft Edge WebView2 (already on Win 11)', recommended: 'Latest WebView2' },
  { label: 'RAM (Memory)', minimum: '4 GB RAM (2 GB for Vanilla)', recommended: '8–16 GB RAM for big modpacks' },
  { label: 'Java', minimum: 'Handled automatically by Solara', recommended: 'No manual Java install needed' },
  { label: 'Minecraft Account', minimum: 'Official Microsoft Minecraft Java account', recommended: 'Official Microsoft account' },
] as const;

export const CHANGELOG_HIGHLIGHTS = [
  {
    version: 'V-0.0.84',
    date: 'November 22, 2026',
    tag: 'Latest Preview',
    title: 'Fixed Description Images & Full CurseForge Modpack Detail Pages',
    highlights: [
      'Fixed images not loading inside modpack descriptions so banners, screenshots, and badges display cleanly.',
      'Added full CurseForge modpack detail pages with cover banners, rich descriptions, filterable versions, and screenshot gallery.',
      'Overhauled the official Solara website with a Bento showcase, live OLED/White theme preview, and smooth animations.',
    ],
  },
  {
    version: 'V-0.0.83',
    date: 'November 22, 2026',
    tag: 'Feature Release',
    title: 'Fresh Modpack Browser, New Create Instance UI & Solara Icon Studio',
    highlights: [
      'Fixed modpack catalog loading so all packs load fast and reliably.',
      'Rebuilt the modpack browser with big cover art cards, Grid/List switch, and clean Solara filters.',
      'Redesigned the Create Instance popup with side-by-side choices and a clean loader picker.',
      'Added 30 crisp 3D icons and built Solara Icon Studio with custom text badges, colors, patterns, and frames.',
    ],
  },
  {
    version: 'V-0.0.82',
    date: 'November 21, 2026',
    tag: 'Feature Release',
    title: 'Multi-Catalog Modpacks & Native .zip Pack Installs',
    highlights: [
      'Added a clean switcher at the top of the modpack browser to browse multiple catalogs.',
      'Added built-in .zip modpack installer so packs set up their loader and mods automatically.',
      'Simplified Create Instance into two clear choices: browse modpacks or build a custom setup.',
    ],
  },
  {
    version: 'V-0.0.81',
    date: 'November 20, 2026',
    tag: 'UI Polish',
    title: 'Cleaner Settings & Realistic Theme Previews',
    highlights: [
      'Polished the Settings menu so every tab looks clean, simple, and easy to read.',
      'Added realistic mini-window previews for OLED and White themes.',
    ],
  },
  {
    version: 'V-0.0.79',
    date: 'November 20, 2026',
    tag: 'Redesign',
    title: 'New Settings Control Center & Storage Panel',
    highlights: [
      'Rebuilt the Storage & Resources panel with a clear usage bar and quick folder tools.',
      'Removed extra unused tabs to keep navigation light and simple.',
    ],
  },
  {
    version: 'V-0.0.63',
    date: 'November 2026',
    tag: 'Core Milestone',
    title: 'OLED & White Themes, 3D Skin Studio & Safe Auto-Updates',
    highlights: [
      'Added automatic background updates that check safety signatures before installing.',
      'Removed all ads and extra banners across the entire app.',
      'Added a live 3D skin viewer and cape switcher right inside the launcher.',
    ],
  },
] as const;

export const ROADMAP_ITEMS = [
  {
    phase: 'Working on now',
    title: 'Full Windows PC Testing',
    copy: 'Testing fresh installs, Microsoft login, modded play, and live updates on real Windows 10 and 11 PCs.',
  },
  {
    phase: 'Up next',
    title: 'Custom Solara Core Engine',
    copy: 'Continuing our work toward a 100% independent Solara launcher engine built from the ground up.',
  },
  {
    phase: 'Planned',
    title: 'One-Click World Backups',
    copy: 'Save quick world backups before updating a modpack so your builds stay safe.',
  },
  {
    phase: 'Planned',
    title: 'Custom Accent Colors',
    copy: 'Pick your own favorite accent color for OLED and White modes.',
  },
] as const;

export const SECURITY_PILLARS = [
  {
    title: 'Direct Microsoft Login',
    badge: 'Your PC only',
    copy: 'Solara signs you in directly through Microsoft’s official login page on your own computer. We never ask for or see your Microsoft password.',
  },
  {
    title: 'Locked in Windows Storage',
    badge: 'Encrypted',
    copy: 'Your login session is saved safely inside Windows’ built-in secure vault—never sent to any third-party server.',
  },
  {
    title: 'Checked Before Updating',
    badge: 'Signed updates',
    copy: 'Every solaralauncher.exe release has a unique digital signature (Minisign) and file fingerprint (SHA-256) so tampered files are blocked automatically.',
  },
  {
    title: 'Zero Ads & Open Source',
    badge: 'GPL-3.0',
    copy: 'No popups, no ad banners, and the full app code is open on GitHub so anyone can inspect how it works.',
  },
] as const;

export const GUIDES = [
  {
    id: 'install',
    step: 'Step 01',
    title: 'How to install Solara on Windows',
    points: [
      'Click Download to grab solaralauncher.exe from our Downloads page or GitHub.',
      'Run the file to install. You only need to install it once — future updates happen automatically.',
      'Want to double-check the file? Copy the PowerShell check command on the Downloads page to verify its SHA-256 fingerprint.',
    ],
  },
  {
    id: 'modpacks',
    step: 'Step 02',
    title: 'How to install a modpack in one click',
    points: [
      'Click the Modpacks icon on the left sidebar of the launcher.',
      'Pick your favorite catalog at the top right, then filter by loader or Minecraft version.',
      'Hit Install — Solara creates a fresh folder, sets up the loader and Java, and gets the pack ready to play.',
    ],
  },
  {
    id: 'icons',
    step: 'Step 03',
    title: 'Make custom icons in Icon Studio',
    points: [
      'Click the + button in the sidebar and choose Create your custom setup.',
      'Hit Randomize for an instant look, or click Choose specific to open Solara Icon Studio.',
      'Pick from 30 3D icons, make a custom text badge (like SMP or 1.21), or upload your own picture with custom colors and frames.',
    ],
  },
  {
    id: 'troubleshooting',
    step: 'Step 04',
    title: 'Quick fixes if something acts up',
    points: [
      'Make sure you are on 64-bit Windows 10 or 11 with Microsoft Edge WebView2.',
      'If a heavy modpack lags or crashes, open Instance Settings → Java & Memory and give it 4–8 GB of RAM.',
      'Low on space? Open Settings → Resources to clear old cached files in one click.',
    ],
  },
] as const;
