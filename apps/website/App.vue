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

useInteractions();

const mobile = ref(false);
const faq = ref<number | null>(0);
const copied = ref('');
const showAllReleases = ref(false);
const activeShot = ref<'modpacks' | 'dashboard' | 'icon-studio' | 'white'>('modpacks');
let timer: ReturnType<typeof setInterval> | null = null;

const releases = computed(() => snapshot.value.data?.releases || []);
const visibleReleases = computed(() =>
  showAllReleases.value ? releases.value : releases.value.slice(0, 6),
);

const showcaseTabs = [
  {
    id: 'modpacks' as const,
    label: 'Modpacks',
    badge: 'Multi-Catalog',
    src: '/screenshots/modpacks.png',
    caption: 'Browse, filter, and install thousands of Minecraft modpacks with one-click loader setup.',
  },
  {
    id: 'dashboard' as const,
    label: 'Dashboard',
    badge: 'OLED Dark',
    src: '/screenshots/dashboard.png',
    caption: 'Pure #000000 OLED interface with clean instance cards, instant launch, and zero ads.',
  },
  {
    id: 'icon-studio' as const,
    label: 'Icon Studio',
    badge: '3D + Monograms',
    src: '/screenshots/icon-studio.png',
    caption: 'Craft crisp 512px instance icons with 3D artwork, custom monograms, gradients, and frames.',
  },
  {
    id: 'white' as const,
    label: 'White Theme',
    badge: 'Light Mode',
    src: '/screenshots/downloads-white.png',
    caption: 'Switch seamlessly between pure OLED black and a crisp daytime White theme.',
  },
];

const currentShot = computed(
  () => showcaseTabs.find((t) => t.id === activeShot.value) ?? showcaseTabs[0],
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
    copy: 'Personalize instances with 3D icons, monogram badges, and custom frames, plus a real-time 3D skin previewer.',
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
  void refreshPublic();
  timer = setInterval(() => void refreshPublic(), 60000);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});

