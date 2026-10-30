<script setup lang="ts">
import {ref,computed} from 'vue';
import {ArrowRight,Plus,Play,Square,Layers3,Server,Compass,ArrowUpRight,RefreshCw,Box,Coffee,ShieldCheck,Download,Sparkles,Check,ChevronDown,ChevronUp,MoreVertical,Copy,ExternalLink,User,Lock} from '../../packages/ui/icons';

const props=withDefaults(defineProps<{
  instances:any[],
  projects:any[],
  services:any[],
  account:any,
  minecraft?:any[],
  loading:boolean,
  apiState:string
}>(),{
  minecraft:()=>[]
});

const emit=defineEmits(['open-instance','create','navigate','external','refresh','play','account','copy','solara-signin','create-preset']);
const jumpInOpen=ref(true);
const librarySearch=ref('');
const avatarFailed=ref(false);
const activeLibraryTab=ref<'instances'|'presets'>('instances');

const ready=computed(()=>props.instances.filter(i=>i.state==='installed').length);
const activeInst=computed(()=>props.instances.find(i=>i.running)||props.instances[0]);
const activeMinecraftAccount=computed(()=>props.minecraft?.find(a=>a.active)||props.minecraft?.[0]||null);
const hasLicense=computed(()=>!!activeMinecraftAccount.value);

const scenes=['forest-valley','alpine-lake','coastal-hamlet','canyon-settlement','moss-cave','workshop'];
function instanceScene(inst:any):string{const name=inst?.name||'';let hash=0;for(let i=0;i<name.length;i++){hash=(hash<<5)-hash+name.charCodeAt(i);hash|=0}return scenes[Math.abs(hash)%scenes.length]}
function instanceArtSrc(inst:any):string{return `/art/${instanceScene(inst)}-v1-small.webp`}

const jumpInServers=ref([
  {
    id:'solara',
    name:'Solara Network',
    ip:'play.solaranetwork.in',
    online:true,
    players:'142 online',
    subtitle:'Solara 1.21.1 · Official',
    motd:'Solara Official Network [1.21.1] Survival, Creative & Minigames',
    loader:'Fabric'
  },
  {
    id:'hypixel',
    name:'Hypixel Network',
    ip:'mc.hypixel.net',
    online:true,
    players:'25,697 online',
    subtitle:'Fabric 1.21.1 · 2 days ago',
    motd:'Hypixel Network [1.8/1.21]  5B GHOST VISITORS! SPOOKY EVENT',
    loader:'Fabric'
  },
  {
    id:'cubecraft',
    name:'CubeCraft Games',
    ip:'play.cubecraft.net',
    online:true,
    players:'3,450 online',
    subtitle:'Vanilla 1.21.1 · 1 day ago',
    motd:'CubeCraft Network · SkyWars, EggWars, Lucky Islands & Parkour',
    loader:'Vanilla'
  },
  {
    id:'arix',
    name:'Arix SMP',
    ip:'play.arix.gg',
    online:true,
    players:'2 online',
    subtitle:'Fabric 1.21.1 · 2 days ago',
    motd:'A Minecraft Server · Vanilla Survival Community',
    loader:'Fabric'
  },
  {
    id:'wynncraft',
    name:'Wynncraft MMORPG',
    ip:'play.wynncraft.com',
    online:true,
    players:'1,840 online',
    subtitle:'Fabric 1.21.1 · 3 days ago',
    motd:'Wynncraft · The Largest Minecraft MMORPG with Quests & Dungeons',
    loader:'Fabric'
  },
  {
    id:'manacube',
    name:'ManaCube Network',
    ip:'play.manacube.com',
    online:true,
    players:'1,220 online',
    subtitle:'Fabric 1.21.1 · 4 days ago',
    motd:'ManaCube · Survival, Islands, Earth SMP, Parkour & KitPvP',
    loader:'Fabric'
  },
  {
    id:'2b2t',
    name:'2b2t Anarchy',
    ip:'2b2t.org',
    online:true,
    players:'420 online',
    subtitle:'Vanilla 1.21.1 · 1 week ago',
    motd:'2b2t · The Oldest Anarchy Server in Minecraft',
    loader:'Vanilla'
  },
  {
    id:'spinex',
    name:'Spinex',
    ip:'spinex.solara.network',
    online:false,
    players:'Offline',
    subtitle:'Quilt 1.21 · 2 weeks ago',
    motd:"Can't connect to server",
    loader:'Quilt'
  }
]);

