<template>
	<div class="flex flex-col gap-4" :aria-busy="ctx.navigating.value">
		<template v-if="ctx.flowType === 'instance'">
			<div class="flex flex-col gap-1">
				<span class="text-xs font-bold uppercase tracking-wider text-secondary">
					New Instance
				</span>
				<p class="m-0 text-sm text-secondary">
					Start with a ready-made modpack or build your own Minecraft setup from scratch.
				</p>
			</div>

			<div class="grid grid-cols-1 gap-3.5 sm:grid-cols-2">
				<!-- Card 1: Browse & download modpacks -->
				<button
					type="button"
					:disabled="ctx.navigating.value"
					class="solara-setup-card group flex min-h-[208px] cursor-pointer flex-col justify-between gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-2 p-5 text-left transition-all disabled:cursor-not-allowed disabled:opacity-60"
					@click="browseModpacks"
				>
					<div class="flex flex-col gap-3.5">
						<div class="flex items-start justify-between gap-2">
							<div
								class="flex size-11 shrink-0 items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-3 text-contrast transition-colors group-hover:border-brand group-hover:text-brand"
							>
								<CompassIcon class="size-5" aria-hidden="true" />
							</div>
							<div class="flex items-center gap-1.5">
								<span
									class="inline-flex items-center gap-1 rounded-md border border-solid border-surface-5 bg-surface-3 px-2 py-0.5 text-[10px] font-semibold text-secondary"
								>
									<span class="size-1.5 rounded-full bg-emerald-400" aria-hidden="true" />
									Modrinth
								</span>
								<span
									class="inline-flex items-center gap-1 rounded-md border border-solid border-surface-5 bg-surface-3 px-2 py-0.5 text-[10px] font-semibold text-secondary"
								>
									<span class="size-1.5 rounded-full bg-orange-400" aria-hidden="true" />
									CurseForge
								</span>
							</div>
						</div>

						<div class="flex flex-col gap-1.5">
							<span class="text-base font-bold leading-snug text-contrast">
								Browse and download modpacks
							</span>
							<span class="text-xs leading-relaxed text-secondary">
								Explore curated modpacks from Modrinth and CurseForge with one-click setup.
							</span>
						</div>
					</div>

					<div
						class="flex items-center justify-between border-0 border-t border-solid border-surface-5 pt-3 text-xs font-semibold text-secondary transition-colors group-hover:text-brand"
					>
						<span>Open catalog</span>
						<RightArrowIcon
							class="size-4 transition-transform group-hover:translate-x-0.5"
							aria-hidden="true"
						/>
					</div>
				</button>

				<!-- Card 2: Create your custom setup -->
				<button
					type="button"
					:disabled="ctx.navigating.value"
					class="solara-setup-card group flex min-h-[208px] cursor-pointer flex-col justify-between gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-2 p-5 text-left transition-all disabled:cursor-not-allowed disabled:opacity-60"
					@click="setSetupType('custom')"
				>
					<div class="flex flex-col gap-3.5">
						<div class="flex items-start justify-between gap-2">
							<div
								class="flex size-11 shrink-0 items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-3 text-contrast transition-colors group-hover:border-brand group-hover:text-brand"
							>
								<BoxesIcon class="size-5" aria-hidden="true" />
							</div>
							<span
								class="rounded-md border border-solid border-surface-5 bg-surface-3 px-2 py-0.5 text-[10px] font-semibold text-secondary"
							>
								Vanilla · Modded
							</span>
						</div>

						<div class="flex flex-col gap-1.5">
							<span class="text-base font-bold leading-snug text-contrast">
								Create your custom setup
							</span>
							<span class="text-xs leading-relaxed text-secondary">
								Pick any Minecraft version, mod loader, and design a custom icon in Icon Studio.
							</span>
						</div>
					</div>

					<div
						class="flex items-center justify-between border-0 border-t border-solid border-surface-5 pt-3 text-xs font-semibold text-secondary transition-colors group-hover:text-brand"
					>
						<span>Configure setup</span>
						<RightArrowIcon
							class="size-4 transition-transform group-hover:translate-x-0.5"
							aria-hidden="true"
						/>
					</div>
				</button>
			</div>
		</template>

		<template v-else>
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.knownProjectPrompt) }}
			</span>
			<Combobox
				ref="projectSearchCombobox"
				v-model="ctx.projectSearchProjectId.value"
				v-tooltip="ctx.finishDisabled.value ? ctx.finishDisabledTooltip.value : undefined"
				:options="ctx.projectSearchOptions.value"
				searchable
				show-search-icon
				:show-chevron="false"
				:disabled="ctx.finishDisabled.value || ctx.navigating.value"
				:search-placeholder="formatMessage(messages.searchProjectPlaceholder)"
				:no-options-message="
					searchLoading
						? formatMessage(commonMessages.loadingLabel)
						: formatMessage(messages.noResultsFound)
				"
				:disable-search-filter="true"
				@search-input="handleSearch"
			>
				<template #option-suffix="{ item }">
					<div
						class="flex shrink-0 items-center gap-1.5 text-sm font-semibold text-secondary opacity-0 transition-opacity group-hover/option:opacity-100 group-data-[focused=true]/option:opacity-100"
					>
						<span>
							{{
								formatMessage(
									isModpackOption(item.value)
										? messages.installModpack
										: commonMessages.continueButton,
								)
							}}
						</span>
						<DownloadIcon v-if="isModpackOption(item.value)" class="size-5 shrink-0" />
						<RightArrowIcon v-else class="size-5 shrink-0" />
					</div>
				</template>
			</Combobox>

			<div class="flex items-center gap-3">
				<div class="h-[1px] w-full flex-1 bg-surface-5" />
				<span class="text-sm text-secondary">{{ formatMessage(commonMessages.orLabel) }}</span>
				<div class="h-[1px] w-full flex-1 bg-surface-5" />
			</div>

			<span class="font-semibold text-contrast">
				{{ setupTypeTitle }}
			</span>

			<div class="flex flex-col gap-3">
				<BigOptionButton
					:disabled="ctx.navigating.value"
					:icon="CompassIcon"
					:title="formatMessage(messages.modpackBaseTitle)"
					:description="formatMessage(messages.modpackBaseDescription)"
					@click="browseModpacks"
				/>
				<BigOptionButton
					:disabled="ctx.navigating.value"
					:icon="UploadIcon"
					:title="formatMessage(messages.uploadModpackTitle)"
					:description="formatMessage(messages.uploadModpackDescription)"
					@click="triggerFileInput"
				/>
				<BigOptionButton
					:disabled="ctx.navigating.value"
					:icon="BoxesIcon"
					:title="formatMessage(messages.customSetupTitle)"
					:description="formatMessage(messages.customSetupDescription)"
					@click="setSetupType('custom')"
				/>
				<BigOptionButton
					v-if="ctx.flowType === 'server-onboarding' || ctx.flowType === 'reset-server'"
					v-tooltip="ctx.finishDisabled.value ? ctx.finishDisabledTooltip.value : undefined"
					:disabled="ctx.navigating.value || ctx.finishDisabled.value"
					:icon="AnvilIcon"
					:title="formatMessage(curseforgeMessages.title)"
					:description="formatMessage(curseforgeMessages.description)"
					@click="ctx.setSetupType('curseforge')"
				/>
				<BigOptionButton
					:disabled="ctx.navigating.value"
					:icon="BoxIcon"
					:title="formatMessage(messages.vanillaMinecraftTitle)"
					:description="formatMessage(messages.vanillaMinecraftDescription)"
					@click="setSetupType('vanilla')"
				/>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import {
	AnvilIcon,
	BoxesIcon,
	BoxIcon,
	BoxImportIcon,
	CompassIcon,
	DownloadIcon,
	RightArrowIcon,
	UploadIcon,
} from '@modrinth/assets'
import { commonMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, h, markRaw, onMounted, ref, watch } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'

