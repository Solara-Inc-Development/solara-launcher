<script setup lang="ts">
import TaskProgress from './TaskProgress.vue';
withDefaults(defineProps<{label?:string;count?:number}>(),{label:'Getting things ready…',count:3});
</script>
<template><section class="workspace-loading" aria-busy="true"><TaskProgress :title="label" detail="Getting the latest details for your workspace."/><div class="workspace-loading-cards" aria-hidden="true"><div v-for="n in count" :key="n" class="workspace-loading-card"><span class="loading-icon-placeholder"/><span/><span/></div></div></section></template>