async function copyChecksum(r: { sha256: string; buildId: string }) {
  await navigator.clipboard.writeText(r.sha256);
  copied.value = r.buildId;
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
        <a class="web-brand" href="#">
          <img src="/logo.svg" alt="Solara" />
          <span>Solara</span>
          <span v-if="latest" class="nav-version-badge">{{ latest.publicVersion }}</span>
        </a>

        <nav :class="{ expanded: mobile }" aria-label="Website navigation">
          <a href="#showcase" @click="mobile = false">Showcase</a>
          <a href="#features" @click="mobile = false">Features</a>
          <a href="#security" @click="mobile = false">Security</a>
          <a href="#downloads" @click="mobile = false">Downloads</a>
          <a :href="links.docs" target="_blank" rel="noopener">Docs <ArrowUpRight /></a>
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
          <a v-else href="#downloads" class="header-download">
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
      <!-- Centered Modern Hero -->
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
          <a v-else href="#downloads" class="button primary hero-cta">
            <Download />
            <span>Download for Windows</span>
            <ArrowRight />
          </a>

          <a
            v-if="links.github"
            :href="links.github"
            target="_blank"
            rel="noopener"
            class="button secondary hero-secondary"
          >
            <GitBranch />
            <span>View on GitHub</span>
            <ArrowUpRight />
          </a>
        </div>

        <div class="hero-trust">
          <span><Check /> Windows 10/11 x64</span>
          <span><ShieldCheck /> Official Microsoft OAuth</span>
          <span><Check /> Minisign Verified Updates</span>
          <span><Check /> Zero Ads</span>
        </div>

        <!-- Interactive App Showcase Window -->
        <div id="showcase" class="showcase-shell">
          <div class="showcase-toolbar">
            <div class="showcase-dots" aria-hidden="true">
              <span />
              <span />
              <span />
            </div>

            <div class="showcase-tabs" role="tablist" aria-label="Launcher views">
              <button
                v-for="tab in showcaseTabs"
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
            <!-- Keep dashboard.png referenced directly in template for preload & contract verification -->
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
              v-show="activeShot === 'white'"
              src="/screenshots/downloads-white.png"
              alt="Solara Launcher clean White theme"
              width="1440"
              height="950"
            />
          </div>

          <div class="showcase-footer">
            <p>{{ currentShot.caption }}</p>
            <a href="#downloads" class="text-link">
              Get solaralauncher.exe <ChevronRight />
            </a>
          </div>
        </div>
      </section>

      <!-- Features Bento Grid -->
      <section id="features" class="web-container web-section">
        <div class="section-header">
          <span class="eyebrow">BUILT FOR PLAYERS</span>
          <h2>Everything you need.<br />Nothing in the way.</h2>
          <p>
            Clean desktop architecture focused on fast launches, safe modpack installs, and zero clutter.
          </p>
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

      <!-- Security & Privacy Card -->
      <section id="security" class="web-container web-section">
        <div class="security-card">
          <div class="security-copy">
            <span class="eyebrow">PRIVACY FIRST</span>
            <h2>Your Microsoft account stays on your PC.</h2>
            <p>
              Solara uses official Microsoft OAuth PKCE directly on your device. Passwords never enter a Solara form, tokens stay inside your OS keyring, and every release is cryptographically verified.
            </p>
            <a
              v-if="links.github"
              :href="links.github"
              target="_blank"
              rel="noopener"
              class="text-link"
            >
              Inspect source code on GitHub <ArrowUpRight />
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

      <!-- Downloads & Release Archive -->
      <section id="downloads" class="web-container web-section">
        <div class="section-header">
          <span class="eyebrow">WINDOWS RELEASE</span>
          <h2>Install once.<br />Stay updated automatically.</h2>
          <p>
            Windows 10/11 (x64) with Microsoft Edge WebView2. Java runtimes are managed automatically per instance.
          </p>
        </div>

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
                  x86_64 · {{ (latest.size / 1048576).toFixed(1) }} MB · Released
                  {{ new Date(latest.publishedAt).toLocaleDateString() }}
                </template>
                <template v-else>
                  Waiting for verified release metadata
                </template>
              </p>
              <div v-if="latest?.sha256" class="download-hash-row">
                <code>SHA-256: {{ latest.sha256.slice(0, 24) }}…</code>
                <button
                  type="button"
                  class="hash-copy-btn"
                  @click="copyChecksum(latest)"
                >
                  <Check v-if="copied === latest.buildId" />
                  <Copy v-else />
                  <span>{{ copied === latest.buildId ? 'Copied' : 'Copy hash' }}</span>
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
            <small>Feed: {{ snapshot.state }} · Signed via Minisign</small>
          </div>
        </div>

        <!-- 3-Step Quickstart -->
        <div class="steps-grid">
          <div class="step-card">
            <span class="step-num">01</span>
            <h3>Run the installer</h3>
            <p>Download <code>solaralauncher.exe</code> and install in seconds on Windows 10 or 11 x64.</p>
          </div>
          <div class="step-card">
            <span class="step-num">02</span>
            <h3>Sign in locally</h3>
            <p>Connect your licensed Microsoft Minecraft account securely through on-device OAuth.</p>
          </div>
          <div class="step-card">
            <span class="step-num">03</span>
            <h3>Play your way</h3>
            <p>Pick a community modpack or create a custom Vanilla/modded instance with your own icon.</p>
          </div>
        </div>

        <!-- Release Archive -->
        <div id="changelog" class="archive">
          <div class="archive-head">
            <div>
              <h3>Release Archive</h3>
              <p>Immutable historical Windows builds with verified SHA-256 checksums.</p>
            </div>
            <span class="archive-count">{{ releases.length }} verified builds</span>
          </div>

          <div v-if="!releases.length" class="archive-empty">
            <GitBranch />
            <p>Loading verified release feed…</p>
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
                  @click="copyChecksum(r)"
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

          <div v-if="releases.length > 6" class="archive-more">
            <button
              type="button"
              class="button secondary"
              @click="showAllReleases = !showAllReleases"
            >
              {{ showAllReleases ? 'Show fewer releases' : `Show all ${releases.length} releases` }}
            </button>
          </div>
        </div>
      </section>

      <!-- FAQ Section -->
      <section class="web-container web-section faq-section">
        <div class="faq-lead">
          <span class="eyebrow">FAQ</span>
          <h2>Common questions.</h2>
          <p>Everything you need to know about accounts, updates, and open-source builds.</p>
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
    </main>

    <!-- Minimal Footer -->
    <footer class="web-footer web-container">
      <div class="footer-top">
        <a class="web-brand" href="#">
          <img src="/logo.svg" alt="Solara" />
          <span>Solara Launcher</span>
        </a>
        <div class="footer-links">
          <a :href="links.home" target="_blank" rel="noopener">Solara-Inc <ArrowUpRight /></a>
          <a :href="links.docs" target="_blank" rel="noopener">Documentation <ArrowUpRight /></a>
          <a :href="links.support" target="_blank" rel="noopener">Support <ArrowUpRight /></a>
          <a v-if="links.github" :href="links.github" target="_blank" rel="noopener">
            GitHub <ArrowUpRight />
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
