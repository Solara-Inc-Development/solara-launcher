<template>
	<div class="flex gap-2 items-center">
		<div v-if="offline" class="flex items-center gap-1">
			<UnplugIcon class="text-secondary" />
			<span class="text-sm text-contrast"> {{ formatMessage(messages.offline) }} </span>
		</div>
		<AppUpdateButton />
		<DownloadManager />
		<div
			class="flex border-solid border-surface-5 text-sm font-medium items-center gap-2 py-1.5 px-3 rounded-xl border"
		>
			<template v-if="selectedProcess">
				<OnlineIndicatorIcon />
				<div class="text-contrast flex items-center gap-2">
					<router-link
						v-tooltip="formatMessage(messages.viewInstance)"
						:to="`/instance/${encodeURIComponent(selectedProcess.instance.id)}`"
						class="hover:underline"
					>
						{{ selectedProcess.instance.name }}
					</router-link>
					<FloatingMenu
						v-if="currentProcesses.length > 1"
						placement="bottom"
						@open="showInstances = true"
						@close="showInstances = false"
					>
						<IconButton
							v-tooltip="
								showInstances
									? formatMessage(messages.hideMoreRunningInstances)
									: formatMessage(messages.showMoreRunningInstances)
							"
							class="!size-6"
							type="quiet"
							size="xs"
							:label="
								showInstances
									? formatMessage(messages.hideMoreRunningInstances)
									: formatMessage(messages.showMoreRunningInstances)
							"
						>
							<DropdownIcon :class="{ 'rotate-180': !!showInstances }" />
						</IconButton>
						<template #popper>
							<div class="flex w-[20rem] max-h-[24rem] flex-col gap-2 overflow-auto">
								<div
									v-for="process in currentProcesses"
									:key="process.uuid"
									class="flex w-full items-center gap-2 rounded-xl bg-surface-4 p-2 text-sm"
								>
									<button
										v-tooltip.left="
											process.uuid === selectedProcess.uuid
												? formatMessage(messages.primaryInstance)
												: formatMessage(messages.makePrimaryInstance)
										"
										class="flex flex-grow items-center gap-2"
										:class="{
											'active:scale-95 transition-transform': process.uuid !== selectedProcess.uuid,
										}"
										:disabled="process.uuid === selectedProcess.uuid"
										@click="selectProcess(process)"
									>
										<OnlineIndicatorIcon />
										<span class="mr-auto text-contrast flex items-center gap-2">
											{{ process.instance.name }}
											<StarIcon v-if="process.uuid === selectedProcess.uuid" class="text-orange" />
										</span>
									</button>
									<button
										v-tooltip="formatMessage(messages.stopInstance)"
										class="active:scale-95 flex"
										@click.stop="stop(process)"
									>
										<StopCircleIcon class="text-red size-5" />
									</button>
									<button
										v-tooltip="formatMessage(messages.viewLogs)"
										class="active:scale-95 flex"
										@click.stop="goToTerminal(process.instance.id)"
									>
										<TerminalSquareIcon class="text-secondary size-5" />
									</button>
								</div>
							</div>
						</template>
					</FloatingMenu>
				</div>
				<button
					v-tooltip="formatMessage(messages.stopInstance)"
					class="active:scale-95 flex"
					@click="stop(selectedProcess)"
				>
					<StopCircleIcon class="text-red size-5" />
				</button>
				<button
					v-tooltip="formatMessage(messages.viewLogs)"
					class="active:scale-95 flex"
					@click="goToTerminal()"
				>
					<TerminalSquareIcon class="text-secondary size-5" />
				</button>
			</template>
			<template v-else>
				<button
					type="button"
					class="flex items-center gap-2 bg-transparent border-0 cursor-pointer text-sm font-medium text-secondary hover:text-contrast transition-colors p-0 select-none"
					:disabled="isCheckingUpdates"
					@click="handleCheckForUpdates"
				>
					<RefreshCwIcon
						class="size-3.5 text-brand"
						:class="{ 'animate-spin': isCheckingUpdates }"
					/>
					<span v-if="isCheckingUpdates" class="text-contrast">Checking for updates...</span>
					<span v-else-if="hasAvailableUpdate" class="text-brand font-semibold flex items-center gap-1.5">
						<span class="size-2 rounded-full bg-brand animate-pulse" />
						Update Available
					</span>
					<span v-else-if="justCheckedUpToDate" class="text-brand font-medium">Up to date</span>
					<span v-else>Check for updates</span>
				</button>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	DropdownIcon,
	OnlineIndicatorIcon,
	RefreshCwIcon,
	StarIcon,
	StopCircleIcon,
	TerminalSquareIcon,
	UnplugIcon,
} from '@modrinth/assets'
import {
	defineMessages,
	FloatingMenu,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import AppUpdateButton from '@/components/ui/app-update-button/index.vue'
import DownloadManager from '@/components/ui/download-manager/index.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { trackEvent } from '@/helpers/analytics'
import { get_many as getInstances } from '@/helpers/instance'
import { get_all as getRunningProcesses, kill as killProcess } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'
import { appUpdateState } from '@/providers/app-update'

const {
	isCheckingUpdates,
	justCheckedUpToDate,
	availableUpdate,
	manualCheckTrigger,
	showUpdateModalTrigger,
} = appUpdateState

const hasAvailableUpdate = computed(() => !!availableUpdate.value)

async function handleCheckForUpdates() {
	if (isCheckingUpdates.value) return
	if (hasAvailableUpdate.value) {
		showUpdateModalTrigger.value?.()
		return
	}
	if (manualCheckTrigger.value) {
		await manualCheckTrigger.value()
		if (hasAvailableUpdate.value) {
			showUpdateModalTrigger.value?.()
		}
	}
}

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const router = useRouter()

const showInstances = ref(false)

interface RunningProcess {
	uuid: string
	instance_id: string
	instance: GameInstance
}

const messages = defineMessages({
	offline: {
		id: 'app.action-bar.offline',
		defaultMessage: 'Offline',
	},
	viewInstance: {
		id: 'app.action-bar.view-instance',
		defaultMessage: 'View instance',
	},
	showMoreRunningInstances: {
		id: 'app.action-bar.show-more-running-instances',
		defaultMessage: 'Show more running instances',
	},
	hideMoreRunningInstances: {
		id: 'app.action-bar.hide-more-running-instances',
		defaultMessage: 'Hide more running instances',
	},
	primaryInstance: {
		id: 'app.action-bar.primary-instance',
		defaultMessage: 'Primary instance',
	},
	makePrimaryInstance: {
		id: 'app.action-bar.make-primary-instance',
		defaultMessage: 'Make primary instance',
	},
	stopInstance: {
		id: 'app.action-bar.stop-instance',
		defaultMessage: 'Stop instance',
	},
	viewLogs: {
		id: 'app.action-bar.view-logs',
		defaultMessage: 'View logs',
	},
	noInstancesRunning: {
		id: 'app.action-bar.no-instances-running',
		defaultMessage: 'No instances running',
	},
})

const currentProcesses = ref<RunningProcess[]>([])
const selectedProcess = ref<RunningProcess | undefined>()

const refresh = async () => {
	const processes = ((await getRunningProcesses().catch((error) => {
		handleError(error)
		return []
	})) ?? []) as Array<{ uuid: string; instance_id: string }>
	const instanceIds = processes.map((process) => process.instance_id)
	const instances: GameInstance[] = await getInstances(instanceIds).catch((error) => {
		handleError(error)
		return []
	})

	currentProcesses.value = processes
		.map((process) => {
			const instance = instances.find((item) => process.instance_id === item.id)
			if (!instance) {
				return null
			}
			return {
				...process,
				instance,
			}
		})
		.filter((process): process is RunningProcess => process !== null)
	if (!selectedProcess.value || !currentProcesses.value.includes(selectedProcess.value)) {
		selectedProcess.value = currentProcesses.value[0]
	}
}

await refresh()

const offline = ref(!navigator.onLine)
function handleOffline() {
	offline.value = true
}
function handleOnline() {
	offline.value = false
}

onMounted(() => {
	window.addEventListener('offline', handleOffline)
	window.addEventListener('online', handleOnline)
})

useAppEvent('process', async () => {
	await refresh()
})

const stop = async (process: RunningProcess) => {
	try {
		await killProcess(process.uuid).catch(handleError)

		trackEvent('InstanceStop', {
			loader: process.instance.loader,
			game_version: process.instance.game_version,
			source: 'AppBar',
		})
	} catch (e) {
		console.error(e)
	}
	await refresh()
}

function goToTerminal(instanceId?: string) {
	const selectedInstanceId = instanceId ?? selectedProcess.value?.instance.id
	if (!selectedInstanceId) {
		return
	}
	router.push(`/instance/${encodeURIComponent(selectedInstanceId)}/logs`)
}

function selectProcess(process: RunningProcess) {
	selectedProcess.value = process
}

onBeforeUnmount(() => {
	window.removeEventListener('offline', handleOffline)
	window.removeEventListener('online', handleOnline)
})
</script>
