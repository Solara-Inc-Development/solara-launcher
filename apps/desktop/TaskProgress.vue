<script setup lang="ts">
import {computed} from 'vue';
import {Download} from '../../packages/ui/icons';
const props=defineProps<{title:string;detail?:string;value?:number|null;paused?:boolean}>();
const fraction=computed(()=>typeof props.value==='number'&&Number.isFinite(props.value)?Math.max(0,Math.min(1,props.value)):undefined);
</script>
<template><div class="task-progress" :class="{'is-paused':paused}" role="status"><span class="task-progress-icon"><Download/></span><div class="task-progress-body"><div class="task-progress-heading"><strong>{{title}}</strong><span v-if="fraction!==undefined">{{Math.round(fraction*100)}}%</span></div><p v-if="detail">{{detail}}</p><progress v-if="!paused" :value="fraction" max="1" :aria-label="title"/></div></div></template>
