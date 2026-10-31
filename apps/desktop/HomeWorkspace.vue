<script setup lang="ts">
import InstanceAvatar from './InstanceAvatar.vue';
import {ref,computed} from 'vue';
import {ArrowRight,Plus,Play,Square,Layers3,Server,Compass,ArrowUpRight,RefreshCw,Box,Coffee,ShieldCheck,ChevronDown,FolderOpen,User,Search,Sparkles,Globe,Package,Gamepad2,Activity,Check,Monitor,Copy} from '../../packages/ui/icons';
import LoaderMark from '../../packages/ui/LoaderMark.vue';
import SelectControl from '../../packages/ui/SelectControl.vue';
const props=withDefaults(defineProps<{instances:any[],projects:any[],services:any[],account:any,minecraft?:any[],loading:boolean,apiState:string}>(),{minecraft:()=>[]});
const emit=defineEmits(['open-instance','create','navigate','external','refresh','play','stop','account','copy','solara-signin','create-preset','duplicate','open-folder','discover']);
const librarySearch=ref(''),activeLibraryTab=ref('instances'),sortBy=ref('name'),loaderFilter=ref('all'),avatarFailed=ref(false);
const activeAccount=computed(()=>props.minecraft.find(a=>a.active)||null);
const recentInstances=computed(()=>[...props.instances].filter(i=>i.running||i.state==='installed').sort((a,b)=>Number(b.running)-Number(a.running)).slice(0,3));
const runningInstance=computed(()=>props.instances.find(i=>i.running)||null);
const uniqueLoaders=computed(()=>Array.from(new Set(props.instances.map(i=>i.loader).filter(Boolean))));
const today=new Intl.DateTimeFormat('en-IN',{weekday:'long',day:'numeric',month:'short'}).format(new Date());
function readPins():string[]{try{const value=JSON.parse(localStorage.getItem('solara:pinned-instances')||'[]');return Array.isArray(value)?value.filter(v=>typeof v==='string'):[]}catch{return []}}
const pinned=ref(readPins());
function togglePin(id:string){pinned.value=pinned.value.includes(id)?pinned.value.filter(v=>v!==id):[...pinned.value,id];try{localStorage.setItem('solara:pinned-instances',JSON.stringify(pinned.value))}catch{}}
const visibleInstances=computed(()=>props.instances.filter(i=>(loaderFilter.value==='all'||i.loader===loaderFilter.value)&&`${i.name} ${i.gameVersion} ${i.loader}`.toLowerCase().includes(librarySearch.value.trim().toLowerCase())).sort((a,b)=>Number(pinned.value.includes(b.id))-Number(pinned.value.includes(a.id))||(sortBy.value==='version'?(b.gameVersion||'').localeCompare(a.gameVersion||'',undefined,{numeric:true}):sortBy.value==='active'?Number(b.running)-Number(a.running)||a.name.localeCompare(b.name):a.name.localeCompare(b.name))));
const templates=[
 {id:'fabric',name:'Fabric essentials',loader:'fabric',gameVersion:'1.21.1',desc:'A lightweight foundation for your favourite mods.',tone:'mint'},
 {id:'vanilla',name:'Simply Minecraft',loader:'vanilla',gameVersion:'1.21.1',desc:'The original game. A fresh world of possibilities.',tone:'amber'},
 {id:'forge',name:'Forge workshop',loader:'forge',gameVersion:'1.20.1',desc:'Start a classic Forge setup and add your own mods.',tone:'rose'},
 {id:'neoforge',name:'A new adventure',loader:'neoforge',gameVersion:'1.21.1',desc:'A clean NeoForge instance, ready to make your own.',tone:'violet'},
 {id:'quilt',name:'Quilt canvas',loader:'quilt',gameVersion:'1.21.1',desc:'Build a personal mod collection with Quilt.',tone:'sky'}
];
const visibleTemplates=computed(()=>templates.filter(t=>`${t.name} ${t.loader}`.toLowerCase().includes(librarySearch.value.trim().toLowerCase())));
const discoveries=[{type:'mod',name:'A little more possibility.',label:'EXPLORE MODS',copy:'Small additions. A whole new experience.',scene:'workshop',tone:'amber',icon:Package},{type:'modpack',name:'Your next obsession.',label:'DISCOVER MODPACKS',copy:'Find a world someone else dreamed up.',scene:'coastal-hamlet',tone:'mint',icon:Box},{type:'shader',name:'See it differently.',label:'FIND SHADERS',copy:'Give a familiar world a fresh perspective.',scene:'alpine-lake',tone:'violet',icon:Sparkles}];
function play(instance:any){if(instance.running){emit('stop',instance);return}if(!activeAccount.value){emit('account');return}emit('play',instance)}
function status(value:string){return ['online','operational','connected'].includes(value?.toLowerCase())?'Available':value||'Status not reported'}
</script>
<template>
<section class="new-home">
  <div class="home-welcome">
    <div class="welcome-copy">
      <div class="welcome-kicker-row">
        <span class="workspace-kicker"><Sparkles :size="12"/> A LITTLE WORLD OF POSSIBILITIES</span>
        <span class="welcome-status-badge" :class="{ 'is-running': runningInstance }">
          <span class="live-status-dot"></span>
          <span>{{ runningInstance ? 'Playing: ' + runningInstance.name : 'Launcher Ready' }}</span>
        </span>
      </div>
      <h1>{{account?'Welcome back, '+(account.name||account.username)+'.':'Welcome to Solara.'}}</h1>
      <p>Your worlds. Your people. Your next adventure.</p>
      <div class="welcome-telemetry-chips">
        <span class="telemetry-chip">
          <Box :size="13"/>
          <strong>{{ instances.length }}</strong> {{ instances.length === 1 ? 'instance' : 'instances' }}
        </span>
        <span class="telemetry-chip">
          <Layers3 :size="13"/>
          <strong>{{ uniqueLoaders.length || 1 }}</strong> active {{ uniqueLoaders.length === 1 ? 'loader' : 'loaders' }}
        </span>
        <span v-if="account" class="telemetry-chip account-chip">
          <ShieldCheck :size="13"/>
          <span>{{ account.name || account.username }}</span>
        </span>
      </div>
      <div class="new-home-actions">
        <button class="workspace-play" @click="emit('create')"><Plus/> Create a world <ArrowUpRight :size="14"/></button>
        <button class="welcome-discover" @click="emit('navigate','explore')">Find your inspiration <ArrowRight :size="14"/></button>
      </div>
    </div>
    <div class="welcome-scene" aria-hidden="true">
      <img src="/art/alpine-lake-v1-small.webp" alt="">
      <span class="welcome-sticker"><Sparkles/> Go somewhere new.</span>
      <span class="welcome-star star-one">✦</span>
      <span class="welcome-star star-two">✦</span>
    </div>
  </div>

  <section class="home-telemetry-banner" aria-label="Launcher status and telemetry">
    <div class="telemetry-card" :class="{ 'is-running': runningInstance }">
      <div class="telemetry-icon-wrap" :class="{ 'beacon-active': runningInstance }">
        <Gamepad2 v-if="runningInstance" :size="17"/>
        <Activity v-else :size="17"/>
      </div>
      <div class="telemetry-info">
        <span class="telemetry-tag">
          <span class="live-dot" :class="{ pulse: runningInstance }"></span>
          {{ runningInstance ? 'Live Session' : 'Game Engine' }}
        </span>
        <strong>{{ runningInstance ? runningInstance.name : (instances.length ? 'Ready to Adventure' : 'No Instances Yet') }}</strong>
        <small v-if="runningInstance">{{ runningInstance.loader }} · Minecraft {{ runningInstance.gameVersion }}</small>
        <small v-else-if="instances.length">{{ instances.length }} {{ instances.length === 1 ? 'world' : 'worlds' }} ready · Java verified</small>
        <small v-else>Pick a version and start playing</small>
      </div>
      <div class="telemetry-action">
        <button v-if="runningInstance" class="telemetry-btn stop" :aria-label="'Stop ' + runningInstance.name" @click="emit('stop', runningInstance)"><Square :size="13"/> Stop</button>
        <button v-else-if="recentInstances.length" class="telemetry-btn play" :aria-label="'Play ' + recentInstances[0].name" @click="play(recentInstances[0])"><Play :size="13"/> Play</button>
        <button v-else class="telemetry-btn primary" @click="emit('create')"><Plus :size="13"/> Create</button>
      </div>
    </div>

    <div class="telemetry-card">
      <div class="telemetry-icon-wrap">
        <Layers3 :size="17"/>
      </div>
      <div class="telemetry-info">
        <span class="telemetry-tag">LOADER ECOSYSTEM</span>
        <strong>{{ uniqueLoaders.length ? uniqueLoaders.map(l => l[0].toUpperCase() + l.slice(1)).join(', ') : 'Fabric, Forge, NeoForge, Quilt' }}</strong>
        <div class="telemetry-chips">
          <button v-for="l in ['fabric', 'forge', 'neoforge', 'quilt', 'vanilla']" :key="l" class="mini-loader-pill" :class="{ active: loaderFilter === l || (loaderFilter === 'all' && uniqueLoaders.includes(l)) }" :aria-label="'Filter instances by ' + l" @click="loaderFilter = (loaderFilter === l ? 'all' : l); activeLibraryTab = 'instances'">
            <LoaderMark :loader="l"/> {{ l }}
          </button>
        </div>
      </div>
    </div>

    <div class="telemetry-card">
      <div class="telemetry-icon-wrap">
        <ShieldCheck :size="17"/>
      </div>
      <div class="telemetry-info">
        <span class="telemetry-tag">SOLARA STATUS</span>
        <strong>{{ apiState === 'connected' ? 'Ecosystem Connected' : apiState === 'cached' ? 'Cached Mode' : 'Standalone Mode' }}</strong>
        <small>{{ activeAccount ? 'Minecraft: ' + activeAccount.name : 'Microsoft accounts saved locally' }}</small>
      </div>
      <button class="telemetry-link" @click="emit('navigate', 'services')">Services <ArrowUpRight :size="13"/></button>
    </div>
  </section>

  <section v-if="recentInstances.length" class="jump-back-section" aria-label="Jump back in">
    <div class="workspace-section-heading"><h2>Jump back in</h2><span class="jump-section-note">Your local instances</span></div>
    <article v-for="inst in recentInstances" :key="inst.id" class="jump-instance-row" :class="{ 'is-running': inst.running }">
      <button class="jump-instance-open" :aria-label="'Continue '+inst.name" @click="emit('open-instance',inst)"><InstanceAvatar :instance="inst"/><span><strong>{{inst.name}}</strong><small><LoaderMark :loader="inst.loader"/>{{inst.loader}} · Minecraft {{inst.gameVersion}}</small></span></button>
      <span class="jump-instance-state" :class="{running:inst.running}"><span class="dot"/>{{inst.running?'Running':'Ready to play'}}</span>
      <button class="workspace-play" :aria-label="(inst.running?'Stop recent instance ':'Play recent instance ')+inst.name" @click="play(inst)"><Square v-if="inst.running"/><Play v-else/>{{inst.running?'Stop':'Play'}}</button>
      <button class="jump-manage" :aria-label="'Manage recent instance '+inst.name" @click="emit('open-instance',inst)"><ArrowUpRight/></button>
    </article>
  </section>

  <div class="home-main-layout">
    <div class="home-content-column">
      <section class="home-library-section">
        <div class="library-header-row"><div class="library-title-group"><h2>Your library <span class="count-badge">{{instances.length}}</span></h2><p>A home for every kind of Minecraft.</p></div><button class="solara-btn-primary" @click="emit('create')"><Plus :size="14"/> New instance</button></div>
        <div class="library-tab-row"><div class="library-tab-pills" aria-label="Library categories"><button class="library-tab-pill" :class="{active:activeLibraryTab==='instances'}" :aria-pressed="activeLibraryTab==='instances'" @click="activeLibraryTab='instances'">My instances</button><button class="library-tab-pill" :class="{active:activeLibraryTab==='templates'}" :aria-pressed="activeLibraryTab==='templates'" @click="activeLibraryTab='templates'"><Sparkles :size="14"/> Quick starts</button></div><button class="text-link" @click="emit('navigate','launcher')">View library <ArrowRight :size="13"/></button></div>
        <div class="library-controls-bar"><label class="library-search-box"><Search :size="15"/><input v-model="librarySearch" placeholder="Find an instance…" aria-label="Search instances"></label><SelectControl v-if="activeLibraryTab==='instances'" v-model="sortBy" label="Sort home instances" :options="[{value:'name',label:'Name'},{value:'active',label:'Running first'},{value:'version',label:'Game version'}]"/></div>
        <div v-if="activeLibraryTab==='instances'&&instances.length" class="library-filter-pills"><button v-for="loader in ['all','fabric','quilt','forge','neoforge','vanilla'].filter(l=>l==='all'||instances.some(i=>i.loader===l))" :key="loader" class="filter-pill" :class="{active:loaderFilter===loader}" :aria-pressed="loaderFilter===loader" @click="loaderFilter=loader">{{loader==='all'?'All instances':loader}}</button></div>
        <template v-if="activeLibraryTab==='instances'">
          <div v-if="visibleInstances.length" class="solara-instance-grid">
            <article v-for="inst in visibleInstances" :key="inst.id" class="solara-instance-card" :class="{pinned:pinned.includes(inst.id), 'is-running': inst.running}">
              <button class="home-instance-cover" :aria-label="'Manage '+inst.name" @click="emit('open-instance',inst)"><InstanceAvatar :instance="inst"/><span class="cover-loader"><LoaderMark :loader="inst.loader"/>{{inst.loader}}</span><span v-if="inst.running" class="cover-running"><span class="dot"/> Running</span></button>
              <div class="home-instance-body"><button class="home-instance-name" @click="emit('open-instance',inst)"><h3>{{inst.name}}</h3><span>Minecraft {{inst.gameVersion}} · <strong :class="{running:inst.running}">{{inst.running?'Running':inst.state}}</strong></span></button><div class="instance-card-actions"><button class="card-action-btn pin-btn" :class="{pinned:pinned.includes(inst.id)}" :aria-label="(pinned.includes(inst.id)?'Unpin ':'Pin ')+inst.name" :aria-pressed="pinned.includes(inst.id)" @click="togglePin(inst.id)">★</button><button class="card-action-btn" title="Open instance folder" @click="emit('open-folder',inst)"><FolderOpen :size="14"/></button><button class="card-action-btn" title="Duplicate instance" @click.stop="emit('duplicate',inst)"><Copy :size="14"/></button><button class="instance-quick-play" :disabled="!inst.running&&inst.state!=='installed'" :aria-label="(inst.running?'Stop ':'Play ')+inst.name" @click="play(inst)"><Square v-if="inst.running" :size="13"/><Play v-else :size="13"/></button></div></div>
            </article>
          </div>
          <div v-else-if="instances.length" class="workspace-empty"><Search/><h3>No matching instances</h3><p>Try a different name or loader.</p><button @click="librarySearch='';loaderFilter='all'">Clear filters</button></div>
          <div v-else class="new-home-empty"><div class="empty-blocks" aria-hidden="true"><Box/><span><Plus/></span></div><h3>Your first world starts here.</h3><p>A fresh survival world? A modded adventure?<br>Start with a version and make it your own.</p><button class="solara-btn-primary" @click="emit('create')"><Plus :size="14"/> Create your first instance</button><button class="text-link" @click="activeLibraryTab='templates'">Or try a quick start <ArrowRight :size="13"/></button></div>
        </template>
        <template v-else><p class="template-explanation">Clean game and loader setups. Add the mods you love afterwards.</p><div class="solara-preset-grid"><article v-for="preset in visibleTemplates" :key="preset.id" class="preset-card" :data-tone="preset.tone"><InstanceAvatar :instance="{id:'quickstart:'+preset.id,loader:preset.loader}" class="preset-avatar"/><div class="preset-body"><span class="preset-loader"><LoaderMark :loader="preset.loader"/>{{preset.loader}} · {{preset.gameVersion}}</span><h3>{{preset.name}}</h3><p>{{preset.desc}}</p><button class="solara-btn-secondary" @click="emit('create-preset',preset)"><Plus :size="13"/> Create instance</button></div></article></div><div v-if="!visibleTemplates.length" class="workspace-empty"><Search/><p>No quick starts match your search.</p></div></template>
      </section>
      <section class="home-community-section"><div class="workspace-section-heading"><div><span class="workspace-kicker">A LITTLE INSPIRATION</span><h2>Make something of your own.</h2></div><button class="text-link" @click="emit('navigate','explore')">Discover more <ArrowRight :size="13"/></button></div><div class="home-discovery-grid"><button v-for="item in discoveries" :key="item.type" class="home-discovery-card" :data-tone="item.tone" @click="emit('discover',item.type)"><div class="discovery-card-art"><img :src="'/art/'+item.scene+'-v1-small.webp'" alt="" loading="lazy"><span><component :is="item.icon"/></span></div><div class="discovery-card-copy"><small>{{item.label}}</small><h3>{{item.name}}</h3><p>{{item.copy}}</p><ArrowUpRight :size="15"/></div></button></div></section>
      <section v-if="projects.length" class="home-projects"><div class="workspace-section-heading"><div><span class="workspace-kicker">THE SOLARA ECOSYSTEM</span><h2>More to explore.</h2></div></div><div class="home-project-grid"><button v-for="project in projects.slice(0,4)" :key="project.id" :disabled="!project.url" @click="emit('external',project.url)"><span class="project-sigil"><Layers3/></span><span><strong>{{project.name}}</strong><small>{{project.description}}</small></span><ArrowUpRight :size="14"/></button></div></section>
    </div>
  </div>
</section>
</template>
