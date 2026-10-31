<script setup lang="ts">
import {ref} from 'vue';
import {ArrowUpRight as Upload,RotateCcw} from '../../packages/ui/icons';
import InstanceAvatar from './InstanceAvatar.vue';
import {instanceIcons,uploadInstanceIcon,resetInstanceIcon} from './instance-icons';
const props=defineProps<{instance:any}>();
const picker=ref<HTMLInputElement|null>(null),busy=ref(false),error=ref(''),saved=ref('');
async function select(event:Event){
  const input=event.target as HTMLInputElement,file=input.files?.[0];if(!file)return;
  busy.value=true;error.value='';saved.value='';
  try {await uploadInstanceIcon(props.instance.id,file);saved.value='Your icon is saved on this device.'}catch(e){error.value=e instanceof Error?e.message:String(e)}finally{busy.value=false;input.value=''}
}
function reset(){try{resetInstanceIcon(props.instance.id);error.value='';saved.value='Default icon restored.'}catch{error.value='Could not save this change on your device.'}}
</script>
<template><section class="instance-appearance" aria-label="Instance icon"><InstanceAvatar :instance="instance"/><div><h3>Make it your own</h3><p>Use the {{instance.loader}} icon or add your own image.</p><div class="row"><button type="button" :disabled="busy" @click="picker?.click()"><Upload/>{{busy?'Saving icon…':'Upload icon'}}</button><button v-if="instanceIcons[instance.id]" type="button" :disabled="busy" @click="reset"><RotateCcw/>Reset</button></div><small>PNG, JPG or WebP · up to 5 MB · saved on this device</small><p v-if="error" class="icon-error" role="alert">{{error}}</p><p v-if="saved" class="icon-saved" role="status">{{saved}}</p><input ref="picker" type="file" accept="image/png,image/jpeg,image/webp" aria-label="Upload instance icon" class="sr-only" @change="select"></div></section></template>
