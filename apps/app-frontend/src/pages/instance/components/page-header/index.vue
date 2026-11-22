<template>
	<section
		class="solara-instance-hero relative overflow-hidden rounded-[24px] border border-solid border-surface-5 bg-surface-3 px-6 py-5"
	>
		<div class="flex flex-wrap items-center justify-between gap-4">
			<!-- Left: Icon + Title + Metadata Chips -->
			<div class="flex min-w-0 flex-1 items-center gap-4">
				<div class="relative shrink-0">
					<Avatar
						:src="iconSrc"
						:alt="instance.name"
						size="72px"
						:tint-by="instance.id"
						pad-transparent-corners
						class="rounded-[20px] ring-1 ring-surface-5 shadow-md"
					/>
					<span
						v-if="playing"
						class="absolute -bottom-1 -right-1 flex size-4 items-center justify-center rounded-full bg-surface-3"
						aria-label="Instance running"
					>
						<span class="size-2.5 rounded-full bg-brand animate-pulse" />
					</span>
				</div>

				<div class="flex min-w-0 flex-col gap-1.5">
					<div class="flex flex-wrap items-center gap-2">
						<h1 class="m-0 truncate text-2xl font-extrabold tracking-tight text-contrast">
							{{ instance.name }}
						</h1>

						<span
							v-if="playing"
							class="inline-flex items-center gap-1.5 rounded-full border border-solid border-brand/40 bg-brand/15 px-2.5 py-0.5 text-xs font-bold text-brand"
						>
							<span class="size-1.5 rounded-full bg-brand animate-pulse" />
							Running
						</span>
						<span
							v-else-if="isInstalling"
							class="inline-flex items-center gap-1.5 rounded-full border border-solid border-brand/40 bg-brand/15 px-2.5 py-0.5 text-xs font-bold text-brand"
						>
							Installing...
						</span>
						<PageHeaderBadgeItem
							v-if="instance.launch_blocked"
							:icon="LockIcon"
							aria-label="Locked instance information"
							class="!border-orange !bg-highlight-orange !text-orange"
						>
							Locked
						</PageHeaderBadgeItem>
						<PageHeaderBadgeItem
							v-else-if="instance.shared_instance && !instance.quarantined"
							:tooltip="sharedInstanceTooltip"
							aria-label="Shared instance information"
							class="!border-blue !bg-highlight-blue !text-blue"
						>
							Shared
							<UnknownIcon class="block size-4 shrink-0 text-current" aria-hidden="true" />
						</PageHeaderBadgeItem>
					</div>

					<InstanceHeaderServerMetadata
						v-if="isServerInstance"
						:loading-server-ping="loadingServerPing"
						:players-online="playersOnline"
						:status-online="statusOnline"
						:hosting-status="hostingStatus"
						:ping="ping"
						:minecraft-server="minecraftServer"
						:show-instance-play-time="showInstancePlayTime"
						:playtime-label="playtimeLabel ?? 'Ready to launch'"
					/>

					<div v-else class="flex flex-wrap items-center gap-2 text-xs font-medium text-secondary">
						<span
							v-tooltip="'Mod loader and Minecraft version'"
							class="inline-flex items-center gap-1.5 rounded-full border border-solid border-surface-5 bg-surface-2 px-3 py-1 font-semibold text-contrast"
						>
							<TagIcon :tag="loaderDisplayName" enforce-type="loader" class="size-3.5 shrink-0" />
							<span>{{ loaderLabel }}</span>
						</span>

						<span
							v-if="showInstancePlayTime && playtimeLabel"
							v-tooltip="'Total playtime'"
							class="inline-flex items-center gap-1.5 rounded-full border border-solid border-surface-5 bg-surface-2 px-3 py-1"
						>
							<TimerIcon class="size-3.5 shrink-0 text-secondary" />
							<span>Played {{ playtimeLabel }}</span>
						</span>

						<span
							v-if="lastPlayedRelative"
							v-tooltip="'Last launched'"
							class="inline-flex items-center gap-1.5 rounded-full border border-solid border-surface-5 bg-surface-2 px-3 py-1"
						>
							<ClockIcon class="size-3.5 shrink-0 text-secondary" />
							<span>Last launched {{ lastPlayedRelative }}</span>
						</span>
						<span
							v-else
							class="inline-flex items-center gap-1.5 rounded-full border border-solid border-surface-5 bg-surface-2 px-3 py-1"
						>
							<ClockIcon class="size-3.5 shrink-0 text-secondary" />
							<span>Ready to launch</span>
						</span>
					</div>
				</div>
			</div>

			<!-- Right: Solara Quick Action Dock -->
			<div class="flex flex-wrap items-center gap-2 shrink-0">
				<Button
					v-if="playing"
					type="colored"
					color="red"
					size="lg"
					native-type="button"
					:disabled="stopping"
					class="!rounded-xl !px-5 !font-bold"
					@click="emit('stop')"
				>
					<StopCircleIcon />
					<span>{{ stopping ? 'Closing...' : 'Close Game' }}</span>
				</Button>
				<Button
					v-else-if="isInstalling"
					type="colored"
					color="brand"
					size="lg"
					native-type="button"
					disabled
					class="!rounded-xl !px-5 !font-bold"
				>
					<span>Installing...</span>
				</Button>
				<Button
					v-else-if="instance.launch_blocked"
					v-tooltip="'This instance has been locked'"
					type="colored"
					color="brand"
					size="lg"
					native-type="button"
					disabled
					class="!rounded-xl !px-5 !font-bold"
				>
					<PlayIcon />
					<span>Launch Game</span>
				</Button>
				<Button
					v-else-if="instance.install_stage !== 'installed'"
					type="colored"
					color="brand"
					size="lg"
					native-type="button"
					class="!rounded-xl !px-5 !font-bold"
					@click="emit('repair')"
				>
					<DownloadIcon />
					<span>Fix &amp; Verify</span>
				</Button>
				<SplitButton
					v-else-if="!loading && isServerInstance"
					type="colored"
					color="brand"
					size="lg"
					:options="serverPlayOptions"
					menu-label="Launch Game"
					@click="emit('playServer')"
				>
					<PlayIcon />
					<span>Join Server</span>
				</SplitButton>
				<Button
					v-else-if="!loading"
					type="colored"
					color="brand"
					size="lg"
					native-type="button"
					class="!rounded-xl !px-5 !font-bold"
					@click="emit('play')"
				>
					<PlayIcon />
					<span>Launch Game</span>
				</Button>
				<Button
					v-else
					type="colored"
					color="brand"
					size="lg"
					native-type="button"
					disabled
					class="!rounded-xl !px-5 !font-bold"
				>
					<span>Launching...</span>
				</Button>

				<Button
					v-tooltip="'Open instance game directory'"
					type="outlined"
					size="lg"
					native-type="button"
					class="!rounded-xl !px-3.5"
					@click="emit('openFolder')"
				>
					<FolderOpenIcon class="size-4" />
					<span class="hidden sm:inline">Open Directory</span>
				</Button>

				<Button
					v-tooltip="'Customize profile icon, loader, Java & RAM'"
					type="outlined"
					size="lg"
					native-type="button"
					class="!rounded-xl !px-3.5"
					@click="emit('settings')"
				>
					<SettingsIcon class="size-4" />
					<span class="hidden sm:inline">Customize</span>
				</Button>

				<TeleportOverflowMenu
					type="outlined"
					size="lg"
					label="More options"
					tooltip="More options"
					:options="moreActions"
					class="!rounded-xl"
				>
					<MoreVerticalIcon />
				</TeleportOverflowMenu>
			</div>
		</div>
	</section>
