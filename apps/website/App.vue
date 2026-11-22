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
let timer: ReturnType<typeof setInterval> | null = null;

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
    title: 'Isolated Instances',
    copy: 'Run Vanilla, Fabric, Forge, NeoForge, and Quilt profiles side by side in clean, isolated directories.',
    tag: 'INSTANCES',
  },
  {
    icon: Package,
    tone: 'emerald',
    title: 'One-Click Modpacks',
    copy: 'Search, filter by loader or Minecraft version, and install community modpacks with verified hashes.',
    tag: 'MODPACKS',
  },
  {
    icon: Sparkles,
    tone: 'purple',
    title: 'Icon Studio & 3D Skins',
    copy: 'Personalize instances with 30 online 3D icons, monogram badges, and custom frames, plus a real-time 3D skin previewer.',
    tag: 'CUSTOMIZE',
  },
  {
    icon: Monitor,
    tone: 'amber',
    title: 'Automatic Java Runtimes',
    copy: 'Detects and installs the exact verified Azul Zulu or Adoptium Java build needed for every Minecraft version.',
    tag: 'RUNTIMES',
  },
  {
    icon: ShieldCheck,
    tone: 'sky',
    title: 'On-Device Microsoft Auth',
    copy: 'Official Microsoft OAuth PKCE sign-in. Credentials stay encrypted locally in your Windows credential store.',
    tag: 'SECURITY',
  },
  {
    icon: Download,
    tone: 'rose',
    title: 'Signed Auto-Updates',
    copy: 'Updates download and verify Minisign cryptographic signatures automatically when idle—no manual reinstalls.',
    tag: 'UPDATES',
  },
];

