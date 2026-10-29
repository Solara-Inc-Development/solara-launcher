<script setup lang="ts">
import {computed} from 'vue';
import {ArrowRight,Plus,Play,Layers3,Server,Compass,ArrowUpRight,RefreshCw,Box,Coffee,ShieldCheck,Download,Sparkles} from '../../packages/ui/icons';
const props=defineProps<{instances:any[],projects:any[],services:any[],account:any,loading:boolean,apiState:string}>();
const emit=defineEmits(['open-instance','create','navigate','external','refresh']);
const ready=computed(()=>props.instances.filter(i=>i.state==='installed').length);
const activeInst=computed(()=>props.instances.find(i=>i.running)||props.instances[0]);
const scenes=['forest-valley','alpine-lake','coastal-hamlet','canyon-settlement','moss-cave','workshop'];
function instanceScene(inst:any):string{const name=inst?.name||'';let hash=0;for(let i=0;i<name.length;i++){hash=(hash<<5)-hash+name.charCodeAt(i);hash|=0}return scenes[Math.abs(hash)%scenes.length]}
function instanceArtSrc(inst:any):string{return `/art/${instanceScene(inst)}-v1-small.webp`}
const popularHighlights=[
  {id:'fabulously-optimized',title:'Fabulously Optimized',type:'modpack',description:'Fast, smooth Minecraft with shaders and mod support.',icon:'https://cdn.modrinth.com/data/1KVo5zza/d8152911f8fd5d7e9a8c499fe89045af81fe816e_96.webp',downloads:'19.5M'},
  {id:'sodium',title:'Sodium',type:'mod',description:'Modern rendering engine providing massive FPS gains.',icon:'https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp',downloads:'256.8M'},
  {id:'iris',title:'Iris Shaders',type:'mod',description:'Modern shader pack support with full Sodium optimization.',icon:'https://cdn.modrinth.com/data/YL57xq9U/18d0e7f076d3d6ed5bedd472b853909aac5da202_96.webp',downloads:'198.6M'},
  {id:'cobblemon',title:'Cobblemon',type:'modpack',description:'Catch, train, and battle Pokemon in open Minecraft.',icon:'https://cdn.modrinth.com/data/5FFgwNNP/e7f9ee2e9d361623847853fe2ddce42f519ee64f.png',downloads:'11.6M'}
];
</script>
<template>
<section class="new-home">
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
          <button class="hero-active-play" :aria-label="'Play '+activeInst.name" @click.stop="emit('open-instance',activeInst)">
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

  <div class="new-home-setup">
    <div><Coffee/><span><strong>The right Java, handled.</strong><small>Missing Java is set up automatically for your instance.</small></span></div>
    <div><Box/><span><strong>A home for every setup.</strong><small>Separate mods, configs and saves for each instance.</small></span></div>
    <div><ShieldCheck/><span><strong>Your accounts stay yours.</strong><small>Minecraft and Solara have separate sign-ins.</small></span></div>
  </div>

  <section class="new-home-library">
    <div class="workspace-section-heading">
      <div><span class="workspace-kicker">PICK UP WHERE YOU LEFT OFF</span><h2>Your library <span>{{instances.length}}</span></h2></div>
      <button class="text-link" @click="emit('navigate','launcher')">View all <ArrowRight :size="15"/></button>
    </div>
    <div v-if="instances.length" class="new-home-instance-list">
      <button v-for="instance in instances.slice(0,5)" :key="instance.id" @click="emit('open-instance',instance)">
        <img :src="instanceArtSrc(instance)" alt="" class="home-instance-thumb">
        <span class="workspace-row-name">
          <strong>{{instance.name}}</strong>
          <small>{{instance.gameVersion}} · <span class="capitalize">{{instance.loader}}</span></small>
        </span>
        <span class="home-instance-state" :class="{running:instance.running}">{{instance.running?'Running':instance.state==='installed'?'Ready':instance.state}}</span>
        <ArrowRight/>
      </button>
    </div>
    <div v-else class="new-home-empty">
      <div><span class="empty-library-symbol"><Plus/></span><h3>Your first world starts here.</h3><p>Pick a Minecraft version. We will take care of its files and Java.</p></div>
      <button @click="emit('create')">Create your first instance <ArrowRight :size="16"/></button>
    </div>
    <div v-if="instances.length" class="new-home-library-note">{{ready}} {{ready===1?'instance has':'instances have'}} game files installed. Account and Java checks happen before play.</div>
  </section>

  <!-- Popular Community Content Showcase -->
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

  <section class="new-home-bottom">
    <div>
      <div class="workspace-section-heading">
        <div><span class="workspace-kicker">THERE'S MORE HERE</span><h2>Your Solara tools</h2></div>
        <button class="text-link" @click="emit('navigate','services')">All services <ArrowUpRight :size="15"/></button>
      </div>
      <div v-if="projects.length||services.length" class="new-home-services">
        <button v-for="item in [...projects,...services].filter((p,i,a)=>a.findIndex(x=>x.id===p.id)===i).slice(0,4)" :key="item.id" @click="item.url&&emit('external',item.url)">
          <span class="service-letter"><Layers3/></span>
          <span class="workspace-row-name"><strong>{{item.name}}</strong><small>{{item.description}}</small></span>
          <ArrowUpRight :size="17"/>
        </button>
      </div>
      <div v-else class="new-home-service-empty">
        <p>{{loading?'Checking your services…':'Service listings could not load. Your local library stays available.'}}</p>
        <button @click="emit('refresh')"><RefreshCw :size="14"/> Refresh</button>
      </div>
    </div>
    <aside class="new-home-links">
      <span class="workspace-kicker">GO SOMEWHERE</span>
      <button @click="emit('navigate','explore')"><Compass/><div><strong>Find your next favourite mod</strong><small>Browse community mods and modpacks.</small></div><ArrowRight/></button>
      <button @click="emit('navigate','servers')"><Server/><div><strong>Bring your friends</strong><small>Local servers and Solara Network.</small></div><ArrowRight/></button>
    </aside>
  </section>
</section>
</template>
