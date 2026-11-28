<script setup lang="ts">
import { ChevronLeftIcon, ChevronRightIcon, SolaraIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { RouterLink, useRouter } from 'vue-router'

import AppActionBar from './action-bar.vue'
import QuickSearchBar from './quick-search.vue'
import WindowControls from './window-controls.vue'

defineProps<{
	canNavigateBack: boolean
	canNavigateForward: boolean
	showSidebarToggle?: boolean
	sidebarToggled?: boolean
}>()

const emit = defineEmits<{
	'create-instance': []
	'open-settings': []
	'toggle-sidebar': []
}>()

const router = useRouter()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	goBack: { id: 'app.navigation.go-back', defaultMessage: 'Go back' },
	goForward: { id: 'app.navigation.go-forward', defaultMessage: 'Go forward' },
})
</script>

<template>
	<header
		data-tauri-drag-region
		class="app-title-bar relative z-[20] flex items-center justify-between gap-3 bg-bg-raised pl-2.5 [grid-area:status] [.mac-traffic-lights_&]:pl-20"
	>
		<!-- LEFT: Solara Crest + History Squircle Capsule -->
		<div data-tauri-drag-region class="flex shrink-0 items-center gap-2.5">
			<RouterLink
				to="/"
				data-tauri-drag-region-exclude
				class="group flex shrink-0 items-center gap-2 rounded-xl px-1.5 py-1 no-underline select-none transition-colors hover:bg-[color-mix(in_srgb,var(--color-contrast)_5%,transparent)]"
			>
				<SolaraIcon class="size-7 shrink-0 transition-transform duration-200 group-hover:scale-105" />
				<span class="font-extrabold text-[13.5px] tracking-tight text-contrast whitespace-nowrap">
					Solara Launcher
				</span>
			</RouterLink>

			<div
				data-tauri-drag-region-exclude
				class="solara-history-capsule flex shrink-0 items-center gap-0.5 rounded-xl p-0.5"
			>
				<button
					type="button"
					:aria-label="formatMessage(messages.goBack)"
					v-tooltip.bottom="formatMessage(messages.goBack)"
					:disabled="!canNavigateBack"
					class="size-7 rounded-[9px] border-0 bg-transparent flex items-center justify-center text-primary hover:text-contrast hover:bg-[color-mix(in_srgb,var(--color-contrast)_7%,transparent)] cursor-pointer transition-colors disabled:opacity-25 disabled:pointer-events-none"
					@click="router.back()"
				>
					<ChevronLeftIcon class="size-4" />
				</button>
				<button
					type="button"
					:aria-label="formatMessage(messages.goForward)"
					v-tooltip.bottom="formatMessage(messages.goForward)"
					:disabled="!canNavigateForward"
					class="size-7 rounded-[9px] border-0 bg-transparent flex items-center justify-center text-primary hover:text-contrast hover:bg-[color-mix(in_srgb,var(--color-contrast)_7%,transparent)] cursor-pointer transition-colors disabled:opacity-25 disabled:pointer-events-none"
					@click="router.forward()"
				>
					<ChevronRightIcon class="size-4" />
				</button>
			</div>
		</div>

		<!-- CENTER: Live Quick Search Bar (Replaces Modrinth Breadcrumbs!) -->
		<div data-tauri-drag-region class="flex flex-1 items-center justify-center min-w-0 px-2">
			<Suspense>
				<QuickSearchBar
					@create-instance="emit('create-instance')"
					@open-settings="emit('open-settings')"
				/>
			</Suspense>
		</div>

		<!-- RIGHT: Status / Updates Capsule + Seamless Window Controls -->
		<section data-tauri-drag-region class="flex shrink-0 items-center gap-2 pr-2">
			<div data-tauri-drag-region-exclude class="flex items-center">
				<Suspense>
					<AppActionBar />
				</Suspense>
			</div>
			<WindowControls />
		</section>
	</header>
</template>

<style scoped>
.app-title-bar {
	height: var(--top-bar-height);
	padding-right: calc(var(--window-controls-width, 0px) + 0.35rem);
}

.solara-history-capsule {
	background-color: color-mix(in srgb, var(--color-bg) 76%, transparent);
	border: 1px solid var(--surface-4);
	box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
}
</style>
