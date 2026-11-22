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
  Copy,
  Sparkles,
  GitBranch,
  Menu,
  X,
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
import { isDownloadable } from '../../packages/shared/domain.mjs';
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
const copied = ref('');
const releaseSearch = ref('');
const showAllReleases = ref(false);
const activeShot = ref<(typeof SHOWCASE_TABS)[number]['id']>('modpacks');
const showcasePaused = ref(false);
const previewTheme = ref<'oled' | 'white'>('oled');
let timer: ReturnType<typeof setInterval> | null = null;
let showcaseTimer: ReturnType<typeof setInterval> | null = null;
let themeTimer: ReturnType<typeof setInterval> | null = null;

const DIRECT_GITHUB_DOWNLOAD_URL =
  'https://github.com/Solara-Inc-Development/solara-launcher/releases/latest/download/solaralauncher.exe';

const DEFAULT_VT_SHA256 =
  '21cc653fd39275144b45c2962aa6e10e63d10a594a8711d382e6a96d7df36b0a';

const githubDownloadUrl = computed(() => DIRECT_GITHUB_DOWNLOAD_URL);

const virusTotalUrl = computed(() => {
  const sha = latest.value?.sha256 || DEFAULT_VT_SHA256;
  return `https://www.virustotal.com/gui/file/${sha}/`;
});

const marqueeItems = [
  'ONE-CLICK MODPACKS',
  '30+ 3D FLUENT ICONS',
  'SOLARA ICON STUDIO',
  'TRUE #000000 OLED',
  'AUTO JAVA 8 / 17 / 21',
  'MINISIGN VERIFIED UPDATES',
  'VIRUSTOTAL TRANSPARENT SCAN',
  'ZERO ADS FOREVER',
  '100% OPEN SOURCE GPL-3.0',
] as const;

function onCardPointerMove(event: PointerEvent) {
  const el = event.currentTarget as HTMLElement | null;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;
  const rx = ((y / rect.height) - 0.5) * -5.5;
  const ry = ((x / rect.width) - 0.5) * 5.5;
  el.style.setProperty('--mx', `${x}px`);
  el.style.setProperty('--my', `${y}px`);
  el.style.setProperty('--rx', `${rx.toFixed(2)}deg`);
  el.style.setProperty('--ry', `${ry.toFixed(2)}deg`);
}

function onCardPointerLeave(event: PointerEvent) {
  const el = event.currentTarget as HTMLElement | null;
  if (!el) return;
  el.style.setProperty('--rx', '0deg');
  el.style.setProperty('--ry', '0deg');
}

function selectShot(id: (typeof SHOWCASE_TABS)[number]['id']) {
  activeShot.value = id;
}

function stepShowcase() {
  if (showcasePaused.value || currentRoute.value !== '/') return;
  const idx = SHOWCASE_TABS.findIndex((t) => t.id === activeShot.value);
  const next = SHOWCASE_TABS[(idx + 1) % SHOWCASE_TABS.length];
  if (next) activeShot.value = next.id;
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

const releases = computed(() => snapshot.value.data?.releases || []);

const filteredReleases = computed(() => {
  const q = releaseSearch.value.trim().toLowerCase();
  if (!q) return releases.value;
  return releases.value.filter(
    (r: { publicVersion: string; notes?: string; sha256?: string }) =>
      r.publicVersion.toLowerCase().includes(q) ||
      (r.notes || '').toLowerCase().includes(q) ||
      (r.sha256 || '').toLowerCase().includes(q),
  );
});

const visibleReleases = computed(() =>
  showAllReleases.value ? filteredReleases.value : filteredReleases.value.slice(0, 8),
);

const currentShot = computed(
  () => SHOWCASE_TABS.find((t) => t.id === activeShot.value) ?? SHOWCASE_TABS[0],
);

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
    title: 'Separate Game Folders',
    copy: 'Keep Vanilla, Fabric, Forge, NeoForge, and Quilt worlds in their own clean folders so mods never clash.',
    tag: 'WORLDS',
  },
  {
    icon: Package,
    tone: 'emerald',
    title: 'One-Click Modpacks',
    copy: 'Browse huge community catalogs, filter by Minecraft version or loader, and install with a single click.',
    tag: 'MODPACKS',
  },
  {
    icon: Sparkles,
    tone: 'purple',
    title: 'Icon Studio & 3D Skins',
    copy: 'Customize every instance with 30 3D icons, text badges, and frames — plus a live 3D skin viewer.',
    tag: 'STYLE',
  },
  {
    icon: Monitor,
    tone: 'amber',
    title: 'Auto Java Setup',
    copy: 'No hunting for Java downloads. Solara detects and installs the right Java version automatically.',
    tag: 'AUTO JAVA',
  },
  {
    icon: ShieldCheck,
    tone: 'sky',
    title: 'Safe Microsoft Login',
    copy: 'Sign in directly with Microsoft on your own PC. Your password never touches Solara servers.',
    tag: 'PRIVATE',
  },
  {
    icon: Download,
    tone: 'rose',
    title: 'Evergreen Auto-Updates',
    copy: 'Download once forever. Signed updates download and apply automatically whenever no game is running.',
    tag: 'UPDATES',
  },
];

