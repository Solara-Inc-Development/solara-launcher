<template>
	<Button
		v-if="showUpdatePill"
		type="outlined"
		native-type="button"
		class="!h-[34px] text-sm !transition-[opacity,transform,background-color,color,filter] !duration-200 ease-out !text-brand [&>svg]:!text-inherit !shadow-[inset_0_0_0_1px_var(--color-brand)] hover:!bg-brand focus-visible:!bg-brand hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
		:disabled="isUpdateDownloading"
		:aria-busy="isUpdateDownloading"
		@click="handleUpdateClick"
	>
		<RefreshCwIcon v-if="!isUpdateDownloading" :class="{ 'animate-spin': restarting }" />
		<DownloadIcon v-else />
		<span v-if="isUpdateDownloading">
			{{ formatMessage(messages.downloadingUpdate) }}
			<span class="inline-block w-[3ch] text-right tabular-nums">{{ downloadPercent }}%</span>
		</span>
		<span v-else>{{ updateLabel }}</span>
	</Button>
</template>

<script setup lang="ts">
import { DownloadIcon, RefreshCwIcon } from '@modrinth/assets'
import { Button, defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import {
	appUpdateState,
	installAvailableAppUpdate,
} from '@/providers/app-update'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	restartToUpdate: {
		id: 'app.action-bar.restart-to-update',
		defaultMessage: 'Restart to Update',
	},
	downloadingUpdate: {
		id: 'app.action-bar.downloading-update',
		defaultMessage: 'Downloading update',
	},
})

const {
	downloading,
	downloadPercent,
	downloadProgress,
	finishedDownloading,
	isVisible: isUpdateVisible,
	restarting,
} = appUpdateState

const isUpdateDownloading = computed(
	() =>
		downloading.value ||
		(downloadProgress.value > 0 && downloadProgress.value < 1 && !finishedDownloading.value),
)
const showUpdatePill = computed(() => isUpdateVisible.value)
const updateLabel = computed(() =>
	isUpdateDownloading.value
		? formatMessage(messages.downloadingUpdate)
		: formatMessage(messages.restartToUpdate),
)

async function handleUpdateClick() {
	if (isUpdateDownloading.value) {
		return
	}
	await installAvailableAppUpdate()
}
</script>
