<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CodeIcon,
	CoffeeIcon,
	FolderOpenIcon,
	InfoIcon,
	SolaraIcon,
	SpinnerIcon,
	WindowIcon,
	WrenchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	formatLoaderLabel,
	NewModal,
	UnsavedChangesPopup,
} from '@modrinth/ui'
import type { PlatformTag } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { type Component, computed, ref, watch } from 'vue'

import { get_project_v3 } from '@/helpers/cache'
import { get_linked_modpack_info, getInstanceIconUrl } from '@/helpers/instance'
import { get_loader_versions } from '@/helpers/metadata'
import { get_game_versions, get_loaders } from '@/helpers/tags'
import type { GameInstance } from '@/helpers/types'
import { showInstanceInFolder } from '@/helpers/utils.js'
import { useUnsavedChangesHost } from '@/shared/unsaved-changes/use-unsaved-changes-host'

import GeneralSettings from './general-settings.vue'
import HooksSettings from './hooks-settings.vue'
import InstallationSettings from './installation-settings.vue'
import { provideInstanceSettings } from './instance-settings-context.ts'
import JavaSettings from './java-settings.vue'
import WindowSettings from './window-settings.vue'

interface SolaraSettingsTab {
	id: string
	label: string
	subtitle: string
	group: 'Profile' | 'Runtime & Launch'
	icon: Component
	content: Component
}

const queryClient = useQueryClient()

const props = defineProps<{
	instance: GameInstance
	offline?: boolean
}>()
const emit = defineEmits<{
	unlinked: []
}>()

const isMinecraftServer = ref(false)
const handleUnlinked = () => emit('unlinked')

const instanceRef = computed(() => props.instance)
const modalRef = ref<InstanceType<typeof NewModal> | null>(null)
const selectedTabIndex = ref(0)
let onAfterClose: (() => void) | undefined

const {
	popup: unsavedChangesPopup,
	original: originalUnsavedChangesState,
	modified: modifiedUnsavedChangesState,
	saving: savingUnsavedChanges,
	hasUnsavedChanges,
	canLeave: canLeaveCurrentTab,
	register: registerUnsavedChangesController,
	reset: resetUnsavedChanges,
	save: saveUnsavedChanges,
} = useUnsavedChangesHost()

function hide(callback?: () => void) {
	if (!canLeaveCurrentTab()) return
	onAfterClose = callback
	modalRef.value?.hide()
}

function handleModalHide() {
	const callback = onAfterClose
	onAfterClose = undefined
	callback?.()
}

function selectTab(index: number) {
	if (index === selectedTabIndex.value) return
	if (!canLeaveCurrentTab()) return
	selectedTabIndex.value = index
}

provideInstanceSettings({
	instance: instanceRef,
	offline: props.offline,
	isMinecraftServer,
	onUnlinked: handleUnlinked,
	closeModal: hide,
	registerUnsavedChangesController,
})

watch(
	() => props.instance,
	(instance) => {
		isMinecraftServer.value = false
		if (instance.link?.project_id) {
			get_project_v3(instance.link.project_id, 'must_revalidate')
				.then((project: Labrinth.Projects.v3.Project | undefined) => {
					if (project?.minecraft_server != null) {
						isMinecraftServer.value = true
					}
				})
				.catch(() => {})
		}
	},
	{ immediate: true },
)

const tabs: SolaraSettingsTab[] = [
	{
		id: 'profile',
		label: 'Profile & Icon',
		subtitle: 'Rename this profile, customize its icon, pick an update channel, or duplicate it.',
		group: 'Profile',
		icon: InfoIcon,
		content: GeneralSettings,
	},
	{
		id: 'loader',
		label: 'Loader & Version',
		subtitle: 'Switch Minecraft versions, change mod loaders, or repair game files.',
		group: 'Profile',
		icon: WrenchIcon,
		content: InstallationSettings,
	},
	{
		id: 'java',
		label: 'Java & RAM',
		subtitle: 'Allocate memory, choose a Java runtime, and configure JVM arguments.',
		group: 'Runtime & Launch',
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		id: 'window',
		label: 'Game Window',
		subtitle: 'Set default fullscreen mode and startup resolution for Minecraft.',
		group: 'Runtime & Launch',
		icon: WindowIcon,
		content: WindowSettings,
	},
	{
		id: 'hooks',
		label: 'Launch Commands',
		subtitle: 'Run custom pre-launch, wrapper, or post-exit commands around gameplay.',
		group: 'Runtime & Launch',
		icon: CodeIcon,
		content: HooksSettings,
	},
]

