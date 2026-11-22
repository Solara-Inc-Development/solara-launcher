<script setup lang="ts">
import type { IconConfig } from '@modrinth/api-client'
import {
	getIconSymbolAsset,
	IconEditorModal,
	injectNotificationManager,
	renderStudioIconDataUrl,
} from '@modrinth/ui'
import { ref } from 'vue'

import { toError } from '@/helpers/errors'
import {
	cache_generated_icon,
	edit_generated_icon,
	edit_generated_icon_if_empty,
	get_recent_icon_configs,
} from '@/helpers/instance'

const props = defineProps<{
	instanceId?: string
	config?: IconConfig | null
}>()

const emit = defineEmits<{
	saved: [iconPath: string, config: IconConfig]
}>()

const { handleError } = injectNotificationManager()
const editor = ref<InstanceType<typeof IconEditorModal> | null>(null)

async function loadSymbolBytes(asset: string): Promise<number[]> {
	if (asset.startsWith('data:')) {
		const commaIndex = asset.indexOf(',')
		if (commaIndex === -1) throw new Error('Invalid rendered icon image.')
		const b64 = asset.slice(commaIndex + 1)
		const bin = atob(b64)
		const out = new Array<number>(bin.length)
		for (let i = 0; i < bin.length; i++) {
			out[i] = bin.charCodeAt(i)
		}
		return out
	}
	const response = await fetch(asset)
	if (!response.ok) throw new Error('Failed to load the icon symbol.')
	return Array.from(new Uint8Array(await response.arrayBuffer()))
}

async function save(config: IconConfig, asset: string): Promise<string> {
	const symbolBytes = await loadSymbolBytes(asset)
	return props.instanceId
		? edit_generated_icon(props.instanceId, config, symbolBytes)
		: cache_generated_icon(config, symbolBytes, true)
}

function onSaved(result: string | void, config: IconConfig) {
	if (result) emit('saved', result, config)
}

function show() {
	editor.value?.show()
}

function hide() {
	editor.value?.hide()
}

async function randomizeAndSave() {
	try {
		const selection = await editor.value?.randomize()
		if (!selection) return null
		const iconPath = await cache_generated_icon(
			selection.config,
			await loadSymbolBytes(selection.symbolAsset),
		)
		return { iconPath, config: selection.config }
	} catch (error) {
		handleError(toError(error))
		return null
	}
}

async function applyGeneratedIcon(instanceId: string, config: IconConfig) {
	const asset = getIconSymbolAsset(config.symbol)
	if (!asset) return false
	try {
		const primaryColor =
			config.background.type === 'color' ? config.background.value : config.background.top_color
		const secondaryColor =
			config.background.type === 'color' ? config.background.value : config.background.bottom_color
		const rendered = await renderStudioIconDataUrl({
			mode: 'icon',
			backdropMode: config.background.type === 'color' ? 'solid' : 'gradient',
			primaryColor,
			secondaryColor,
			angle: 135,
			pattern: 'none',
			frame: 'none',
			scale: 0.74,
			shadow: true,
			symbolAsset: asset,
		})
		return await edit_generated_icon_if_empty(instanceId, config, await loadSymbolBytes(rendered))
	} catch (error) {
		handleError(toError(error))
		return false
	}
}

defineExpose({ show, hide, randomize: randomizeAndSave, randomizeAndSave, applyGeneratedIcon })
</script>

<template>
	<IconEditorModal
		ref="editor"
		:config="config"
		:load-recents="get_recent_icon_configs"
		:save="save"
		@saved="onSaved"
	/>
</template>
