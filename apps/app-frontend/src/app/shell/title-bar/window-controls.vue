<template>
	<Teleport to="body">
		<section
			v-if="showControls"
			ref="controlsEl"
			class="solara-window-controls pointer-events-auto fixed top-0 right-0 z-[10001] flex h-[3.25rem] items-center gap-1 px-2.5 pt-[2px]"
			data-tauri-drag-region-exclude
		>
			<IconButton
				type="quiet"
				label="Minimize window"
				class="solara-win-btn !h-7 !w-7 !min-w-7 !rounded-lg !p-0 hover:!bg-white/10"
				@click="getCurrentWindow().minimize()"
			>
				<MinimizeIcon class="!size-3.5" />
			</IconButton>
			<IconButton
				type="quiet"
				label="Toggle maximize window"
				class="solara-win-btn !h-7 !w-7 !min-w-7 !rounded-lg !p-0 hover:!bg-white/10"
				@click="getCurrentWindow().toggleMaximize()"
			>
				<RestoreIcon v-if="isMaximized" class="!size-3.5" />
				<MaximizeIcon v-else class="!size-3.5" />
			</IconButton>
			<IconButton
				type="quiet"
				label="Close window"
				class="close-button solara-win-btn !h-7 !w-7 !min-w-7 !rounded-lg !p-0 hover:!bg-red focus-visible:!bg-red"
				@click="handleClose"
			>
				<XIcon class="!size-3.5" />
			</IconButton>
		</section>
	</Teleport>
</template>

<script setup>
import { MaximizeIcon, MinimizeIcon, RestoreIcon, XIcon } from '@modrinth/assets'
import { IconButton } from '@modrinth/ui'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get as getSettings } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils.js'

const appSettings = useAppSettings()

const nativeDecorations = ref(false)
const isMaximized = ref(false)
const os = ref('')
const controlsEl = ref(null)

const alwaysShowAppControls = computed(() => appSettings.getFeatureFlag('always_show_app_controls'))

const showControls = computed(
	() =>
		alwaysShowAppControls.value ||
		(!nativeDecorations.value && (os.value === 'Windows' || os.value === 'Linux')),
)

function setWindowControlsWidth(width) {
	document.documentElement.style.setProperty('--window-controls-width', `${width}px`)
}

let resizeObserver
watch(controlsEl, (el) => {
	resizeObserver?.disconnect()
	resizeObserver = undefined
	if (!el) {
		setWindowControlsWidth(0)
		return
	}

	resizeObserver = new ResizeObserver(() => {
		setWindowControlsWidth(el.getBoundingClientRect().width)
	})
	resizeObserver.observe(el)
	setWindowControlsWidth(el.getBoundingClientRect().width)
})

onMounted(async () => {
	os.value = await getOS()

	await getSettings()
	nativeDecorations.value = false

	if (os.value !== 'MacOS') {
		await getCurrentWindow().setDecorations(false)
	}

	isMaximized.value = await getCurrentWindow().isMaximized()

	const unlisten = await getCurrentWindow().onResized(async () => {
		isMaximized.value = await getCurrentWindow().isMaximized()
	})

	onUnmounted(() => {
		unlisten()
	})
})

onUnmounted(() => {
	resizeObserver?.disconnect()
	document.documentElement.style.removeProperty('--window-controls-width')
})

const handleClose = async () => {
	await saveWindowState(StateFlags.ALL)
	await getCurrentWindow().close()
}
</script>
<style scoped>
.close-button:hover {
	color: var(--color-accent-contrast) !important;
}
</style>
