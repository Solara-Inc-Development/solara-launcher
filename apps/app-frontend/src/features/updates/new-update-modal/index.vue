<script setup lang="ts">
import { DownloadIcon, RefreshCwIcon, XIcon } from '@modrinth/assets'
import { Button, NewModal } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { appUpdateState, installAvailableAppUpdate } from '@/providers/app-update'

export interface SolaraUpdateNotification {
	version: string
	currentVersion?: string
	notes?: string | null
	publishedAt?: string | null
}

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const updateDetails = ref<SolaraUpdateNotification | null>(null)

const displayVersion = computed(() => {
	const raw = updateDetails.value?.version ?? appUpdateState.availableUpdate.value?.version ?? ''
	if (!raw) return 'Latest'
	return raw.startsWith('V-') || raw.startsWith('v') ? raw : `v${raw}`
})

const releaseNotes = computed(
	() =>
		updateDetails.value?.notes?.trim() ||
		'Includes the latest Solara Launcher improvements, security updates, and stability fixes.',
)

const isInstalling = computed(
	() => appUpdateState.downloading.value || appUpdateState.restarting.value,
)

function show(details?: SolaraUpdateNotification) {
	if (details) {
		updateDetails.value = details
	}
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

async function notifyForVersion(_version?: string, _pendingVersion?: string | null) {
	// Startup update check triggers show() when an update is available on GitHub Releases CDN.
}

async function handleUpdateNow() {
	await installAvailableAppUpdate()
	hide()
}

defineExpose({ show, hide, notifyForVersion })
</script>

<template>
	<NewModal ref="modal" header="New Version Available" max-width="30rem">
		<div class="flex flex-col gap-4">
			<p class="m-0 text-secondary">
				Solara Launcher <strong class="text-contrast">{{ displayVersion }}</strong> is ready to
				install.
			</p>
			<div
				class="max-h-48 overflow-y-auto rounded-xl border border-solid border-surface-5 bg-surface-3 p-3 text-sm text-primary whitespace-pre-wrap"
			>
				{{ releaseNotes }}
			</div>
			<div class="flex items-center justify-end gap-2 pt-2">
				<Button type="outlined" :disabled="isInstalling" @click="hide">
					<XIcon />
					I'll do it later
				</Button>
				<Button color="brand" :disabled="isInstalling" @click="handleUpdateNow">
					<RefreshCwIcon v-if="isInstalling" class="animate-spin" />
					<DownloadIcon v-else />
					Update Now
				</Button>
			</div>
		</div>
	</NewModal>
</template>
