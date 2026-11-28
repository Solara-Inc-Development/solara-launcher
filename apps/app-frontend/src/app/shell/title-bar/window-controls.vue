<template>
	<Teleport to="body">
		<section
			v-if="showControls"
			ref="controlsEl"
			class="pointer-events-auto fixed top-0 right-0 z-[10001] flex h-[var(--top-bar-height,3.25rem)] items-center gap-1 bg-bg-raised px-2"
			data-tauri-drag-region-exclude
		>
			<IconButton
				type="quiet"
				label="Minimize window"
				class="relative before:absolute before:-inset-y-[9px] before:-inset-x-[6px] before:content-['']"
				@click="getCurrentWindow().minimize()"
			>
				<MinimizeIcon />
			</IconButton>
			<IconButton
				type="quiet"
				label="Toggle maximize window"
				class="relative before:absolute before:-inset-y-[9px] before:-inset-x-[6px] before:content-['']"
				@click="getCurrentWindow().toggleMaximize()"
			>
				<RestoreIcon v-if="isMaximized" />
				<MaximizeIcon v-else />
			</IconButton>
			<IconButton
				type="quiet"
				label="Close window"
				class="close-button relative before:absolute before:-inset-y-[9px] before:-left-[6px] before:-right-[9px] before:content-[''] hover:!bg-red focus-visible:!bg-red"
				@click="handleClose"
			>
				<XIcon />
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
