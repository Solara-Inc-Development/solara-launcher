<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue';
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
let timer: ReturnType<typeof setInterval> | null = null;
let revealObserver: IntersectionObserver | null = null;

function setupScrollReveals() {
  revealObserver?.disconnect();
  const els = document.querySelectorAll<HTMLElement>('.reveal');
  if (!els.length) return;

  if (
    preferences.value.motion ||
    window.matchMedia('(prefers-reduced-motion: reduce)').matches ||
    typeof IntersectionObserver === 'undefined'
  ) {
    els.forEach((el) => el.classList.add('is-visible'));
    return;
  }

  revealObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          entry.target.classList.add('is-visible');
          revealObserver?.unobserve(entry.target);
        }
      }
    },
    { threshold: 0.08, rootMargin: '0px 0px -30px 0px' },
  );

  els.forEach((el) => {
    el.classList.remove('is-visible');
    revealObserver?.observe(el);
  });
}

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
    copy: 'Search thousands of community modpacks, pick your Minecraft version, and install in one click.',
    tag: 'MODPACKS',
  },
  {
    icon: Sparkles,
    tone: 'purple',
    title: 'Icon Studio & 3D Skins',
    copy: 'Give every world a custom icon with 30 3D icons, text badges, and frames — plus a live 3D skin viewer.',
    tag: 'STYLE',
  },
  {
    icon: Monitor,
    tone: 'amber',
    title: 'Auto Java Setup',
    copy: 'No more hunting for Java downloads. Solara picks and sets up the right Java version automatically.',
    tag: 'AUTO JAVA',
  },
  {
    icon: ShieldCheck,
    tone: 'sky',
    title: 'Safe Microsoft Login',
    copy: 'Sign in directly with your official Microsoft account on your own PC. We never see your password.',
    tag: 'PRIVATE',
  },
  {
    icon: Download,
    tone: 'rose',
    title: 'Automatic Updates',
    copy: 'Install once and relax. Solara checks digital signatures and updates itself safely when you are not playing.',
    tag: 'UPDATES',
  },
];

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

watch(currentRoute, () => {
  void nextTick(() => {
    setTimeout(setupScrollReveals, 80);
  });
});

onMounted(() => {
  window.addEventListener('popstate', onPopState);
  void refreshPublic();
  timer = setInterval(() => void refreshPublic(), 60000);
  void nextTick(setupScrollReveals);
});