const instancePresets=ref([
  {
    id:'fabric-1.21.1',
    name:'Modern Fabric 1.21.1',
    loader:'fabric',
    gameVersion:'1.21.1',
    desc:'Lightweight & ultra fast. Compatible with Sodium, Iris shaders, and modern mods.',
    tag:'Recommended',
    icon:'⚡'
  },
  {
    id:'fabulously-optimized',
    name:'Fabulously Optimized',
    loader:'fabric',
    gameVersion:'1.21.1',
    desc:'Maximum FPS, shaders support, dynamic lights, and zoom out-of-the-box.',
    tag:'Extreme FPS',
    icon:'🚀'
  },
  {
    id:'cobblemon-modpack',
    name:'Cobblemon Adventure',
    loader:'fabric',
    gameVersion:'1.20.1',
    desc:'Catch, train, and battle Pokémon with open-world Minecraft exploration.',
    tag:'Modpack',
    icon:'🔴'
  },
  {
    id:'neoforge-1.21.1',
    name:'NeoForge Standard',
    loader:'neoforge',
    gameVersion:'1.21.1',
    desc:'The modern standard fork of Forge for 1.21+ mod compatibility.',
    tag:'Modern Forge',
    icon:'🔥'
  },
  {
    id:'forge-1.20.1',
    name:'Forge Classic 1.20.1',
    loader:'forge',
    gameVersion:'1.20.1',
    desc:'Massive mod library support: Create, Twilight Forest, Mekanism, and more.',
    tag:'Classic Mods',
    icon:'🛠'
  },
  {
    id:'pvp-1.8.9',
    name:'PvP Legacy 1.8.9',
    loader:'vanilla',
    gameVersion:'1.8.9',
    desc:'Classic 1.8 combat mechanics for Hypixel and competitive Bedwars.',
    tag:'PvP Arena',
    icon:'⚔'
  },
  {
    id:'quilt-1.21.1',
    name:'Quilt Performance',
    loader:'quilt',
    gameVersion:'1.21.1',
    desc:'Modern community-driven loader with full Fabric mod compatibility.',
    tag:'Next-Gen',
    icon:'💎'
  },
  {
    id:'vanilla-1.21.1',
    name:'Pure Vanilla 1.21.1',
    loader:'vanilla',
    gameVersion:'1.21.1',
    desc:'Untouched vanilla release with latest Tricky Trials updates.',
    tag:'Vanilla',
    icon:'🟩'
  }
]);

const visibleInstances=computed(()=>{
  const q=librarySearch.value.trim().toLowerCase();
  if(!q)return props.instances;
  return props.instances.filter(i=>`${i.name} ${i.gameVersion} ${i.loader}`.toLowerCase().includes(q));
});

const visiblePresets=computed(()=>{
  const q=librarySearch.value.trim().toLowerCase();
  if(!q)return instancePresets.value;
  return instancePresets.value.filter(p=>`${p.name} ${p.gameVersion} ${p.loader} ${p.desc}`.toLowerCase().includes(q));
});

function handlePlayServer(server:any){
  if(!hasLicense.value){
    emit('account');
    return;
  }
  const target=props.instances.find(i=>i.state==='installed')||props.instances[0];
  if(target){
    emit('play',target);
  }else{
    emit('create');
  }
}

function handlePlayInstance(instance:any){
  if(!hasLicense.value){
    emit('account');
    return;
  }
  emit('play',instance);
}

function installPreset(p:any){
  emit('create-preset',{
    name:p.name,
    loader:p.loader,
    gameVersion:p.gameVersion
  });
}

