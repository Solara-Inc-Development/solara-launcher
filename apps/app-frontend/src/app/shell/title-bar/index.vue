<script setup lang="ts">
import {
	ChevronLeftIcon,
	ChevronRightIcon,
	HomeIcon,
	LibraryIcon,
	PackageIcon,
	PaletteIcon,
	PlusIcon,
	SettingsIcon,
	SolaraIcon,
} from '@modrinth/assets'
import { commonMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { RouterLink, useRouter } from 'vue-router'

import NavButton from '@/components/ui/NavButton.vue'
import QuickInstanceSwitcher from '@/components/ui/QuickInstanceSwitcher.vue'

import AppActionBar from './action-bar.vue'
import WindowControls from './window-controls.vue'

withDefaults(
	defineProps<{
		canNavigateBack: boolean
		canNavigateForward: boolean
		showSkinTab?: boolean
		offline?: boolean
		showSidebarToggle?: boolean
		sidebarToggled?: boolean
	}>(),
	{
		showSkinTab: true,
		offline: false,
		showSidebarToggle: false,
		sidebarToggled: false,
	},
)

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
	home: { id: 'app.nav.home', defaultMessage: 'Home' },
	createNewInstance: { id: 'app.nav.create-new-instance', defaultMessage: 'Create new instance' },
})
</script>

<template>
	<header
		data-tauri-drag-region
		class="app-title-bar relative z-[2] flex items-center justify-between gap-3 bg-bg-raised pl-3 [grid-area:status] [.mac-traffic-lights_&]:pl-20"
	>
		<!-- LEFT: Solara Brand Crest + History Capsule -->
		<div data-tauri-drag-region class="flex shrink-0 items-center gap-2.5">
			<RouterLink
				to="/"
				data-tauri-drag-region-exclude
				class="solara-brand-badge group flex shrink-0 items-center gap-2 rounded-xl px-1.5 py-1 no-underline select-none transition-colors hover:bg-[color-mix(in_srgb,var(--color-contrast)_5%,transparent)]"
			>
				<SolaraIcon class="size-7 shrink-0 transition-transform duration-200 group-hover:scale-105" />
				<span class="font-extrabold text-[13.5px] tracking-tight text-contrast whitespace-nowrap">
					Solara Launcher
				</span>
			</RouterLink>

			<div
				data-tauri-drag-region-exclude
				class="solara-recessed-capsule flex shrink-0 items-center gap-0.5 rounded-xl p-0.5"
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

		<!-- CENTER: Lunar / Feather Style Top Navigation Deck -->
		<nav
			aria-label="Primary navigation"
			data-tauri-drag-region
			class="flex flex-1 items-center justify-center min-w-0"
		>
			<div
				data-tauri-drag-region-exclude
				class="solara-top-nav-deck flex items-center gap-1 rounded-2xl p-1"
			>
				<NavButton
					:to="'/'"
					:is-primary="(r) => r.path === '/'"
				>
					<HomeIcon />
					<span>{{ formatMessage(messages.home) }}</span>
				</NavButton>

				<NavButton
					:to="'/browse/modpack'"
					:is-primary="(r) => r.path.startsWith('/browse') && !r.query.i && !r.query.sid"
					:is-subpage="(r) => r.path.startsWith('/project') && !r.query.i && !r.query.sid"
				>
					<PackageIcon />
					<span>Modpacks</span>
				</NavButton>

				<NavButton
					:to="'/library'"
					:is-primary="(r) => r.path === '/library'"
					:is-subpage="
						(r) =>
							r.path.startsWith('/instance/') ||
							Boolean((r.path.startsWith('/browse') || r.path.startsWith('/project')) && r.query.i)
					"
				>
					<LibraryIcon />
					<span>Library</span>
				</NavButton>

				<NavButton
					v-if="showSkinTab"
					:to="'/skins'"
					:is-primary="(r) => r.path.startsWith('/skins')"
				>
					<PaletteIcon />
					<span>Skins</span>
				</NavButton>

				<Suspense>
					<QuickInstanceSwitcher />
				</Suspense>

				<div class="h-4 w-px bg-surface-5/80 mx-0.5 shrink-0" />

				<NavButton
					variant="cta"
					v-tooltip.bottom="formatMessage(messages.createNewInstance)"
					:to="() => emit('create-instance')"
					:disabled="offline"
				>
					<PlusIcon />
					<span class="hidden xl:inline">New</span>
				</NavButton>
			</div>
		</nav>

		<!-- RIGHT: Downloads / Status Pill + Settings + Window Controls -->
		<section data-tauri-drag-region class="flex shrink-0 items-center gap-2 pr-2">
			<div data-tauri-drag-region-exclude class="flex items-center">
				<Suspense>
					<AppActionBar />
				</Suspense>
			</div>

			<button
				type="button"
				data-tauri-drag-region-exclude
				:aria-label="formatMessage(commonMessages.settingsLabel)"
				v-tooltip.bottom="formatMessage(commonMessages.settingsLabel)"
				class="solara-recessed-capsule size-9 rounded-xl flex items-center justify-center text-secondary hover:text-contrast hover:border-brand/35 cursor-pointer transition-all active:scale-95"
				@click="emit('open-settings')"
			>
				<SettingsIcon class="size-[1.1rem]" />
			</button>

			<WindowControls />
		</section>
	</header>
</template>

<style scoped>
.app-title-bar {
	height: var(--top-bar-height);
	padding-right: calc(var(--window-controls-width, 0px) + 0.35rem);
}

.solara-top-nav-deck,
.solara-recessed-capsule {
	background-color: color-mix(in srgb, var(--color-bg) 76%, transparent);
	border: 1px solid var(--surface-4);
	box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
}
</style>
