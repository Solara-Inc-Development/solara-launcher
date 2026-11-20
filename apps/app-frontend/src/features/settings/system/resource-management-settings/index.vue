<script setup>
import { defineMessages, injectNotificationManager, Slider, useVIntl } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { ref, watch } from 'vue'

import { appSettingsKeys, get, set } from '@/helpers/settings.ts'

import AppDirectorySettings from './app-directory/index.vue'
import ContentStorageSettings from './content-storage.vue'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const settings = ref(await get())

const messages = defineMessages({
	maximumConcurrentDownloadsTitle: {
		id: 'app.settings.resource-management.maximum-concurrent-downloads.title',
		defaultMessage: 'Maximum concurrent downloads',
	},
	maximumConcurrentDownloadsDescription: {
		id: 'app.settings.resource-management.maximum-concurrent-downloads.description',
		defaultMessage:
			'The maximum amount of files the launcher can download at the same time. Set this to a lower value if you have a poor internet connection. (Restart required to take effect)',
	},
	maximumConcurrentWritesTitle: {
		id: 'app.settings.resource-management.maximum-concurrent-writes.title',
		defaultMessage: 'Maximum concurrent writes',
	},
	maximumConcurrentWritesDescription: {
		id: 'app.settings.resource-management.maximum-concurrent-writes.description',
		defaultMessage:
			'The maximum amount of files the launcher can write to the disk at once. Set this to a lower value if you are frequently getting I/O errors. (Restart required to take effect)',
	},
})

const limitsMutation = useMutation({
	networkMode: 'always',
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (limits) => set({ ...(await get()), ...limits }),
	onError: handleError,
	onSettled: () => queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
})

watch(
	() => ({
		max_concurrent_downloads: settings.value.max_concurrent_downloads,
		max_concurrent_writes: settings.value.max_concurrent_writes,
	}),
	(limits) => limitsMutation.mutate(limits),
)
</script>

<template>
	<div class="flex flex-col gap-6">
		<ContentStorageSettings />
		<AppDirectorySettings />

		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.maximumConcurrentDownloadsTitle) }}
			</h2>
			<Slider
				id="max-downloads"
				v-model="settings.max_concurrent_downloads"
				:min="1"
				:max="10"
				:step="1"
			/>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.maximumConcurrentDownloadsDescription) }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.maximumConcurrentWritesTitle) }}
			</h2>
			<Slider
				id="max-writes"
				v-model="settings.max_concurrent_writes"
				:min="1"
				:max="50"
				:step="1"
			/>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.maximumConcurrentWritesDescription) }}
			</p>
		</div>
	</div>
</template>
