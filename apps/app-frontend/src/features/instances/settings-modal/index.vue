<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CodeIcon,
	CoffeeIcon,
	FolderOpenIcon,
	InfoIcon,
	Settings2Icon,
	SolaraIcon,
	WindowIcon,
	WrenchIcon,
} from '@modrinth/assets'
import {
	Avatar,
	defineMessage,
	defineMessages,
	formatLoaderLabel,
	TabbedModal,
	type TabbedModalTab,
	UnsavedChangesPopup,
} from '@modrinth/ui'
import type { PlatformTag } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'

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
import SyncedOptionsSettings from './synced-options-settings.vue'
import WindowSettings from './window-settings.vue'

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
const tabbedModal = ref<InstanceType<typeof TabbedModal> | null>(null)
let onAfterClose: (() => void) | undefined

function hide(callback?: () => void) {
	onAfterClose = callback
	if (!tabbedModal.value?.hide()) onAfterClose = undefined
}

function handleAfterHide() {
	const callback = onAfterClose
	onAfterClose = undefined
	callback?.()
}

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

const tabCategories = defineMessages({
	instance: {
		id: 'solara.instance.settings.sidebar.profile',
		defaultMessage: 'Profile',
	},
	game: {
		id: 'solara.instance.settings.sidebar.runtime',
		defaultMessage: 'Runtime & Launch',
	},
})

const tabs = computed<TabbedModalTab[]>(() => [
	{
		name: defineMessage({
			id: 'solara.instance.settings.tabs.profile',
			defaultMessage: 'Profile & Icon',
		}),
		icon: InfoIcon,
		category: tabCategories.instance,
		content: GeneralSettings,
	},
	{
		name: defineMessage({
			id: 'solara.instance.settings.tabs.loader',
			defaultMessage: 'Loader & Version',
		}),
		icon: WrenchIcon,
		category: tabCategories.instance,
		content: InstallationSettings,
	},
	{
		name: defineMessage({
			id: 'solara.instance.settings.tabs.java',
			defaultMessage: 'Java & RAM',
		}),
		icon: CoffeeIcon,
		category: tabCategories.game,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'solara.instance.settings.tabs.window',
			defaultMessage: 'Game Window',
		}),
		icon: WindowIcon,
		category: tabCategories.game,
		content: WindowSettings,
	},
	{
		name: defineMessage({
			id: 'solara.instance.settings.tabs.commands',
			defaultMessage: 'Launch Commands',
		}),
		icon: CodeIcon,
		category: tabCategories.game,
		content: HooksSettings,
	},
	{
		name: defineMessage({
			id: 'solara.instance.settings.tabs.sync',
			defaultMessage: 'Cloud Sync',
		}),
		icon: Settings2Icon,
		category: tabCategories.game,
		content: SyncedOptionsSettings,
	},
])

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
	if (props.instance.link?.project_id) {
		queryClient.prefetchQuery({
			networkMode: 'always' as const,
			queryKey: ['linkedModpackInfo', props.instance.id],
			queryFn: () => get_linked_modpack_info(props.instance.id, 'stale_while_revalidate'),
		})
	}
	tabbedModal.value?.show(undefined, tabIndex)
}

defineExpose({ show, hide })
</script>
<template>
	<TabbedModal
		ref="tabbedModal"
		:tabs="tabs"
		header="Instance Settings"
		:on-after-hide="handleAfterHide"
		:before-hide="canLeaveCurrentTab"
		:before-tab-change="canLeaveCurrentTab"
		:floating-action-bar-shown="hasUnsavedChanges"
	>
		<template #title>
			<span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
				Instance Settings
			</span>
		</template>
		<template #breadcrumb>
			<Avatar
				:src="getInstanceIconUrl(instance.icon_path)"
				size="24px"
				:tint-by="props.instance.id"
				pad-transparent-corners
			/>
			<span class="truncate">{{ instance.name }}</span>
		</template>
		<template #sidebar-header>
			<div class="mb-2 flex items-center gap-3 rounded-2xl bg-bg-raised p-3">
				<Avatar
					:src="getInstanceIconUrl(instance.icon_path)"
					:alt="instance.name"
					size="40px"
					:tint-by="props.instance.id"
					pad-transparent-corners
					class="shrink-0 rounded-xl"
				/>
				<div class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="truncate text-sm font-bold leading-tight text-contrast">
						{{ instance.name }}
					</span>
					<span class="truncate text-[11px] font-semibold text-secondary">
						{{ formatLoaderLabel(instance.loader) }} · {{ instance.game_version }}
					</span>
				</div>
			</div>
		</template>
		<template #footer>
			<div class="mt-auto flex flex-col gap-2.5 pt-3">
				<button
					type="button"
					class="flex w-full cursor-pointer items-center justify-center gap-2 rounded-xl border-0 bg-bg-raised px-3 py-2 text-xs font-semibold text-contrast transition-colors hover:bg-surface-4"
					@click="showInstanceInFolder(instance.id)"
				>
					<FolderOpenIcon class="size-3.5 text-brand" />
					<span>Open Game Folder</span>
				</button>
				<div class="flex items-center gap-2 px-1 text-secondary">
					<SolaraIcon aria-hidden="true" class="size-4 text-brand" />
					<span class="text-[11px] font-medium">Solara Profile Controls</span>
				</div>
			</div>
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
	</TabbedModal>
</template>