const quickSteps = [
  {
    num: '01',
    title: 'Download once from GitHub',
    copy: 'Grab solaralauncher.exe directly from GitHub Releases CDN and run the quick installer.',
  },
  {
    num: '02',
    title: 'Sign in with Microsoft',
    copy: 'Log in safely through Microsoft’s official window on your own PC.',
  },
  {
    num: '03',
    title: 'Pick a pack & play',
    copy: 'Create a custom world or click Install on any modpack — Java sets up automatically.',
  },
] as const;

const questions = [
  [
    'Do I need to download a new .exe for every update?',
    'Nope! You only download solaralauncher.exe once. After that, Solara automatically downloads and installs verified updates on launch whenever no game is running.',
  ],
  [
    'Where does the download come from?',
    'Directly from our official GitHub Releases CDN — fast, verifiable with SHA-256 and VirusTotal, and open source.',
  ],
  [
    'Is my Microsoft account safe?',
    'Yes. You sign in straight through Microsoft on your own PC. Your password never goes through Solara, and your login stays locked inside Windows.',
  ],
  [
    'Which mod loaders work in Solara?',
    'Vanilla, Fabric, NeoForge, Forge, and Quilt all work out of the box — and Solara automatically sets up the right Java version for each one.',
  ],
  [
    'Is Solara really free with no ads?',
    'Yes! Zero ad banners, zero popups, and the full open-source code is public on GitHub.',
  ],
] as const;

onMounted(() => {
  // Ensure animations are always enabled by default
  preferences.value.motion = false;
  try {
    localStorage.removeItem('solara:motion');
  } catch {
    // ignore
  }
  window.addEventListener('popstate', onPopState);
  void refreshPublic();
  timer = setInterval(() => void refreshPublic(), 60000);
  showcaseTimer = setInterval(stepShowcase, 4500);
  themeTimer = setInterval(() => {
    if (!showcasePaused.value && currentRoute.value === '/') {
      previewTheme.value = previewTheme.value === 'oled' ? 'white' : 'oled';
    }
  }, 6000);
});

