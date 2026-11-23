<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import {
  ArrowUpRight,
  ArrowRight,
  Download,
  Gamepad2,
  Monitor,
  ShieldCheck,
  Package,
  Plus,
  Minus,
  Check,
  ChevronRight,
  ChevronDown,
  Copy,
  Sparkles,
  GitBranch,
  Menu,
  X,
  Box,
  Discord,
  Lock,
  Coffee,
  Layers3,
} from './src/ui/icons';
import { useInteractions } from './src/ui/interactions';
import './src/ui/motion.css';
import {
  snapshot,
  refreshPublic,
  preferences,
  latest,
  links,
} from '../../packages/shared/client';
import {
  CHANGELOG_HIGHLIGHTS,
  GUIDES,
  LOADER_MATRIX,
  ROADMAP_ITEMS,
  SECURITY_PILLARS,
  SHOWCASE_TABS,
  type SiteRoute,
  SYSTEM_REQUIREMENTS,
} from './src/data/site-content';

useInteractions();

const VALID_ROUTES: SiteRoute[] = ['/', '/features', '/downloads', '/changelog', '/security', '/docs'];

function resolveRoute(pathname: string): SiteRoute {
  const clean = pathname.replace(/\/+$/, '') || '/';
  return VALID_ROUTES.includes(clean as SiteRoute) ? (clean as SiteRoute) : '/';
}

const currentRoute = ref<SiteRoute>(resolveRoute(window.location.pathname));
const mobile = ref(false);
const faq = ref<number | null>(0);
const openSubsystem = ref<number | null>(null);
const copied = ref('');
const activeShot = ref<(typeof SHOWCASE_TABS)[number]['id']>('dashboard');
let timer: ReturnType<typeof setInterval> | null = null;

// Interactive Hero Runtime Simulator (100% real Java & loader mapping used by Solara)
const MC_PRESETS = [
  { version: '1.12.2', java: 8, runtime: 'Temurin 8', loaders: ['Forge', 'Vanilla'] },
  { version: '1.16.5', java: 17, runtime: 'Temurin 17', loaders: ['Fabric', 'Forge', 'Vanilla'] },
  { version: '1.18.2', java: 17, runtime: 'Temurin 17', loaders: ['Fabric', 'Forge', 'Quilt', 'Vanilla'] },
  { version: '1.19.4', java: 17, runtime: 'Temurin 17', loaders: ['Fabric', 'Forge', 'Quilt', 'Vanilla'] },
  { version: '1.20.1', java: 17, runtime: 'Temurin 17', loaders: ['Fabric', 'NeoForge', 'Forge', 'Quilt', 'Vanilla'] },
  { version: '1.20.4', java: 17, runtime: 'Temurin 17', loaders: ['Fabric', 'NeoForge', 'Forge', 'Quilt', 'Vanilla'] },
  { version: '1.21.1', java: 21, runtime: 'Temurin 21', loaders: ['Fabric', 'NeoForge', 'Forge', 'Quilt', 'Vanilla'] },
  { version: '1.21.4', java: 21, runtime: 'Temurin 21', loaders: ['Fabric', 'NeoForge', 'Forge', 'Quilt', 'Vanilla'] },
] as const;

const selectedMcIndex = ref(6); // Default: 1.21.1 -> Java 21
const selectedLoader = ref<'Fabric' | 'NeoForge' | 'Forge' | 'Quilt' | 'Vanilla'>('Fabric');

const currentPreset = computed(() => MC_PRESETS[selectedMcIndex.value] ?? MC_PRESETS[6]);
const javaBarPercent = computed(() => {
  if (currentPreset.value.java === 8) return 44;
  if (currentPreset.value.java === 17) return 74;
  return 100;
});
const sliderFillPercent = computed(() =>
  Math.round((selectedMcIndex.value / (MC_PRESETS.length - 1)) * 100),
);

function pickLoader(loader: 'Fabric' | 'NeoForge' | 'Forge' | 'Quilt' | 'Vanilla') {
  selectedLoader.value = loader;
  if (loader === 'NeoForge' && selectedMcIndex.value < 4) {
    selectedMcIndex.value = 6;
  } else if (loader === 'Quilt' && selectedMcIndex.value < 2) {
    selectedMcIndex.value = 6;
  } else if (loader === 'Fabric' && selectedMcIndex.value < 1) {
    selectedMcIndex.value = 6;
  }
}

const DIRECT_GITHUB_DOWNLOAD_URL =
  'https://github.com/Solara-Inc-Development/solara-launcher/releases/download/V-0.0.94/solaralauncher.exe';

const DEFAULT_VT_SHA256 =
  '73a0f741eac4fbd8ee06d9d726d3382ec15d907ceef5c0afa299faf6825076e0';

const DISCORD_URL = 'https://discord.gg/solara-inc';

const githubDownloadUrl = computed(() => DIRECT_GITHUB_DOWNLOAD_URL);

const virusTotalUrl = computed(() => {
  return `https://www.virustotal.com/gui/file/${DEFAULT_VT_SHA256}/`;
});

const currentShot = computed(
  () => SHOWCASE_TABS.find((t) => t.id === activeShot.value) ?? SHOWCASE_TABS[1],
);

function selectShot(id: (typeof SHOWCASE_TABS)[number]['id']) {
  activeShot.value = id;
}

function navigate(route: SiteRoute, hash?: string) {
  mobile.value = false;
  if (currentRoute.value !== route) {
    currentRoute.value = route;
    window.history.pushState({}, '', hash ? `${route}#${hash}` : route);
  } else if (hash) {
    window.history.replaceState({}, '', `${route}#${hash}`);
  }
  if (hash) {
    requestAnimationFrame(() => {
      document.getElementById(hash)?.scrollIntoView({ behavior: 'smooth' });
    });
  } else {
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }
}

