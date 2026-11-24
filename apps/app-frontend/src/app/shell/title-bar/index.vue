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
	<header
		data-tauri-drag-region
		class="solara-title-bar relative z-[2] flex items-center [grid-area:status] [.mac-traffic-lights_&]:pl-20"
	>
		<!-- Left Brand & Compact Nav -->
		<div data-tauri-drag-region class="flex min-w-0 flex-1 items-center gap-3 overflow-hidden px-3.5">
			<div
				data-tauri-drag-region
				class="flex shrink-0 items-center gap-2 pointer-events-none select-none"
			>
				<SolaraIcon class="size-[26px] shrink-0" />
				<span class="font-extrabold text-[13.5px] tracking-tight text-contrast">
					Solara Launcher
				</span>
			</div>

			<div
				data-tauri-drag-region
				class="solara-nav-capsule flex shrink-0 items-center gap-0.5 rounded-full p-0.5"
			>
				<IconButton
					type="quiet"
					:label="formatMessage(messages.goBack)"
					class="!h-7 !min-w-7 !w-7 !rounded-full !border-0 !p-0 !opacity-100 hover:!bg-white/10"
					:disabled="!canNavigateBack"
					@click="router.back()"
				>
					<ChevronLeftIcon
						class="!size-3.5 !text-contrast"
						:class="{ 'opacity-25': !canNavigateBack }"
					/>
				</IconButton>
				<IconButton
					type="quiet"
					:label="formatMessage(messages.goForward)"
					class="!h-7 !min-w-7 !w-7 !rounded-full !border-0 !p-0 !opacity-100 hover:!bg-white/10"
					:disabled="!canNavigateForward"
					@click="router.forward()"
				>
					<ChevronRightIcon
						class="!size-3.5 !text-contrast"
						:class="{ 'opacity-25': !canNavigateForward }"
					/>
				</IconButton>
			</div>

			<Breadcrumbs />
		</div>

		<!-- Right Status Pill, Downloads, Update & Flush Window Controls -->
		<section data-tauri-drag-region class="flex shrink-0 ml-auto items-center h-full">
			<IconButton
				v-if="showSidebarToggle"
				:type="sidebarToggled ? 'base' : 'quiet'"
				:label="formatMessage(messages.nextImage)"
				class="mr-2 transition-transform"
				:class="{ 'rotate-180': !sidebarToggled }"
				@click="emit('toggle-sidebar')"
			>
				<RightArrowIcon />
			</IconButton>
			<div class="flex items-center mr-2">
				<Suspense>
					<AppActionBar />
				</Suspense>
			</div>
			<WindowControls />
		</section>
	</header>
</template>

<style scoped lang="scss">
.solara-title-bar {
	height: var(--top-bar-height);
	padding-right: var(--window-controls-width, 0px);
	background: var(--color-bg);
}

.solara-nav-capsule {
	background: #0a0e16;
	border: 1px solid rgba(255, 255, 255, 0.075);
}

:global(.light-mode) .solara-nav-capsule {
	background: #ffffff;
	border: 1px solid rgba(15, 23, 42, 0.08);
}
</style>
