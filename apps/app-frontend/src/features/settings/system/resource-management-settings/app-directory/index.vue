<script setup lang="ts">
import {
	ClipboardCopyIcon,
	FolderOpenIcon,
	FolderSearchIcon,
	RefreshCwIcon,
	UndoIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Button,
	type ButtonMenuOption,
	defineMessages,
	injectNotificationManager,
	SplitButton,
	useVIntl,
} from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'

import { getOS, openPath } from '@/helpers/utils.js'

import ChangeDirectoryModal from './change-directory-modal.vue'
import { normalizePath } from './paths'
import { useAppDirectory } from './use-app-directory'
import { useCopyPath } from './use-copy-path'

const { formatMessage } = useVIntl()
const copyPath = useCopyPath()
const { handleError } = injectNotificationManager()
const { currentDir, defaultDir, pendingDir, busy, changeDirectory, cancelMove, restartNow } =
	useAppDirectory()

const messages = defineMessages({
	title: {
		id: 'app.settings.resource-management.app-directory.title',
		defaultMessage: 'App directory',
	},
	description: {
		id: 'app.settings.resource-management.app-directory.description',
		defaultMessage: 'Where Solara Launcher stores instances, worlds, and cached files.',
	},
	pending: {
		id: 'app.settings.resource-management.app-directory.pending',
		defaultMessage: 'Your app directory will move to {path} the next time Solara Launcher opens.',
	},
	cancelMove: {
		id: 'app.settings.resource-management.app-directory.cancel-move',
		defaultMessage: 'Cancel move',
	},
	moveNow: {
		id: 'app.settings.resource-management.app-directory.move-now',
		defaultMessage: 'Move and restart now',
	},
	openInFinder: {
		id: 'app.settings.resource-management.app-directory.open-in-finder',
		defaultMessage: 'Open in Finder',
	},
	openInExplorer: {
		id: 'app.settings.resource-management.app-directory.open-in-explorer',
		defaultMessage: 'Open in File Explorer',
	},
	openFolder: {
		id: 'app.settings.resource-management.app-directory.open-folder',
		defaultMessage: 'Open folder',
	},
	changeDirectory: {
		id: 'app.settings.resource-management.app-directory.change',
		defaultMessage: 'Change directory',
	},
	changeToDefault: {
		id: 'app.settings.resource-management.app-directory.change-to-default',
		defaultMessage: 'Change to default',
	},
	moreChangeOptions: {
		id: 'app.settings.resource-management.app-directory.more-change-options',
		defaultMessage: 'More directory options',
	},
})

const changeModal = ref<InstanceType<typeof ChangeDirectoryModal>>()
const os = ref<string | null>(null)

onMounted(async () => {
	os.value = await getOS()
})

const isDefaultDir = computed(() => {
	if (!currentDir.value || !defaultDir.value) return true
	const caseInsensitive = os.value === 'Windows'
	return (
		normalizePath(currentDir.value, caseInsensitive) ===
		normalizePath(defaultDir.value, caseInsensitive)
	)
})

const changeOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'change-to-default',
		label: formatMessage(messages.changeToDefault),
		icon: UndoIcon,
		action: (event) => {
			if (defaultDir.value) changeModal.value?.show(event, defaultDir.value)
		},
	},
])

const openFolderLabel = computed(() => {
	if (os.value === 'MacOS') return formatMessage(messages.openInFinder)
	if (os.value === 'Windows') return formatMessage(messages.openInExplorer)
	return formatMessage(messages.openFolder)
})

function openCurrentDir() {
	if (currentDir.value) {
		openPath(currentDir.value).catch(handleError)
	}
}
</script>

<template>
	<section
		class="solara-dir-card flex flex-col gap-4 rounded-2xl p-5"
		aria-labelledby="app-directory-title"
	>
		<div class="flex items-center gap-3">
			<div
				class="solara-dir-icon flex size-10 shrink-0 items-center justify-center rounded-xl"
				aria-hidden="true"
			>
				<FolderOpenIcon class="size-5 text-amber-400" />
			</div>
			<div class="flex min-w-0 flex-col gap-0.5">
				<h2 id="app-directory-title" class="m-0 text-base font-bold text-contrast">
					{{ formatMessage(messages.title) }}
				</h2>
				<p class="m-0 text-xs font-medium text-secondary">
					{{ formatMessage(messages.description) }}
				</p>
			</div>
		</div>

		<button
			v-if="currentDir"
			type="button"
			class="solara-path-pill flex w-full cursor-pointer items-center justify-between gap-3 rounded-xl border-0 px-3.5 py-3 text-left transition-colors"
			@click="copyPath(currentDir)"
		>
			<span class="min-w-0 truncate font-mono text-xs font-semibold text-contrast">
				{{ currentDir }}
			</span>
			<span class="flex shrink-0 items-center gap-1.5 text-xs font-semibold text-secondary">
				<ClipboardCopyIcon class="size-4" aria-hidden="true" />
				<span>Copy</span>
			</span>
		</button>

		<Admonition
			v-if="pendingDir"
			type="warning"
			:body="formatMessage(messages.pending, { path: pendingDir })"
		>
			<template #actions>
				<div class="flex flex-wrap gap-2">
					<Button type="outlined" :disabled="busy" @click="cancelMove">
						<XIcon aria-hidden="true" />
						{{ formatMessage(messages.cancelMove) }}
					</Button>
					<Button type="colored" color="red" :disabled="busy" @click="restartNow">
						<RefreshCwIcon aria-hidden="true" />
						{{ formatMessage(messages.moveNow) }}
					</Button>
				</div>
			</template>
		</Admonition>

		<div class="flex flex-wrap items-center gap-2.5">
			<Button :disabled="!currentDir" @click="openCurrentDir">
				<FolderOpenIcon aria-hidden="true" />
				{{ openFolderLabel }}
			</Button>
			<Button
				v-if="isDefaultDir"
				:disabled="!currentDir || busy"
				@click="changeModal?.show($event)"
			>
				<FolderSearchIcon aria-hidden="true" />
				{{ formatMessage(messages.changeDirectory) }}
			</Button>
			<SplitButton
				v-else
				:menu-label="formatMessage(messages.moreChangeOptions)"
				:options="changeOptions"
				:disabled="busy"
				@click="changeModal?.show($event)"
			>
				<FolderSearchIcon aria-hidden="true" />
				{{ formatMessage(messages.changeDirectory) }}
			</SplitButton>
		</div>

		<ChangeDirectoryModal
			v-if="currentDir"
			ref="changeModal"
			:current-dir="currentDir"
			:case-insensitive="os === 'Windows'"
			@confirm="changeDirectory"
		/>
	</section>
</template>

<style scoped>
.solara-dir-card {
	background: color-mix(in srgb, var(--surface-3) 86%, transparent);
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--surface-5) 55%, transparent);
}

:global(.light-mode) .solara-dir-card {
	background: #ffffff;
	box-shadow:
		0 2px 10px rgba(15, 23, 42, 0.04),
		inset 0 0 0 1px rgba(15, 23, 42, 0.07);
}

.solara-dir-icon {
	background: linear-gradient(135deg, rgba(251, 191, 36, 0.18), rgba(245, 158, 11, 0.1));
}

.solara-path-pill {
	background: color-mix(in srgb, var(--surface-2) 78%, transparent);
}

.solara-path-pill:hover {
	background: color-mix(in srgb, var(--surface-4) 80%, transparent);
}

:global(.light-mode) .solara-path-pill {
	background: #f4f7fb;
}

:global(.light-mode) .solara-path-pill:hover {
	background: #e9eff7;
}
</style>
