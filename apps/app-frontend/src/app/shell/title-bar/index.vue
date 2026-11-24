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
		<div class="solara-top-hairline" aria-hidden="true" />

		<!-- Left Brand Dock aligned with Left Sidebar -->
		<div
			data-tauri-drag-region
			class="solara-brand-zone flex shrink-0 items-center gap-2.5 px-3.5 h-full select-none"
		>
			<div class="solara-crest-badge pointer-events-none">
				<SolaraIcon class="size-5 shrink-0" />
			</div>
			<div class="flex flex-col leading-none min-w-0 pointer-events-none">
				<span class="font-extrabold text-[13px] tracking-tight text-contrast truncate">
					Solara Launcher
				</span>
				<span class="text-[9.5px] font-bold uppercase tracking-[0.14em] text-brand/80 mt-0.5">
					Minecraft Java
				</span>
			</div>
		</div>

		<!-- Navigation Controls & Pill Breadcrumbs -->
		<div data-tauri-drag-region class="flex min-w-0 flex-1 items-center gap-2.5 overflow-hidden px-3.5">
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

		<!-- Right Status Pill, Downloads, Update & Window Controls -->
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
	background: #05070b;
	border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

:global(.light-mode) .solara-title-bar {
	background: var(--color-raised-bg);
	border-bottom: 1px solid rgba(15, 23, 42, 0.08);
}

.solara-top-hairline {
	position: absolute;
	top: 0;
	left: 0;
	right: 0;
	height: 2px;
	background: linear-gradient(90deg, #3dd6f5 0%, #6ce4ff 52%, #f4c15d 100%);
	opacity: 0.85;
	pointer-events: none;
	z-index: 5;
}

.solara-brand-zone {
	width: var(--left-bar-width);
	border-right: 1px solid rgba(255, 255, 255, 0.06);
}

:global(.light-mode) .solara-brand-zone {
	border-right: 1px solid rgba(15, 23, 42, 0.08);
}

.solara-crest-badge {
	width: 30px;
	height: 30px;
	border-radius: 10px;
	background: rgba(61, 214, 245, 0.11);
	display: flex;
	align-items: center;
	justify-content: center;
	flex-shrink: 0;
}

.solara-nav-capsule {
	background: rgba(255, 255, 255, 0.045);
}

:global(.light-mode) .solara-nav-capsule {
	background: rgba(15, 23, 42, 0.06);
}
</style>