</template>

<script setup lang="ts">
import type { Labrinth, SharedInstances } from '@modrinth/api-client'
import {
	ClockIcon,
	DownloadIcon,
	ExternalIcon,
	EyeIcon,
	EyeOffIcon,
	FolderOpenIcon,
	LockIcon,
	MoreVerticalIcon,
	PackageIcon,
	PlayIcon,
	ReportIcon,
	SettingsIcon,
	StopCircleIcon,
	TimerIcon,
	UnknownIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	type ButtonMenuOption,
	commonMessages,
	formatLoaderLabel,
	PageHeaderBadgeItem,
	type ServerLoader,
	SplitButton,
	TagIcon,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed } from 'vue'

import type { GameInstance } from '@/helpers/types'

import InstanceHeaderServerMetadata from './instance-page-header-server-metadata.vue'

dayjs.extend(relativeTime)

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		iconSrc?: string | null
		isServerInstance?: boolean
		showInstancePlayTime?: boolean
		timePlayed?: number
		playing?: boolean
		loading?: boolean
		stopping?: boolean
		loadingServerPing?: boolean
		playersOnline?: number
		hostingStatus?: SharedInstances.Instances.v1.OnlineStatus
		statusOnline?: boolean
		ping?: number
		minecraftServer?: Labrinth.Projects.v3.Project['minecraft_server']
		canHideFromLibrary?: boolean
		hiddenFromLibrary?: boolean
	}>(),
	{
		iconSrc: null,
		isServerInstance: false,
		showInstancePlayTime: false,
		timePlayed: 0,
		playing: false,
		loading: false,
		stopping: false,
		loadingServerPing: false,
		playersOnline: undefined,
		statusOnline: false,
		hostingStatus: undefined,
		ping: undefined,
		minecraftServer: undefined,
		canHideFromLibrary: false,
		hiddenFromLibrary: false,
	},
)