const activeTab = computed(() => tabs[selectedTabIndex.value] ?? tabs[0])

function getSupportedModpackLoaders() {
	return get_loaders().then((value: PlatformTag[]) =>
		value
			.filter((item) => item.supported_project_types.includes('modpack') || item.name === 'vanilla')
			.sort((a, b) => (a.name === 'vanilla' ? -1 : b.name === 'vanilla' ? 1 : 0)),
	)
}

// Preload
useQuery({
	networkMode: 'always' as const,
	queryKey: ['instance-settings', 'loader-versions', 'fabric'],
	queryFn: () => get_loader_versions('fabric'),
})
useQuery({
	networkMode: 'always' as const,
	queryKey: ['instance-settings', 'loader-versions', 'forge'],
	queryFn: () => get_loader_versions('forge'),
})
useQuery({
	networkMode: 'always' as const,
	queryKey: ['instance-settings', 'loader-versions', 'quilt'],
	queryFn: () => get_loader_versions('quilt'),
})
useQuery({
	networkMode: 'always' as const,
	queryKey: ['instance-settings', 'loader-versions', 'neo'],
	queryFn: () => get_loader_versions('neo'),
})
useQuery({
	networkMode: 'always' as const,
	queryKey: ['instance-settings', 'game-versions'],
	queryFn: get_game_versions,
})
useQuery({
	networkMode: 'always' as const,
	queryKey: ['instance-settings', 'loaders', 'modpack'],
	queryFn: getSupportedModpackLoaders,
})
useQuery({
	networkMode: 'always' as const,
	queryKey: computed(() => ['linkedModpackInfo', props.instance.id]),
	queryFn: () => get_linked_modpack_info(props.instance.id, 'stale_while_revalidate'),
	enabled: computed(() => !!props.instance.link?.project_id && !props.offline),
})

function show(tabIndex?: number) {
	if (typeof tabIndex === 'number' && tabIndex >= 0 && tabIndex < tabs.length) {
		selectedTabIndex.value = tabIndex
	} else {
		selectedTabIndex.value = 0
	}
	if (props.instance.link?.project_id) {
		queryClient.prefetchQuery({
			networkMode: 'always' as const,
			queryKey: ['linkedModpackInfo', props.instance.id],
			queryFn: () => get_linked_modpack_info(props.instance.id, 'stale_while_revalidate'),
		})
	}
	modalRef.value?.show()
}

defineExpose({ show, hide })
</script>