function onPopState() {
  currentRoute.value = resolveRoute(window.location.pathname);
}

const features: ReadonlyArray<{
  icon: typeof Gamepad2;
  tone: string;
  title: string;
  copy: string;
  tag: string;
}> = [
  {
    icon: Gamepad2,
    tone: 'cobalt',
    title: 'Isolated Game Folders',
    copy: 'Keep Vanilla, Fabric, NeoForge, Forge, and Quilt worlds in their own clean folders so mods never clash.',
    tag: 'WORLDS',
  },
  {
    icon: Package,
    tone: 'emerald',
    title: 'One-Click Modpacks',
    copy: 'Browse top community catalogs with rich descriptions, screenshots, and instant .mrpack & .zip installs.',
    tag: 'MODPACKS',
  },
  {
    icon: Sparkles,
    tone: 'purple',
    title: 'Solara Icon Studio',
    copy: 'Customize every world with 30 crisp 3D icons, text badges, gradient backgrounds, and clean frames.',
    tag: 'ICONS',
  },
  {
    icon: Monitor,
    tone: 'amber',
    title: 'Pure OLED & White Mode',
    copy: 'True #000000 OLED black for late-night sessions and a crisp White theme for bright rooms — zero ads.',
    tag: 'THEMES',
  },
  {
    icon: ShieldCheck,
    tone: 'sky',
    title: 'Local Microsoft Login',
    copy: 'Sign in directly through Microsoft OAuth 2.0 PKCE on your own PC. Credentials never touch Solara servers.',
    tag: 'SECURITY',
  },
  {
    icon: GitBranch,
    tone: 'rose',
    title: 'One Evergreen .exe Forever',
    copy: 'Download solaralauncher.exe once. Every future release checks at launch and applies silently from GitHub.',
    tag: 'AUTO-UPDATE',
  },
];

const SUBSYSTEMS = [
  {
    id: '01',
    title: 'Isolated instance directories',
    detail: 'Every profile gets its own mods, configs, resource packs, shader packs, and saves folder so different Minecraft versions never corrupt each other.',
  },
  {
    id: '02',
    title: 'One-click modpack & mod browser',
    detail: 'Search multiple community catalogs, inspect full rich-image descriptions and screenshots, filter by loader/version, and install automatically.',
  },
  {
    id: '03',
    title: 'Automatic Temurin Java runtimes',
    detail: 'Solara automatically downloads and pairs the right Eclipse Temurin Java version (Java 8, 17, or 21) for each Minecraft version.',
  },
  {
    id: '04',
    title: 'All 5 modern mod loaders',
    detail: 'Full native support for Vanilla, Fabric, NeoForge, Forge, and Quilt with automatic loader metadata resolution.',
  },
  {
    id: '05',
    title: 'Solara Icon Studio (3D + Badges)',
    detail: 'Built-in icon designer with 30 3D voxel icons, custom text badges, color palettes, patterns, and frames saved locally per instance.',
  },
  {
    id: '06',
    title: 'Permanent evergreen solaralauncher.exe',
    detail: 'One locked public Windows installer with a fixed SHA-256 and VirusTotal report — no need to manually download a new .exe on future releases.',
  },
  {
    id: '07',
    title: '0ms Minisign auto-updates via GitHub CDN',
    detail: 'On startup, Solara checks GitHub Releases CDN, verifies the Minisign cryptographic signature in memory, and silently updates in place when idle.',
  },
  {
    id: '08',
    title: 'Local Microsoft OAuth 2.0 PKCE',
    detail: 'Official Microsoft loopback authentication runs strictly on your own PC and stores tokens encrypted in Windows Credential Manager.',
  },
  {
    id: '09',
    title: 'True OLED #000000 & White UI',
    detail: 'Zero ad banners, zero sponsor popups, and instant theme switching between true pitch-black OLED and crisp White mode.',
  },
  {
    id: '10',
    title: 'Open-source GPL-3.0 & notice verification',
    detail: 'Full source code, exact release source archives, and bundled third-party license notices are published transparently on GitHub.',
  },
] as const;

const questions = [
  [
    'Do I need to download a new .exe for every update?',
    'No! You only download solaralauncher.exe once. Even if you install from this same file months later, Solara checks GitHub Releases on launch and automatically updates itself in place.',
  ],
  [
    'Where does the download come from?',
    'Directly from our official GitHub Releases CDN — fast, open source, and verifiable with its permanent SHA-256 fingerprint and VirusTotal report.',
  ],
  [
    'Is my Microsoft account safe?',
    'Yes. You sign in straight through Microsoft on your own PC using OAuth 2.0 PKCE. Your password never goes through Solara servers.',
  ],
  [
    'Which mod loaders work in Solara?',
    'Vanilla, Fabric, NeoForge, Forge, and Quilt all work out of the box — and Solara automatically sets up the right Java version for each one.',
  ],
  [
    'Is Solara really free with no ads?',
    'Yes. Zero ad banners, zero popups, and the full GPL-3.0 source code is public on GitHub.',
  ],
] as const;

onMounted(() => {
  preferences.value.motion = false;
  try {
    localStorage.removeItem('solara:motion');
  } catch {
    // ignore
  }
  window.addEventListener('popstate', onPopState);
  void refreshPublic();
  timer = setInterval(() => void refreshPublic(), 60000);
});

onUnmounted(() => {
  window.removeEventListener('popstate', onPopState);
  if (timer) clearInterval(timer);
});

async function copyText(text: string, key: string) {
  await navigator.clipboard.writeText(text);
  copied.value = key;
  setTimeout(() => {
    copied.value = '';
  }, 2400);
}
</script>