const emit = defineEmits<{
	repair: []
	stop: []
	play: []
	playServer: []
	settings: []
	openFolder: []
	export: []
	createShortcut: []
	toggleHidden: []
	report: [event?: MouseEvent]
}>()

const installingStages = ['installing', 'pack_installing', 'minecraft_installing']

const { formatMessage } = useVIntl()

const isInstalling = computed(() => installingStages.includes(props.instance.install_stage))

const loaderDisplayName = computed(() => formatLoaderLabel(props.instance.loader) as ServerLoader)
const loaderLabel = computed(() =>
	[loaderDisplayName.value, props.instance.game_version].filter(Boolean).join(' · '),
)
const lastPlayedRelative = computed(() =>
	props.instance.last_played ? dayjs(props.instance.last_played).fromNow() : undefined,
)
const sharedInstanceTooltip = computed(() =>
	props.instance.shared_instance?.role === 'owner'
		? "This instance's content is being shared to other users."
		: "This instance's content is being managed by someone else.",
)
const playtimeLabel = computed(() => {
	const seconds = Math.floor(props.timePlayed)
	if (seconds <= 0) {
		return undefined
	}

	const hours = Math.floor(seconds / 3600)
	if (hours >= 1) {
		return `${hours}h`
	}

	const minutes = Math.floor(seconds / 60)
	if (minutes >= 1) {
		return `${minutes}m`
	}

	return `${seconds}s`
})
const serverPlayOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'launch_instance',
		label: 'Launch without joining',
		icon: PlayIcon,
		action: () => emit('play'),
	},
])
const moreActions = computed<ButtonMenuOption[]>(() => {
	const actions: ButtonMenuOption[] = [
		{
			id: 'open-folder',
			label: 'Open Game Folder',
			icon: FolderOpenIcon,
			action: () => emit('openFolder'),
		},
	]

	if (!props.instance.quarantined) {
		actions.push({
			id: 'export-mrpack',
			label: 'Export Pack (.mrpack)',
			icon: PackageIcon,
			action: () => emit('export'),
		})
	}

	if (!props.instance.launch_blocked) {
		actions.push({
			id: 'create-shortcut',
			label: 'Pin to Desktop',
			icon: ExternalIcon,
			action: () => emit('createShortcut'),
		})
	}

	if (props.canHideFromLibrary) {
		actions.push(
			props.hiddenFromLibrary
				? {
						id: 'show-in-library',
						label: 'Show in Library',
						icon: EyeIcon,
						action: () => emit('toggleHidden'),
					}
				: {
						id: 'hide-from-library',
						label: 'Hide from Library',
						icon: EyeOffIcon,
						action: () => emit('toggleHidden'),
					},
		)
	}

	if (props.instance.shared_instance?.role === 'member') {
		actions.push(
			{ type: 'divider' },
			{
				id: 'report-shared-instance',
				label: formatMessage(commonMessages.reportButton),
				icon: ReportIcon,
				tone: 'red',
				action: (event) => emit('report', event),
			},
		)
	}

	return actions
})
</script>
