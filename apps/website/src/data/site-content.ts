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
    badge: 'Multi-Catalog',
    src: '/screenshots/modpacks.png',
    caption: 'Browse, filter by loader or Minecraft version, and install thousands of modpacks with one click.',
  },
  {
    id: 'dashboard',
    label: 'Dashboard',
    badge: 'OLED Dark',
    src: '/screenshots/dashboard.png',
    caption: 'Pure #000000 OLED home screen with instant instance launch, live status, and zero ads.',
  },
  {
    id: 'icon-studio',
    label: 'Icon Studio',
    badge: '3D + Monograms',
    src: '/screenshots/icon-studio.png',
    caption: 'Design crisp 512x512 instance icons using 30 online 3D assets, monogram badges, gradients, and HUD frames.',
  },
  {
    id: 'settings',
    label: 'Resources',
    badge: 'Storage & Java',
    src: '/screenshots/settings-resources.png',
    caption: 'Monitor disk usage, configure download concurrency, and manage automatic Java runtimes cleanly.',
  },
  {
    id: 'white',
    label: 'White Theme',
    badge: 'Light Mode',
    src: '/screenshots/downloads-white.png',
    caption: 'Switch effortlessly between pure OLED obsidian dark mode and a crisp daytime White theme.',
  },
];

export const LOADER_MATRIX = [
  {
    name: 'Vanilla',
    focus: 'Official Mojang releases & snapshots',
    mcRange: 'All supported releases',
    javaAuto: 'Auto-matched (Java 8 / 17 / 21)',
    bestFor: 'Pure survival, realms, and snapshot testing',
  },
  {
    name: 'Fabric',
    focus: 'Lightweight, fast-updating modular loader',
    mcRange: '1.14 → Latest',
    javaAuto: 'Auto-matched per version',
    bestFor: 'FPS optimization packs, modern SMPs, quick updates',
  },
  {
    name: 'NeoForge',
    focus: 'Modern high-compatibility Forge successor',
    mcRange: '1.20.1 → Latest',
    javaAuto: 'Auto-matched (Java 21)',
    bestFor: 'Large kitchen-sink & tech/magic modpacks',
  },
  {
    name: 'Forge',
    focus: 'Classic ecosystem with massive mod library',
    mcRange: '1.12.2 → Latest',
    javaAuto: 'Auto-matched (Java 8 / 17 / 21)',
    bestFor: 'Classic adventure, RPG, and legacy packs',
  },
  {
    name: 'Quilt',
    focus: 'Community fork compatible with Fabric mods',
    mcRange: '1.18.2 → Latest',
    javaAuto: 'Auto-matched per version',
    bestFor: 'Quilt-native packs and custom mod setups',
  },
] as const;

export const SYSTEM_REQUIREMENTS = [
  { label: 'Operating System', minimum: 'Windows 10 (64-bit, 1903+)', recommended: 'Windows 11 (64-bit)' },
  { label: 'Architecture', minimum: 'x86_64 (64-bit CPU)', recommended: 'Modern 4+ core x86_64 CPU' },
  { label: 'WebView Runtime', minimum: 'Microsoft Edge WebView2 (pre-installed on Win 11)', recommended: 'Latest WebView2 Evergreen' },
  { label: 'Memory (RAM)', minimum: '4 GB system RAM (2 GB for Vanilla)', recommended: '8–16 GB RAM (4–8 GB for heavy modpacks)' },
  { label: 'Java Runtime', minimum: 'Managed automatically by Solara', recommended: 'Auto-provisioned Temurin / Zulu per MC version' },
  { label: 'Minecraft Account', minimum: 'Licensed Microsoft Minecraft: Java Edition', recommended: 'Official Microsoft account (OAuth PKCE)' },
] as const;

export const CHANGELOG_HIGHLIGHTS = [
  {
    version: 'V-0.0.83',
    date: 'November 22, 2026',
    tag: 'Latest Preview',
    title: 'Cover-Banner ModPackInstaller, Redesigned Create Instance & Solara Icon Studio',
    highlights: [
      'Fixed live catalog loading with v3 search query syntax and full loader/version/gallery extraction.',
      'Redesigned ModPackInstaller with 3-column cover-banner cards, Grid/List view toggle, and native Solara dropdown filters.',
      'Redesigned Create Instance into 2-column hero setup cards, unified icon & name identity card, and 5-column visual loader selector.',
      'Introduced Solara Icon Studio with 30 online MIT Microsoft Fluent 3D icons, Monogram Badge generator, Framed Upload, dual custom color pickers, gradient angles, texture patterns, and HUD/ring/border frames.',
    ],
  },
  {
    version: 'V-0.0.82',
    date: 'November 21, 2026',
    tag: 'Feature Release',
    title: 'Multi-Catalog ModPackInstaller & Native .zip Modpack Support',
    highlights: [
      'Added top source switcher supporting multiple community modpack catalogs side by side.',
      'Added native Rust .zip modpack installer with automatic manifest parsing, file resolution, and isolated instance creation.',
      'Streamlined Create Instance Step 1 into Browse & download modpacks or Create your custom setup.',
    ],
  },
  {
    version: 'V-0.0.81',
    date: 'November 20, 2026',
    tag: 'UI Polish',
    title: 'Authentic Native Desktop Settings UI',
    highlights: [
      'Cleaned up Settings modal, sidebar navigation, Theme selector previews, and Resources tab for crisp native desktop clarity.',
      'Added realistic OLED and White mini-window theme preview cards.',
    ],
  },
  {
    version: 'V-0.0.79',
    date: 'November 20, 2026',
    tag: 'Redesign',
    title: 'Solara Settings Control Center & Card-Based Resources UI',
    highlights: [
      'Built card-based Resources panel with segmented storage bar, app directory copy pill, and transfer concurrency sliders.',
      'Removed unused clutter tabs and streamlined sidebar navigation.',
    ],
  },
  {
    version: 'V-0.0.63',
    date: 'November 2026',
    tag: 'Core Milestone',
    title: 'OLED & White Theme System, 3D Skin Studio & Minisign Auto-Updater',
    highlights: [
      'Integrated GitHub Releases CDN auto-updater with cryptographic Minisign verification and idle-only safe install.',
      'Removed all ads and promotional sidebars across the entire desktop interface.',
      'Enforced 8 MiB Windows thread stack and redacted local crash logging.',
    ],
  },
] as const;

