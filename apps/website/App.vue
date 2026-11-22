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
const previewTheme = ref<'oled' | 'white'>('oled');
let timer: ReturnType<typeof setInterval> | null = null;

function onCardPointerMove(event: PointerEvent) {
  const el = event.currentTarget as HTMLElement | null;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  el.style.setProperty('--mx', `${event.clientX - rect.left}px`);
  el.style.setProperty('--my', `${event.clientY - rect.top}px`);
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
    title: 'Automatic Updates',
    copy: 'Install once and relax. Signed updates download and apply safely whenever no game is running.',
    tag: 'UPDATES',
  },
];

const quickSteps = [
  {
    num: '01',
    title: 'Download once',
    copy: 'Grab solaralauncher.exe (~16 MB) and run the quick Windows installer.',
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
    'Do I need to create a new Solara account?',
    'Nope! You only need your official Microsoft Minecraft: Java Edition account to play. Anyone can grab the Windows installer for free.',
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
    'How do updates work?',
    'You only download solaralauncher.exe once. After that, Solara checks for signed updates in the background and installs them safely when no game is running.',
  ],
  [
    'Is Solara really free with no ads?',
    'Yes! Zero ad banners, zero popups, and the full open-source code is public on GitHub.',
  ],
] as const;

onMounted(() => {
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
    <!-- Layered OLED Aurora & Subtle Grid -->
    <div class="ambient-bg" aria-hidden="true">
      <span class="ambient-orb orb-1" />
      <span class="ambient-orb orb-2" />
      <span class="ambient-orb orb-3" />
      <span class="ambient-grid" />
    </div>

    <a class="skip-link" href="#main">Skip to content</a>

    <!-- Sticky Frosted Navbar -->
    <header class="web-header">
      <div class="web-header-inner">
        <a class="web-brand" href="/" @click.prevent="navigate('/')">
          <img src="/logo.svg" alt="Solara" />
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
            v-if="latest"
            :href="latest.url"
            download="solaralauncher.exe"
            class="header-download shimmer-btn"
          >
            <Download />
            <span>Download</span>
          </a>
          <a v-else href="/downloads" class="header-download shimmer-btn" @click.prevent="navigate('/downloads')">
            <Download />
            <span>Get Solara</span>
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
              <strong>{{ latest ? `${latest.publicVersion} is live` : 'Windows x64 Preview' }}</strong>
              <span class="hero-pill-sep">·</span>
              <span>Modpacks, Icon Studio &amp; OLED Themes</span>
              <ChevronRight class="pill-arrow" />
            </a>

            <h1 class="hero-anim-2">
              Play Minecraft,<br />
              <span class="hero-gradient-text">the fast &amp; clean way.</span>
            </h1>

            <p class="hero-description hero-anim-3">
              Keep your worlds tidy, install modpacks in one click, and customize 3D icons &amp; skins — all inside a sleek, ad-free Windows launcher.
            </p>

            <div class="hero-actions hero-anim-4">
              <a
                v-if="latest"
                :href="latest.url"
                download="solaralauncher.exe"
                class="button primary hero-cta shimmer-btn"
              >
                <Download />
                <span>Download for Windows</span>
                <span class="cta-meta">{{ latest.publicVersion }}</span>
              </a>
              <a v-else href="/downloads" class="button primary hero-cta shimmer-btn" @click.prevent="navigate('/downloads')">
                <Download />
                <span>Download for Windows</span>
                <ArrowRight />
              </a>

              <a
                href="/features"
                class="button secondary hero-secondary"
                @click.prevent="navigate('/features')"
              >
                <span>See all features</span>
                <ChevronRight />
              </a>

              <a
                v-if="links.github"
                :href="links.github"
                target="_blank"
                rel="noopener"
                class="button secondary hero-secondary"
              >
                <GitBranch />
                <span>GitHub</span>
                <ArrowUpRight />
              </a>
            </div>

            <div class="hero-trust hero-anim-5">
              <span><Check /> Windows 10 &amp; 11</span>
              <span><ShieldCheck /> Official Microsoft Login</span>
              <span><Check /> Safe Auto-Updates</span>
              <span><Check /> 100% Free &amp; Ad-Free</span>
            </div>

            <!-- Interactive 5-View App Showcase -->
            <div id="showcase" class="showcase-wrap hero-anim-6">
              <div class="showcase-glow" aria-hidden="true" />
              <div class="showcase-shell">
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
                      :class="['showcase-tab', { active: activeShot === tab.id }]"
                      @click="activeShot = tab.id"
                    >
                      <span>{{ tab.label }}</span>
                      <small>{{ tab.badge }}</small>
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
                  <a href="/downloads" class="text-link" @click.prevent="navigate('/downloads')">
                    Get solaralauncher.exe <ChevronRight />
                  </a>
                </div>
              </div>
            </div>
          </section>

          <!-- Key Stats Strip -->
          <section class="web-container stats-strip">
            <div class="stat-box spotlight-hover" @pointermove="onCardPointerMove">
              <strong>{{ latest ? `${(latest.size / 1048576).toFixed(1)} MB` : '~16 MB' }}</strong>
              <span>Tiny, fast Windows installer</span>
            </div>
            <div class="stat-box spotlight-hover" @pointermove="onCardPointerMove">
              <strong>5 Loaders</strong>
              <span>Vanilla, Fabric, NeoForge, Forge &amp; Quilt</span>
            </div>
            <div class="stat-box spotlight-hover" @pointermove="onCardPointerMove">
              <strong>30+ 3D Icons</strong>
              <span>Plus custom text badges &amp; frames</span>
            </div>
            <div class="stat-box spotlight-hover" @pointermove="onCardPointerMove">
              <strong>Zero Ads</strong>
              <span>Clean &amp; open source forever</span>
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
              <article class="bento-card bento-wide bento-tone-emerald spotlight-hover" @pointermove="onCardPointerMove">
                <div class="bento-copy">
                  <span class="bento-pill">ONE-CLICK MODPACKS</span>
                  <h3>Browse cover-art packs &amp; install instantly.</h3>
                  <p>Search popular community catalogs, filter by loader or Minecraft version, view screenshots, and install in one click.</p>
                </div>
                <div class="bento-media">
                  <img src="/screenshots/modpacks.png" alt="Solara Modpack Browser" />
                </div>
              </article>

              <!-- Bento 2: Icon Studio -->
              <article class="bento-card bento-narrow bento-tone-purple spotlight-hover" @pointermove="onCardPointerMove">
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
              <article class="bento-card bento-narrow bento-tone-cobalt spotlight-hover" @pointermove="onCardPointerMove">
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
              <article class="bento-card bento-wide bento-tone-sky spotlight-hover" @pointermove="onCardPointerMove">
                <div class="bento-copy bento-copy-row">
                  <div>
                    <span class="bento-pill">DUAL THEMES</span>
                    <h3>True OLED Black or crisp White mode.</h3>
                    <p>Click below to preview both themes — switch anytime inside Settings.</p>
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
                :class="['feature', 'spotlight-hover', `feature--${f.tone}`]"
                @pointermove="onCardPointerMove"
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

          <!-- Security Highlight -->
          <section id="security" class="web-container web-section">
            <div class="security-card spotlight-hover" @pointermove="onCardPointerMove">
              <div class="security-copy">
                <span class="eyebrow">BUILT FOR SAFETY</span>
                <h2>Your Microsoft account stays on your PC.</h2>
                <p>
                  You sign in directly through Microsoft’s official page on your own computer. Solara never sees your password, and every update is checked with a digital signature before installing.
                </p>
                <a href="/security" class="text-link" @click.prevent="navigate('/security')">
                  See how we keep you safe <ArrowRight />
                </a>
              </div>

              <div class="security-list">
                <div class="security-item">
                  <span class="security-check"><Check /></span>
                  <div>
                    <strong>Official Microsoft Login</strong>
                    <span>Works with real Minecraft: Java Edition accounts only.</span>
                  </div>
                </div>
                <div class="security-item">
                  <span class="security-check"><Check /></span>
                  <div>
                    <strong>Saved Safely on Windows</strong>
                    <span>Your login never leaves your computer or touches another server.</span>
                  </div>
                </div>
                <div class="security-item">
                  <span class="security-check"><Check /></span>
                  <div>
                    <strong>Verified Updates</strong>
                    <span>Every installer and update is checked before it ever runs.</span>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <!-- Luminous Download CTA Panel -->
          <section id="downloads" class="web-container web-section">
            <div class="download-panel spotlight-hover" @pointermove="onCardPointerMove">
              <div class="download-platform">
                <div class="download-icon-box">
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
                      Windows 10/11 (64-bit) · {{ (latest.size / 1048576).toFixed(1) }} MB · Released
                      {{ new Date(latest.publishedAt).toLocaleDateString() }}
                    </template>
                    <template v-else>
                      Windows 10/11 (64-bit) · Checking latest release…
                    </template>
                  </p>
                </div>
              </div>

              <div class="download-cta-col">
                <div class="cta-btn-row">
                  <a
                    v-if="latest"
                    :href="latest.url"
                    download="solaralauncher.exe"
                    class="button primary download-btn shimmer-btn"
                  >
                    <Download />
                    <span>Download solaralauncher.exe</span>
                  </a>
                  <a
                    href="/downloads"
                    class="button secondary"
                    @click.prevent="navigate('/downloads')"
                  >
                    <span>All versions &amp; hashes</span>
                  </a>
                </div>
                <small>Signed updates · Auto-updates built in</small>
              </div>
            </div>
          </section>

          <!-- Explore Pages Grid -->
          <section class="web-container web-section explore-section">
            <div class="explore-grid">
              <a
                href="/changelog"
                class="explore-card spotlight-hover"
                @pointermove="onCardPointerMove"
                @click.prevent="navigate('/changelog')"
              >
                <span class="eyebrow">WHAT’S NEW</span>
                <h3>Updates &amp; Roadmap</h3>
                <p>See what we just shipped in {{ latest?.publicVersion || 'V-0.0.83' }} and what’s coming next.</p>
                <span class="text-link">See changelog <ArrowRight /></span>
              </a>
              <a
                href="/docs"
                class="explore-card spotlight-hover"
                @pointermove="onCardPointerMove"
                @click.prevent="navigate('/docs')"
              >
                <span class="eyebrow">NEED HELP?</span>
                <h3>Easy Guides &amp; FAQ</h3>
                <p>Quick step-by-step tips for installing modpacks, making icons, and fixing lag.</p>
                <span class="text-link">Open guides <ArrowRight /></span>
              </a>
              <a
                href="/security"
                class="explore-card spotlight-hover"
                @pointermove="onCardPointerMove"
                @click.prevent="navigate('/security')"
              >
                <span class="eyebrow">TRUST &amp; PRIVACY</span>
                <h3>Safety &amp; Open Source</h3>
                <p>Learn how Microsoft login works on your PC and how we check every release.</p>
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
                :class="['feature', 'spotlight-hover', `feature--${f.tone}`]"
                @pointermove="onCardPointerMove"
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

          <section class="web-container sub-section">
            <div class="spotlight-list">
              <article class="spotlight-card">
                <div class="spotlight-copy">
                  <span class="eyebrow">MODPACK BROWSER</span>
                  <h2>Find and install modpacks in seconds.</h2>
                  <p>
                    Switch between top community catalogs right at the top. Filter by Fabric, Forge, NeoForge, or Quilt, pick your Minecraft version, and hit Install — Solara handles the setup for you.
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
            <span class="eyebrow">WINDOWS INSTALLER</span>
            <h1>Get <span class="hero-gradient-text">Solara Launcher.</span></h1>
            <p class="page-lead">
              One clean file: <code>solaralauncher.exe</code>. Install it once, and new updates arrive automatically in the background.
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
                      <span class="dot pulse-dot" />
                      {{ latest ? latest.publicVersion : 'Preview' }}
                    </span>
                  </div>
                  <p class="download-meta">
                    <template v-if="latest">
                      {{ (latest.size / 1048576).toFixed(2) }} MB · Released
                      {{ new Date(latest.publishedAt).toLocaleDateString() }} · Testing Preview
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
                <a
                  v-if="latest"
                  :href="latest.url"
                  download="solaralauncher.exe"
                  class="button primary download-btn shimmer-btn"
                >
                  <Download />
                  <span>Download solaralauncher.exe</span>
                </a>
                <a
                  v-if="links.github"
                  :href="`${links.github}/releases`"
                  target="_blank"
                  rel="noopener"
                  class="text-link"
                >
                  View on GitHub Releases <ArrowUpRight />
                </a>
              </div>
            </div>
          </section>

          <section v-if="latest?.sha256" class="web-container sub-section">
            <div class="info-card">
              <div class="info-card-head">
                <div>
                  <span class="eyebrow">OPTIONAL CHECK</span>
                  <h3>Double-check your download on Windows</h3>
                </div>
                <button
                  type="button"
                  class="button secondary compact-btn"
                  @click="copyText(`(Get-FileHash .\\solaralauncher.exe -Algorithm SHA256).Hash -eq '${latest.sha256.toUpperCase()}'`, 'ps-cmd')"
                >
                  <Check v-if="copied === 'ps-cmd'" />
                  <Copy v-else />
                  <span>{{ copied === 'ps-cmd' ? 'Copied!' : 'Copy command' }}</span>
                </button>
              </div>
              <pre class="code-box"><code>(Get-FileHash .\solaralauncher.exe -Algorithm SHA256).Hash -eq '{{ latest.sha256.toUpperCase() }}'</code></pre>
              <p class="info-note">
                Paste this into Windows PowerShell in your Downloads folder. If it prints <code>True</code>, your file matches our official release 100%.
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
                <span class="eyebrow">PAST VERSIONS</span>
                <h2>Release Archive ({{ releases.length }})</h2>
              </div>
              <input
                v-model="releaseSearch"
                type="search"
                class="archive-search"
                placeholder="Search version (e.g. V-0.0.83)..."
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
                  <a
                    v-if="isDownloadable(r)"
                    :href="r.url"
                    download="solaralauncher.exe"
                    class="button secondary compact-btn"
                  >
                    <Download />
                    <span>Download</span>
                  </a>
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
              Every major Solara update in plain English — so you always know what got better.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="timeline">
              <article v-for="item in CHANGELOG_HIGHLIGHTS" :key="item.version" class="timeline-card spotlight-hover" @pointermove="onCardPointerMove">
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
              <article v-for="r in ROADMAP_ITEMS" :key="r.title" class="roadmap-card spotlight-hover" @pointermove="onCardPointerMove">
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
            <span class="eyebrow">SAFETY &amp; PRIVACY</span>
            <h1>Built to be safe.<br /><span class="hero-gradient-text">Open for anyone to check.</span></h1>
            <p class="page-lead">
              Your Minecraft login stays on your PC, every download is verified, and the full code is open on GitHub.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="pillar-grid">
              <article v-for="p in SECURITY_PILLARS" :key="p.title" class="pillar-card spotlight-hover" @pointermove="onCardPointerMove">
                <span class="roadmap-phase">{{ p.badge }}</span>
                <h3>{{ p.title }}</h3>
                <p>{{ p.copy }}</p>
              </article>
            </div>
          </section>

          <section class="web-container sub-section page-end">
            <div class="info-card">
              <span class="eyebrow">HONEST STATUS &amp; OPEN SOURCE</span>
              <h3>Current Testing Preview &amp; License</h3>
              <ul class="bullet-list">
                <li><strong>Testing Preview:</strong> Solara Launcher is currently in active Windows testing preview. We share real progress openly and never hide known limits.</li>
                <li><strong>Official Accounts Only:</strong> Solara only works with real Microsoft Minecraft: Java Edition accounts — no cracked logins.</li>
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
              <article v-for="g in GUIDES" :key="g.id" class="guide-card spotlight-hover" @pointermove="onCardPointerMove">
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
              Fast, ad-free OLED Minecraft launcher for Windows with one-click modpacks, Icon Studio, and automatic updates.
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
            <h4>Help &amp; Links</h4>
            <a href="/security" @click.prevent="navigate('/security')">Safety &amp; Privacy</a>
            <a href="/docs" @click.prevent="navigate('/docs')">Help &amp; FAQ</a>
            <a v-if="links.github" :href="links.github" target="_blank" rel="noopener">GitHub <ArrowUpRight /></a>
            <a v-if="links.home" :href="links.home" target="_blank" rel="noopener">Solara-Inc <ArrowUpRight /></a>
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
            <span class="dot" />
            {{ snapshot.status === 'ready' ? 'Release feed: connected' : 'Release feed: checking' }}
          </span>
        </div>
      </div>
    </footer>
  </div>
</template>