import { injectFilePicker } from '../../../../providers'
import BigOptionButton from '../../../base/BigOptionButton.vue'
import Combobox from '../../../base/Combobox.vue'
import { injectCreationFlowContext } from '../creation-flow-context'
import { curseforgeMessages } from '../curseforge'

const debug = useDebugLogger('SetupTypeStage')
const ctx = injectCreationFlowContext()
const filePicker = injectFilePicker()
const { setSetupType: _setSetupType } = ctx
const { formatMessage } = useVIntl()

const searchLoading = ref(false)
const projectSearchCombobox = ref<{ $el: HTMLElement }>()

const messages = defineMessages({
	knownProjectPrompt: {
		id: 'creation-flow.modal.project.known-project.prompt',
		defaultMessage: 'Already know what you want to play?',
	},
	searchProjectPlaceholder: {
		id: 'creation-flow.modal.project.search.placeholder',
		defaultMessage: 'Search mods, modpacks, and more...',
	},
	noResultsFound: {
		id: 'creation-flow.modal.project.search.no-results',
		defaultMessage: 'No results found',
	},
	installModpack: {
		id: 'creation-flow.modal.project.search.install-modpack',
		defaultMessage: 'Install modpack',
	},
	createInstance: {
		id: 'creation-flow.modal.project.search.create-instance',
		defaultMessage: 'Create instance',
	},
	instanceTypeTitle: {
		id: 'creation-flow.modal.setup-type.title.instance',
		defaultMessage: 'Choose instance type',
	},
	installationTypeTitle: {
		id: 'creation-flow.modal.setup-type.title.installation',
		defaultMessage: 'Select installation type',
	},
	worldTypeTitle: {
		id: 'creation-flow.modal.setup-type.title.world',
		defaultMessage: 'Select world type',
	},
	customSetupTitle: {
		id: 'creation-flow.modal.setup-type.option.custom-setup.title',
		defaultMessage: 'Custom setup',
	},
	customSetupDescription: {
		id: 'creation-flow.modal.setup-type.option.custom-setup.description',
		defaultMessage: 'Start from scratch by picking a loader and game version.',
	},
	modpackBaseTitle: {
		id: 'creation-flow.modal.setup-type.option.modpack-base.title',
		defaultMessage: 'Start from a mod or modpack',
	},
	modpackBaseDescription: {
		id: 'creation-flow.modal.setup-type.option.modpack-base.description',
		defaultMessage: 'Choose a project and we’ll use its latest version.',
	},
	uploadModpackTitle: {
		id: 'creation-flow.modal.setup-type.option.upload-modpack.title',
		defaultMessage: 'Upload a modpack',
	},
	uploadModpackDescription: {
		id: 'creation-flow.modal.setup-type.option.upload-modpack.description',
		defaultMessage: 'Install a modpack from an .mrpack file on your device.',
	},
	importInstanceTitle: {
		id: 'creation-flow.modal.setup-type.option.import-instance.title',
		defaultMessage: 'Import instance',
	},
	importInstanceDescription: {
		id: 'creation-flow.modal.setup-type.option.import-instance.description',
		defaultMessage: 'Import an instance from Prism, CurseForge, or similar.',
	},
	vanillaMinecraftTitle: {
		id: 'creation-flow.modal.setup-type.option.vanilla-minecraft.title',
		defaultMessage: 'Vanilla Minecraft',
	},
	vanillaMinecraftDescription: {
		id: 'creation-flow.modal.setup-type.option.vanilla-minecraft.description',
		defaultMessage: 'Classic Minecraft with no mods or plugins.',
	},
})