<template>
	<NewModal
		ref="modalRef"
		hide-header
		no-padding
		max-width="940px"
		width="min(940px, 92vw)"
		:on-hide="handleModalHide"
	>
		<div class="solara-inst-settings relative flex h-[min(640px,82vh)] w-full flex-col overflow-hidden bg-bg">
			<!-- Signature Solara top accent hairline -->
			<div class="solara-settings-hairline h-[2px] w-full shrink-0" />

			<div class="flex min-h-0 flex-1 flex-col md:flex-row">
				<!-- Left Custom Solara Profile Sidebar -->
				<aside class="solara-settings-sidebar flex w-full shrink-0 flex-col gap-3 p-4 md:w-[244px]">
					<!-- Profile Identity Badge -->
					<div class="flex items-center gap-3 rounded-2xl bg-bg-raised p-3">
						<Avatar
							:src="getInstanceIconUrl(instance.icon_path)"
							:alt="instance.name"
							size="44px"
							:tint-by="props.instance.id"
							pad-transparent-corners
							class="shrink-0 rounded-xl"
						/>
						<div class="flex min-w-0 flex-1 flex-col gap-1">
							<span class="truncate text-sm font-extrabold leading-tight text-contrast">
								{{ instance.name }}
							</span>
							<span
								class="inline-flex w-fit max-w-full items-center truncate rounded-md bg-brand/15 px-2 py-0.5 text-[10.5px] font-bold text-brand"
							>
								{{ formatLoaderLabel(instance.loader) }} · {{ instance.game_version }}
							</span>
						</div>
					</div>

					<!-- Categorized Custom Nav Tabs -->
					<nav class="flex flex-1 flex-col gap-4 overflow-y-auto pr-0.5" aria-label="Instance settings">
						<div v-for="group in (['Profile', 'Runtime & Launch'] as const)" :key="group" class="flex flex-col gap-1">
							<span class="px-2.5 text-[10.5px] font-bold uppercase tracking-wider text-secondary/75">
								{{ group }}
							</span>
							<button
								v-for="(tab, idx) in tabs"
								v-show="tab.group === group"
								:key="tab.id"
								type="button"
								:class="[
									'solara-settings-nav-item relative flex w-full cursor-pointer items-center gap-2.5 rounded-xl border-0 px-3 py-2 text-left text-[13px] font-semibold transition-all duration-150',
									selectedTabIndex === idx
										? 'solara-settings-nav-active text-brand'
										: 'bg-transparent text-secondary hover:bg-bg-raised hover:text-contrast',
								]"
								@click="selectTab(idx)"
							>
								<span
									v-if="selectedTabIndex === idx"
									class="absolute left-0 top-1/2 h-4 w-[3px] -translate-y-1/2 rounded-r-full bg-brand"
								/>
								<component :is="tab.icon" class="size-4 shrink-0" />
								<span class="truncate">{{ tab.label }}</span>
							</button>
						</div>
					</nav>

					<!-- Sidebar Footer: Quick Folder + Solara Crest -->
					<div class="solara-sidebar-footer flex flex-col gap-2.5 pt-3">
						<button
							type="button"
							class="flex w-full cursor-pointer items-center justify-center gap-2 rounded-xl border-0 bg-bg-raised px-3 py-2 text-xs font-semibold text-contrast transition-colors hover:bg-surface-4"
							@click="showInstanceInFolder(instance.id)"
						>
							<FolderOpenIcon class="size-3.5 text-brand" />
							<span>Open Game Folder</span>
						</button>
						<div class="flex items-center justify-center gap-1.5 text-[11px] font-semibold text-secondary/80">
							<SolaraIcon aria-hidden="true" class="size-3.5 text-brand" />
							<span>Solara Instance Controls</span>
						</div>
					</div>
				</aside>

				<!-- Right Pure-OLED Content Stage -->
				<div class="relative flex min-h-0 min-w-0 flex-1 flex-col bg-bg">
					<!-- Clean Top Stage Header -->
					<header class="solara-stage-header flex items-center justify-between gap-4 px-6 py-4">
						<div class="flex min-w-0 flex-col gap-0.5">
							<h2 class="m-0 truncate text-lg font-extrabold tracking-tight text-contrast">
								{{ activeTab.label }}
							</h2>
							<p class="m-0 truncate text-xs text-secondary">
								{{ activeTab.subtitle }}
							</p>
						</div>
						<button
							type="button"
							aria-label="Close instance settings"
							class="flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-xl border-0 bg-bg-raised text-secondary transition-colors hover:bg-surface-4 hover:text-contrast"
							@click="hide()"
						>
							<XIcon class="size-4" />
						</button>
					</header>

					<!-- Scrollable Active Tab Content -->
					<div class="min-h-0 flex-1 overflow-y-auto px-6 py-5">
						<Suspense>
							<component :is="activeTab.content" :key="`${instance.id}-${activeTab.id}`" />
							<template #fallback>
								<div class="flex items-center justify-center gap-2.5 py-16 text-secondary">
									<SpinnerIcon class="size-5 animate-spin text-brand" />
									<span class="text-sm font-medium">Loading settings...</span>
								</div>
							</template>
						</Suspense>
					</div>

					<!-- Floating Unsaved Changes Bar -->
					<div
						v-if="hasUnsavedChanges"
						class="solara-unsaved-wrap shrink-0 px-6 py-3"
					>
						<UnsavedChangesPopup
							ref="unsavedChangesPopup"
							:original="originalUnsavedChangesState"
							:modified="modifiedUnsavedChangesState"
							:saving="savingUnsavedChanges"
							inline
							@reset="resetUnsavedChanges"
							@save="saveUnsavedChanges"
						/>
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>

<style scoped>
.solara-settings-hairline {
	background: linear-gradient(90deg, transparent 0%, #3dd6f5 32%, #ffb55a 78%, transparent 100%);
}

.solara-settings-sidebar {
	background-color: color-mix(in srgb, var(--surface-2) 85%, var(--surface-1));
	border-right: 1px solid color-mix(in srgb, var(--surface-5) 32%, transparent);
}

.solara-settings-nav-active {
	background-color: color-mix(in srgb, var(--color-brand) 14%, transparent);
}

.solara-sidebar-footer {
	border-top: 1px solid color-mix(in srgb, var(--surface-5) 30%, transparent);
}

.solara-stage-header {
	border-bottom: 1px solid color-mix(in srgb, var(--surface-5) 30%, transparent);
}

.solara-unsaved-wrap {
	border-top: 1px solid color-mix(in srgb, var(--surface-5) 35%, transparent);
	background-color: color-mix(in srgb, var(--surface-2) 92%, transparent);
}
</style>