<template>
  <div class="website" :style="{ '--accent': '#3b82f6' }" :data-reduced-motion="preferences.motion">
    <a class="skip-link" href="#main">Skip to content</a>

    <!-- Minimal Sticky Pure OLED Navbar -->
    <header class="web-header">
      <div class="web-header-inner">
        <a class="web-brand" href="/" @click.prevent="navigate('/')">
          <img src="/logo.svg" alt="Solara" />
          <span>Solara</span>
        </a>

        <nav :class="{ expanded: mobile }" aria-label="Website navigation">
          <a href="/" :class="{ active: currentRoute === '/' }" @click.prevent="navigate('/')">About</a>
          <a href="/features" :class="{ active: currentRoute === '/features' }" @click.prevent="navigate('/features')">Features</a>
          <a href="/downloads" :class="{ active: currentRoute === '/downloads' }" @click.prevent="navigate('/downloads')">Downloads</a>
          <a href="/changelog" :class="['nav-with-pill', { active: currentRoute === '/changelog' }]" @click.prevent="navigate('/changelog')">
            <span>Releases</span>
            <span class="nav-release-pill">{{ latest ? latest.publicVersion : 'V-0.0.94' }}</span>
          </a>
          <a href="/security" :class="{ active: currentRoute === '/security' }" @click.prevent="navigate('/security')">Safety</a>
          <a href="/docs" :class="{ active: currentRoute === '/docs' }" @click.prevent="navigate('/docs')">Docs</a>
        </nav>

        <div class="header-actions">
          <a
            v-if="links.github"
            :href="links.github"
            target="_blank"
            rel="noopener"
            class="header-github"
          >
            <GitBranch />
            <span>GitHub</span>
          </a>

          <a
            :href="githubDownloadUrl"
            download="solaralauncher.exe"
            rel="noopener"
            class="header-download"
          >
            <span>Download</span>
          </a>

          <button
            class="mobile-menu"
            aria-label="Toggle navigation"
            :aria-expanded="mobile"
            @click="mobile = !mobile"
          >
            <X v-if="mobile" />
            <Menu v-else />
          </button>
        </div>
      </div>
    </header>

    <main id="main">
      <Transition name="page-slide" mode="out-in">
        <!-- ==================== HOME PAGE ==================== -->
        <div v-if="currentRoute === '/'" key="home" class="page-stack">
          <!-- Asymmetric Split Hero with Pure OLED Pixel-Voxel Skyline Backdrop -->
          <section class="hero-section">
            <div class="hero-backdrop" aria-hidden="true" />
            <div class="web-container hero-grid">
              <div class="hero-left">
                <div class="hero-eyebrow-row">
                  <span class="hero-eyebrow-item"><Box class="eyebrow-ic" /> Minecraft: Java Edition</span>
                  <span class="hero-sep">·</span>
                  <span class="hero-eyebrow-item">Built with Rust &amp; Vue</span>
                </div>

                <h1 class="display-title hero-h1">
                  Minecraft, the way<br />
                  <span class="accent-text">it should feel.</span>
                </h1>

                <p class="hero-lead">
                  A fast, ad-free pure OLED Minecraft launcher for Windows — one-click modpacks, automatic Java runtimes, and one permanent evergreen installer.
                </p>

                <div class="hero-cta-row">
                  <a
                    :href="githubDownloadUrl"
                    download="solaralauncher.exe"
                    rel="noopener"
                    class="btn-primary"
                  >
                    <Download />
                    <span>Download the launcher</span>
                  </a>

                  <a
                    v-if="links.github"
                    :href="links.github"
                    target="_blank"
                    rel="noopener"
                    class="btn-secondary"
                  >
                    <GitBranch />
                    <span>View on GitHub</span>
                  </a>
                </div>

                <div class="hero-meta-strip">
                  <span class="meta-os"><Monitor /> Windows 10/11 (64-bit)</span>
                  <span class="hero-sep">·</span>
                  <a :href="virusTotalUrl" target="_blank" rel="noopener" class="meta-vt">
                    <ShieldCheck /> VirusTotal Verified
                  </a>
                </div>
                <p class="hero-meta-sub">
                  Standalone <code>solaralauncher.exe</code> · {{ latest ? `${(latest.size / 1048576).toFixed(1)} MB` : '71.6 MB' }} · Latest {{ latest ? latest.publicVersion : 'V-0.0.94' }} · Direct GitHub CDN
                </p>
              </div>

              <!-- Right Floating Frosted OLED Interactive Runtime Card -->
              <div class="hero-right">
                <div class="runtime-card">
                  <span class="runtime-label">Smart Runtime Engine</span>
                  <h2 class="display-title runtime-title">
                    <span class="accent-text">Zero</span> manual setup.
                  </h2>
                  <p class="runtime-sub">
                    Automatic Eclipse Temurin Java pairing and isolated folders for every Minecraft version.
                  </p>

                  <div class="runtime-metric">
                    <div class="runtime-metric-head">
                      <span class="runtime-metric-name">
                        <img src="/logo.svg" alt="" class="mini-mark" />
                        <span>Solara Auto-Java</span>
                      </span>
                      <span class="runtime-metric-val">
                        <strong>Java {{ currentPreset.java }}</strong>
                        <small>{{ currentPreset.runtime }}</small>
                      </span>
                    </div>
                    <div class="runtime-bar-track">
                      <div class="runtime-bar-fill" :style="{ width: `${javaBarPercent}%` }" />
                    </div>
                  </div>

                  <div class="runtime-metric">
                    <div class="runtime-metric-head">
                      <span class="runtime-metric-name">
                        <Coffee class="metric-ic" />
                        <span>Minecraft Version</span>
                      </span>
                      <span class="runtime-metric-val muted-val">
                        <strong>{{ currentPreset.version }}</strong>
                        <small>{{ selectedLoader }}</small>
                      </span>
                    </div>
                    <div class="runtime-slider-wrap">
                      <div class="runtime-slider-track">
                        <div class="runtime-slider-fill" :style="{ width: `${sliderFillPercent}%` }" />
                      </div>
                      <input
                        v-model.number="selectedMcIndex"
                        type="range"
                        min="0"
                        :max="MC_PRESETS.length - 1"
                        step="1"
                        class="runtime-slider"
                        aria-label="Preview Minecraft version Java auto-pairing"
                      />
                    </div>
                  </div>

                  <div class="runtime-loaders" role="group" aria-label="Select mod loader">
                    <button
                      v-for="loader in (['Fabric', 'NeoForge', 'Forge', 'Quilt', 'Vanilla'] as const)"
                      :key="loader"
                      type="button"
                      :class="['loader-chip', { active: selectedLoader === loader }]"
                      @click="pickLoader(loader)"
                    >
                      {{ loader }}
                    </button>
                  </div>

                  <p class="runtime-foot">
                    Drag the version bar to preview Solara’s automatic Java &amp; loader pairing.
                  </p>
                </div>
              </div>
            </div>
          </section>

          <!-- Section 2: "What it is" Asymmetric Split with Rich Voxel World Artwork + 3 Borderless Columns -->
          <section id="about" class="web-container editorial-section">
            <div class="editorial-split">
              <div class="editorial-copy">
                <div class="section-kicker">
                  <span>What it is</span>
                  <img src="/logo.svg" alt="" class="kicker-logo" />
                </div>
                <h2 class="display-title section-h2">
                  Same Minecraft,<br />
                  cleaner workflow.
                </h2>
                <p class="section-body">
                  Solara Launcher gives every modpack and vanilla world its own clean directory, installs required Java runtimes automatically, and stays up to date from a single permanent Windows installer.
                </p>
                <a href="/features" class="inline-accent-link" @click.prevent="navigate('/features')">
                  <span>Explore all features</span>
                  <ArrowUpRight />
                </a>
              </div>

              <div class="editorial-media">
                <div class="art-feature-frame">
                  <img
                    src="/art/alpine-lake-v1.webp"
                    alt="Voxel alpine castle town by a mountain lake"
                    width="1440"
                    height="810"
                  />
                </div>
              </div>
            </div>

            <!-- 3 Borderless Minimal Pillars -->
            <div class="pillars-row">
              <div class="pillar-minimal">
                <h3><Lock class="pillar-ic" /> Official Microsoft login</h3>
                <p>
                  Sign in with your legally owned Minecraft: Java Edition account through Microsoft OAuth 2.0 PKCE directly on your own PC. Solara never sees your password.
                </p>
              </div>
              <div class="pillar-minimal">
                <h3><Layers3 class="pillar-ic" /> One permanent .exe</h3>
                <p>
                  Install <code>solaralauncher.exe</code> once from GitHub Releases. Every future update checks on launch, verifies its Minisign signature, and upgrades silently.
                </p>
              </div>
              <div class="pillar-minimal">
                <h3><GitBranch class="pillar-ic" /> Open source (GPL-3.0)</h3>
                <p>
                  Solara’s launcher code and exact release source archives are public on GitHub with permanent SHA-256 fingerprints and verifiable VirusTotal reports.
                </p>
              </div>
            </div>
          </section>

          <!-- Section 3: "Built in the open." Halftone Wave + 4-Tile High-Contrast Action Strip -->
          <section id="community" class="open-band">
            <div class="open-band-glow" aria-hidden="true" />
            <div class="open-band-dots" aria-hidden="true" />
            <div class="web-container open-band-inner">
              <span class="band-kicker">Community &amp; Transparency</span>
              <h2 class="display-title section-h2">Built in the open.</h2>
              <p class="section-body band-sub">
                Every release lands on GitHub in public. Inspect the source code, verify the permanent VirusTotal scan, or join the community.
              </p>

              <div class="action-tiles">
                <a
                  :href="DISCORD_URL"
                  target="_blank"
                  rel="noopener"
                  class="action-tile tile-discord"
                >
                  <div class="tile-head">
                    <Discord />
                    <strong>Discord</strong>
                  </div>
                  <p>Questions, live progress updates, and screenshots.</p>
                </a>

                <a
                  :href="virusTotalUrl"
                  target="_blank"
                  rel="noopener"
                  class="action-tile tile-dark"
                >
                  <div class="tile-head">
                    <ShieldCheck />
                    <strong>VirusTotal</strong>
                  </div>
                  <p>Inspect the permanent SHA-256 scan report anytime.</p>
                </a>

                <a
                  :href="links.github || 'https://github.com/Solara-Inc-Development/solara-launcher'"
                  target="_blank"
                  rel="noopener"
                  class="action-tile tile-white"
                >
                  <div class="tile-head">
                    <GitBranch />
                    <strong>Source</strong>
                  </div>
                  <p>Issues, pull requests, and official releases on GitHub.</p>
                </a>

                <a
                  href="/security"
                  class="action-tile tile-dark"
                  @click.prevent="navigate('/security')"
                >
                  <div class="tile-head">
                    <Lock />
                    <strong>How it’s built</strong>
                  </div>
                  <p>Minisign verification, local login, and why it’s safe to run.</p>
                </a>
              </div>

              <p class="band-foot">
                Want to verify your installer? <a href="/downloads" @click.prevent="navigate('/downloads')">Check the permanent SHA-256 &amp; PowerShell command</a>.
              </p>
            </div>
          </section>

          <!-- Section 4: Asymmetric Voxel Biome Gallery (Horizon Style) -->
          <section id="gallery" class="web-container editorial-section">
            <div class="gallery-grid">
              <div class="gallery-col-left">
                <figure class="gallery-shot">
                  <img src="/art/forest-valley-v1.webp" alt="Misty voxel pine valley and riverside cabin" loading="lazy" />
                </figure>
                <figure class="gallery-shot">
                  <img src="/art/moss-cave-v1.webp" alt="Sunlit voxel lush moss cavern" loading="lazy" />
                </figure>
              </div>

              <div class="gallery-col-right">
                <div class="gallery-heading-block">
                  <div class="section-kicker">
                    <span>Crafted for OLED</span>
                    <img src="/logo.svg" alt="" class="kicker-logo" />
                  </div>
                  <h2 class="display-title section-h2">
                    The same game you love, <span class="accent-text">zero clutter.</span>
                  </h2>
                  <p class="section-body">
                    Keep every survival world, modded SMP, and shader pack isolated in its own tidy instance — zero ads or popups.
                  </p>
                </div>

                <figure class="gallery-shot">
                  <img src="/art/canyon-settlement-v1.webp" alt="Golden voxel canyon village" loading="lazy" />
                </figure>
                <figure class="gallery-shot">
                  <img src="/art/coastal-hamlet-v1.webp" alt="Coastal voxel fishing hamlet" loading="lazy" />
                </figure>
              </div>
            </div>

            <!-- Clean Interactive Desktop Preview Showcase -->
            <div class="launcher-preview-wrap">
              <div class="launcher-preview-head">
                <div>
                  <span class="band-kicker">INSIDE THE LAUNCHER</span>
                  <h3 class="display-title preview-h3">Built for OLED Black &amp; Crisp White.</h3>
                </div>
                <div class="media-pills" role="tablist" aria-label="Launcher views">
                  <button
                    v-for="tab in SHOWCASE_TABS"
                    :key="tab.id"
                    type="button"
                    role="tab"
                    :aria-selected="activeShot === tab.id"
                    :class="['media-pill', { active: activeShot === tab.id }]"
                    @click="selectShot(tab.id)"
                  >
                    {{ tab.label }}
                  </button>
                </div>
              </div>
              <div class="media-frame">
                <Transition name="shot-fade" mode="out-in">
                  <img
                    v-if="activeShot === 'dashboard'"
                    key="dashboard"
                    src="/screenshots/dashboard.png"
                    alt="Solara Launcher Home screen in OLED dark theme"
                    width="1440"
                    height="950"
                  />
                  <img
                    v-else-if="activeShot === 'modpacks'"
                    key="modpacks"
                    src="/screenshots/modpacks.png"
                    alt="Solara Launcher Modpack Browser"
                    width="1380"
                    height="860"
                  />
                  <img
                    v-else-if="activeShot === 'icon-studio'"
                    key="icon-studio"
                    src="/screenshots/icon-studio.png"
                    alt="Solara Icon Studio"
                    width="1380"
                    height="860"
                  />
                  <img
                    v-else-if="activeShot === 'settings'"
                    key="settings"
                    src="/screenshots/settings-resources.png"
                    alt="Solara Storage and Resources"
                    width="1440"
                    height="950"
                  />
                  <img
                    v-else
                    key="white"
                    src="/screenshots/downloads-white.png"
                    alt="Solara Launcher White Theme"
                    width="1440"
                    height="950"
                  />
                </Transition>
              </div>
            </div>
          </section>

          <!-- Section 5: Big Typographic Stat + 2-Column Numbered Expandable Subsystems -->
          <section id="architecture" class="web-container editorial-section">
            <div class="stat-header-grid">
              <div class="stat-hero-left">
                <div class="big-stat-number display-title">100%</div>
                <p class="section-body">
                  ad-free and open source: every subsystem in Solara Launcher is transparent, isolated on your disk, and cryptographically verified before updating.
                </p>
              </div>
              <div class="stat-hero-right">
                <h2 class="display-title section-h2">Built for everyday play.</h2>
                <p class="section-body">
                  Below is an overview of Solara’s core launcher subsystems — click any row to inspect how it works under the hood on Windows.
                </p>
                <p class="stat-meta-line">
                  Current preview {{ latest ? latest.publicVersion : 'V-0.0.94' }} ·
                  <a href="/changelog" class="inline-accent-link" @click.prevent="navigate('/changelog')">
                    <span>read full release notes</span>
                    <ArrowUpRight />
                  </a>
                </p>
              </div>
            </div>

            <div class="subsystem-grid">
              <div
                v-for="(item, idx) in SUBSYSTEMS"
                :key="item.id"
                :class="['subsystem-row', { open: openSubsystem === idx }]"
              >
                <button
                  type="button"
                  class="subsystem-trigger"
                  :aria-expanded="openSubsystem === idx"
                  @click="openSubsystem = openSubsystem === idx ? null : idx"
                >
                  <span class="subsystem-left">
                    <span class="subsystem-num">{{ item.id }}</span>
                    <span class="subsystem-title">{{ item.title }}</span>
                  </span>
                  <span class="subsystem-right">
                    <span class="status-bars" aria-hidden="true">
                      <i /><i /><i /><i />
                    </span>
                    <ChevronDown class="subsystem-chev" />
                  </span>
                </button>
                <div v-if="openSubsystem === idx" class="subsystem-body">
                  <p>{{ item.detail }}</p>
                </div>
              </div>
            </div>
          </section>

          <!-- Section 6: Clean Download CTA Panel -->
          <section id="downloads" class="web-container editorial-section">
            <div class="download-panel">
              <div class="download-platform">
                <div class="download-icon-box">
                  <Monitor />
                </div>
                <div>
                  <div class="download-title-row">
                    <h3>Solara Launcher for Windows</h3>
                    <span class="status-badge">
                      <span class="dot" />
                      {{ latest ? latest.publicVersion : 'V-0.0.94' }}
                    </span>
                  </div>
                  <p class="download-meta">
                    Windows 10/11 (64-bit) · {{ latest ? `${(latest.size / 1048576).toFixed(1)} MB` : '30.0 MB' }} · One permanent evergreen installer
                  </p>
                </div>
              </div>

              <div class="download-cta-col">
                <div class="cta-btn-row">
                  <a
                    :href="githubDownloadUrl"
                    download="solaralauncher.exe"
                    rel="noopener"
                    class="btn-primary"
                  >
                    <Download />
                    <span>Download solaralauncher.exe</span>
                  </a>
                  <a
                    :href="virusTotalUrl"
                    target="_blank"
                    rel="noopener"
                    class="btn-secondary"
                  >
                    <ShieldCheck />
                    <span>VirusTotal</span>
                    <ArrowUpRight />
                  </a>
                </div>
                <small>Permanent SHA-256 · Streams directly from GitHub Releases CDN</small>
              </div>
            </div>
          </section>
        </div>

        <!-- ==================== FEATURES PAGE ==================== -->
        <div v-else-if="currentRoute === '/features'" key="features" class="page-stack">
          <section class="web-container sub-hero">
            <span class="band-kicker">FEATURES</span>
            <h1 class="display-title sub-h1">
              Made for smooth,<br /><span class="accent-text">stress-free Minecraft.</span>
            </h1>
            <p class="hero-lead">
              Every screen in Solara is built to feel snappy, clean on OLED and bright screens, and easy for any player to jump right into.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="feature-grid">
              <article
                v-for="(f, i) in features"
                :key="f.title"
                :class="['feature', `feature--${f.tone}`]"
              >
                <div class="feature-top">
                  <span class="feature-icon-wrap">
                    <component :is="f.icon" class="feature-icon" />
                  </span>
                  <span class="feature-tag">0{{ i + 1 }} · {{ f.tag }}</span>
                </div>
                <h3>{{ f.title }}</h3>
                <p>{{ f.copy }}</p>
              </article>
            </div>
          </section>

          <section class="web-container sub-section page-end">
            <div class="section-header-left">
              <span class="band-kicker">MOD LOADERS</span>
              <h2 class="display-title section-h2">All 5 Minecraft setups supported.</h2>
              <p class="section-body">Pick whichever loader you like — Solara matches the right Java version automatically.</p>
            </div>
            <div class="data-table-wrap">
              <table class="data-table">
                <thead>
                  <tr>
                    <th>Loader</th>
                    <th>What it is</th>
                    <th>Versions</th>
                    <th>Java Setup</th>
                    <th>Best For</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="row in LOADER_MATRIX" :key="row.name">
                    <td><strong>{{ row.name }}</strong></td>
                    <td>{{ row.focus }}</td>
                    <td>{{ row.mcRange }}</td>
                    <td>{{ row.javaAuto }}</td>
                    <td>{{ row.bestFor }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </section>
        </div>

        <!-- ==================== DOWNLOADS PAGE ==================== -->
        <div v-else-if="currentRoute === '/downloads'" key="downloads" class="page-stack">
          <section class="web-container sub-hero">
            <span class="band-kicker">PERMANENT EVERGREEN INSTALLER · DIRECT FROM GITHUB</span>
            <h1 class="display-title sub-h1">
              Get <span class="accent-text">Solara Launcher.</span>
            </h1>
            <p class="hero-lead">
              One single permanent file: <code>solaralauncher.exe</code>. Download it once from GitHub Releases — every future update fetches and installs automatically on launch.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="download-panel">
              <div class="download-platform">
                <div class="download-icon-box">
                  <Monitor />
                </div>
                <div>
                  <div class="download-title-row">
                    <h3>Solara Launcher for Windows (64-bit)</h3>
                    <span class="status-badge">
                      <span class="dot" />
                      {{ latest ? latest.publicVersion : 'V-0.0.94' }}
                    </span>
                  </div>
                  <p class="download-meta">
                    {{ latest ? `${(latest.size / 1048576).toFixed(2)} MB` : '30.02 MB' }} · Permanent Universal Installer · Hosted on GitHub Releases CDN
                  </p>
                  <div class="download-hash-row">
                    <code>SHA-256: {{ DEFAULT_VT_SHA256 }}</code>
                    <button
                      type="button"
                      class="hash-copy-btn"
                      @click="copyText(DEFAULT_VT_SHA256, 'latest-sha')"
                    >
                      <Check v-if="copied === 'latest-sha'" />
                      <Copy v-else />
                      <span>{{ copied === 'latest-sha' ? 'Copied!' : 'Copy hash' }}</span>
                    </button>
                  </div>
                </div>
              </div>

              <div class="download-cta-col">
                <div class="cta-btn-row">
                  <a
                    :href="githubDownloadUrl"
                    download="solaralauncher.exe"
                    rel="noopener"
                    class="btn-primary"
                  >
                    <Download />
                    <span>Download from GitHub</span>
                  </a>
                  <a
                    :href="virusTotalUrl"
                    target="_blank"
                    rel="noopener"
                    class="btn-secondary"
                  >
                    <ShieldCheck />
                    <span>VirusTotal Scan</span>
                    <ArrowUpRight />
                  </a>
                </div>
                <a
                  v-if="links.github"
                  :href="`${links.github}/releases/latest`"
                  target="_blank"
                  rel="noopener"
                  class="inline-accent-link"
                >
                  <span>View latest release on GitHub</span>
                  <ArrowUpRight />
                </a>
              </div>
            </div>
          </section>

          <section class="web-container sub-section">
            <div class="info-card">
              <div class="info-card-head">
                <div>
                  <span class="band-kicker">VERIFICATION</span>
                  <h3>Verify your download with PowerShell or VirusTotal</h3>
                </div>
                <div class="cta-btn-row">
                  <a
                    :href="virusTotalUrl"
                    target="_blank"
                    rel="noopener"
                    class="btn-secondary compact-btn"
                  >
                    <ShieldCheck />
                    <span>Open VirusTotal Report</span>
                    <ArrowUpRight />
                  </a>
                  <button
                    type="button"
                    class="btn-secondary compact-btn"
                    @click="copyText(`(Get-FileHash .\\solaralauncher.exe -Algorithm SHA256).Hash -eq '${DEFAULT_VT_SHA256.toUpperCase()}'`, 'ps-cmd')"
                  >
                    <Check v-if="copied === 'ps-cmd'" />
                    <Copy v-else />
                    <span>{{ copied === 'ps-cmd' ? 'Copied!' : 'Copy PowerShell check' }}</span>
                  </button>
                </div>
              </div>
              <pre class="code-box"><code>(Get-FileHash .\solaralauncher.exe -Algorithm SHA256).Hash -eq '{{ DEFAULT_VT_SHA256.toUpperCase() }}'</code></pre>
              <p class="info-note">
                Paste this into Windows PowerShell in your Downloads folder. If it prints <code>True</code>, your file matches our official GitHub release 100%.
              </p>
            </div>
          </section>

          <section class="web-container sub-section">
            <div class="section-header-left">
              <span class="band-kicker">REQUIREMENTS</span>
              <h2 class="display-title section-h2">PC Requirements</h2>
            </div>
            <div class="data-table-wrap">
              <table class="data-table">
                <thead>
                  <tr>
                    <th>Requirement</th>
                    <th>Minimum</th>
                    <th>Recommended</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="req in SYSTEM_REQUIREMENTS" :key="req.label">
                    <td><strong>{{ req.label }}</strong></td>
                    <td>{{ req.minimum }}</td>
                    <td>{{ req.recommended }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </section>

          <section class="web-container sub-section page-end">
            <div class="info-card">
              <div class="info-card-head">
                <div>
                  <span class="band-kicker">EVERGREEN ARCHITECTURE</span>
                  <h3>One single <code>solaralauncher.exe</code> for every future update</h3>
                </div>
                <a
                  href="/changelog"
                  class="btn-secondary compact-btn"
                  @click.prevent="navigate('/changelog')"
                >
                  <span>View What’s New</span>
                  <ChevronRight />
                </a>
              </div>
              <ul class="bullet-list">
                <li><strong>One File Forever:</strong> You never need to download a separate <code>.exe</code> when a new version is released.</li>
                <li><strong>Always Fetches the Latest Version:</strong> Even if you run this same <code>solaralauncher.exe</code> months later, it automatically checks GitHub Releases on launch and upgrades itself in seconds.</li>
                <li><strong>Zero-Click Background Updates:</strong> Installed copies verify Minisign signatures and apply updates silently whenever no game is running.</li>
              </ul>
            </div>
          </section>
        </div>

        <!-- ==================== CHANGELOG PAGE ==================== -->
        <div v-else-if="currentRoute === '/changelog'" key="changelog" class="page-stack">
          <section class="web-container sub-hero">
            <span class="band-kicker">RELEASES</span>
            <h1 class="display-title sub-h1">
              Recent updates &amp;<br /><span class="accent-text">what’s coming next.</span>
            </h1>
            <p class="hero-lead">
              Every major Solara update in plain English — delivered automatically to your installed launcher.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="timeline">
              <article
                v-for="item in CHANGELOG_HIGHLIGHTS"
                :key="item.version"
                class="timeline-card"
              >
                <div class="timeline-head">
                  <span class="status-badge">
                    <span class="dot" />
                    {{ item.version }}
                  </span>
                  <span class="timeline-tag">{{ item.tag }}</span>
                  <span class="timeline-date">{{ item.date }}</span>
                </div>
                <h3>{{ item.title }}</h3>
                <ul class="bullet-list">
                  <li v-for="point in item.highlights" :key="point">{{ point }}</li>
                </ul>
              </article>
            </div>
          </section>

          <section class="web-container sub-section page-end">
            <div class="section-header-left">
              <span class="band-kicker">ROADMAP</span>
              <h2 class="display-title section-h2">Public Roadmap</h2>
              <p class="section-body">What we are working on right now and what is planned next.</p>
            </div>
            <div class="roadmap-grid">
              <article
                v-for="r in ROADMAP_ITEMS"
                :key="r.title"
                class="roadmap-card"
              >
                <span class="roadmap-phase">{{ r.phase }}</span>
                <h3>{{ r.title }}</h3>
                <p>{{ r.copy }}</p>
              </article>
            </div>
          </section>
        </div>

        <!-- ==================== SECURITY PAGE ==================== -->
        <div v-else-if="currentRoute === '/security'" key="security" class="page-stack">
          <section class="web-container sub-hero">
            <span class="band-kicker">SAFETY &amp; TRANSPARENCY</span>
            <h1 class="display-title sub-h1">
              Built to be safe.<br /><span class="accent-text">Open for anyone to check.</span>
            </h1>
            <p class="hero-lead">
              Your Minecraft login stays on your PC, every download streams straight from GitHub Releases, and you can inspect our permanent VirusTotal report anytime.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="pillar-grid">
              <article
                v-for="p in SECURITY_PILLARS"
                :key="p.title"
                class="pillar-card"
              >
                <span class="roadmap-phase">{{ p.badge }}</span>
                <h3>{{ p.title }}</h3>
                <p>{{ p.copy }}</p>
              </article>
            </div>
          </section>

          <section class="web-container sub-section page-end">
            <div class="info-card">
              <div class="info-card-head">
                <div>
                  <span class="band-kicker">PUBLIC VERIFICATION</span>
                  <h3>Permanent VirusTotal Report &amp; Open-Source Status</h3>
                </div>
                <a
                  :href="virusTotalUrl"
                  target="_blank"
                  rel="noopener"
                  class="btn-secondary compact-btn"
                >
                  <ShieldCheck />
                  <span>Inspect on VirusTotal</span>
                  <ArrowUpRight />
                </a>
              </div>
              <ul class="bullet-list">
                <li><strong>Direct GitHub Hosting:</strong> All installer and auto-update payloads come directly from GitHub Releases CDN.</li>
                <li><strong>Permanent Installer SHA-256:</strong> <code>{{ DEFAULT_VT_SHA256 }}</code> — verified directly on VirusTotal.</li>
                <li><strong>Testing Preview:</strong> Solara Launcher is in active Windows testing preview. Official Microsoft accounts only — no cracked logins.</li>
                <li><strong>Open Source (GPL-3.0):</strong> The full launcher source and third-party license notices are included in every release and on GitHub.</li>
              </ul>
            </div>
          </section>
        </div>

        <!-- ==================== DOCS & FAQ PAGE ==================== -->
        <div v-else key="docs" class="page-stack">
          <section class="web-container sub-hero">
            <span class="band-kicker">DOCS &amp; HELP</span>
            <h1 class="display-title sub-h1">
              Easy setup &amp;<br /><span class="accent-text">quick answers.</span>
            </h1>
            <p class="hero-lead">
              New to Solara? Follow these quick steps to get playing in under two minutes.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="guide-grid">
              <article
                v-for="g in GUIDES"
                :key="g.id"
                class="guide-card"
              >
                <span class="roadmap-phase">{{ g.step }}</span>
                <h3>{{ g.title }}</h3>
                <ul class="bullet-list">
                  <li v-for="pt in g.points" :key="pt">{{ pt }}</li>
                </ul>
              </article>
            </div>
          </section>

          <section id="faq" class="web-container sub-section page-end">
            <div class="section-header-left">
              <span class="band-kicker">FAQ</span>
              <h2 class="display-title section-h2">Common questions</h2>
            </div>

            <div class="faq-list">
              <div v-for="([question, answer], i) in questions" :key="question" class="faq-item" :class="{ open: faq === i }">
                <button
                  type="button"
                  class="faq-trigger"
                  :aria-expanded="faq === i"
                  @click="faq = faq === i ? null : i"
                >
                  <span>{{ question }}</span>
                  <span class="faq-icon">
                    <Minus v-if="faq === i" />
                    <Plus v-else />
                  </span>
                </button>
                <div v-if="faq === i" class="faq-answer">
                  <p>{{ answer }}</p>
                </div>
              </div>
            </div>
          </section>
        </div>
      </Transition>
    </main>

    <!-- Minimal Editorial 3-Column Footer (Horizon Rhythm on Pure OLED Black) -->
    <footer class="web-footer">
      <div class="web-container">
        <div class="footer-head">
          <a class="web-brand" href="/" @click.prevent="navigate('/')">
            <img src="/logo.svg" alt="Solara" />
            <span>Solara</span>
          </a>
          <p class="footer-disclaimer">
            Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft.
          </p>
        </div>

        <div class="footer-cols">
          <div class="footer-col">
            <h2>Who makes it</h2>
            <p>
              Solara Launcher is developed by Solara Inc. Development for Windows players who want a fast, pitch-black OLED launcher without ad clutter. This site offers no Minecraft game files — you need your own legally owned copy of Minecraft: Java Edition.
            </p>
          </div>

          <div class="footer-col">
            <h2>Licences</h2>
            <p>
              Solara Launcher is open source under the <a :href="`${links.github || 'https://github.com/Solara-Inc-Development/solara-launcher'}/blob/main/LICENSE`" target="_blank" rel="noopener">GPL-3.0-only licence</a>. Third-party dependency notices are bundled in every build and listed in <code>THIRD_PARTY_NOTICES.md</code>. Fonts are Geist, Bricolage Grotesque, and JetBrains Mono under the SIL Open Font License.
            </p>
          </div>

          <div class="footer-col">
            <h2>Privacy &amp; Verification</h2>
            <p>
              Your Microsoft login stays strictly on your own PC via OAuth 2.0 PKCE. All installer downloads stream directly from GitHub Releases CDN and can be inspected anytime via our <a :href="virusTotalUrl" target="_blank" rel="noopener">permanent VirusTotal report</a>.
            </p>
            <div class="footer-inline-links">
              <a href="/downloads" @click.prevent="navigate('/downloads')">Downloads</a>
              <a href="/changelog" @click.prevent="navigate('/changelog')">Releases</a>
              <a href="/security" @click.prevent="navigate('/security')">Safety</a>
              <a href="/docs" @click.prevent="navigate('/docs')">Docs</a>
            </div>
          </div>
        </div>

        <div class="footer-bottom">
          <span>Made in the open, with Rust &amp; Vue.</span>
          <a
            :href="links.github || 'https://github.com/Solara-Inc-Development/solara-launcher'"
            target="_blank"
            rel="noopener"
            class="footer-repo-link"
          >
            <GitBranch />
            <span>Solara-Inc-Development/solara-launcher</span>
          </a>
        </div>
      </div>
    </footer>
  </div>
</template>