const questions = [
  [
    'Do I need any extra account to use Solara?',
    'No. Only your licensed Microsoft Minecraft: Java Edition account is needed to play. Public Windows releases are free to download with no sign-up wall.',
  ],
  [
    'Where are my Microsoft credentials stored?',
    'Sign-in happens directly on your device with official Microsoft OAuth PKCE. Your password never touches a Solara form, and tokens remain encrypted locally on your PC.',
  ],
  [
    'Which mod loaders and Minecraft versions are supported?',
    'Solara supports Vanilla, Fabric, NeoForge, Forge, and Quilt across modern and classic Minecraft Java releases, with automatic Java runtime selection.',
  ],
  [
    'How are Windows updates verified?',
    'Every solaralauncher.exe release is SHA-256 checksummed and Minisign-signed. The launcher checks signatures automatically before applying updates when no game is running.',
  ],
  [
    'Is Solara open source and ad-free?',
    'Yes. The desktop application and website are open source under GPL-3.0-only with zero ads, zero telemetry clutter, and full third-party licence notices bundled.',
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
          <a
            href="/"
            :class="{ active: currentRoute === '/' }"
            @click.prevent="navigate('/')"
          >Overview</a>
          <a
            href="/features"
            :class="{ active: currentRoute === '/features' }"
            @click.prevent="navigate('/features')"
          >Features</a>
          <a
            href="/downloads"
            :class="{ active: currentRoute === '/downloads' }"
            @click.prevent="navigate('/downloads')"
          >Downloads</a>
          <a
            href="/changelog"
            :class="{ active: currentRoute === '/changelog' }"
            @click.prevent="navigate('/changelog')"
          >Changelog</a>
          <a
            href="/security"
            :class="{ active: currentRoute === '/security' }"
            @click.prevent="navigate('/security')"
          >Security</a>
          <a
            href="/docs"
            :class="{ active: currentRoute === '/docs' }"
            @click.prevent="navigate('/docs')"
          >Guides &amp; FAQ</a>
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
      <!-- ==================== HOME PAGE ==================== -->
      <template v-if="currentRoute === '/'">
        <section class="web-hero web-container">
          <div class="hero-pill">
            <span class="dot" />
            <span>{{ latest ? `Latest release ${latest.publicVersion}` : 'Windows x64 Preview' }}</span>
            <span class="hero-pill-sep">·</span>
            <span>OLED &amp; White Themes</span>
          </div>

          <h1>
            The pure Minecraft launcher,<br />
            <span>built for speed.</span>
          </h1>

          <p class="hero-description">
            Manage isolated Minecraft Java instances, install community modpacks in one click, and customize icons &amp; 3D skins—in a fast, ad-free desktop app.
          </p>

          <div class="hero-actions">
            <a
              v-if="latest"
              :href="latest.url"
              download="solaralauncher.exe"
              class="button primary hero-cta"
            >
              <Download />
              <span>Download for Windows</span>
              <span class="cta-meta">{{ latest.publicVersion }}</span>
            </a>
            <a v-else href="/downloads" class="button primary hero-cta" @click.prevent="navigate('/downloads')">
              <Download />
              <span>Download for Windows</span>
              <ArrowRight />
            </a>

            <a
              href="/features"
              class="button secondary hero-secondary"
              @click.prevent="navigate('/features')"
            >
              <span>Explore features</span>
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

          <div class="hero-trust">
            <span><Check /> Windows 10/11 x64</span>
            <span><ShieldCheck /> Official Microsoft OAuth</span>
            <span><Check /> Minisign Verified Updates</span>
            <span><Check /> Zero Ads · Open Source</span>
          </div>

          <!-- Interactive 5-View App Showcase -->
          <div id="showcase" class="showcase-shell">
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
              <img
                v-show="activeShot === 'dashboard'"
                src="/screenshots/dashboard.png"
                alt="Solara Launcher Dashboard in OLED dark theme"
                width="1440"
                height="950"
              />
              <img
                v-show="activeShot === 'modpacks'"
                src="/screenshots/modpacks.png"
                alt="Solara Launcher ModPackInstaller with cover cards and filters"
                width="1380"
                height="860"
              />
              <img
                v-show="activeShot === 'icon-studio'"
                src="/screenshots/icon-studio.png"
                alt="Solara Icon Studio with 3D icons and custom monogram badges"
                width="1380"
                height="860"
              />
              <img
                v-show="activeShot === 'settings'"
                src="/screenshots/settings-resources.png"
                alt="Solara Settings Resources and Storage management"
                width="1440"
                height="950"
              />
              <img
                v-show="activeShot === 'white'"
                src="/screenshots/downloads-white.png"
                alt="Solara Launcher clean White theme"
                width="1440"
                height="950"
              />
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
          <div class="stat-box">
            <strong>{{ latest ? `${(latest.size / 1048576).toFixed(1)} MB` : '~16 MB' }}</strong>
            <span>Lightweight Windows installer</span>
          </div>
          <div class="stat-box">
            <strong>5 Loaders</strong>
            <span>Vanilla, Fabric, NeoForge, Forge, Quilt</span>
          </div>
          <div class="stat-box">
            <strong>30+ 3D Icons</strong>
            <span>Plus Monogram Badges &amp; HUD frames</span>
          </div>
          <div class="stat-box">
            <strong>{{ releases.length || 83 }} Builds</strong>
            <span>Immutable SHA-256 &amp; Minisign archive</span>
          </div>
        </section>

        <!-- Features Bento Grid -->
        <section id="features" class="web-container web-section">
          <div class="section-header-row">
            <div class="section-header">
              <span class="eyebrow">BUILT FOR PLAYERS</span>
              <h2>Everything you need.<br />Nothing in the way.</h2>
              <p>
                Clean desktop architecture focused on fast launches, safe modpack installs, and zero clutter.
              </p>
            </div>
            <a href="/features" class="button secondary" @click.prevent="navigate('/features')">
              <span>Deep dive into features</span>
              <ArrowRight />
            </a>
          </div>

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

        <!-- Security Highlight -->
        <section id="security" class="web-container web-section">
          <div class="security-card">
            <div class="security-copy">
              <span class="eyebrow">PRIVACY FIRST</span>
              <h2>Your Microsoft account stays on your PC.</h2>
              <p>
                Solara uses official Microsoft OAuth PKCE directly on your device. Passwords never enter a Solara form, tokens stay inside your OS keyring, and every release is cryptographically verified.
              </p>
              <a href="/security" class="text-link" @click.prevent="navigate('/security')">
                Read full security model <ArrowRight />
              </a>
            </div>

            <div class="security-list">
              <div class="security-item">
                <span class="security-check"><Check /></span>
                <div>
                  <strong>Official Microsoft OAuth PKCE</strong>
                  <span>Licensed Minecraft: Java Edition accounts only. No cracked flows.</span>
                </div>
              </div>
              <div class="security-item">
                <span class="security-check"><Check /></span>
                <div>
                  <strong>Local Encrypted Keyring</strong>
                  <span>Session tokens never leave your computer or touch a third-party server.</span>
                </div>
              </div>
              <div class="security-item">
                <span class="security-check"><Check /></span>
                <div>
                  <strong>Minisign &amp; SHA-256 Verified</strong>
                  <span>Every Windows installer and auto-update payload is cryptographically signed.</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        <!-- Quick Download Panel on Home -->
        <section id="downloads" class="web-container web-section">
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
                    {{ latest ? latest.publicVersion : 'Preview' }}
                  </span>
                </div>
                <p class="download-meta">
                  <template v-if="latest">
                    Windows 10/11 x64 · {{ (latest.size / 1048576).toFixed(1) }} MB · Released
                    {{ new Date(latest.publishedAt).toLocaleDateString() }}
                  </template>
                  <template v-else>
                    Windows 10/11 x64 · Waiting for verified release feed
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
                  class="button primary download-btn"
                >
                  <Download />
                  <span>Download solaralauncher.exe</span>
                </a>
                <a
                  href="/downloads"
                  class="button secondary"
                  @click.prevent="navigate('/downloads')"
                >
                  <span>Checksums &amp; Archive</span>
                </a>
              </div>
              <small>Feed: {{ snapshot.state }} · Signed via Minisign</small>
            </div>
          </div>
        </section>

        <!-- Explore Pages Grid -->
        <section class="web-container web-section explore-section">
          <div class="explore-grid">
            <a href="/changelog" class="explore-card" @click.prevent="navigate('/changelog')">
              <span class="eyebrow">WHATS NEW</span>
              <h3>Changelog &amp; Roadmap</h3>
              <p>See every verified release milestone, bug fix, and upcoming launcher feature.</p>
              <span class="text-link">Browse releases <ArrowRight /></span>
            </a>
            <a href="/docs" class="explore-card" @click.prevent="navigate('/docs')">
              <span class="eyebrow">HELP &amp; SETUP</span>
              <h3>Guides &amp; Troubleshooting</h3>
              <p>Step-by-step setup guides for Windows install, modpacks, Icon Studio, and RAM tuning.</p>
              <span class="text-link">Read guides <ArrowRight /></span>
            </a>
            <a href="/security" class="explore-card" @click.prevent="navigate('/security')">
              <span class="eyebrow">TRUST &amp; LICENSING</span>
              <h3>Security &amp; Open Source</h3>
              <p>Verify SHA-256 hashes, inspect Minisign signatures, and review our GPL-3.0-only licence.</p>
              <span class="text-link">Inspect security <ArrowRight /></span>
            </a>
          </div>
        </section>
      </template>

      <!-- ==================== FEATURES PAGE ==================== -->
      <template v-else-if="currentRoute === '/features'">
        <section class="web-container page-hero">
          <span class="eyebrow">ARCHITECTURE &amp; CAPABILITIES</span>
          <h1>Engineered for clean,<br /><span>effortless Minecraft play.</span></h1>
          <p class="page-lead">
            Every view in Solara Launcher is crafted for speed, legibility on OLED and bright displays, and clean separation between your instances.
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

        <!-- Visual Deep-Dive Cards -->
        <section class="web-container sub-section">
          <div class="spotlight-list">
            <article class="spotlight-card">
              <div class="spotlight-copy">
                <span class="eyebrow">MODPACK INSTALLER</span>
                <h2>Cover-banner browsing across catalogs.</h2>
                <p>
                  Switch between supported community modpack catalogs in one click. Filter by Fabric, Forge, NeoForge, or Quilt, search exact Minecraft versions, toggle between Grid and List layouts, and install directly into an isolated instance.
                </p>
              </div>
              <div class="spotlight-media">
                <img src="/screenshots/modpacks.png" alt="ModPackInstaller cover cards" loading="lazy" />
              </div>
            </article>

            <article class="spotlight-card reverse">
              <div class="spotlight-copy">
                <span class="eyebrow">SOLARA ICON STUDIO</span>
                <h2>3D icons, Monogram Badges &amp; HUD frames.</h2>
                <p>
                  Make every instance recognizable at a glance. Pick from 30 online MIT-licensed 3D icons, type custom monogram tags (like SMP or 1.21), or frame your own PNG with dual custom color pickers, gradient angles, geometric textures, and HUD corners—exported crisp at 512×512.
                </p>
              </div>
              <div class="spotlight-media">
                <img src="/screenshots/icon-studio.png" alt="Solara Icon Studio" loading="lazy" />
              </div>
            </article>

            <article class="spotlight-card">
              <div class="spotlight-copy">
                <span class="eyebrow">DUAL THEME SYSTEM</span>
                <h2>True #000000 OLED &amp; crisp daytime White.</h2>
                <p>
                  Designed from the ground up for both pitch-black OLED monitors and bright workspaces. Switch instantly under Settings → Theme with live mini-window previews and zero contrast glitches.
                </p>
              </div>
              <div class="spotlight-media">
                <img src="/screenshots/downloads-white.png" alt="Solara White Theme" loading="lazy" />
              </div>
            </article>
          </div>
        </section>

        <!-- Loader Matrix Table -->
        <section class="web-container sub-section page-end">
          <div class="section-header">
            <span class="eyebrow">LOADER SUPPORT</span>
            <h2>Supported Minecraft Loaders</h2>
            <p>All five major Minecraft Java setups work out of the box with automatic Java matching.</p>
          </div>
          <div class="data-table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th>Loader</th>
                  <th>Focus</th>
                  <th>Minecraft Versions</th>
                  <th>Java Management</th>
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
      </template>

      <!-- ==================== DOWNLOADS PAGE ==================== -->
      <template v-else-if="currentRoute === '/downloads'">
        <section class="web-container page-hero">
          <span class="eyebrow">WINDOWS INSTALLER &amp; ARCHIVE</span>
          <h1>Download <span>Solara Launcher.</span></h1>
          <p class="page-lead">
            Every release uses the clean filename <code>solaralauncher.exe</code>, is SHA-256 verified, Minisign signed, and paired with its exact source archive.
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
                  <h3>Solara Launcher for Windows (x64)</h3>
                  <span class="status-badge">
                    <span class="dot" />
                    {{ latest ? latest.publicVersion : 'Preview' }}
                  </span>
                </div>
                <p class="download-meta">
                  <template v-if="latest">
                    {{ (latest.size / 1048576).toFixed(2) }} MB · Released
                    {{ new Date(latest.publishedAt).toLocaleDateString() }} · Testing Preview
                  </template>
                  <template v-else>
                    Loading release metadata…
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
                    <span>{{ copied === 'latest-sha' ? 'Copied' : 'Copy SHA-256' }}</span>
                  </button>
                </div>
              </div>
            </div>

            <div class="download-cta-col">
              <a
                v-if="latest"
                :href="latest.url"
                download="solaralauncher.exe"
                class="button primary download-btn"
              >
                <Download />
                <span>Download solaralauncher.exe</span>
              </a>
              <button v-else disabled class="primary download-btn">
                <Download />
                <span>Installer unavailable</span>
              </button>
              <small>Feed: {{ snapshot.state }} · GitHub Releases CDN</small>
            </div>
          </div>

          <!-- Verify Checksum Box -->
          <div class="verify-box">
            <div class="verify-head">
              <strong>Verify your download on Windows PowerShell</strong>
              <button
                type="button"
                class="hash-copy-btn"
                @click="copyText('Get-FileHash .\\solaralauncher.exe -Algorithm SHA256', 'ps-cmd')"
              >
                <Check v-if="copied === 'ps-cmd'" />
                <Copy v-else />
                <span>{{ copied === 'ps-cmd' ? 'Copied command' : 'Copy command' }}</span>
              </button>
            </div>
            <pre><code>Get-FileHash .\solaralauncher.exe -Algorithm SHA256</code></pre>
            <p>Compare the output hash with the SHA-256 checksum listed above before running the installer.</p>
          </div>
        </section>

        <!-- System Requirements -->
        <section class="web-container sub-section">
          <div class="section-header">
            <span class="eyebrow">COMPATIBILITY</span>
            <h2>System Requirements</h2>
          </div>
          <div class="data-table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th>Component</th>
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
        <section class="web-container sub-section page-end">
          <div class="archive">
            <div class="archive-head">
              <div>
                <h3>Complete Release Archive</h3>
                <p>All historical Windows builds remain preserved and immutable.</p>
              </div>
              <div class="archive-controls">
                <input
                  v-model="releaseSearch"
                  type="search"
                  placeholder="Filter by version, note, or SHA-256..."
                  class="archive-search"
                />
                <span class="archive-count">{{ filteredReleases.length }} builds</span>
              </div>
            </div>

            <div v-if="!filteredReleases.length" class="archive-empty">
              <GitBranch />
              <p>{{ releases.length ? 'No releases matched your search.' : 'Loading verified release feed…' }}</p>
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
                    :title="copied === r.buildId ? 'Copied SHA-256' : 'Copy SHA-256'"
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
                {{ showAllReleases ? 'Show fewer releases' : `Show all ${filteredReleases.length} releases` }}
              </button>
            </div>
          </div>
        </section>
      </template>

      <!-- ==================== CHANGELOG & ROADMAP PAGE ==================== -->
      <template v-else-if="currentRoute === '/changelog'">
        <section class="web-container page-hero">
          <span class="eyebrow">RELEASE TIMELINE &amp; ROADMAP</span>
          <h1>Built in the open,<br /><span>shipped steadily.</span></h1>
          <p class="page-lead">
            Track major UI upgrades, fixes, and upcoming milestones across Solara Launcher releases.
          </p>
        </section>

        <section class="web-container sub-section">
          <div class="timeline-list">
            <article v-for="entry in CHANGELOG_HIGHLIGHTS" :key="entry.version" class="timeline-card">
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
          <div class="section-header">
            <span class="eyebrow">WHAT’S NEXT</span>
            <h2>Public Roadmap</h2>
            <p>Honest priorities we are actively building and testing.</p>
          </div>
          <div class="roadmap-grid">
            <article v-for="item in ROADMAP_ITEMS" :key="item.title" class="roadmap-card">
              <span class="roadmap-phase">{{ item.phase }}</span>
              <h3>{{ item.title }}</h3>
              <p>{{ item.copy }}</p>
            </article>
          </div>
        </section>
      </template>

      <!-- ==================== SECURITY PAGE ==================== -->
      <template v-else-if="currentRoute === '/security'">
        <section class="web-container page-hero">
          <span class="eyebrow">SECURITY, PRIVACY &amp; LICENSING</span>
          <h1>Transparent by design.<br /><span>Verified at every step.</span></h1>
          <p class="page-lead">
            No account lock-in, no hidden telemetry, no embedded API keys—and cryptographic verification for every update.
          </p>
        </section>

        <section class="web-container sub-section">
          <div class="pillar-grid">
            <article v-for="p in SECURITY_PILLARS" :key="p.title" class="pillar-card">
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
          <div class="security-card">
            <div class="security-copy">
              <span class="eyebrow">HONEST STATUS</span>
              <h2>Testing Preview &amp; Signing Boundary</h2>
              <p>
                Solara Launcher releases are signed with our persistent Minisign updater key so the built-in auto-updater verifies every download automatically. Windows Authenticode publisher signing is separate and not claimed.
              </p>
              <a
                v-if="links.github"
                :href="links.github"
                target="_blank"
                rel="noopener"
                class="text-link"
              >
                Inspect repository &amp; licence notices <ArrowUpRight />
              </a>
            </div>
            <div class="security-list">
              <div class="security-item">
                <span class="security-check"><Check /></span>
                <div>
                  <strong>Exact Corresponding Source</strong>
                  <span>Every release ships with an immutable source.zip verified against the exact CI commit.</span>
                </div>
              </div>
              <div class="security-item">
                <span class="security-check"><Check /></span>
                <div>
                  <strong>Bundled Third-Party Notices</strong>
                  <span>714 locked dependency packages and licence texts are packaged inside every installer.</span>
                </div>
              </div>
              <div class="security-item">
                <span class="security-check"><Check /></span>
                <div>
                  <strong>Zero Embedded Secrets</strong>
                  <span>Catalog proxies run server-side; signing keys and provider credentials never enter public source.</span>
                </div>
              </div>
            </div>
          </div>
        </section>
      </template>

      <!-- ==================== GUIDES & FAQ PAGE ==================== -->
      <template v-else-if="currentRoute === '/docs'">
        <section class="web-container page-hero">
          <span class="eyebrow">DOCUMENTATION &amp; SUPPORT</span>
          <h1>Guides, setup &amp; <span>answers.</span></h1>
          <p class="page-lead">
            Everything you need to install Solara Launcher, set up modpacks, customize instance icons, and troubleshoot smoothly.
          </p>
        </section>

        <section class="web-container sub-section">
          <div class="guides-grid">
            <article v-for="g in GUIDES" :key="g.id" class="guide-card">
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

        <section class="web-container sub-section faq-section">
          <div class="faq-lead">
            <span class="eyebrow">FAQ</span>
            <h2>Common questions.</h2>
            <p>Still need help or found a bug? Report it directly on GitHub.</p>
            <a :href="links.support" target="_blank" rel="noopener" class="text-link">
              Open an issue on GitHub <ArrowUpRight />
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
              <p v-show="faq === i" :id="'faq-' + i">{{ q[1] }}</p>
            </article>
          </div>
        </section>
      </template>
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
            Fast, ad-free OLED Minecraft Java launcher for Windows with one-click modpacks, Icon Studio, and signed auto-updates.
          </p>
        </div>

        <div class="footer-col">
          <h4>Product</h4>
          <a href="/" @click.prevent="navigate('/')">Overview</a>
          <a href="/features" @click.prevent="navigate('/features')">Features</a>
          <a href="/downloads" @click.prevent="navigate('/downloads')">Downloads</a>
          <a href="/changelog" @click.prevent="navigate('/changelog')">Changelog</a>
        </div>

        <div class="footer-col">
          <h4>Resources</h4>
          <a href="/security" @click.prevent="navigate('/security')">Security &amp; Privacy</a>
          <a href="/docs" @click.prevent="navigate('/docs')">Guides &amp; FAQ</a>
          <a v-if="links.github" :href="links.github" target="_blank" rel="noopener">
            GitHub <ArrowUpRight />
          </a>
          <a :href="links.home" target="_blank" rel="noopener">
            Solara-Inc <ArrowUpRight />
          </a>
        </div>
      </div>

      <div class="footer-bottom">
        <span>© {{ new Date().getFullYear() }} Solara Launcher. Open source (GPL-3.0-only).</span>
        <span>
          <span class="dot" /> API {{ snapshot.state }}
        </span>
      </div>

      <p class="third-party">
        Independent Minecraft Java launcher. Not an official Minecraft product; not approved by or associated with Mojang or Microsoft.
      </p>
    </footer>
  </div>
</template>
