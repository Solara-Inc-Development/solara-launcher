<script setup lang="ts">
import {
	DownloadIcon,
	FolderOpenIcon,
	LoaderCircleIcon,
	TrashIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	Toggle,
	useVIntl } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import {
	exportDebugInfo,
	exportingDebugInfo,
} from '@/components/ui/download-manager/debug-info-export'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { purge_cache_types } from '@/helpers/cache.js'
import { appSettingsKeys, get, set } from '@/helpers/settings.ts'
import { showAppDbBackupsFolder } from '@/helpers/utils.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const appSettings = useAppSettings()
const queryClient = useQueryClient()
const clearCacheConfirmModal = ref<InstanceType<typeof ConfirmModalWrapper> | null>(null)

const copyDetailsMutation = useMutation({
	networkMode: 'always',
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (enabled: boolean) => {
		const settings = await get()
		const nextSettings = { ...settings, always_show_copy_details: enabled }
		await set(nextSettings)
		return nextSettings
	},
	onMutate: () => queryClient.cancelQueries({ queryKey: appSettingsKeys.all }),
	onSuccess: (settings) => {
		appSettings.alwaysShowCopyDetails = settings.always_show_copy_details
		queryClient.setQueryData(appSettingsKeys.all, settings)
	},
	onError: handleError,
	onSettled: () => queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
})

const messages = defineMessages({
	debugInfoTitle: {
		id: 'app.settings.resource-management.debug-info.title',
		defaultMessage: 'Debug info',
	},
	debugInfoDescription: {
		id: 'app.settings.resource-management.debug-info.description',
		defaultMessage:
			'Export app logs and diagnostic data to a ZIP file to share with Solara Support.',
	},
	exportDebugInfo: {
		id: 'app.settings.resource-management.debug-info.export',
		defaultMessage: 'Export debug info',
	},
	appCacheTitle: {
		id: 'app.settings.resource-management.app-cache.title',
		defaultMessage: 'App cache',
	},
	clearCache: {
		id: 'app.settings.troubleshooting.clear-cache',
		defaultMessage: 'Clear cache',
	},
	clearCacheConfirmTitle: {
		id: 'app.settings.troubleshooting.clear-cache.confirm.title',
		defaultMessage: 'Clear the app cache?',
	},
	clearCacheConfirmDescription: {
		id: 'app.settings.resource-management.app-cache.confirm.description',
		defaultMessage: 'The app may load more slowly until the cache is rebuilt.',
	},
	appCacheDescription: {
		id: 'app.settings.resource-management.app-cache.description',
		defaultMessage:
			'Clear cached data and download it again from the catalog. The app may load more slowly until the cache is rebuilt.',
	},
	appDatabaseBackupsTitle: {
		id: 'app.settings.resource-management.app-database-backups.title',
		defaultMessage: 'App database backups',
	},
	openBackupsFolder: {
		id: 'app.settings.resource-management.app-database-backups.open-folder',
		defaultMessage: 'Open backups folder',
	},
	appDatabaseBackupsDescription: {
		id: 'app.settings.resource-management.app-database-backups.description',
		defaultMessage:
			'Backups of important app data are stored here in case you need to recover them later.',
	},
	alwaysShowCopyDetailsTitle: {
		id: 'app.settings.resource-management.always-show-copy-details.title',
		defaultMessage: 'Always show copy details',
	},
	alwaysShowCopyDetailsDescription: {
		id: 'app.settings.resource-management.always-show-copy-details.description',
		defaultMessage:
			'Show the Copy details action while an install is queued or running. It is always available for failed or interrupted installs.',
	},
})

async function clearCache() {
	await purge_cache_types([
		'project',
		'project_v3',
		'version',
		'user',
		'team',
		'organization',
		'file',
		'loader_manifest',
		'minecraft_manifest',
		'categories',
		'report_types',
		'loaders',
		'game_versions',
		'donation_platforms',
		'file_hash',
		'file_update',
		'search_results',
		'search_results_v3',
	]).catch(handleError)
}

