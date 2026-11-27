<script setup>
import {
	DropdownIcon,
	EyeIcon,
	FolderOpenIcon,
	PlayIcon,
	SpinnerIcon,
	StopCircleIcon,
} from '@modrinth/assets'
import {
	Avatar,
	commonMessages,
	ContextMenu,
	defineMessages,
	FloatingMenu,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRoute, useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { useHiddenInstances } from '@/features/library/hidden-instances/use-hidden-instances'
import { trackEvent } from '@/helpers/analytics'
import { getInstanceIconUrl, kill, run } from '@/helpers/instance'
import { get_all } from '@/helpers/process'
import { showInstanceInFolder } from '@/helpers/utils'
import { instanceListQueryOptions } from '@/pages/instance/query-options'

const INLINE_MAX = 3
const DROPDOWN_MAX = 8

const { handleError } = injectNotificationManager()
const instancesQuery = useQuery(instanceListQueryOptions())
const router = useRouter()
const route = useRoute()
const instanceOptions = ref()
const runningInstances = ref([])
const moreMenuOpen = ref(false)

const { formatMessage } = useVIntl()
const { hiddenIds } = useHiddenInstances()

const allInstances = computed(() =>
	(instancesQuery.data.value ?? [])
		.filter((instance) => !hiddenIds.value.has(instance.id))
		.sort((a, b) => {
			const dateACreated = dayjs(a.created)
			const dateAPlayed = a.last_played ? dayjs(a.last_played) : dayjs(0)

			const dateBCreated = dayjs(b.created)
			const dateBPlayed = b.last_played ? dayjs(b.last_played) : dayjs(0)

			const dateA = dateACreated.isAfter(dateAPlayed) ? dateACreated : dateAPlayed
			const dateB = dateBCreated.isAfter(dateBPlayed) ? dateBCreated : dateBPlayed

			if (dateA.isSame(dateB)) {
				return a.name.localeCompare(b.name)
			}

			return dateB - dateA
		}),
)

const inlineInstances = computed(() => allInstances.value.slice(0, INLINE_MAX))
const dropdownInstances = computed(() => allInstances.value.slice(0, DROPDOWN_MAX))
const overflowCount = computed(() => Math.max(0, allInstances.value.length - INLINE_MAX))

function isInstanceRouteActive(instanceId) {
	return route.path.startsWith(`/instance/${encodeURIComponent(instanceId)}`)
}

function isInstanceRunning(instanceId) {
	return runningInstances.value.includes(instanceId)
}

await instancesQuery.suspense().catch(handleError)

useAppEvent('process', checkProcesses)

onMounted(() => {
	checkProcesses()
})

const messages = defineMessages({
	viewInstance: {
		id: 'app.quick-instance-switcher.view-instance',
		defaultMessage: 'View instance',
	},
	instanceActions: {
		id: 'app.quick-instance-switcher.actions.label',
		defaultMessage: 'Instance actions',
	},
	instanceLocked: {
		id: 'app.quick-instance-switcher.instance-locked',
		defaultMessage: 'This instance has been locked',
	},
	title: {
		id: 'app.quick-instance-switcher.title',
		defaultMessage: 'Recent instances',
	},
})

async function checkProcesses() {
	const processes = (await get_all().catch(handleError)) ?? []
	runningInstances.value = processes.map((process) => process.instance_id)
}

async function playInstance(instance) {
	if (instance.launch_blocked || instance.install_stage !== 'installed') return
	await run(instance.id)
		.catch((err) => handleSevereError(err, { instanceId: instance.id }))
		.finally(() => {
			trackEvent('InstanceStart', {
				loader: instance.loader,
				game_version: instance.game_version,
				source: 'QuickInstanceSwitcher',
			})
		})
}

async function stopInstance(instance) {
	await kill(instance.id).catch(handleError)
	trackEvent('InstanceStop', {
		loader: instance.loader,
		game_version: instance.game_version,
		source: 'QuickInstanceSwitcher',
	})
}

function openContextMenu(event, instance) {
	const playing = isInstanceRunning(instance.id)
	instanceOptions.value?.open(event, [
		playing
			? {
					id: 'stop',
					label: formatMessage(commonMessages.stopButton),
					icon: StopCircleIcon,
					tone: 'red',
					action: () => stopInstance(instance),
				}
			: {
					id: 'play',
					label: formatMessage(commonMessages.playButton),
					icon: PlayIcon,
					tone: 'brand',
					disabled: instance.launch_blocked || instance.install_stage !== 'installed',
					tooltip: instance.launch_blocked ? formatMessage(messages.instanceLocked) : undefined,
					action: () => playInstance(instance),
				},
		{ type: 'divider' },
		{
			id: 'open-instance',
			label: formatMessage(messages.viewInstance),
			icon: EyeIcon,
			action: () => router.push(`/instance/${encodeURIComponent(instance.id)}`),
		},
		{
			id: 'open-folder',
			label: formatMessage(commonMessages.openFolderButton),
			icon: FolderOpenIcon,
			action: () => showInstanceInFolder(instance.id),
		},
	])
}
</script>

<template>
	<div
		v-if="inlineInstances.length > 0"
		data-tauri-drag-region-exclude
		:aria-label="formatMessage(messages.title)"
		class="hidden lg:flex items-center gap-1"
	>
		<div class="h-4 w-px bg-surface-5/80 mx-0.5 shrink-0" />

		<RouterLink
			v-for="(instance, index) in inlineInstances"
			:key="instance.id"
			v-tooltip.bottom="instance.name"
			:to="`/instance/${encodeURIComponent(instance.id)}`"
			class="solara-quick-tile group relative size-8 rounded-xl items-center justify-center transition-all duration-200 select-none no-underline"
			:class="[
				index === 2 ? 'hidden xl:flex' : 'flex',
				isInstanceRouteActive(instance.id)
					? 'bg-brand/15 ring-1 ring-brand/55 shadow-[0_0_12px_rgba(61,214,245,0.18)]'
					: 'hover:bg-[color-mix(in_srgb,var(--color-contrast)_7%,transparent)]',
			]"
			@contextmenu.prevent.stop="(event) => openContextMenu(event, instance)"
		>
			<Avatar
				:src="getInstanceIconUrl(instance.icon_path)"
				size="22px"
				:tint-by="instance.id"
				:class="`rounded-[7px] transition-transform duration-200 ${
					instance.install_stage !== 'installed'
						? 'brightness-[0.3] scale-[0.88]'
						: 'group-hover:scale-105'
				}`"
				pad-transparent-corners
			/>
			<span
				v-if="isInstanceRunning(instance.id)"
				class="absolute -bottom-0.5 -right-0.5 size-2.5 rounded-full bg-emerald-400 ring-2 ring-[var(--color-bg)] animate-pulse pointer-events-none"
			/>
			<div
				v-else-if="instance.install_stage !== 'installed'"
				class="absolute inset-0 flex items-center justify-center z-10 pointer-events-none"
			>
				<SpinnerIcon class="animate-spin size-3.5 text-brand" />
			</div>
		</RouterLink>

		<FloatingMenu
			v-if="overflowCount > 0"
			placement="bottom"
			@open="moreMenuOpen = true"
			@close="moreMenuOpen = false"
		>
			<button
				type="button"
				v-tooltip.bottom="'Recent instances'"
				class="h-7 px-1.5 rounded-lg border-0 bg-transparent hover:bg-[color-mix(in_srgb,var(--color-contrast)_7%,transparent)] text-xs font-bold text-secondary hover:text-contrast flex items-center gap-0.5 cursor-pointer transition-colors"
			>
				<span>+{{ overflowCount }}</span>
				<DropdownIcon class="size-3.5 transition-transform" :class="{ 'rotate-180': moreMenuOpen }" />
			</button>
			<template #popper="{ hide }">
				<div class="flex w-[17rem] max-h-[20rem] flex-col gap-1 overflow-y-auto p-1">
					<div class="px-2 py-1 text-[11px] font-bold uppercase tracking-wider text-secondary">
						Recent Instances
					</div>
					<div
						v-for="inst in dropdownInstances"
						:key="inst.id"
						class="flex items-center justify-between gap-2 rounded-xl px-2 py-1.5 hover:bg-surface-4 transition-colors"
					>
						<RouterLink
							:to="`/instance/${encodeURIComponent(inst.id)}`"
							class="flex items-center gap-2.5 min-w-0 flex-1 no-underline text-contrast"
							@click="hide()"
						>
							<Avatar
								:src="getInstanceIconUrl(inst.icon_path)"
								size="24px"
								:tint-by="inst.id"
								class="rounded-md shrink-0"
								pad-transparent-corners
							/>
							<div class="flex flex-col min-w-0">
								<span class="text-xs font-bold text-contrast truncate">{{ inst.name }}</span>
								<span class="text-[11px] text-secondary truncate capitalize">
									{{ inst.loader }} {{ inst.game_version }}
								</span>
							</div>
						</RouterLink>
						<button
							v-if="isInstanceRunning(inst.id)"
							type="button"
							v-tooltip="'Stop'"
							class="size-7 rounded-lg bg-red/15 hover:bg-red/25 border-0 text-red flex items-center justify-center cursor-pointer shrink-0"
							@click.stop="stopInstance(inst)"
						>
							<StopCircleIcon class="size-4" />
						</button>
						<button
							v-else
							type="button"
							v-tooltip="'Play'"
							:disabled="inst.launch_blocked || inst.install_stage !== 'installed'"
							class="size-7 rounded-lg bg-brand/15 hover:bg-brand hover:text-[var(--color-accent-contrast)] border-0 text-brand flex items-center justify-center cursor-pointer shrink-0 disabled:opacity-40"
							@click.stop="playInstance(inst)"
						>
							<PlayIcon class="size-3.5" />
						</button>
					</div>
				</div>
			</template>
		</FloatingMenu>

		<ContextMenu ref="instanceOptions" :label="formatMessage(messages.instanceActions)" />
	</div>
</template>