onUnmounted(() => {
  window.removeEventListener('popstate', onPopState);
  if (timer) clearInterval(timer);
  if (showcaseTimer) clearInterval(showcaseTimer);
  if (themeTimer) clearInterval(themeTimer);
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
    <!-- Animated Multi-Layer Cosmic OLED Background -->
    <div class="ambient-bg" aria-hidden="true">
      <span class="ambient-orb orb-1" />
      <span class="ambient-orb orb-2" />
      <span class="ambient-orb orb-3" />
      <span class="ambient-beam beam-1" />
      <span class="ambient-beam beam-2" />
      <span class="ambient-particles">
        <i class="p-dot p-1" />
        <i class="p-dot p-2" />
        <i class="p-dot p-3" />
        <i class="p-dot p-4" />
        <i class="p-dot p-5" />
        <i class="p-dot p-6" />
      </span>
      <span class="ambient-grid" />
    </div>

    <a class="skip-link" href="#main">Skip to content</a>

    <!-- Sticky Frosted Navbar -->
    <header class="web-header">
      <div class="web-header-inner">
        <a class="web-brand" href="/" @click.prevent="navigate('/')">
          <span class="brand-logo-wrap">
            <img src="/logo.svg" alt="Solara" />
          </span>
          <span>Solara</span>
          <span v-if="latest" class="nav-version-badge">{{ latest.publicVersion }}</span>
        </a>

        <nav :class="{ expanded: mobile }" aria-label="Website navigation">
          <a href="/" :class="{ active: currentRoute === '/' }" @click.prevent="navigate('/')">Home</a>
          <a href="/features" :class="{ active: currentRoute === '/features' }" @click.prevent="navigate('/features')">Features</a>
          <a href="/downloads" :class="{ active: currentRoute === '/downloads' }" @click.prevent="navigate('/downloads')">Downloads</a>
          <a href="/changelog" :class="{ active: currentRoute === '/changelog' }" @click.prevent="navigate('/changelog')">What’s New</a>
          <a href="/security" :class="{ active: currentRoute === '/security' }" @click.prevent="navigate('/security')">Safety</a>
          <a href="/docs" :class="{ active: currentRoute === '/docs' }" @click.prevent="navigate('/docs')">Help &amp; FAQ</a>
        </nav>

        <div class="header-actions">
          <a
            :href="githubDownloadUrl"
            download="solaralauncher.exe"
            rel="noopener"
            class="header-download shimmer-btn"
          >
            <Download />
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
          <!-- Hero -->
          <section class="web-hero web-container">
            <a
              href="/changelog"
              class="hero-pill hero-anim-1"
              @click.prevent="navigate('/changelog')"
            >
              <span class="dot pulse-dot" />
              <strong>{{ latest ? `${latest.publicVersion} Live on GitHub` : 'Windows x64 Preview' }}</strong>
              <span class="hero-pill-sep">·</span>
              <span>One Evergreen Installer &amp; Auto-Updates</span>
              <ChevronRight class="pill-arrow" />
            </a>

            <h1 class="hero-anim-2">
              Play Minecraft,<br />
              <span class="hero-gradient-text">the fast &amp; clean way.</span>
            </h1>

            <p class="hero-description hero-anim-3">
              Keep your worlds tidy, install modpacks in one click, and customize 3D icons &amp; skins — all inside a sleek, ad-free Windows launcher that updates itself automatically.
            </p>

            <div class="hero-actions hero-anim-4">
              <a
                :href="githubDownloadUrl"
                download="solaralauncher.exe"
                rel="noopener"
                class="button primary hero-cta shimmer-btn cta-pulse"
              >
                <Download />
                <span>Download from GitHub</span>
                <span v-if="latest" class="cta-meta">{{ latest.publicVersion }}</span>
              </a>

              <a
                :href="virusTotalUrl"
                target="_blank"
                rel="noopener"
                class="button secondary hero-secondary vt-badge-btn"
              >
                <ShieldCheck />
                <span>VirusTotal Scan</span>
                <ArrowUpRight />
              </a>

              <a
                href="/features"
                class="button secondary hero-secondary"
                @click.prevent="navigate('/features')"
              >
                <span>Explore Features</span>
                <ChevronRight />
              </a>
            </div>

            <div class="hero-trust hero-anim-5">
              <span><Check /> Direct GitHub CDN Download</span>
              <span><ShieldCheck /> Official Microsoft Login</span>
              <span><Check /> Evergreen Auto-Updates</span>
              <span><Check /> 100% Free &amp; Open Source</span>
            </div>

            <!-- Interactive Auto-Cycling 5-View App Showcase -->
            <div
              id="showcase"
              class="showcase-wrap hero-anim-6"
              @mouseenter="showcasePaused = true"
              @mouseleave="showcasePaused = false"
            >
              <div class="showcase-glow" aria-hidden="true" />
              <div class="showcase-shell animated-border-card">
                <div class="showcase-toolbar">
                  <div class="showcase-dots" aria-hidden="true">
                    <span />
                    <span />
                    <span />
                  </div>

                  <div class="showcase-tabs" role="tablist" aria-label="Launcher views">
                    <button
                      v-for="tab in SHOWCASE_TABS"
                      :key="tab.id"
                      type="button"
                      role="tab"
                      :aria-selected="activeShot === tab.id"
                      :class="['showcase-tab', { active: activeShot === tab.id, cycling: activeShot === tab.id && !showcasePaused }]"
                      @click="selectShot(tab.id)"
                    >
                      <span>{{ tab.label }}</span>
                      <small>{{ tab.badge }}</small>
                      <i v-if="activeShot === tab.id" class="tab-progress-bar" />
                    </button>
                  </div>
                </div>

                <div class="showcase-stage">
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
                      alt="Solara Launcher Modpack Browser with cover art cards"
                      width="1380"
                      height="860"
                    />
                    <img
                      v-else-if="activeShot === 'icon-studio'"
                      key="icon-studio"
                      src="/screenshots/icon-studio.png"
                      alt="Solara Icon Studio with 3D icons and custom badges"
                      width="1380"
                      height="860"
                    />
                    <img
                      v-else-if="activeShot === 'settings'"
                      key="settings"
                      src="/screenshots/settings-resources.png"
                      alt="Solara Storage and Resources screen"
                      width="1440"
                      height="950"
                    />
                    <img
                      v-else
                      key="white"
                      src="/screenshots/downloads-white.png"
                      alt="Solara Launcher clean White theme"
                      width="1440"
                      height="950"
                    />
                  </Transition>
                </div>

                <div class="showcase-footer">
                  <p>{{ currentShot.caption }}</p>
                  <a :href="githubDownloadUrl" download="solaralauncher.exe" rel="noopener" class="text-link">
                    Download solaralauncher.exe <ChevronRight />
                  </a>
                </div>
              </div>
            </div>
          </section>

          <!-- Infinite Animated Feature Marquee Ticker -->
          <div class="marquee-strip" aria-hidden="true">
            <div class="marquee-track">
              <div v-for="loop in 2" :key="loop" class="marquee-group">
                <span v-for="item in marqueeItems" :key="`${loop}-${item}`" class="marquee-item">
                  <i class="marquee-dot" />
                  {{ item }}
                </span>
              </div>
            </div>
          </div>

          <!-- Key Stats Strip -->
          <section class="web-container stats-strip">
            <div class="stat-box spotlight-hover tilt-card" @pointermove="onCardPointerMove" @pointerleave="onCardPointerLeave">
              <strong>1 File Forever</strong>
              <span>Evergreen auto-updating .exe</span>
            </div>
            <div class="stat-box spotlight-hover tilt-card" @pointermove="onCardPointerMove" @pointerleave="onCardPointerLeave">
              <strong>5 Loaders</strong>
              <span>Vanilla, Fabric, NeoForge, Forge &amp; Quilt</span>
            </div>
            <div class="stat-box spotlight-hover tilt-card" @pointermove="onCardPointerMove" @pointerleave="onCardPointerLeave">
              <strong>30+ 3D Icons</strong>
              <span>Plus custom text badges &amp; frames</span>
            </div>
            <div class="stat-box spotlight-hover tilt-card" @pointermove="onCardPointerMove" @pointerleave="onCardPointerLeave">
              <strong>GitHub CDN</strong>
              <span>Direct downloads &amp; VirusTotal verified</span>
            </div>
          </section>

          <!-- Visual Bento Showcase -->
          <section class="web-container web-section">
            <div class="section-header-row">
              <div class="section-header">
                <span class="eyebrow">BUILT FOR PLAYERS</span>
                <h2>Designed to feel effortless.</h2>
                <p>No bloated menus or messy setup — everything you actually use is right where you expect it.</p>
              </div>
              <a href="/features" class="button secondary" @click.prevent="navigate('/features')">
                <span>All features</span>
                <ArrowRight />
              </a>
            </div>

            <div class="bento-grid">
              <!-- Bento 1: Modpacks (Wide) -->
              <article
                class="bento-card bento-wide bento-tone-emerald spotlight-hover tilt-card"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
              >
                <div class="bento-copy">
                  <span class="bento-pill">ONE-CLICK MODPACKS</span>
                  <h3>Browse cover-art packs &amp; full detail pages.</h3>
                  <p>Search popular community catalogs, inspect full descriptions with images, galleries, and versions, and install in one click.</p>
                </div>
                <div class="bento-media">
                  <img src="/screenshots/modpacks.png" alt="Solara Modpack Browser" />
                </div>
              </article>

              <!-- Bento 2: Icon Studio -->
              <article
                class="bento-card bento-narrow bento-tone-purple spotlight-hover tilt-card"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
              >
                <div class="bento-copy">
                  <span class="bento-pill">ICON STUDIO</span>
                  <h3>30 3D icons &amp; custom badges.</h3>
                  <p>Style every world with 3D icons, text badges, custom colors, patterns, and frames.</p>
                </div>
                <div class="bento-media">
                  <img src="/screenshots/icon-studio.png" alt="Solara Icon Studio" />
                </div>
              </article>

              <!-- Bento 3: 3-Step Quick Start -->
              <article
                class="bento-card bento-narrow bento-tone-cobalt spotlight-hover tilt-card"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
              >
                <div class="bento-copy">
                  <span class="bento-pill">READY IN 60 SECONDS</span>
                  <h3>Three simple steps to play.</h3>
                </div>
                <div class="steps-list">
                  <div v-for="step in quickSteps" :key="step.num" class="step-row">
                    <span class="step-num">{{ step.num }}</span>
                    <div>
                      <strong>{{ step.title }}</strong>
                      <p>{{ step.copy }}</p>
                    </div>
                  </div>
                </div>
              </article>

              <!-- Bento 4: Interactive OLED vs White Theme Switcher (Wide) -->
              <article
                class="bento-card bento-wide bento-tone-sky spotlight-hover tilt-card"
                @mouseenter="showcasePaused = true"
                @mouseleave="showcasePaused = false"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
              >
                <div class="bento-copy bento-copy-row">
                  <div>
                    <span class="bento-pill">LIVE THEME PREVIEW</span>
                    <h3>True OLED Black or crisp White mode.</h3>
                    <p>Auto-switching preview — click to toggle between pure OLED Black and White mode.</p>
                  </div>
                  <div class="theme-switch-pills" role="group" aria-label="Preview launcher theme">
                    <button
                      type="button"
                      :class="['theme-pill', { active: previewTheme === 'oled' }]"
                      @click="previewTheme = 'oled'"
                    >
                      OLED Black
                    </button>
                    <button
                      type="button"
                      :class="['theme-pill', { active: previewTheme === 'white' }]"
                      @click="previewTheme = 'white'"
                    >
                      White Mode
                    </button>
                  </div>
                </div>
                <div class="bento-media">
                  <Transition name="shot-fade" mode="out-in">
                    <img
                      v-if="previewTheme === 'oled'"
                      key="oled-preview"
                      src="/screenshots/dashboard.png"
                      alt="Solara OLED Black theme"
                    />
                    <img
                      v-else
                      key="white-preview"
                      src="/screenshots/downloads-white.png"
                      alt="Solara White theme"
                    />
                  </Transition>
                </div>
              </article>
            </div>
          </section>

          <!-- Colorful 6-Card Feature Grid -->
          <section id="features" class="web-container web-section">
            <div class="section-header-row">
              <div class="section-header">
                <span class="eyebrow">EVERYTHING INCLUDED</span>
                <h2>Simple to use. Built to last.</h2>
                <p>No add-on paywalls, no ad clutter — every core launcher tool comes built in.</p>
              </div>
            </div>

            <div class="feature-grid">
              <article
                v-for="(f, i) in features"
                :key="f.title"
                :class="['feature', 'spotlight-hover', 'tilt-card', `feature--${f.tone}`]"
                :style="{ '--float-delay': `${i * 0.4}s` }"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
              >
                <div class="feature-top">
                  <span class="feature-icon-wrap float-icon">
                    <component :is="f.icon" class="feature-icon" />
                  </span>
                  <span class="feature-tag">0{{ i + 1 }} · {{ f.tag }}</span>
                </div>
                <h3>{{ f.title }}</h3>
                <p>{{ f.copy }}</p>
              </article>
            </div>
          </section>

          <!-- Security & VirusTotal Highlight -->
          <section id="security" class="web-container web-section">
            <div class="security-card spotlight-hover" @pointermove="onCardPointerMove">
              <div class="security-copy">
                <span class="eyebrow">TRANSPARENT &amp; VERIFIED</span>
                <h2>Direct from GitHub. Checked on VirusTotal.</h2>
                <p>
                  You sign in directly through Microsoft’s official page on your own PC. Every release is built cleanly on GitHub Actions, signed with Minisign, and publicly verifiable on VirusTotal.
                </p>
                <div class="security-cta-row">
                  <a
                    :href="virusTotalUrl"
                    target="_blank"
                    rel="noopener"
                    class="button secondary compact-btn vt-badge-btn"
                  >
                    <ShieldCheck />
                    <span>View Live VirusTotal Report</span>
                    <ArrowUpRight />
                  </a>
                  <a href="/security" class="text-link" @click.prevent="navigate('/security')">
                    How we keep you safe <ArrowRight />
                  </a>
                </div>
              </div>

              <div class="security-list">
                <div class="security-item">
                  <span class="security-check"><Check /></span>
                  <div>
                    <strong>Direct GitHub Releases Download</strong>
                    <span>Every solaralauncher.exe download streams straight from GitHub’s official CDN.</span>
                  </div>
                </div>
                <div class="security-item">
                  <span class="security-check"><Check /></span>
                  <div>
                    <strong>Public VirusTotal &amp; SHA-256 Verification</strong>
                    <span>Check the exact SHA-256 fingerprint and live VirusTotal scan anytime in one click.</span>
                  </div>
                </div>
                <div class="security-item">
                  <span class="security-check"><Check /></span>
                  <div>
                    <strong>Official Microsoft Login On Your PC</strong>
                    <span>Your password never leaves your computer or touches Solara servers.</span>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <!-- Luminous Download CTA Panel -->
          <section id="downloads" class="web-container web-section">
            <div class="download-panel animated-border-card spotlight-hover" @pointermove="onCardPointerMove">
              <div class="download-platform">
                <div class="download-icon-box float-icon">
                  <Monitor />
                </div>
                <div>
                  <div class="download-title-row">
                    <h3>Solara Launcher for Windows</h3>
                    <span class="status-badge">
                      <span class="dot pulse-dot" />
                      {{ latest ? latest.publicVersion : 'Preview' }}
                    </span>
                  </div>
                  <p class="download-meta">
                    <template v-if="latest">
                      Windows 10/11 (64-bit) · {{ (latest.size / 1048576).toFixed(1) }} MB · Direct from GitHub Releases
                    </template>
                    <template v-else>
                      Windows 10/11 (64-bit) · Direct from GitHub Releases
                    </template>
                  </p>
                </div>
              </div>

              <div class="download-cta-col">
                <div class="cta-btn-row">
                  <a
                    :href="githubDownloadUrl"
                    download="solaralauncher.exe"
                    rel="noopener"
                    class="button primary download-btn shimmer-btn cta-pulse"
                  >
                    <Download />
                    <span>Download solaralauncher.exe</span>
                  </a>
                  <a
                    :href="virusTotalUrl"
                    target="_blank"
                    rel="noopener"
                    class="button secondary vt-badge-btn"
                  >
                    <ShieldCheck />
                    <span>VirusTotal Scan</span>
                    <ArrowUpRight />
                  </a>
                </div>
                <small>One evergreen .exe · Streams directly from GitHub CDN</small>
              </div>
            </div>
          </section>

          <!-- Explore Pages Grid -->
          <section class="web-container web-section explore-section">
            <div class="explore-grid">
              <a
                href="/changelog"
                class="explore-card spotlight-hover tilt-card"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
                @click.prevent="navigate('/changelog')"
              >
                <span class="eyebrow">WHAT’S NEW</span>
                <h3>Updates &amp; Roadmap</h3>
                <p>See what we just shipped in {{ latest?.publicVersion || 'V-0.0.87' }} and what’s coming next.</p>
                <span class="text-link">See changelog <ArrowRight /></span>
              </a>
              <a
                href="/docs"
                class="explore-card spotlight-hover tilt-card"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
                @click.prevent="navigate('/docs')"
              >
                <span class="eyebrow">NEED HELP?</span>
                <h3>Easy Guides &amp; FAQ</h3>
                <p>Quick step-by-step tips for installing modpacks, making icons, and fixing lag.</p>
                <span class="text-link">Open guides <ArrowRight /></span>
              </a>
              <a
                href="/security"
                class="explore-card spotlight-hover tilt-card"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
                @click.prevent="navigate('/security')"
              >
                <span class="eyebrow">TRUST &amp; PRIVACY</span>
                <h3>Safety &amp; VirusTotal</h3>
                <p>Inspect our live VirusTotal scan, SHA-256 hashes, and open-source GPL-3.0 code.</p>
                <span class="text-link">Read safety info <ArrowRight /></span>
              </a>
            </div>
          </section>
        </div>

        <!-- ==================== FEATURES PAGE ==================== -->
        <div v-else-if="currentRoute === '/features'" key="features" class="page-stack">
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">FEATURES</span>
            <h1>Made for smooth,<br /><span class="hero-gradient-text">stress-free Minecraft.</span></h1>
            <p class="page-lead">
              Every screen in Solara is built to feel snappy, clean on OLED and bright screens, and easy for any player to jump right into.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="feature-grid">
              <article
                v-for="(f, i) in features"
                :key="f.title"
                :class="['feature', 'spotlight-hover', 'tilt-card', `feature--${f.tone}`]"
                :style="{ '--float-delay': `${i * 0.4}s` }"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
              >
                <div class="feature-top">
                  <span class="feature-icon-wrap float-icon">
                    <component :is="f.icon" class="feature-icon" />
                  </span>
                  <span class="feature-tag">0{{ i + 1 }} · {{ f.tag }}</span>
                </div>
                <h3>{{ f.title }}</h3>
                <p>{{ f.copy }}</p>
              </article>
            </div>
          </section>

          <section class="web-container sub-section">
            <div class="spotlight-list">
              <article class="spotlight-card">
                <div class="spotlight-copy">
                  <span class="eyebrow">MODPACK BROWSER</span>
                  <h2>Find and install modpacks in seconds.</h2>
                  <p>
                    Switch between top community catalogs right at the top. Open rich modpack detail pages with full image descriptions, versions, and screenshot galleries, then hit Install.
                  </p>
                </div>
                <div class="spotlight-media">
                  <img src="/screenshots/modpacks.png" alt="Modpack browser cover cards" loading="lazy" />
                </div>
              </article>

              <article class="spotlight-card reverse">
                <div class="spotlight-copy">
                  <span class="eyebrow">SOLARA ICON STUDIO</span>
                  <h2>3D icons, text badges &amp; cool frames.</h2>
                  <p>
                    Make every world stand out. Choose from 30 crisp 3D icons, type your own badge like SMP or 1.21, or upload your own picture — then mix custom colors, patterns, and frames.
                  </p>
                </div>
                <div class="spotlight-media">
                  <img src="/screenshots/icon-studio.png" alt="Solara Icon Studio" loading="lazy" />
                </div>
              </article>

              <article class="spotlight-card">
                <div class="spotlight-copy">
                  <span class="eyebrow">TWO CLEAN THEMES</span>
                  <h2>Pure OLED Black or crisp White mode.</h2>
                  <p>
                    Playing late at night or in a bright room? Switch smoothly between true pitch-black OLED and a crisp White theme right from Settings → Theme.
                  </p>
                </div>
                <div class="spotlight-media">
                  <img src="/screenshots/downloads-white.png" alt="Solara White Theme" loading="lazy" />
                </div>
              </article>
            </div>
          </section>

          <section class="web-container sub-section page-end">
            <div class="section-header">
              <span class="eyebrow">MOD LOADERS</span>
              <h2>All 5 Minecraft setups supported</h2>
              <p>Pick whichever loader you like — Solara matches the right Java version automatically.</p>
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
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">ONE EVERGREEN INSTALLER · DIRECT FROM GITHUB</span>
            <h1>Get <span class="hero-gradient-text">Solara Launcher.</span></h1>
            <p class="page-lead">
              You only ever need one file: <code>solaralauncher.exe</code>, downloaded straight from GitHub Releases. Install it once and it automatically updates itself to every new release.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="download-panel animated-border-card">
              <div class="download-platform">
                <div class="download-icon-box float-icon">
                  <Monitor />
                </div>
                <div>
                  <div class="download-title-row">
                    <h3>Solara Launcher for Windows (64-bit)</h3>
                    <span class="status-badge">
                      <span class="dot pulse-dot" />
                      {{ latest ? latest.publicVersion : 'Preview' }}
                    </span>
                  </div>
                  <p class="download-meta">
                    <template v-if="latest">
                      {{ (latest.size / 1048576).toFixed(2) }} MB · Released
                      {{ new Date(latest.publishedAt).toLocaleDateString() }} · Hosted on GitHub Releases CDN
                    </template>
                    <template v-else>
                      Loading latest release info…
                    </template>
                  </p>
                  <div v-if="latest?.sha256" class="download-hash-row">
                    <code>SHA-256: {{ latest.sha256 }}</code>
                    <button
                      type="button"
                      class="hash-copy-btn"
                      @click="copyText(latest.sha256, 'latest-sha')"
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
                    class="button primary download-btn shimmer-btn cta-pulse"
                  >
                    <Download />
                    <span>Download from GitHub</span>
                  </a>
                  <a
                    :href="virusTotalUrl"
                    target="_blank"
                    rel="noopener"
                    class="button secondary vt-badge-btn"
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
                  class="text-link"
                >
                  View latest release on GitHub <ArrowUpRight />
                </a>
              </div>
            </div>
          </section>

          <section v-if="latest?.sha256" class="web-container sub-section">
            <div class="info-card">
              <div class="info-card-head">
                <div>
                  <span class="eyebrow">OPTIONAL CHECK</span>
                  <h3>Verify your download with PowerShell or VirusTotal</h3>
                </div>
                <div class="cta-btn-row">
                  <a
                    :href="virusTotalUrl"
                    target="_blank"
                    rel="noopener"
                    class="button secondary compact-btn vt-badge-btn"
                  >
                    <ShieldCheck />
                    <span>Open VirusTotal Report</span>
                    <ArrowUpRight />
                  </a>
                  <button
                    type="button"
                    class="button secondary compact-btn"
                    @click="copyText(`(Get-FileHash .\\solaralauncher.exe -Algorithm SHA256).Hash -eq '${latest.sha256.toUpperCase()}'`, 'ps-cmd')"
                  >
                    <Check v-if="copied === 'ps-cmd'" />
                    <Copy v-else />
                    <span>{{ copied === 'ps-cmd' ? 'Copied!' : 'Copy PowerShell check' }}</span>
                  </button>
                </div>
              </div>
              <pre class="code-box"><code>(Get-FileHash .\solaralauncher.exe -Algorithm SHA256).Hash -eq '{{ latest.sha256.toUpperCase() }}'</code></pre>
              <p class="info-note">
                Paste this into Windows PowerShell in your Downloads folder. If it prints <code>True</code>, your file matches our official GitHub release 100%.
              </p>
            </div>
          </section>

          <section class="web-container sub-section">
            <div class="section-header">
              <span class="eyebrow">WHAT YOU NEED</span>
              <h2>PC Requirements</h2>
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
            <div class="archive-head">
              <div>
                <span class="eyebrow">RELEASE HISTORY</span>
                <h2>Version History ({{ releases.length }})</h2>
              </div>
              <input
                v-model="releaseSearch"
                type="search"
                class="archive-search"
                placeholder="Search version (e.g. V-0.0.87)..."
                aria-label="Search releases"
              />
            </div>

            <div class="archive-list">
              <article v-for="r in visibleReleases" :key="r.publicVersion" class="archive-row">
                <div class="archive-meta">
                  <div class="archive-title">
                    <strong>{{ r.publicVersion }}</strong>
                    <span class="archive-date">{{ new Date(r.publishedAt).toLocaleDateString() }}</span>
                    <span class="archive-size">{{ (r.size / 1048576).toFixed(2) }} MB</span>
                  </div>
                  <p v-if="r.notes" class="archive-notes">{{ r.notes.slice(0, 180) }}{{ r.notes.length > 180 ? '…' : '' }}</p>
                  <code v-if="r.sha256" class="archive-sha">SHA-256: {{ r.sha256 }}</code>
                </div>
                <div class="archive-actions">
                  <span v-if="isDownloadable(r)" class="status-badge">
                    <Check />
                    <span>Included in solaralauncher.exe</span>
                  </span>
                </div>
              </article>
            </div>

            <div v-if="filteredReleases.length > 8" class="archive-more">
              <button type="button" class="button secondary" @click="showAllReleases = !showAllReleases">
                {{ showAllReleases ? 'Show fewer versions' : `Show all ${filteredReleases.length} versions` }}
              </button>
            </div>
          </section>
        </div>

        <!-- ==================== CHANGELOG PAGE ==================== -->
        <div v-else-if="currentRoute === '/changelog'" key="changelog" class="page-stack">
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">WHAT’S NEW</span>
            <h1>Recent updates &amp;<br /><span class="hero-gradient-text">what’s coming next.</span></h1>
            <p class="page-lead">
              Every major Solara update in plain English — delivered automatically to your installed launcher.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="timeline">
              <article
                v-for="item in CHANGELOG_HIGHLIGHTS"
                :key="item.version"
                class="timeline-card spotlight-hover"
                @pointermove="onCardPointerMove"
              >
                <div class="timeline-head">
                  <span class="status-badge">
                    <span class="dot pulse-dot" />
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
            <div class="section-header">
              <span class="eyebrow">COMING SOON</span>
              <h2>Public Roadmap</h2>
              <p>What we are working on right now and what is planned next.</p>
            </div>
            <div class="roadmap-grid">
              <article
                v-for="r in ROADMAP_ITEMS"
                :key="r.title"
                class="roadmap-card spotlight-hover tilt-card"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
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
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">SAFETY &amp; VIRUSTOTAL TRANSPARENCY</span>
            <h1>Built to be safe.<br /><span class="hero-gradient-text">Open for anyone to check.</span></h1>
            <p class="page-lead">
              Your Minecraft login stays on your PC, every download comes straight from GitHub Releases, and you can inspect our live VirusTotal scan anytime.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="pillar-grid">
              <article
                v-for="p in SECURITY_PILLARS"
                :key="p.title"
                class="pillar-card spotlight-hover tilt-card"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
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
                  <span class="eyebrow">PUBLIC VERIFICATION</span>
                  <h3>Live VirusTotal Report &amp; Open-Source Status</h3>
                </div>
                <a
                  :href="virusTotalUrl"
                  target="_blank"
                  rel="noopener"
                  class="button secondary compact-btn vt-badge-btn"
                >
                  <ShieldCheck />
                  <span>Inspect on VirusTotal</span>
                  <ArrowUpRight />
                </a>
              </div>
              <ul class="bullet-list">
                <li><strong>Direct GitHub Hosting:</strong> All installer and auto-update downloads come directly from GitHub Releases CDN — never modified in transit.</li>
                <li><strong>Public VirusTotal Scan:</strong> Every release SHA-256 (currently <code>{{ latest?.sha256 || DEFAULT_VT_SHA256 }}</code>) can be checked directly on VirusTotal.</li>
                <li><strong>Testing Preview:</strong> Solara Launcher is in active Windows testing preview. Official Microsoft accounts only — no cracked logins.</li>
                <li><strong>Open Source (GPL-3.0):</strong> The full launcher source and third-party license notices are included in every release and on GitHub.</li>
              </ul>
            </div>
          </section>
        </div>

        <!-- ==================== DOCS & FAQ PAGE ==================== -->
        <div v-else key="docs" class="page-stack">
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">HELP &amp; GUIDES</span>
            <h1>Easy setup &amp;<br /><span class="hero-gradient-text">quick answers.</span></h1>
            <p class="page-lead">
              New to Solara? Follow these quick steps to get playing in under two minutes.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="guide-grid">
              <article
                v-for="g in GUIDES"
                :key="g.id"
                class="guide-card spotlight-hover tilt-card"
                @pointermove="onCardPointerMove"
                @pointerleave="onCardPointerLeave"
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
            <div class="section-header">
              <span class="eyebrow">FAQ</span>
              <h2>Common questions</h2>
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
                <Transition name="faq-slide">
                  <div v-if="faq === i" class="faq-answer">
                    <p>{{ answer }}</p>
                  </div>
                </Transition>
              </div>
            </div>
          </section>
        </div>
      </Transition>
    </main>

    <!-- Footer -->
    <footer class="web-footer">
      <div class="web-container">
        <div class="footer-top">
          <div class="footer-brand-col">
            <a class="web-brand" href="/" @click.prevent="navigate('/')">
              <img src="/logo.svg" alt="Solara" />
              <span>Solara Launcher</span>
            </a>
            <p>
              Fast, ad-free OLED Minecraft launcher for Windows with one-click modpacks, Icon Studio, and evergreen GitHub auto-updates.
            </p>
          </div>

          <div class="footer-links-col">
            <h4>Pages</h4>
            <a href="/" @click.prevent="navigate('/')">Home</a>
            <a href="/features" @click.prevent="navigate('/features')">Features</a>
            <a href="/downloads" @click.prevent="navigate('/downloads')">Downloads</a>
            <a href="/changelog" @click.prevent="navigate('/changelog')">What’s New</a>
          </div>

          <div class="footer-links-col">
            <h4>Verification &amp; Links</h4>
            <a href="/security" @click.prevent="navigate('/security')">Safety &amp; Privacy</a>
            <a :href="virusTotalUrl" target="_blank" rel="noopener">VirusTotal Scan <ArrowUpRight /></a>
            <a href="/docs" @click.prevent="navigate('/docs')">Help &amp; FAQ</a>
            <a v-if="links.github" :href="links.github" target="_blank" rel="noopener">GitHub <ArrowUpRight /></a>
          </div>
        </div>

        <div class="footer-bottom">
          <div>
            <p>© {{ new Date().getFullYear() }} Solara Launcher · Open source (GPL-3.0-only).</p>
            <p class="legal-note">
              Independent Minecraft Java launcher. Not an official Minecraft product; not approved by or associated with Mojang or Microsoft.
            </p>
          </div>
          <span class="feed-status">
            <span class="dot pulse-dot" />
            {{ snapshot.status === 'ready' ? 'GitHub release feed: live' : 'GitHub release feed: synced' }}
          </span>
        </div>
      </div>
    </footer>
  </div>
</template>
