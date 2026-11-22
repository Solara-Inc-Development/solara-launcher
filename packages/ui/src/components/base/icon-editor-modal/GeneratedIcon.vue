<template>
	<div
		class="flex items-center justify-center overflow-hidden p-[13%]"
		:style="iconBackgroundStyle(config.background)"
	>
		<img
			v-if="symbolAsset"
			:src="symbolAsset"
			:alt="alt"
			class="size-full select-none object-contain drop-shadow-[0_4px_8px_rgba(0,0,0,0.35)]"
			draggable="false"
		/>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { getIconSymbolAsset } from './editor-catalog'
import { iconBackgroundStyle } from './generated-icon'
import type { IconConfig } from './types'

const props = withDefaults(
	defineProps<{
		config: IconConfig
		alt?: string
	}>(),
	{ alt: '' },
)

const symbolAsset = computed(() => getIconSymbolAsset(props.config.symbol))
</script>
