<script setup lang="ts">
import { ChevronLeftIcon, ChevronRightIcon, RightArrowIcon, SolaraIcon } from '@modrinth/assets'
import { defineMessages, IconButton, useVIntl } from '@modrinth/ui'
import { useRouter } from 'vue-router'

import AppActionBar from './action-bar.vue'
import Breadcrumbs from './breadcrumbs.vue'
import WindowControls from './window-controls.vue'

defineProps<{
	canNavigateBack: boolean
	canNavigateForward: boolean
	showSidebarToggle: boolean
	sidebarToggled: boolean
}>()

const emit = defineEmits<{
	'toggle-sidebar': []
}>()

const router = useRouter()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	goBack: { id: 'app.navigation.go-back', defaultMessage: 'Go back' },
	goForward: { id: 'app.navigation.go-forward', defaultMessage: 'Go forward' },
	nextImage: { id: 'app.navigation.next-image', defaultMessage: 'Next image' },
})
</script>

<template>
	<div
		data-tauri-drag-region
		class="app-title-bar relative z-[2] flex bg-[#000000] border-0 border-b border-solid border-white/[0.06] pl-1 [grid-area:status] [.mac-traffic-lights_&]:pl-20"
	>
		<div data-tauri-drag-region class="flex min-w-0 flex-1 items-center overflow-hidden px-2.5 py-1.5">
			<div class="flex shrink-0 items-center gap-2 px-1.5 pointer-events-none select-none">
				<SolaraIcon class="size-6 shrink-0" />
				<span class="font-extrabold text-[13px] tracking-tight text-contrast">Solara Launcher</span>
			</div>
			<div data-tauri-drag-region class="ml-2 flex shrink-0 items-center gap-1">
				<IconButton
					type="quiet"
					:label="formatMessage(messages.goBack)"
					class="!h-7 !min-w-7 !w-7 !rounded-lg !bg-white/[0.04] hover:!bg-white/[0.09] !border-0 !p-0 !opacity-100"
					:disabled="!canNavigateBack"
					@click="router.back()"
				>
					<ChevronLeftIcon
						class="!size-4 !text-primary"
						:class="{ 'opacity-25': !canNavigateBack }"
					/>
				</IconButton>
				<IconButton
					type="quiet"
					:label="formatMessage(messages.goForward)"
					class="!h-7 !min-w-7 !w-7 !rounded-lg !bg-white/[0.04] hover:!bg-white/[0.09] !border-0 !p-0 !opacity-100"
					:disabled="!canNavigateForward"
					@click="router.forward()"
				>
					<ChevronRightIcon
						class="!size-4 !text-primary"
						:class="{ 'opacity-25': !canNavigateForward }"
					/>
				</IconButton>
			</div>
			<Breadcrumbs />
		</div>
		<section data-tauri-drag-region class="flex shrink-0 ml-auto items-center">
			<IconButton
				v-if="showSidebarToggle"
				:type="sidebarToggled ? 'base' : 'quiet'"
				:label="formatMessage(messages.nextImage)"
				class="mr-2.5 transition-transform"
				:class="{ 'rotate-180': !sidebarToggled }"
				@click="emit('toggle-sidebar')"
			>
				<RightArrowIcon />
			</IconButton>
			<div class="flex mr-2.5">
				<Suspense>
					<AppActionBar />
				</Suspense>
			</div>
			<WindowControls />
		</section>
	</div>
</template>

<style scoped>
.app-title-bar {
	height: var(--top-bar-height);
	padding-right: var(--window-controls-width, 0px);
}
</style>
