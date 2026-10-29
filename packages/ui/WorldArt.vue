<script setup lang="ts">
import {computed,ref,watch} from 'vue';
import landscapes from '../../public/art/landscapes.json';
const props=withDefaults(defineProps<{scene?:string;eager?:boolean;small?:boolean;decorative?:boolean}>(),{scene:'forest-valley',eager:false,small:false,decorative:true});
const emit=defineEmits<{settled:[]}>();
const item=computed(()=>landscapes.find(x=>x.id===`${props.scene}-v1`)||landscapes[0]);
const source=computed(()=>props.small?item.value.small:item.value.src);
const loaded=ref(false),failed=ref(false);
watch(source,()=>{loaded.value=false;failed.value=false});
async function ready(event:Event){try{await (event.target as HTMLImageElement).decode()}catch{}loaded.value=true;emit('settled')}
function fail(){failed.value=true;emit('settled')}
</script>
<template>
  <div class="world-art image-art" :class="{'is-loaded':loaded,'is-failed':failed}" :aria-busy="!loaded&&!failed">
    <img :key="source" :src="source" :alt="decorative?'':item.alt" :width="item.width" :height="item.height" :loading="eager?'eager':'lazy'" :fetchpriority="eager?'high':'auto'" decoding="async" @load="ready" @error="fail">
    <span v-if="!loaded&&!failed" class="art-loading" aria-hidden="true"><i></i></span>
    <span v-if="failed&&!decorative" class="art-fallback">Landscape unavailable</span>
  </div>
</template>