onUnmounted(() => {
  window.removeEventListener('popstate', onPopState);
  revealObserver?.disconnect();
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
    <!-- Subtle Ambient Aurora Background -->
    <div class="ambient-bg" aria-hidden="true">
      <span class="ambient-orb orb-1" />
      <span class="ambient-orb orb-2" />
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
            class="header-download"
          >
            <Download />
            <span>Download</span>
          </a>
          <a v-else href="/downloads" class="header-download" @click.prevent="navigate('/downloads')">
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
        <div v-if="currentRoute === '/'" key="home">
          <section class="web-hero web-container">
            <div class="hero-pill hero-anim-1">
              <span class="dot pulse-dot" />
              <span>{{ latest ? `New ${latest.publicVersion} is out` : 'Windows x64 Preview' }}</span>
              <span class="hero-pill-sep">·</span>
              <span>OLED Black &amp; White Themes</span>
            </div>

            <h1 class="hero-anim-2">
              Play Minecraft,<br />
              <span>the fast &amp; clean way.</span>
            </h1>

            <p class="hero-description hero-anim-3">
              Keep your worlds tidy, install modpacks in one click, and design custom icons &amp; 3D skins — in a lightweight Windows launcher with zero ads.
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
            <div id="showcase" class="showcase-shell hero-anim-6">
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
          </section>

          <!-- Key Stats Strip -->
          <section class="web-container stats-strip">
            <div class="stat-box reveal" style="--delay: 0ms">
              <strong>{{ latest ? `${(latest.size / 1048576).toFixed(1)} MB` : '~16 MB' }}</strong>
              <span>Tiny, fast Windows installer</span>
            </div>
            <div class="stat-box reveal" style="--delay: 70ms">
              <strong>5 Loaders</strong>
              <span>Vanilla, Fabric, NeoForge, Forge &amp; Quilt</span>
            </div>
            <div class="stat-box reveal" style="--delay: 140ms">
              <strong>30+ 3D Icons</strong>
              <span>Plus custom text badges &amp; frames</span>
            </div>
            <div class="stat-box reveal" style="--delay: 210ms">
              <strong>Zero Ads</strong>
              <span>Clean &amp; open source forever</span>
            </div>
          </section>

          <!-- Features Bento Grid -->
          <section id="features" class="web-container web-section">
            <div class="section-header-row reveal">
              <div class="section-header">
                <span class="eyebrow">WHY PLAYERS LOVE SOLARA</span>
                <h2>Simple to use.<br />Ready in seconds.</h2>
                <p>
                  Everything you need to jump into Minecraft fast — without menus getting in your way.
                </p>
              </div>
              <a href="/features" class="button secondary" @click.prevent="navigate('/features')">
                <span>Explore all features</span>
                <ArrowRight />
              </a>
            </div>

            <div class="feature-grid">
              <article
                v-for="(f, i) in features"
                :key="f.title"
                :class="['feature', 'reveal', 'spotlight-hover', `feature--${f.tone}`]"
                :style="{ '--delay': `${i * 60}ms` }"
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
            <div class="security-card reveal">
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

          <!-- Quick Download Panel on Home -->
          <section id="downloads" class="web-container web-section">
            <div class="download-panel reveal">
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
                class="explore-card reveal spotlight-hover"
                style="--delay: 0ms"
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
                class="explore-card reveal spotlight-hover"
                style="--delay: 80ms"
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
                class="explore-card reveal spotlight-hover"
                style="--delay: 160ms"
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
        <div v-else-if="currentRoute === '/features'" key="features">
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">FEATURES</span>
            <h1>Made for smooth,<br /><span>stress-free Minecraft.</span></h1>
            <p class="page-lead">
              Every screen in Solara is built to feel snappy, clean on OLED and bright screens, and easy for any player to jump right into.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="feature-grid">
              <article
                v-for="(f, i) in features"
                :key="f.title"
                :class="['feature', 'reveal', 'spotlight-hover', `feature--${f.tone}`]"
                :style="{ '--delay': `${i * 55}ms` }"
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

          <!-- Visual Deep-Dive Cards -->
          <section class="web-container sub-section">
            <div class="spotlight-list">
              <article class="spotlight-card reveal">
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

              <article class="spotlight-card reverse reveal">
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

              <article class="spotlight-card reveal">
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

          <!-- Loader Matrix Table -->
          <section class="web-container sub-section page-end reveal">
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
        <div v-else-if="currentRoute === '/downloads'" key="downloads">
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">WINDOWS INSTALLER</span>
            <h1>Get <span>Solara Launcher.</span></h1>
            <p class="page-lead">
              One clean file: <code>solaralauncher.exe</code>. Install it once, and new updates arrive automatically in the background.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="download-panel reveal">
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
                <button v-else disabled class="primary download-btn">
                  <Download />
                  <span>Installer unavailable</span>
                </button>
                <small>Signed via Minisign · Hosted on GitHub Releases</small>
              </div>
            </div>

            <!-- Verify Checksum Box -->
            <div class="verify-box reveal">
              <div class="verify-head">
                <strong>Want to double-check the file on Windows?</strong>
                <button
                  type="button"
                  class="hash-copy-btn"
                  @click="copyText('Get-FileHash .\\solaralauncher.exe -Algorithm SHA256', 'ps-cmd')"
                >
                  <Check v-if="copied === 'ps-cmd'" />
                  <Copy v-else />
                  <span>{{ copied === 'ps-cmd' ? 'Copied!' : 'Copy command' }}</span>
                </button>
              </div>
              <pre><code>Get-FileHash .\solaralauncher.exe -Algorithm SHA256</code></pre>
              <p>Run this in PowerShell and make sure the code matches the SHA-256 hash shown above.</p>
            </div>
          </section>

          <!-- System Requirements -->
          <section class="web-container sub-section reveal">
            <div class="section-header">
              <span class="eyebrow">WHAT YOUR PC NEEDS</span>
              <h2>System Requirements</h2>
            </div>
            <div class="data-table-wrap">
              <table class="data-table">
                <thead>
                  <tr>
                    <th>Part</th>
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

          <!-- Full Searchable Release Archive -->
          <section class="web-container sub-section page-end reveal">
            <div class="archive">
              <div class="archive-head">
                <div>
                  <h3>Older Versions &amp; Archive</h3>
                  <p>Every previous release stays saved with its own checksum.</p>
                </div>
                <div class="archive-controls">
                  <input
                    v-model="releaseSearch"
                    type="search"
                    placeholder="Search version or note..."
                    class="archive-search"
                  />
                  <span class="archive-count">{{ filteredReleases.length }} builds</span>
                </div>
              </div>

              <div v-if="!filteredReleases.length" class="archive-empty">
                <GitBranch />
                <p>{{ releases.length ? 'No builds matched your search.' : 'Loading release list…' }}</p>
              </div>

              <div v-else class="release-list">
                <article v-for="r in visibleReleases" :key="r.buildId" class="release-row">
                  <div class="release-info">
                    <div class="release-title-row">
                      <h4>{{ r.publicVersion }}</h4>
                      <span class="release-state">{{ r.state }}</span>
                      <span class="release-date">
                        {{ r.publishedAt ? new Date(r.publishedAt).toLocaleDateString() : 'Unpublished' }}
                        · {{ (r.size / 1048576).toFixed(1) }} MB
                      </span>
                    </div>
                    <p class="release-notes">{{ r.notes }}</p>
                    <code class="release-sha">{{ r.sha256 }}</code>
                  </div>

                  <div class="release-actions">
                    <button
                      type="button"
                      class="icon-action-btn"
                      :disabled="!r.sha256"
                      :title="copied === r.buildId ? 'Copied!' : 'Copy SHA-256'"
                      @click="copyText(r.sha256, r.buildId)"
                    >
                      <Check v-if="copied === r.buildId" />
                      <Copy v-else />
                    </button>
                    <a
                      v-if="isDownloadable(r)"
                      :href="r.url"
                      download="solaralauncher.exe"
                      class="button release-dl-btn"
                    >
                      <Download />
                      <span>{{ r.publicVersion }}</span>
                    </a>
                  </div>
                </article>
              </div>

              <div v-if="filteredReleases.length > 8" class="archive-more">
                <button
                  type="button"
                  class="button secondary"
                  @click="showAllReleases = !showAllReleases"
                >
                  {{ showAllReleases ? 'Show fewer builds' : `Show all ${filteredReleases.length} builds` }}
                </button>
              </div>
            </div>
          </section>
        </div>

        <!-- ==================== CHANGELOG & ROADMAP PAGE ==================== -->
        <div v-else-if="currentRoute === '/changelog'" key="changelog">
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">WHAT’S NEW</span>
            <h1>Recent updates &amp;<br /><span>what’s coming next.</span></h1>
            <p class="page-lead">
              Follow every new feature, design refresh, and fix we ship to Solara Launcher.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="timeline-list">
              <article
                v-for="(entry, idx) in CHANGELOG_HIGHLIGHTS"
                :key="entry.version"
                class="timeline-card reveal"
                :style="{ '--delay': `${idx * 60}ms` }"
              >
                <div class="timeline-meta">
                  <span class="status-badge">
                    <span class="dot" />
                    {{ entry.version }}
                  </span>
                  <span class="timeline-tag">{{ entry.tag }}</span>
                  <span class="timeline-date">{{ entry.date }}</span>
                </div>
                <h2>{{ entry.title }}</h2>
                <ul>
                  <li v-for="item in entry.highlights" :key="item">
                    <Check />
                    <span>{{ item }}</span>
                  </li>
                </ul>
              </article>
            </div>
          </section>

          <section class="web-container sub-section page-end">
            <div class="section-header reveal">
              <span class="eyebrow">ROADMAP</span>
              <h2>What we’re building next</h2>
              <p>Honest plans we are working on right now.</p>
            </div>
            <div class="roadmap-grid">
              <article
                v-for="(item, idx) in ROADMAP_ITEMS"
                :key="item.title"
                class="roadmap-card reveal spotlight-hover"
                :style="{ '--delay': `${idx * 70}ms` }"
                @pointermove="onCardPointerMove"
              >
                <span class="roadmap-phase">{{ item.phase }}</span>
                <h3>{{ item.title }}</h3>
                <p>{{ item.copy }}</p>
              </article>
            </div>
          </section>
        </div>

        <!-- ==================== SECURITY PAGE ==================== -->
        <div v-else-if="currentRoute === '/security'" key="security">
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">SAFETY &amp; PRIVACY</span>
            <h1>Built to be safe.<br /><span>Open for anyone to check.</span></h1>
            <p class="page-lead">
              No shady logins, no ads, and no hidden tricks. Your account stays on your PC and our code is open on GitHub.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="pillar-grid">
              <article
                v-for="(p, idx) in SECURITY_PILLARS"
                :key="p.title"
                class="pillar-card reveal spotlight-hover"
                :style="{ '--delay': `${idx * 70}ms` }"
                @pointermove="onCardPointerMove"
              >
                <div class="pillar-top">
                  <span class="pillar-icon"><ShieldCheck /></span>
                  <span class="pillar-badge">{{ p.badge }}</span>
                </div>
                <h3>{{ p.title }}</h3>
                <p>{{ p.copy }}</p>
              </article>
            </div>
          </section>

          <section class="web-container sub-section page-end">
            <div class="security-card reveal">
              <div class="security-copy">
                <span class="eyebrow">HONEST NOTE</span>
                <h2>Testing Preview Status</h2>
                <p>
                  Every Solara release is signed with our updater key so automatic updates can verify the file before running. Windows Authenticode publisher signing is separate and not claimed yet while we are in Testing Preview.
                </p>
                <a
                  v-if="links.github"
                  :href="links.github"
                  target="_blank"
                  rel="noopener"
                  class="text-link"
                >
                  Check our code on GitHub <ArrowUpRight />
                </a>
              </div>
              <div class="security-list">
                <div class="security-item">
                  <span class="security-check"><Check /></span>
                  <div>
                    <strong>Matching Source Code</strong>
                    <span>Every release comes with its exact source.zip snapshot.</span>
                  </div>
                </div>
                <div class="security-item">
                  <span class="security-check"><Check /></span>
                  <div>
                    <strong>Open-Source Credits Included</strong>
                    <span>All third-party library licences are bundled inside the app.</span>
                  </div>
                </div>
                <div class="security-item">
                  <span class="security-check"><Check /></span>
                  <div>
                    <strong>No Secrets in the App</strong>
                    <span>Private keys stay safely on our server, never inside the desktop app.</span>
                  </div>
                </div>
              </div>
            </div>
          </section>
        </div>

        <!-- ==================== GUIDES & FAQ PAGE ==================== -->
        <div v-else-if="currentRoute === '/docs'" key="docs">
          <section class="web-container page-hero hero-anim-1">
            <span class="eyebrow">HELP &amp; GUIDES</span>
            <h1>Easy setup &amp; <span>quick answers.</span></h1>
            <p class="page-lead">
              New to Solara? Follow these quick steps to install, add modpacks, and make your own icons in minutes.
            </p>
          </section>

          <section class="web-container sub-section">
            <div class="guides-grid">
              <article
                v-for="(g, idx) in GUIDES"
                :key="g.id"
                class="guide-card reveal spotlight-hover"
                :style="{ '--delay': `${idx * 70}ms` }"
                @pointermove="onCardPointerMove"
              >
                <span class="step-num">{{ g.step }}</span>
                <h3>{{ g.title }}</h3>
                <ul>
                  <li v-for="pt in g.points" :key="pt">
                    <Check />
                    <span>{{ pt }}</span>
                  </li>
                </ul>
              </article>
            </div>
          </section>

          <section class="web-container sub-section faq-section reveal">
            <div class="faq-lead">
              <span class="eyebrow">FAQ</span>
              <h2>Common questions.</h2>
              <p>Still stuck or found a bug? Tell us on GitHub and we’ll help out.</p>
              <a :href="links.support" target="_blank" rel="noopener" class="text-link">
                Report an issue on GitHub <ArrowUpRight />
              </a>
            </div>

            <div class="faq-list">
              <article v-for="(q, i) in questions" :key="q[0]">
                <button
                  type="button"
                  :aria-expanded="faq === i"
                  :aria-controls="'faq-' + i"
                  @click="faq = faq === i ? null : i"
                >
                  <span>{{ q[0] }}</span>
                  <Minus v-if="faq === i" />
                  <Plus v-else />
                </button>
                <Transition name="faq-slide">
                  <p v-if="faq === i" :id="'faq-' + i">{{ q[1] }}</p>
                </Transition>
              </article>
            </div>
          </section>
        </div>
      </Transition>
    </main>

    <!-- Multi-Column Footer -->
    <footer class="web-footer web-container">
      <div class="footer-grid">
        <div class="footer-brand-col">
          <a class="web-brand" href="/" @click.prevent="navigate('/')">
            <img src="/logo.svg" alt="Solara" />
            <span>Solara Launcher</span>
          </a>
          <p>
            Fast, ad-free OLED Minecraft launcher for Windows with one-click modpacks, Icon Studio, and automatic updates.
          </p>
        </div>

        <div class="footer-col">
          <h4>Pages</h4>
          <a href="/" @click.prevent="navigate('/')">Home</a>
          <a href="/features" @click.prevent="navigate('/features')">Features</a>
          <a href="/downloads" @click.prevent="navigate('/downloads')">Downloads</a>
          <a href="/changelog" @click.prevent="navigate('/changelog')">What’s New</a>
        </div>

        <div class="footer-col">
          <h4>Help &amp; Links</h4>
          <a href="/security" @click.prevent="navigate('/security')">Safety &amp; Privacy</a>
          <a href="/docs" @click.prevent="navigate('/docs')">Help &amp; FAQ</a>
          <a v-if="links.github" :href="links.github" target="_blank" rel="noopener">
            GitHub <ArrowUpRight />
          </a>
          <a :href="links.home" target="_blank" rel="noopener">
            Solara-Inc <ArrowUpRight />
          </a>
        </div>
      </div>

      <div class="footer-bottom">
        <span>© {{ new Date().getFullYear() }} Solara Launcher · Open source (GPL-3.0-only).</span>
        <span>
          <span class="dot pulse-dot" /> Release feed: {{ snapshot.state }}
        </span>
      </div>

      <p class="third-party">
        Independent Minecraft Java launcher. Not an official Minecraft product; not approved by or associated with Mojang or Microsoft.
      </p>
    </footer>
  </div>
</template>
