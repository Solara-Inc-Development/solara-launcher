<template>
	<div v-if="false" />
</template>

<script setup lang="ts">
import type { GameInstance } from '@/helpers/types'

type SharedInstancePublishState = 'idle' | 'reviewing' | 'publishing'

defineProps<{
	instance: GameInstance
}>()

defineEmits<{
	published: []
	'state-change': [state: SharedInstancePublishState]
}>()

async function show(_e?: MouseEvent) {}

defineExpose({ show })
</script>