const setupTypeTitle = computed(() => {
	if (ctx.flowType === 'instance') {
		return formatMessage(messages.instanceTypeTitle)
	}
	if (ctx.flowType === 'server-onboarding' || ctx.flowType === 'reset-server') {
		return formatMessage(messages.installationTypeTitle)
	}
	return formatMessage(messages.worldTypeTitle)
})

function isModpackOption(projectId: string) {
	return ctx.projectSearchHits.value[projectId]?.projectType === 'modpack'
}

function setSetupType(type: 'custom' | 'vanilla') {
	debug('selected:', type)
	_setSetupType(type)
}

function selectModpack() {
	debug('selected: modpack')
	_setSetupType('modpack')
}

function proceedWithModpack() {
	if (ctx.finishDisabled.value) return

	if (ctx.flowType === 'instance') {
		ctx.finish()
	} else {
		ctx.modal.value?.setStage('final-config')
	}
}

function browseModpacks() {
	if (ctx.finishDisabled.value) return

	selectModpack()
	ctx.browseModpacks()
}

async function triggerFileInput() {
	if (ctx.finishDisabled.value) return

	const picked = await filePicker.pickModpackFile({
		readFile: ctx.flowType !== 'instance',
	})
	if (!picked) return

	selectModpack()
	ctx.modpackFile.value = picked.file ?? null
	ctx.modpackFilePath.value = picked.path ?? null
	proceedWithModpack()
}

async function search(query: string) {
	try {
		if (!query.trim()) {
			ctx.projectSearchOptions.value = []
			return
		}
		const results = await ctx.searchProjects(query.trim(), 10)

		ctx.projectSearchHits.value = {}
		for (const hit of results.hits) {
			ctx.projectSearchHits.value[hit.project_id] = {
				title: hit.title,
				iconUrl: hit.icon_url,
				latestVersion: hit.latest_version,
				projectType: hit.project_type ?? 'modpack',
			}
		}

		ctx.projectSearchOptions.value = results.hits.map((hit) => ({
			label: hit.title,
			value: hit.project_id,
			icon: markRaw({
				setup: () => () =>
					h('img', {
						src: hit.icon_url,
						alt: hit.title,
						class: 'h-5 w-5 rounded',
					}),
			}),
		}))
	} catch (error) {
		debug('project search failed:', error)
		ctx.projectSearchOptions.value = []
	} finally {
		searchLoading.value = false
	}
}

async function handleSearch(query: string) {
	searchLoading.value = true
	await search(query)
}

onMounted(() => {
	ctx.projectSearchProjectId.value = undefined
	search('')
	setTimeout(() => {
		projectSearchCombobox.value?.$el.querySelector('input')?.focus()
	}, 150)
})

watch(
	() => ctx.projectSearchProjectId.value,
	async (projectId, oldProjectId) => {
		if (projectId === oldProjectId) return

		if (!projectId) return
		const hit = ctx.projectSearchHits.value[projectId]

		if (ctx.flowType === 'instance' || hit?.projectType !== 'modpack') {
			void ctx.selectProject(projectId, hit?.projectType ?? 'mod')
			return
		}

		try {
			const versions = await ctx.getProjectVersions(projectId)
			if (ctx.projectSearchProjectId.value !== projectId || versions.length === 0) return

			selectModpack()
			ctx.modpackSelection.value = {
				projectId,
				versionId: versions[0].id,
				name: hit?.title ?? '',
				iconUrl: hit?.iconUrl,
			}
			proceedWithModpack()
		} catch (error) {
			debug('failed to load project versions:', error)
		}
	},
)
</script>

<style scoped>
.solara-setup-card:hover {
	border-color: color-mix(in srgb, var(--color-brand) 52%, var(--surface-5));
	background: color-mix(in srgb, var(--surface-4) 55%, var(--surface-2));
	transform: translateY(-1px);
}
</style>
