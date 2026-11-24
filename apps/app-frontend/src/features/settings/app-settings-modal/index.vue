<script setup lang="ts">
import {
	CodeIcon,
	CoffeeIcon,
	GaugeIcon,
	PaintbrushIcon,
	SolaraIcon,
	ToggleRightIcon,
	WindowIcon,
	WrenchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	injectNotificationManager,
	NewModal,
	ProgressBar,
	UnsavedChangesPopup,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { getVersion } from '@tauri-apps/api/app'
import { computed, ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import AppearanceSettings from '@/features/settings/display/appearance-settings.vue'
import FeatureFlagSettings from '@/features/settings/display/feature-flag-settings.vue'
import JavaAndMemorySettings from '@/features/settings/instance-defaults/java-and-memory-settings/index.vue'
import LaunchHooksSettings from '@/features/settings/instance-defaults/launch-hooks-settings.vue'
import WindowSettings from '@/features/settings/instance-defaults/window-settings.vue'
import type { InstanceDefaultsTab } from '@/features/settings/settings-actions'
import ResourceManagementSettings from '@/features/settings/system/resource-management-settings/index.vue'
import TroubleshootingSettings from '@/features/settings/system/troubleshooting-settings.vue'
import { appSettingsKeys, appSettingsQueryOptions, set } from '@/helpers/settings.ts'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'
import { useUnsavedChangesHost } from '@/shared/unsaved-changes/use-unsaved-changes-host'

import { provideAppSettingsModal } from './context'

const appSettings = useAppSettings()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()

const devModeCounter = ref(0)
const selectedTab = ref(0)
const contentScrollRef = ref<HTMLElement | null>(null)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const tabCategories = defineMessages({
	display: {
		id: 'settings.sidebar.label.display',
		defaultMessage: 'Display',
	},
	instanceDefaults: {
		id: 'app.settings.sidebar.label.instance-defaults',
		defaultMessage: 'Instance defaults',
	},
	system: {
		id: 'app.settings.sidebar.label.system',
		defaultMessage: 'System',
	},
})

const tabs = [
	{
		id: 'theme',
		name: defineMessage({
			id: 'app.settings.tabs.theme',
			defaultMessage: 'Theme & Presence',
		}),
		subtitle: 'Customize OLED appearance, Discord Rich Presence, and visual preferences.',
		category: tabCategories.display,
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		id: 'flags',
		name: commonSettingsMessages.featureFlags,
		subtitle: 'Toggle experimental developer flags for Solara Launcher.',
		category: tabCategories.display,
		icon: ToggleRightIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
	{
		id: 'java',
		name: defineMessage({
			id: 'app.settings.tabs.java-and-memory',
			defaultMessage: 'Java & Memory',
		}),
		subtitle: 'Configure default Java runtimes, RAM allocation, and JVM flags for new instances.',
		category: tabCategories.instanceDefaults,
		icon: CoffeeIcon,
		content: JavaAndMemorySettings,
	},
	{
		id: 'window',
		name: defineMessage({
			id: 'app.settings.tabs.window',
			defaultMessage: 'Game Window',
		}),
		subtitle: 'Set default fullscreen mode and window resolution when launching Minecraft.',
		category: tabCategories.instanceDefaults,
		icon: WindowIcon,
		content: WindowSettings,
	},
	{
		id: 'hooks',
		name: defineMessage({
			id: 'app.settings.tabs.launch-hooks',
			defaultMessage: 'Launch Hooks',
		}),
		subtitle: 'Run optional pre-launch, wrapper, or post-exit commands across instances.',
		category: tabCategories.instanceDefaults,
		icon: CodeIcon,
		content: LaunchHooksSettings,
	},
	{
		id: 'resources',
		name: defineMessage({
			id: 'app.settings.tabs.resources',
			defaultMessage: 'Storage & Resources',
		}),
		subtitle: 'Manage Solara Launcher folders, cache limits, and concurrent downloads.',
		category: tabCategories.system,
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		id: 'troubleshooting',
		name: defineMessage({
			id: 'app.settings.tabs.troubleshooting',
			defaultMessage: 'Troubleshooting',
		}),
		subtitle: 'Export debug logs, open backups, and diagnose launcher or connection issues.',
		category: tabCategories.system,
		icon: WrenchIcon,
		content: TroubleshootingSettings,
	},
]

const availableTabs = computed(() =>
	tabs.filter((tab) => !tab.developerOnly || appSettings.devMode),
)

const groupedTabs = computed(() => {
	const groups: Array<{ category: string; items: Array<{ tab: (typeof tabs)[number]; index: number }> }> = []
	availableTabs.value.forEach((tab, index) => {
		const category = formatMessage(tab.category)
		let group = groups.find((g) => g.category === category)
		if (!group) {
			group = { category, items: [] }
			groups.push(group)
		}
		group.items.push({ tab, index })
	})
	return groups
})

const activeTab = computed(() => availableTabs.value[selectedTab.value] ?? availableTabs.value[0])

const modal = ref<InstanceType<typeof NewModal> | null>(null)
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

function setTab(index: number) {
	if (index === selectedTab.value) return
	if (!canLeaveCurrentTab()) return
	const clamped = Math.max(0, Math.min(index, availableTabs.value.length - 1))
	selectedTab.value = clamped
	if (contentScrollRef.value) {
		contentScrollRef.value.scrollTop = 0
	}
}

function close(): boolean {
	if (!canLeaveCurrentTab()) return false
	modal.value?.hide()
	return true
}

function handleBeforeHide(): boolean {
	return canLeaveCurrentTab()
}

provideAppSettingsModal({
	close,
	registerUnsavedChangesController,
})

function openAtIndex(index?: number) {
	if (typeof index === 'number' && index >= 0 && index < availableTabs.value.length) {
		selectedTab.value = index
	} else if (selectedTab.value >= availableTabs.value.length) {
		selectedTab.value = 0
	}
	modal.value?.show()
}

function show() {
	openAtIndex(0)
}

function showProfile(): void {
	openAtIndex(0)
}

function showFeatureFlags(): void {
	const featureFlagsTabIndex = availableTabs.value.findIndex(
		(tab) => tab.content === FeatureFlagSettings,
	)
	openAtIndex(featureFlagsTabIndex >= 0 ? featureFlagsTabIndex : 0)
}

function showSyncedOptions(): void {
	openAtIndex(0)
}

function showInstanceDefaults(tab: InstanceDefaultsTab): void {
	const content = {
		java: JavaAndMemorySettings,
		window: WindowSettings,
		hooks: LaunchHooksSettings,
	}[tab]
	const tabIndex = availableTabs.value.findIndex((t) => t.content === content)
	openAtIndex(tabIndex >= 0 ? tabIndex : 0)
}

defineExpose({ show, showProfile, showFeatureFlags, showSyncedOptions, showInstanceDefaults })

const { progress, version: downloadingVersion } = injectAppUpdateDownloadProgress()

const { data: appInfo } = useQuery({
	queryKey: ['app-info'],
	networkMode: 'always',
	queryFn: async () => ({
		version: await getVersion(),
	}),
	staleTime: Infinity,
})

const developerModeMutation = useMutation({
	networkMode: 'always',
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (enabled: boolean) => {
		const settings = await queryClient.fetchQuery(appSettingsQueryOptions())
		const nextSettings = { ...settings, developer_mode: enabled }
		await set(nextSettings)
		return nextSettings
	},
	onMutate: () => queryClient.cancelQueries({ queryKey: appSettingsKeys.all }),
	onSuccess: (settings) => {
		const previousTab = availableTabs.value[selectedTab.value]

		queryClient.setQueryData(appSettingsKeys.all, settings)
		appSettings.devMode = settings.developer_mode

		const nextIdx = previousTab ? availableTabs.value.indexOf(previousTab) : -1
		selectedTab.value = nextIdx >= 0 ? nextIdx : 0
	},
	onError: handleError,
	onSettled: () => queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
})

function devModeCount() {
	if (developerModeMutation.isPending.value) return
	devModeCounter.value++
	if (devModeCounter.value > 5) {
		devModeCounter.value = 0
		developerModeMutation.mutate(!appSettings.devMode)
	}
}

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
	appVersion: {
		id: 'app.settings.app-version',
		defaultMessage: 'Solara Launcher {version}',
	},
	developerModeButtonLabel: {
		id: 'app.settings.developer-mode-button.label',
		defaultMessage: 'Toggle developer mode',
	},
})
</script>

<template>
	<NewModal
		ref="modal"
		hide-header
		no-padding
		max-width="58rem"
		width="min(58rem, calc(100vw - 3rem))"
		:on-hide="handleBeforeHide"
	>
		<div class="solara-app-settings-shell">
			<div class="solara-settings-accent-bar" aria-hidden="true" />

			<div class="solara-app-settings-body">
				<!-- Left Solara Navigation Sidebar -->
				<aside class="solara-app-settings-sidebar">
					<div class="solara-app-settings-brand-card">
						<div class="solara-brand-emblem">
							<SolaraIcon class="size-6 text-brand" />
						</div>
						<div class="min-w-0 flex-1">
							<div class="text-sm font-extrabold text-contrast tracking-tight truncate">
								Solara {{ formatMessage(commonMessages.settingsLabel) }}
							</div>
							<div class="text-[11px] font-semibold text-brand truncate">
								{{ appInfo?.version ? `V-${appInfo.version}` : 'Desktop Preferences' }}
							</div>
						</div>
					</div>

					<nav class="solara-app-settings-nav" aria-label="Solara Settings">
						<div v-for="group in groupedTabs" :key="group.category" class="flex flex-col gap-1">
							<span class="px-2.5 pt-2 pb-1 text-[10px] font-bold uppercase tracking-[0.12em] text-secondary/75">
								{{ group.category }}
							</span>
							<button
								v-for="{ tab, index } in group.items"
								:key="tab.id"
								type="button"
								class="solara-app-settings-tab"
								:class="{ 'is-active': selectedTab === index }"
								@click="setTab(index)"
							>
								<span v-if="selectedTab === index" class="solara-tab-indicator" aria-hidden="true" />
								<component :is="tab.icon" class="size-4 shrink-0" />
								<span class="truncate">{{ formatMessage(tab.name) }}</span>
							</button>
						</div>
					</nav>

					<div class="solara-app-settings-footer">
						<div v-if="progress > 0 && progress < 1" class="mb-2.5 rounded-xl bg-bg-raised p-2.5">
							<p class="text-[11px] font-semibold text-brand m-0 mb-1.5">
								{{ formatMessage(messages.downloading, { version: downloadingVersion }) }}
							</p>
							<ProgressBar :progress="progress" />
						</div>
						<p v-if="appSettings.devMode" class="text-[11px] text-brand font-bold m-0 mb-2">
							{{ formatMessage(developerModeEnabled) }}
						</p>
						<div class="flex items-center gap-2.5">
							<button
								type="button"
								:aria-label="formatMessage(messages.developerModeButtonLabel)"
								:disabled="developerModeMutation.isPending.value"
								class="p-1.5 rounded-lg bg-bg-raised border-0 cursor-pointer flex items-center justify-center transition-colors hover:bg-white/10"
								:class="appSettings.devMode ? 'text-brand' : 'text-secondary'"
								@click="devModeCount"
							>
								<SolaraIcon aria-hidden="true" class="size-4" />
							</button>
							<div v-if="appInfo" class="min-w-0">
								<p class="m-0 text-xs font-semibold text-contrast truncate">
									{{ formatMessage(messages.appVersion, { version: appInfo.version }) }}
								</p>
								<p class="m-0 text-[10px] text-secondary/70">Pure OLED Edition</p>
							</div>
						</div>
					</div>
				</aside>

				<!-- Right Pure-OLED Stage -->
				<section class="solara-app-settings-stage">
					<header class="solara-app-settings-header">
						<div class="min-w-0">
							<h2 class="m-0 text-lg font-extrabold text-contrast tracking-tight flex items-center gap-2">
								<component :is="activeTab.icon" class="size-5 text-brand shrink-0" />
								<span>{{ formatMessage(activeTab.name) }}</span>
							</h2>
							<p class="m-0 mt-0.5 text-xs text-secondary truncate">
								{{ activeTab.subtitle }}
							</p>
						</div>
						<button
							type="button"
							class="solara-close-btn"
							aria-label="Close settings"
							@click="close"
						>
							<XIcon class="size-4" />
						</button>
					</header>

					<div ref="contentScrollRef" class="solara-app-settings-content">
						<component :is="activeTab.content" />
					</div>

					<div v-if="hasUnsavedChanges" class="solara-unsaved-dock">
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
				</section>
			</div>
		</div>
	</NewModal>
</template>

<style scoped lang="scss">
.solara-app-settings-shell {
	position: relative;
	display: flex;
	flex-direction: column;
	background: #000000;
	border-radius: 20px;
	overflow: hidden;
	height: min(42rem, calc(100vh - 5rem));
	box-shadow:
		0 24px 70px rgba(0, 0, 0, 0.85),
		0 0 0 1px rgba(255, 255, 255, 0.08);
}

.solara-settings-accent-bar {
	height: 2px;
	width: 100%;
	flex-shrink: 0;
	background: linear-gradient(90deg, #3dd6f5 0%, #6ce4ff 52%, #f4c15d 100%);
	opacity: 0.9;
}

.solara-app-settings-body {
	display: grid;
	grid-template-columns: 244px minmax(0, 1fr);
	flex: 1;
	min-height: 0;
}

.solara-app-settings-sidebar {
	display: flex;
	flex-direction: column;
	background: #06080c;
	border-right: 1px solid rgba(255, 255, 255, 0.06);
	padding: 1rem 0.85rem;
	min-height: 0;
}

.solara-app-settings-brand-card {
	display: flex;
	align-items: center;
	gap: 0.7rem;
	padding: 0.65rem 0.75rem;
	border-radius: 14px;
	background: rgba(61, 214, 245, 0.07);
	border: 1px solid rgba(61, 214, 245, 0.16);
	margin-bottom: 0.85rem;
}

.solara-brand-emblem {
	width: 36px;
	height: 36px;
	border-radius: 11px;
	background: rgba(61, 214, 245, 0.12);
	display: flex;
	align-items: center;
	justify-content: center;
	flex-shrink: 0;
}

.solara-app-settings-nav {
	display: flex;
	flex-direction: column;
	gap: 0.45rem;
	overflow-y: auto;
	flex: 1;
	padding-right: 2px;
}

.solara-app-settings-tab {
	position: relative;
	display: flex;
	align-items: center;
	gap: 0.625rem;
	width: 100%;
	padding: 0.58rem 0.78rem;
	border: 0;
	border-radius: 11px;
	background: transparent;
	color: var(--color-secondary);
	font-size: 0.84rem;
	font-weight: 600;
	text-align: left;
	cursor: pointer;
	transition:
		background-color 140ms ease,
		color 140ms ease;

	&:hover {
		background: rgba(255, 255, 255, 0.05);
		color: var(--color-contrast);
	}

	&.is-active {
		background: rgba(61, 214, 245, 0.13);
		color: #7ee8ff;
	}
}

.solara-tab-indicator {
	position: absolute;
	left: 0;
	top: 20%;
	bottom: 20%;
	width: 3px;
	border-radius: 999px;
	background: #3dd6f5;
	box-shadow: 0 0 10px rgba(61, 214, 245, 0.7);
}

.solara-app-settings-footer {
	margin-top: auto;
	padding-top: 0.85rem;
	border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.solara-app-settings-stage {
	display: flex;
	flex-direction: column;
	min-height: 0;
	background: #000000;
	position: relative;
}

.solara-app-settings-header {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 1rem;
	padding: 1.15rem 1.5rem 0.95rem;
	border-bottom: 1px solid rgba(255, 255, 255, 0.06);
	flex-shrink: 0;
}

.solara-close-btn {
	width: 34px;
	height: 34px;
	border-radius: 10px;
	border: 0;
	background: rgba(255, 255, 255, 0.05);
	color: var(--color-secondary);
	display: flex;
	align-items: center;
	justify-content: center;
	cursor: pointer;
	flex-shrink: 0;
	transition:
		background-color 140ms ease,
		color 140ms ease;

	&:hover {
		background: rgba(255, 255, 255, 0.11);
		color: var(--color-contrast);
	}
}

.solara-app-settings-content {
	flex: 1;
	min-height: 0;
	overflow-y: auto;
	padding: 1.25rem 1.5rem 1.75rem;
}

.solara-unsaved-dock {
	padding: 0.75rem 1.25rem 1rem;
	border-top: 1px solid rgba(255, 255, 255, 0.06);
	background: rgba(6, 8, 12, 0.96);
	flex-shrink: 0;
}

@media (max-width: 720px) {
	.solara-app-settings-body {
		grid-template-columns: 1fr;
	}

	.solara-app-settings-sidebar {
		border-right: 0;
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
	}
}
</style>
