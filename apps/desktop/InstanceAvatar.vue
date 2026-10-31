<script setup lang="ts">
import {ref,watch} from 'vue';
import LoaderMark from '../../packages/ui/LoaderMark.vue';
import {instanceIcons} from './instance-icons';
const props=defineProps<{instance:{id:string;loader:string;name?:string}}>();
const failed=ref(false);
watch(()=>[props.instance?.id,instanceIcons.value[props.instance?.id]],()=>failed.value=false);
</script>
<template><span class="instance-avatar" :data-loader="instance.loader" aria-hidden="true"><img v-if="instanceIcons[instance.id]&&!failed" :src="instanceIcons[instance.id]" alt="" @error="failed=true"><template v-else><span class="avatar-tile tile-one"/><span class="avatar-tile tile-two"/><LoaderMark :loader="instance.loader" class="avatar-glyph"/></template></span></template>