function handleClearCacheClick() {
	if (appSettings.skipNonEssentialWarnings) {
		void clearCache()
		return
	}

	clearCacheConfirmModal.value?.show()
}

async function openDbBackupsFolder() {
	await showAppDbBackupsFolder().catch(handleError)
}

async function handleExportDebugInfo() {
	await exportDebugInfo(formatMessage(messages.exportDebugInfo)).catch(handleError)
}
</script>

<template>
	<div class="flex flex-col gap-3">
		<ConfirmModalWrapper
			ref="clearCacheConfirmModal"
			:title="formatMessage(messages.clearCacheConfirmTitle)"
			:description="formatMessage(messages.clearCacheConfirmDescription)"
			:has-to-type="false"
			:proceed-label="formatMessage(messages.clearCache)"
			:show-ad-on-close="false"
			@proceed="clearCache"
		/>
		<div class="rounded-2xl bg-bg-raised p-4 flex items-center justify-between gap-4">
			<div class="flex flex-col gap-1 min-w-0">
				<h2 class="m-0 text-sm font-bold text-contrast">
					{{ formatMessage(messages.appCacheTitle) }}
				</h2>
				<p class="m-0 text-xs text-secondary leading-relaxed">
					{{ formatMessage(messages.appCacheDescription) }}
				</p>
			</div>
			<Button id="clear-cache" class="shrink-0" @click="handleClearCacheClick">
				<TrashIcon aria-hidden="true" />
				{{ formatMessage(messages.clearCache) }}
			</Button>
		</div>
		<div class="rounded-2xl bg-bg-raised p-4 flex items-center justify-between gap-4">
			<div class="flex flex-col gap-1 min-w-0">
				<h2 class="m-0 text-sm font-bold text-contrast">
					{{ formatMessage(messages.appDatabaseBackupsTitle) }}
				</h2>
				<p class="m-0 text-xs text-secondary leading-relaxed">
					{{ formatMessage(messages.appDatabaseBackupsDescription) }}
				</p>
			</div>
			<Button id="open-db-backups-folder" class="shrink-0" @click="openDbBackupsFolder">
				<FolderOpenIcon aria-hidden="true" />
				{{ formatMessage(messages.openBackupsFolder) }}
			</Button>
		</div>
		<div class="rounded-2xl bg-bg-raised p-4 flex items-center justify-between gap-4">
			<div class="flex flex-col gap-1 min-w-0">
				<h2 class="m-0 text-sm font-bold text-contrast">
					{{ formatMessage(messages.debugInfoTitle) }}
				</h2>
				<p class="m-0 text-xs text-secondary leading-relaxed">
					{{ formatMessage(messages.debugInfoDescription) }}
				</p>
			</div>
			<Button
				id="export-debug-info"
				color="brand"
				class="shrink-0"
				:disabled="exportingDebugInfo"
				:loading="exportingDebugInfo"
				@click="handleExportDebugInfo"
			>
				<LoaderCircleIcon
					v-if="exportingDebugInfo"
					class="motion-safe:animate-spin"
					aria-hidden="true"
				/>
				<DownloadIcon v-else aria-hidden="true" />
				{{ formatMessage(messages.exportDebugInfo) }}
			</Button>
		</div>
		<div class="rounded-2xl bg-bg-raised p-4 flex items-center justify-between gap-4">
			<div class="flex flex-col gap-1 min-w-0">
				<h2 class="m-0 text-sm font-bold text-contrast">
					{{ formatMessage(messages.alwaysShowCopyDetailsTitle) }}
				</h2>
				<p class="m-0 text-xs text-secondary leading-relaxed">
					{{ formatMessage(messages.alwaysShowCopyDetailsDescription) }}
				</p>
			</div>
			<Toggle
				id="always-show-copy-details"
				:model-value="appSettings.alwaysShowCopyDetails"
				:disabled="copyDetailsMutation.isPending.value"
				:aria-label="formatMessage(messages.alwaysShowCopyDetailsTitle)"
				@update:model-value="copyDetailsMutation.mutate"
			/>
		</div>
	</div>
</template>
