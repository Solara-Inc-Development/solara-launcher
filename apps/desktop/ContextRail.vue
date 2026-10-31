<script setup lang="ts">
import {computed,ref,watch} from 'vue';
import {User,ChevronDown,ArrowRight,ArrowUpRight,ShieldCheck,Sliders,Layers3,RefreshCw,Server,BookOpen} from '../../packages/ui/icons';
const props=defineProps<{minecraft:any[],account:any,services:any[],apiState:string,loading:boolean,busy:boolean,desktop:boolean}>();
const emit=defineEmits(['account','navigate','solara-signin','external','refresh']);
const active=computed(()=>props.minecraft.find(a=>a.active)||null);
const avatarFailed=ref(false);
watch(()=>active.value?.id,()=>avatarFailed.value=false);
</script>

<template>
<aside class="context-rail" aria-label="Your accounts and services">
  <section class="context-account-section">
    <h2>Playing as</h2>
    <button class="playing-account-card" :disabled="busy||!desktop" @click="active?emit('navigate','launcher'):emit('account')">
      <span class="playing-avatar-wrap"><img v-if="active&&!avatarFailed" :src="'https://mc-heads.net/avatar/'+active.id+'/48'" alt="" @error="avatarFailed=true"><User v-else :size="24"/></span>
      <span class="playing-info"><strong>{{active?active.name:'Add Minecraft account'}}</strong><small>{{active?'Minecraft account':'Sign in with Microsoft'}}</small></span><ChevronDown :size="18"/>
    </button>
    <p class="context-account-note"><ShieldCheck/>Your Minecraft account stays on this device.</p>
  </section>
  <section class="context-solara-section">
    <template v-if="!account"><button class="text-link" :disabled="busy||!desktop" @click="emit('solara-signin')">Sign in to a Solara account</button><p>Connect your hosting, tools and Solara services.</p></template>
    <template v-else><span class="context-identity"><img src="/logo.svg" alt=""><span><strong>{{account.name||account.username}}</strong><small>Solara account</small></span><button aria-label="Open Solara account services" @click="emit('navigate','services')"><ArrowRight/></button></span></template>
  </section>
  <section class="context-news-section">
    <h2>Make it yours</h2>
    <button class="context-tip" @click="emit('navigate','settings')">
      <span class="context-tip-art" aria-hidden="true"><span class="tip-grid"></span><span class="tip-slider"><i/></span><span class="tip-slider second"><i/></span><span class="tip-art-icon"><Sliders/></span></span>
      <strong>Your setup. Your way.</strong><span>Choose your colours, Java and memory preferences.</span><span class="context-tip-link">Open preferences <ArrowUpRight/></span>
    </button>
  </section>
  <section class="context-hosting-section">
    <span class="context-hosting-brand"><img src="/logo.svg" alt="">solara services</span>
    <h2>A place for<br>your next <span>world.</span></h2>
    <p>Discover hosting, community tools and support.</p>
    <button class="context-hosting-art" aria-label="Explore Solara services" @click="emit('navigate','services')"><img src="/art/workshop-v1-small.webp" alt="An illustrated Minecraft workshop"><span><Server/> Explore services <ArrowUpRight/></span></button>
  </section>
  <section class="context-live-section">
    <div class="context-section-heading"><h2>Solara connection</h2><button title="Refresh Solara services" aria-label="Refresh Solara services" @click="emit('refresh')"><RefreshCw/></button></div>
    <p class="side-status"><span class="dot" :data-state="apiState"/>{{apiState==='connected'?'Live catalogue':apiState==='cached'?'Saved catalogue':loading?'Connecting…':'Catalogue unavailable'}}</p>
    <button v-for="service in services.slice(0,3)" :key="service.id" class="side-service-link" :disabled="!service.url" @click="emit('external',service.url)"><span class="service-sigil"><Layers3/></span><span><strong>{{service.name}}</strong><small>{{service.status||'Status not reported'}}</small></span><ArrowUpRight/></button>
  </section>
</aside>
</template>
