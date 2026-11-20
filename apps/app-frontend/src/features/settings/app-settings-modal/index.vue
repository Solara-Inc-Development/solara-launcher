<script setup lang="ts">
import {
	CodeIcon,
	CoffeeIcon,
	EyeOffIcon,
	GaugeIcon,
	PaintbrushIcon,
	SolaraIcon,
	ToggleRightIcon,
	WindowIcon,
	WrenchIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	injectNotificationManager,
	ProgressBar,
	TabbedModal,
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

// TODO: Apply COMPONENT_STRUCTURE.md here and extract out common setting option components
const appSettings = useAppSettings()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()

const devModeCounter = ref(0)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const tabCategories = defineMessages({
	display: {
		id: 'settings.sidebar.label.display',
		defaultMessage: 'Display',
	},
	account: {
		id: 'settings.sidebar.label.account',
		defaultMessage: 'Account',
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
		name: defineMessage({
			id: 'app.settings.tabs.theme',
			defaultMessage: 'Theme',
		}),
		category: tabCategories.display,
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: commonSettingsMessages.featureFlags,
		category: tabCategories.display,
		icon: ToggleRightIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-and-memory',
			defaultMessage: 'Java and memory',
		}),
		category: tabCategories.instanceDefaults,
		icon: CoffeeIcon,
		content: JavaAndMemorySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.window',
			defaultMessage: 'Window',
		}),
		category: tabCategories.instanceDefaults,
		icon: WindowIcon,
		content: WindowSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.launch-hooks',
			defaultMessage: 'Launch hooks',
		}),
		category: tabCategories.instanceDefaults,
		icon: CodeIcon,
		content: LaunchHooksSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resources',
			defaultMessage: 'Resources',
		}),
		category: tabCategories.system,
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.troubleshooting',
			defaultMessage: 'Troubleshooting',
		}),
		category: tabCategories.system,
		icon: WrenchIcon,
		content: TroubleshootingSettings,
	},
]

const availableTabs = computed(() =>
	tabs.filter((tab) => !tab.developerOnly || appSettings.devMode),
)

const modal = ref<InstanceType<typeof TabbedModal> | null>(null)
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

function close(): boolean {
	return modal.value?.hide() ?? false
}

provideAppSettingsModal({
	close,
	registerUnsavedChangesController,
})

function show() {
	modal.value?.show()
}

function showProfile(): void {
	modal.value?.show()
}

function showFeatureFlags(): void {
	const featureFlagsTabIndex = availableTabs.value.findIndex(
		(tab) => tab.content === FeatureFlagSettings,
	)
	modal.value?.show(undefined, featureFlagsTabIndex >= 0 ? featureFlagsTabIndex : undefined)
}

function showSyncedOptions(): void {
	modal.value?.show()
}

function showInstanceDefaults(tab: InstanceDefaultsTab): void {
	const content = {
		java: JavaAndMemorySettings,
		window: WindowSettings,
		hooks: LaunchHooksSettings,
	}[tab]
	const tabIndex = availableTabs.value.findIndex((tab) => tab.content === content)
	modal.value?.show(undefined, tabIndex >= 0 ? tabIndex : undefined)
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
		const selectedTab = modal.value ? availableTabs.value[modal.value.selectedTab] : undefined

		queryClient.setQueryData(appSettingsKeys.all, settings)
		appSettings.devMode = settings.developer_mode

		if (modal.value) {
			const selectedTabIndex = selectedTab ? availableTabs.value.indexOf(selectedTab) : -1
			modal.value.setTab(selectedTabIndex >= 0 ? selectedTabIndex : 0)
		}
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
	macos: {
		id: 'app.settings.operating-system.macos',
		defaultMessage: 'macOS',
	},
	developerModeButtonLabel: {
		id: 'app.settings.developer-mode-button.label',
		defaultMessage: 'Toggle developer mode',
	},
})
</script>
<template>
	<TabbedModal
		ref="modal"
		:tabs="availableTabs"
		:header="formatMessage(commonMessages.settingsLabel)"
		:before-hide="canLeaveCurrentTab"
		:before-tab-change="canLeaveCurrentTab"
		:floating-action-bar-shown="hasUnsavedChanges"
	>
		<template #title>
			<span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
				{{ formatMessage(commonMessages.settingsLabel) }}
			</span>
		</template>
		<template #floating-action-bar>
			<UnsavedChangesPopup
				ref="unsavedChangesPopup"
				:original="originalUnsavedChangesState"
				:modified="modifiedUnsavedChangesState"
				:saving="savingUnsavedChanges"
				inline
				@reset="resetUnsavedChanges"
				@save="saveUnsavedChanges"
			/>
		</template>
		<template #footer>
			<div class="mt-auto pt-3 text-secondary text-sm">
				<div v-if="progress > 0 && progress < 1" class="mb-3">
					<p class="text-xs m-0 mb-2">
						{{ formatMessage(messages.downloading, { version: downloadingVersion }) }}
					</p>
					<ProgressBar :progress="progress" />
				</div>
				<p v-if="appSettings.devMode" class="text-brand font-semibold m-0 mb-2">
					{{ formatMessage(developerModeEnabled) }}
				</p>
				<div class="flex items-center gap-2.5">
					<button
						:aria-label="formatMessage(messages.developerModeButtonLabel)"
						:disabled="developerModeMutation.isPending.value"
						class="p-0 bg-transparent border-none cursor-pointer button-animation flex items-center justify-center"
						:class="{ 'text-brand': appSettings.devMode, 'text-secondary': !appSettings.devMode }"
						@click="devModeCount"
					>
						<SolaraIcon aria-hidden="true" class="w-5 h-5" />
					</button>
					<div v-if="appInfo">
						<p class="m-0 text-xs font-medium text-secondary">
							{{ formatMessage(messages.appVersion, { version: appInfo.version }) }}
						</p>
					</div>
				</div>
			</div>
		</template>
	</TabbedModal>
</template>