export const ROADMAP_ITEMS = [
  {
    phase: 'In Progress',
    title: 'Real-Device Windows Acceptance',
    copy: 'Full fresh-install, Microsoft OAuth persistence, modded gameplay, and live upgrade/relaunch sign-off on physical Windows 10/11 PCs.',
  },
  {
    phase: 'Planned',
    title: 'Independent Core Engine Migration',
    copy: 'Continuing the clean-room architectural transition toward a fully independent Solara launcher core.',
  },
  {
    phase: 'Planned',
    title: 'World Backups & Snapshot Recovery',
    copy: 'One-click local world snapshots before updating modpacks, with instant rollback protection.',
  },
  {
    phase: 'Planned',
    title: 'Custom Accent & Theme Controls',
    copy: 'Fine-tune your OLED or White workspace accent palette while keeping native crisp contrast.',
  },
] as const;

export const SECURITY_PILLARS = [
  {
    title: 'Direct On-Device Microsoft OAuth PKCE',
    badge: 'RFC 7636',
    copy: 'Solara opens Microsoft’s official sign-in flow on your local machine using PKCE and a loopback listener. Your password never touches Solara code or any third-party form.',
  },
  {
    title: 'OS-Protected Credential Storage',
    badge: 'Windows Vault',
    copy: 'Minecraft access and refresh tokens are encrypted and stored locally via Windows Credential Manager—never uploaded to external servers or printed in logs.',
  },
  {
    title: 'Minisign Cryptographic Signatures',
    badge: 'Ed25519',
    copy: 'Every solaralauncher.exe release and auto-update payload is signed with our persistent private Minisign key and bound to its exact version number to block downgrade attacks.',
  },
  {
    title: 'Zero Ads & Open-Source Transparency',
    badge: 'GPL-3.0-only',
    copy: 'Zero banner ads, zero sponsored popups, and full corresponding source archives (source.zip) plus 714 bundled third-party package notices published with every build.',
  },
] as const;

export const GUIDES = [
  {
    id: 'install',
    step: 'Guide 01',
    title: 'Installing Solara on Windows',
    points: [
      'Download solaralauncher.exe from the Downloads page or GitHub Releases.',
      'Optional: verify the SHA-256 hash using PowerShell (Get-FileHash solaralauncher.exe -Algorithm SHA256).',
      'Run the installer—no version numbers in the filename, and future signed updates install automatically when idle.',
    ],
  },
  {
    id: 'modpacks',
    step: 'Guide 02',
    title: 'Installing Modpacks in One Click',
    points: [
      'Open ModPackInstaller from the left navigation rail.',
      'Switch between supported community modpack catalogs at the top-right switcher.',
      'Filter by Fabric, Forge, NeoForge, or Quilt and Minecraft version, then click Install—Solara sets up the loader and Java runtime automatically.',
    ],
  },
  {
    id: 'icons',
    step: 'Guide 03',
    title: 'Customizing Instances with Icon Studio',
    points: [
      'Click the + button in the sidebar and pick Create your custom setup.',
      'Click Randomize for instant combinations, or Choose specific to launch Solara Icon Studio.',
      'Pick from 30 online 3D icons, craft a custom Monogram Badge (e.g. SMP / 1.21), or frame your own uploaded PNG with custom gradients, patterns, and HUD corners.',
    ],
  },
  {
    id: 'troubleshooting',
    step: 'Guide 04',
    title: 'Troubleshooting & Crash Logs',
    points: [
      'Ensure Windows 10/11 x64 has Microsoft Edge WebView2 installed and up to date.',
      'If a modded instance runs out of memory, open Instance Settings → Java & Memory and allocate 4–8 GB RAM.',
      'Check Settings → Resources to clear unused cached files or open the app directory for redacted local logs.',
    ],
  },
] as const;