const popularHighlights=[
  {id:'fabulously-optimized',title:'Fabulously Optimized',type:'modpack',description:'Fast, smooth Minecraft with shaders and mod support.',icon:'https://cdn.modrinth.com/data/1KVo5zza/d8152911f8fd5d7e9a8c499fe89045af81fe816e_96.webp',downloads:'19.5M'},
  {id:'sodium',title:'Sodium',type:'mod',description:'Modern rendering engine providing massive FPS gains.',icon:'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',downloads:'256.8M'},
  {id:'iris',title:'Iris Shaders',type:'mod',description:'Modern shader pack support with full Sodium optimization.',icon:'https://cdn.modrinth.com/data/YL57xq9U/18d0e7f076d3d6ed5bedd472b853909aac5da202_96.webp',downloads:'198.6M'},
  {id:'cobblemon',title:'Cobblemon',type:'modpack',description:'Catch, train, and battle Pokemon in open Minecraft.',icon:'https://cdn.modrinth.com/data/5FFgwNNP/e7f9ee2e9d361623847853fe2ddce42f519ee64f.png',downloads:'11.6M'}
];
</script>

<template>
<section class="new-home">
  <!-- Top Hero Banner -->
  <div class="home-hero-card">
    <img src="/art/forest-valley-v1.webp" alt="" class="home-hero-bg" loading="eager">
    <div class="home-hero-overlay"></div>
    <header class="new-home-intro">
      <div class="hero-left-copy">
        <span class="workspace-kicker">{{account?'WELCOME BACK, '+(account.name||account.username||'PLAYER').toUpperCase():'WELCOME TO SOLARA'}}</span>
        <h1>Less setup.<br><span>More Minecraft.</span></h1>
        <p>Your versions, mods and worlds. High performance, zero bloat, independent.</p>
        <div class="new-home-actions">
          <button class="workspace-play" @click="emit('navigate','launcher')"><Play :size="18"/> Open Minecraft</button>
          <button @click="emit('create')"><Plus :size="18"/> New instance</button>
        </div>
      </div>
      <div v-if="activeInst" class="hero-active-card" @click="emit('open-instance',activeInst)">
        <span class="hero-card-kicker">{{activeInst.running?'CURRENTLY RUNNING':'CONTINUE PLAYING'}}</span>
        <div class="hero-active-content">
          <img :src="instanceArtSrc(activeInst)" alt="" class="hero-active-thumb">
          <div class="hero-active-info">
            <strong>{{activeInst.name}}</strong>
            <small>{{activeInst.gameVersion}} · <span class="capitalize">{{activeInst.loader}}</span></small>
          </div>
          <button class="hero-active-play" :title="(activeInst.running?'Manage ':'Play ')+activeInst.name" :aria-label="(activeInst.running?'Manage ':'Play ')+activeInst.name" @click.stop="handlePlayInstance(activeInst)">
            <Play :size="15"/>
          </button>
        </div>
      </div>
      <div v-else class="new-home-wordmark" aria-hidden="true">
        <span>SO</span><span>LA<span class="wordmark-accent">R</span>A<span class="wordmark-dot">.</span></span>
        <small>MAKE IT YOUR WORLD</small>
      </div>
    </header>
  </div>

  <!-- Main Modrinth-Style 2-Column Layout -->
  <div class="home-main-layout">
    <!-- Left Column: Jump In + Library -->
    <div class="home-content-column">
      <!-- 1. Jump In Servers Section -->
      <section class="home-jumpin-section">
        <div class="jumpin-header" @click="jumpInOpen = !jumpInOpen">
          <div class="jumpin-title">
            <h2>Jump in</h2>
            <span class="jumpin-count-badge">{{jumpInServers.length}} servers</span>
            <component :is="jumpInOpen ? ChevronUp : ChevronDown" :size="18"/>
          </div>
        </div>

        <Transition name="expand">
          <div v-if="jumpInOpen" class="jumpin-list">
            <article v-for="srv in jumpInServers" :key="srv.id" class="jumpin-card" :class="{offline: !srv.online}">
              <div class="jumpin-icon-wrap">
                <span class="jumpin-icon">⚡</span>
              </div>
              <div class="jumpin-body">
                <div class="jumpin-title-row">
                  <h3>{{srv.name}}</h3>
                  <span class="jumpin-link-badge">🔗</span>
                  <span class="jumpin-ping" :class="{online: srv.online, offline: !srv.online}">
                    <span class="ping-bars">📶</span> {{srv.players}}
                  </span>
                </div>
                <div class="jumpin-sub-row">
                  <span class="cake-badge">🍰</span>
                  <span class="jumpin-subtitle">{{srv.subtitle}}</span>
                </div>
                <div class="jumpin-motd" :class="{problem: !srv.online}">
                  {{srv.motd}}
                </div>
              </div>
              <div class="jumpin-actions">
                <button class="modrinth-play-pill" :disabled="!srv.online" :title="hasLicense?'Play on '+srv.name:'Sign in with Minecraft Java account to play'" @click="handlePlayServer(srv)">
                  <Play :size="14"/> Play
                </button>
                <button class="jumpin-dots-btn" title="Copy server address" @click.stop="emit('copy', srv.ip)">
                  <MoreVertical :size="15"/>
                </button>
              </div>
            </article>
          </div>
        </Transition>
      </section>

      <!-- 2. Modrinth-Style Library Section -->
      <section class="home-library-section">
        <div class="library-header-row">
          <div class="library-title-group">
            <h2>Library</h2>
            <div class="library-tab-pills">
              <button class="library-tab-pill" :class="{active: activeLibraryTab==='instances'}" @click="activeLibraryTab='instances'">
                Installed ({{instances.length}})
              </button>
              <button class="library-tab-pill" :class="{active: activeLibraryTab==='presets'}" @click="activeLibraryTab='presets'">
                ✨ Instance Templates ({{instancePresets.length}})
              </button>
            </div>
          </div>
          <div class="library-action-buttons">
            <button class="modrinth-secondary-btn" @click="emit('create')">
              <Plus :size="14"/> New group
            </button>
            <button class="modrinth-primary-btn" @click="emit('create')">
              <Plus :size="14"/> New instance
            </button>
          </div>
        </div>

        <div class="library-controls-bar">
          <div class="library-search-box">
            <input v-model="librarySearch" :placeholder="activeLibraryTab==='instances'?'Search instances…':'Search instance templates…'" aria-label="Search instances">
          </div>
          <div class="library-filter-pills">
            <button class="filter-pill-dropdown">
              <span>⇅ Last played</span> <ChevronDown :size="13"/>
            </button>
            <button class="filter-pill-dropdown">
              <span>⊞ Custom group</span> <ChevronDown :size="13"/>
            </button>
            <button class="filter-pill-btn">
              <span>+ Add filter</span>
            </button>
          </div>
        </div>

        <!-- 2A. Instance Cards Grid -->
        <template v-if="activeLibraryTab==='instances'">
          <div v-if="visibleInstances.length" class="modrinth-instance-grid">
            <article v-for="inst in visibleInstances" :key="inst.id" class="modrinth-instance-card" @click="emit('open-instance', inst)">
              <div class="instance-lime-badge">
                <div class="instance-3d-block">🍰</div>
              </div>
              <div class="instance-card-meta">
                <h3>{{inst.name}}</h3>
                <p>{{inst.loader}} {{inst.gameVersion}}</p>
              </div>
              <div class="instance-hover-action" @click.stop="handlePlayInstance(inst)">
                <button class="instance-quick-play" :title="hasLicense?'Play '+inst.name:'Sign in with Minecraft Java account to play'">
                  <Play :size="16"/>
                </button>
              </div>
            </article>
          </div>

          <!-- Empty State -->
          <div v-else class="new-home-empty">
            <div>
              <span class="empty-library-symbol"><Plus/></span>
              <h3>Your first world starts here.</h3>
              <p>Pick a Minecraft version or 1-click template. We will take care of its files and Java.</p>
            </div>
            <button class="modrinth-primary-btn" @click="emit('create')">Create your first instance <ArrowRight :size="16"/></button>
          </div>
        </template>

        <!-- 2B. Instance Templates & Presets Grid -->
        <template v-else>
          <div class="modrinth-preset-grid">
            <article v-for="preset in visiblePresets" :key="preset.id" class="preset-card">
              <div class="preset-top">
                <span class="preset-icon">{{preset.icon}}</span>
                <span class="preset-tag">{{preset.tag}}</span>
              </div>
              <div class="preset-body">
                <h3>{{preset.name}}</h3>
                <p class="preset-desc">{{preset.desc}}</p>
                <div class="preset-chips">
                  <span class="loader-chip">{{preset.loader}}</span>
                  <span class="version-chip">{{preset.gameVersion}}</span>
                </div>
              </div>
              <button class="modrinth-primary-btn preset-install-btn" @click="installPreset(preset)">
                <Plus :size="14"/> Install instance
              </button>
            </article>
          </div>
        </template>
      </section>

      <!-- 3. Trending on Modrinth Section -->
      <section class="home-community-section">
        <div class="workspace-section-heading">
          <div><span class="workspace-kicker">TRENDING ON MODRINTH</span><h2>Popular mods & modpacks</h2></div>
          <button class="text-link" @click="emit('navigate','explore')">Browse all <ArrowRight :size="15"/></button>
        </div>
        <div class="home-trending-grid">
          <article v-for="item in popularHighlights" :key="item.id" class="card home-mod-card" @click="emit('navigate','explore')">
            <div class="home-mod-top">
              <img :src="item.icon" :alt="item.title" class="home-mod-icon" loading="lazy">
              <div class="home-mod-meta">
                <h3>{{item.title}}</h3>
                <span class="pill download-pill"><Download :size="11"/> {{item.downloads}}</span>
              </div>
            </div>
            <p class="home-mod-desc">{{item.description}}</p>
            <span class="text-link home-mod-explore">Explore <ArrowUpRight :size="14"/></span>
          </article>
        </div>
      </section>
    </div>

    <!-- Right Column: Playing As & News -->
    <aside class="home-side-column">
      <!-- Playing As Box -->
      <div class="side-playing-box">
        <span class="side-kicker">Playing as</span>
        <div class="playing-account-card" @click="emit('account')">
          <div class="playing-avatar-wrap">
            <img v-if="activeMinecraftAccount && !avatarFailed" :src="'https://mc-heads.net/avatar/' + activeMinecraftAccount.name + '/48'" alt="" class="playing-avatar" @error="avatarFailed = true">
            <div v-else class="playing-avatar-placeholder">
              <User :size="20"/>
            </div>
          </div>
          <div class="playing-info">
            <strong>{{activeMinecraftAccount ? activeMinecraftAccount.name : 'No Minecraft account'}}</strong>
            <small v-if="activeMinecraftAccount" class="verified-tag"><Check :size="10"/> Java License Verified</small>
            <small v-else class="unverified-tag">Click to sign in &amp; play</small>
          </div>
          <ChevronDown :size="15" class="playing-chevron"/>
        </div>

        <div class="side-solara-account">
          <button v-if="!account" class="side-solara-signin-link" @click="emit('solara-signin')">
            Sign in to a Solara account to sync settings and see what your friends are playing!
          </button>
          <div v-else class="side-solara-connected">
            <ShieldCheck :size="14"/>
            <span>Solara: <strong>{{account.name || account.username}}</strong></span>
          </div>
        </div>
      </div>

      <!-- News & Feature Promo -->
      <div class="side-news-box">
        <span class="side-kicker">News</span>
        <div class="side-news-card" @click="emit('navigate', 'explore')">
          <div class="side-news-art">
            <div class="news-art-sliders">
              <div class="news-slider-line"><span></span></div>
              <div class="news-slider-line"><span></span></div>
            </div>
            <span class="news-link-badge">🔗</span>
          </div>
          <h4>Sync settings across instances</h4>
          <p>Easily copy configs and options between your local setups.</p>
        </div>
      </div>

      <!-- Solara Hosting Promo Banner -->
      <div class="side-hosting-card">
        <div class="hosting-badge"><span class="brand-glyph">✦</span> solara hosting</div>
        <h3>Get your next server on Solara.</h3>
        <p>Instant provisioning, dedicated IP addresses, and DDoS protection.</p>
        <button class="modrinth-primary-btn hosting-btn" @click="emit('navigate', 'servers')">
          Explore servers <ArrowUpRight :size="14"/>
        </button>
      </div>

      <!-- Quick Feature Setup -->
      <div class="side-feature-pills">
        <div class="feature-item"><Coffee :size="16"/><span>Automatic Java</span></div>
        <div class="feature-item"><Box :size="16"/><span>Isolated Worlds</span></div>
        <div class="feature-item"><ShieldCheck :size="16"/><span>Independent Auth</span></div>
      </div>
    </aside>
  </div>
</section>
</template>
