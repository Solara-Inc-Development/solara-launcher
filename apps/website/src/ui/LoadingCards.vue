<script setup lang="ts">
withDefaults(defineProps<{label?:string;count?:number}>(),{label:'Loading…',count:3});
</script>
<template><div class="loading-cards" role="status" aria-live="polite" aria-busy="true"><div class="loading-caption"><i class="loading-ring" aria-hidden="true"></i>{{label}}</div><div class="skeleton-row" aria-hidden="true"><div v-for="n in count" :key="n" class="skeleton-card"><i class="skeleton"></i><span class="skeleton"></span><span class="skeleton"></span></div></div></div></template>
