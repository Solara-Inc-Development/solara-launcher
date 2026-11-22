<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ArrowLeftIcon,
	CheckIcon,
	DownloadIcon,
	ExternalIcon,
	GridIcon,
	ListIcon,
	PackageIcon,
	SearchIcon,
	SpinnerIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import { Avatar, Button, injectNotificationManager, Pagination } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import {
	type CurseForgeContentType,
	type CurseForgeFile,
	type CurseForgeModpackSummary,
	type CurseForgeSort,
	getCurseForgeModpackDetail,
	installCurseForgeContent,
	searchCurseForgeModpacks,
} from '@/features/modpack-installer/curseforge-api'
import { get_project_versions, get_search_results_v3 } from '@/helpers/cache.js'
import {
	get_installed_project_ids as getInstalledProjectIds,
	getInstanceIconUrl,
} from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { instanceKeys } from '@/pages/instance/query-options'
import { injectContentInstall } from '@/providers/content-install'

dayjs.extend(relativeTime)

type ProviderSource = 'modrinth' | 'curseforge'
type ContentKind = Exclude<CurseForgeContentType, 'modpack'>
type SortOption = 'relevance' | 'downloads' | 'updated' | 'newest'
type ViewMode = 'grid' | 'list'

interface ModrinthHit extends Labrinth.Search.v3.ResultSearchProject {
	featured_gallery?: string | null
	gallery?: string[]
}

const props = defineProps<{
	instance: GameInstance
	initialProjectType?: string
}>()

const route = useRoute()
const router = useRouter()
const queryClient = useQueryClient()
const { addNotification, handleError } = injectNotificationManager()
const { install: installModrinthVersion } = injectContentInstall()

function normalizeContentKind(raw?: string): ContentKind {
	const v = String(raw ?? '').toLowerCase()
	if (v === 'resourcepack' || v === 'resourcepacks') return 'resourcepack'
	if (v === 'shader' || v === 'shaders') return 'shader'
	if (v === 'datapack' || v === 'datapacks') return 'datapack'
	return props.instance.loader === 'vanilla' ? 'resourcepack' : 'mod'
}

const source = ref<ProviderSource>(
	String(route.query.source ?? '') === 'curseforge' ? 'curseforge' : 'modrinth',
)
const contentKind = ref<ContentKind>(normalizeContentKind(props.initialProjectType))
const viewMode = ref<ViewMode>('grid')
const searchQuery = ref('')
const debouncedQuery = ref('')
const selectedSort = ref<SortOption>('relevance')
const page = ref(1)
const pageSize = 18

const loading = ref(false)
const errorMessage = ref<string | null>(null)

// Modrinth state
const modrinthHits = ref<ModrinthHit[]>([])
const modrinthTotal = ref(0)
const installedModrinthIds = ref<Set<string>>(new Set())
const installingModrinthIds = ref<Set<string>>(new Set())

// CurseForge state
const curseforgeItems = ref<CurseForgeModpackSummary[]>([])
const curseforgeTotal = ref(0)
const curseforgeConfigured = ref(true)
const installedCurseForgeIds = ref<Set<number>>(new Set())
const installingCurseForgeIds = ref<Set<number>>(new Set())

const isVanilla = computed(() => props.instance.loader === 'vanilla')

const CONTENT_TABS = computed<{ id: ContentKind; label: string; disabled?: boolean }[]>(() => [
	{ id: 'mod', label: 'Mods', disabled: isVanilla.value },
	{ id: 'resourcepack', label: 'Resource Packs' },
	{ id: 'shader', label: 'Shaders' },
	{ id: 'datapack', label: 'Datapacks' },
])

let searchTimer: ReturnType<typeof setTimeout> | null = null
watch(searchQuery, (val) => {
	if (searchTimer) clearTimeout(searchTimer)
	searchTimer = setTimeout(() => {
		debouncedQuery.value = val.trim()
		page.value = 1
	}, 260)
})

watch([source, contentKind, selectedSort], () => {
	page.value = 1
})

function formatDownloads(count: number): string {
	if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`
	if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`
	return String(count)
}

function mapCurseForgeSort(s: SortOption): CurseForgeSort {
	if (s === 'downloads') return 'downloads'
	if (s === 'updated') return 'updated'
	if (s === 'newest') return 'newest'
	return 'popularity'
}

async function loadInstalledModrinthIds() {
	try {
		const ids = await getInstalledProjectIds(props.instance.id)
		installedModrinthIds.value = new Set(ids ?? [])
	} catch {
		// best effort
	}
}

async function fetchCatalog() {
	loading.value = true
	errorMessage.value = null

	try {
		if (source.value === 'modrinth') {
			const facets: string[][] = [[`project_type:${contentKind.value}`]]
			if (props.instance.game_version) {
				facets.push([`versions:${props.instance.game_version}`])
			}
			if (contentKind.value === 'mod' && props.instance.loader && props.instance.loader !== 'vanilla') {
				facets.push([`categories:${props.instance.loader}`])
			}

			const qs = new URLSearchParams({
				limit: String(pageSize),
				offset: String((page.value - 1) * pageSize),
				index: selectedSort.value,
				facets: JSON.stringify(facets),
			})
			if (debouncedQuery.value) qs.set('query', debouncedQuery.value)

			const res = await get_search_results_v3(`?${qs.toString()}`)
			modrinthHits.value = (res?.hits ?? []) as ModrinthHit[]
			modrinthTotal.value = Number(res?.total_hits ?? 0)
		} else {
			const res = await searchCurseForgeModpacks({
				q: debouncedQuery.value || undefined,
				gameVersion: props.instance.game_version || undefined,
				loader:
					contentKind.value === 'mod' && props.instance.loader !== 'vanilla'
						? props.instance.loader
						: undefined,
				contentType: contentKind.value,
				sort: mapCurseForgeSort(selectedSort.value),
				index: (page.value - 1) * pageSize,
				pageSize,
			})
			curseforgeConfigured.value = res.configured
			curseforgeItems.value = res.items ?? []
			curseforgeTotal.value = res.pagination?.totalCount ?? 0
		}
	} catch (err) {
		errorMessage.value = err instanceof Error ? err.message : 'Failed to load catalog.'
	} finally {
		loading.value = false
	}
}

watch([source, contentKind, debouncedQuery, selectedSort, page], () => {
	void fetchCatalog()
})

onMounted(() => {
	void loadInstalledModrinthIds()
	void fetchCatalog()
})

const totalPages = computed(() => {
	const total = source.value === 'modrinth' ? modrinthTotal.value : curseforgeTotal.value
	return Math.max(1, Math.min(50, Math.ceil(total / pageSize)))
})

async function handleInstallModrinth(hit: ModrinthHit) {
	const projectId = hit.project_id
	if (installingModrinthIds.value.has(projectId)) return

	const next = new Set(installingModrinthIds.value)
	next.add(projectId)
	installingModrinthIds.value = next

	try {
		const versions = (await get_project_versions(projectId)) as Labrinth.Versions.v2.Version[]
		const targetVersion = props.instance.game_version
		const targetLoader = props.instance.loader

		const compatible = (versions ?? []).filter((v) => {
			const mcOk = !targetVersion || v.game_versions.includes(targetVersion)
			const loaderOk =
				contentKind.value !== 'mod' ||
				!targetLoader ||
				targetLoader === 'vanilla' ||
				v.loaders.includes(targetLoader)
			return mcOk && loaderOk
		})

		const picked = compatible[0] ?? versions?.[0]
		if (!picked) {
			throw new Error(`No compatible version found for ${hit.title}.`)
		}

		await installModrinthVersion(projectId, picked.id, props.instance.id, 'SolaraInstanceBrowser')

		const installedNext = new Set(installedModrinthIds.value)
		installedNext.add(projectId)
		installedModrinthIds.value = installedNext

		await queryClient.invalidateQueries({ queryKey: instanceKeys.content(props.instance.id) })
		addNotification({
			type: 'success',
			title: `Installed ${hit.title}`,
			text: `Added to ${props.instance.name}`,
		})
	} catch (err) {
		handleError(err)
	} finally {
		const cleaned = new Set(installingModrinthIds.value)
		cleaned.delete(projectId)
		installingModrinthIds.value = cleaned
	}
}

function pickCompatibleCurseForgeFile(files: CurseForgeFile[]): CurseForgeFile | null {
	const targetMc = props.instance.game_version
	const targetLoader = props.instance.loader?.toLowerCase()

	const matchBoth = files.find((f) => {
		if (!f.downloadUrl || f.restricted) return false
		const mcOk = !targetMc || f.gameVersions.includes(targetMc)
		const loaderOk =
			contentKind.value !== 'mod' ||
			!targetLoader ||
			targetLoader === 'vanilla' ||
			f.loaders.length === 0 ||
			f.loaders.map((l) => l.toLowerCase()).includes(targetLoader)
		return mcOk && loaderOk
	})
	if (matchBoth) return matchBoth

	return files.find((f) => !!f.downloadUrl && !f.restricted) ?? null
}

async function handleInstallCurseForge(item: CurseForgeModpackSummary) {
	if (installingCurseForgeIds.value.has(item.id)) return

	const next = new Set(installingCurseForgeIds.value)
	next.add(item.id)
	installingCurseForgeIds.value = next

	try {
		let chosen: CurseForgeFile | null =
			item.latestFile && pickCompatibleCurseForgeFile([item.latestFile]) ? item.latestFile : null

		if (!chosen) {
			const detail = await getCurseForgeModpackDetail(item.id)
			chosen = pickCompatibleCurseForgeFile(detail.modpack.files ?? [])
		}

		if (!chosen || !chosen.downloadUrl) {
			addNotification({
				type: 'warning',
				title: 'Author restricted direct download',
				text: `${item.name} requires downloading directly from CurseForge in your browser.`,
			})
			if (item.websiteUrl) {
				void openUrl(item.websiteUrl)
			}
			return
		}

		await installCurseForgeContent({
			instanceId: props.instance.id,
			contentType: contentKind.value,
			modId: item.id,
			fileId: chosen.id,
			downloadUrl: chosen.downloadUrl,
			fileName: chosen.fileName,
		})

		const doneNext = new Set(installedCurseForgeIds.value)
		doneNext.add(item.id)
		installedCurseForgeIds.value = doneNext

		await queryClient.invalidateQueries({ queryKey: instanceKeys.content(props.instance.id) })
		addNotification({
			type: 'success',
			title: `Installed ${item.name}`,
			text: `Added ${chosen.fileName} to ${props.instance.name}`,
		})
	} catch (err) {
		handleError(err)
	} finally {
		const cleaned = new Set(installingCurseForgeIds.value)
		cleaned.delete(item.id)
		installingCurseForgeIds.value = cleaned
	}
}

function goBackToInstance() {
	void router.push(`/instance/${encodeURIComponent(props.instance.id)}`)
}
</script>

<template>
	<div class="flex flex-col gap-5 p-6">
		<!-- Top Instance Context + Catalog Switcher Header -->
		<div class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-solid border-surface-4 bg-surface-2 px-5 py-4">
			<div class="flex min-w-0 items-center gap-3.5">
				<button
					type="button"
					class="inline-flex h-9 cursor-pointer items-center gap-1.5 rounded-xl border border-solid border-surface-4 bg-surface-3 px-3 text-xs font-bold text-contrast transition-colors hover:border-brand/50"
					@click="goBackToInstance"
				>
					<ArrowLeftIcon class="size-4" />
					<span>Back</span>
				</button>

				<Avatar
					:src="getInstanceIconUrl(instance.icon_path)"
					:alt="instance.name"
					size="42px"
					:tint-by="instance.id"
					class="rounded-xl ring-1 ring-surface-5"
				/>

				<div class="flex min-w-0 flex-col">
					<span class="text-[11px] font-bold uppercase tracking-wider text-secondary">
						Installing content into
					</span>
					<div class="flex flex-wrap items-center gap-2">
						<h1 class="m-0 truncate text-lg font-extrabold text-contrast">{{ instance.name }}</h1>
						<span class="rounded-md border border-solid border-brand/35 bg-brand/12 px-2 py-0.5 text-xs font-bold capitalize text-brand">
							{{ instance.loader }} · {{ instance.game_version }}
						</span>
					</div>
				</div>
			</div>

			<!-- Dual Provider Switcher: Modrinth & CurseForge -->
			<div class="inline-flex items-center gap-1 rounded-xl border border-solid border-surface-4 bg-surface-1 p-1">
				<button
					type="button"
					:class="[
						'inline-flex cursor-pointer items-center gap-2 rounded-lg border-0 px-4 py-2 text-xs font-extrabold transition-all',
						source === 'modrinth'
							? 'bg-[#1bd96a] text-[#04140a] shadow-sm'
							: 'bg-transparent text-secondary hover:text-contrast',
					]"
					@click="source = 'modrinth'"
				>
					<span>Modrinth</span>
				</button>
				<button
					type="button"
					:class="[
						'inline-flex cursor-pointer items-center gap-2 rounded-lg border-0 px-4 py-2 text-xs font-extrabold transition-all',
						source === 'curseforge'
							? 'bg-[#f16436] text-white shadow-sm'
							: 'bg-transparent text-secondary hover:text-contrast',
					]"
					@click="source = 'curseforge'"
				>
					<span>CurseForge</span>
				</button>
			</div>
		</div>

		<!-- Content Type Tabs + Search + Sort + Grid/List Controls -->
		<div class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-4">
			<div class="flex flex-wrap items-center justify-between gap-3">
				<div class="flex flex-wrap items-center gap-1.5">
					<button
						v-for="tab in CONTENT_TABS"
						:key="tab.id"
						type="button"
						:disabled="tab.disabled"
						:title="tab.disabled ? 'Switch instance to Fabric, Forge, NeoForge, or Quilt to install mods' : undefined"
						:class="[
							'inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-solid px-3.5 py-1.5 text-xs font-bold transition-colors disabled:cursor-not-allowed disabled:opacity-40',
							contentKind === tab.id
								? 'border-brand/50 bg-brand text-[var(--color-accent-contrast)]'
								: 'border-surface-4 bg-surface-3 text-secondary hover:text-contrast',
						]"
						@click="contentKind = tab.id"
					>
						<span>{{ tab.label }}</span>
					</button>
				</div>

				<div class="flex items-center gap-2">
					<select
						v-model="selectedSort"
						aria-label="Sort catalog"
						class="h-9 cursor-pointer rounded-xl border border-solid border-surface-4 bg-surface-1 px-3 text-xs font-semibold text-contrast outline-none focus:border-brand"
					>
						<option value="relevance">Most Popular</option>
						<option value="downloads">Most Downloads</option>
						<option value="updated">Recently Updated</option>
						<option value="newest">Newest</option>
					</select>

					<div class="inline-flex items-center rounded-xl border border-solid border-surface-4 bg-surface-3 p-0.5">
						<button
							type="button"
							title="Grid view"
							:class="[
								'inline-flex size-8 cursor-pointer items-center justify-center rounded-lg border-0',
								viewMode === 'grid'
									? 'bg-brand text-[var(--color-accent-contrast)]'
									: 'bg-transparent text-secondary hover:text-contrast',
							]"
							@click="viewMode = 'grid'"
						>
							<GridIcon class="size-4" />
						</button>
						<button
							type="button"
							title="List view"
							:class="[
								'inline-flex size-8 cursor-pointer items-center justify-center rounded-lg border-0',
								viewMode === 'list'
									? 'bg-brand text-[var(--color-accent-contrast)]'
									: 'bg-transparent text-secondary hover:text-contrast',
							]"
							@click="viewMode = 'list'"
						>
							<ListIcon class="size-4" />
						</button>
					</div>
				</div>
			</div>

			<div class="relative">
				<SearchIcon class="pointer-events-none absolute left-3.5 top-1/2 size-4 -translate-y-1/2 text-secondary" />
				<input
					v-model="searchQuery"
					type="text"
					:placeholder="`Search ${source === 'modrinth' ? 'Modrinth' : 'CurseForge'} ${contentKind}s for ${instance.loader} ${instance.game_version}...`"
					class="h-10 w-full rounded-xl border border-solid border-surface-4 bg-surface-1 pl-10 pr-9 text-sm text-contrast outline-none transition-colors focus:border-brand"
				/>
				<button
					v-if="searchQuery"
					type="button"
					class="absolute right-3 top-1/2 -translate-y-1/2 cursor-pointer border-0 bg-transparent p-0 text-secondary hover:text-contrast"
					@click="searchQuery = ''"
				>
					<XIcon class="size-4" />
				</button>
			</div>
		</div>

		<!-- Error / Loading -->
		<div
			v-if="errorMessage"
			class="rounded-2xl border border-solid border-red/40 bg-red/10 px-5 py-4 text-sm font-medium text-red"
		>
			{{ errorMessage }}
		</div>

		<div
			v-else-if="loading"
			class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3"
		>
			<div
				v-for="n in 9"
				:key="n"
				class="h-44 animate-pulse rounded-2xl border border-solid border-surface-4 bg-surface-2"
			/>
		</div>

		<!-- MODRINTH RESULTS -->
		<template v-else-if="source === 'modrinth'">
			<div
				v-if="modrinthHits.length === 0"
				class="flex flex-col items-center gap-2 rounded-2xl border border-solid border-surface-4 bg-surface-2 py-16 text-center text-secondary"
			>
				<PackageIcon class="size-8 opacity-60" />
				<p class="m-0 text-sm font-semibold text-contrast">No matching Modrinth {{ contentKind }}s found.</p>
			</div>

			<div
				v-else
				:class="
					viewMode === 'grid'
						? 'grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3'
						: 'flex flex-col gap-2.5'
				"
			>
				<article
					v-for="hit in modrinthHits"
					:key="hit.project_id"
					:class="[
						'flex rounded-2xl border border-solid border-surface-4 bg-surface-2 transition-all hover:border-[#1bd96a]/45',
						viewMode === 'grid' ? 'flex-col justify-between gap-3 p-4' : 'items-center justify-between gap-4 px-4 py-3',
					]"
				>
					<div class="flex min-w-0 items-start gap-3">
						<Avatar :src="hit.icon_url" :alt="hit.title" size="50px" class="shrink-0 rounded-xl ring-1 ring-surface-5" />
						<div class="flex min-w-0 flex-1 flex-col gap-1">
							<div class="flex items-center gap-2">
								<h3 class="m-0 truncate text-sm font-extrabold text-contrast">{{ hit.title }}</h3>
								<span class="shrink-0 rounded bg-[#1bd96a]/15 px-1.5 py-0.5 text-[10px] font-bold text-[#1bd96a]">
									Modrinth
								</span>
							</div>
							<span class="truncate text-xs text-secondary">by {{ hit.author }}</span>
							<p class="m-0 line-clamp-2 text-xs leading-relaxed text-secondary">
								{{ hit.description }}
							</p>
						</div>
					</div>

					<div
						:class="[
							'flex items-center justify-between gap-2',
							viewMode === 'grid' ? 'border-0 border-t border-solid border-surface-4 pt-3' : 'shrink-0',
						]"
					>
						<div class="flex items-center gap-3 text-[11px] font-semibold text-secondary">
							<span class="inline-flex items-center gap-1">
								<DownloadIcon class="size-3.5" />
								{{ formatDownloads(hit.downloads) }}
							</span>
							<span v-if="hit.date_modified" class="inline-flex items-center gap-1">
								<UpdatedIcon class="size-3.5" />
								{{ dayjs(hit.date_modified).fromNow() }}
							</span>
						</div>

						<div class="flex items-center gap-1.5">
							<Button
								type="outlined"
								size="sm"
								class="!rounded-lg"
								@click="router.push({ path: `/project/${hit.project_id}`, query: { i: instance.id } })"
							>
								Details
							</Button>
							<Button
								:type="installedModrinthIds.has(hit.project_id) ? 'outlined' : 'colored'"
								:color="installedModrinthIds.has(hit.project_id) ? 'default' : 'brand'"
								size="sm"
								class="!rounded-lg !font-bold"
								:disabled="installingModrinthIds.has(hit.project_id) || installedModrinthIds.has(hit.project_id)"
								@click="handleInstallModrinth(hit)"
							>
								<SpinnerIcon v-if="installingModrinthIds.has(hit.project_id)" class="size-3.5 animate-spin" />
								<CheckIcon v-else-if="installedModrinthIds.has(hit.project_id)" class="size-3.5 text-emerald-400" />
								<DownloadIcon v-else class="size-3.5" />
								<span>
									{{
										installingModrinthIds.has(hit.project_id)
											? 'Installing...'
											: installedModrinthIds.has(hit.project_id)
												? 'Installed'
												: 'Install'
									}}
								</span>
							</Button>
						</div>
					</div>
				</article>
			</div>
		</template>

		<!-- CURSEFORGE RESULTS -->
		<template v-else>
			<div
				v-if="!curseforgeConfigured || curseforgeItems.length === 0"
				class="flex flex-col items-center gap-2 rounded-2xl border border-solid border-surface-4 bg-surface-2 py-16 text-center text-secondary"
			>
				<PackageIcon class="size-8 opacity-60" />
				<p class="m-0 text-sm font-semibold text-contrast">
					{{
						!curseforgeConfigured
							? 'CurseForge catalog is temporarily unavailable.'
							: `No matching CurseForge ${contentKind}s found.`
					}}
				</p>
			</div>

			<div
				v-else
				:class="
					viewMode === 'grid'
						? 'grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3'
						: 'flex flex-col gap-2.5'
				"
			>
				<article
					v-for="item in curseforgeItems"
					:key="item.id"
					:class="[
						'flex rounded-2xl border border-solid border-surface-4 bg-surface-2 transition-all hover:border-[#f16436]/50',
						viewMode === 'grid' ? 'flex-col justify-between gap-3 p-4' : 'items-center justify-between gap-4 px-4 py-3',
					]"
				>
					<div class="flex min-w-0 items-start gap-3">
						<Avatar :src="item.logoUrl" :alt="item.name" size="50px" class="shrink-0 rounded-xl ring-1 ring-surface-5" />
						<div class="flex min-w-0 flex-1 flex-col gap-1">
							<div class="flex items-center gap-2">
								<h3 class="m-0 truncate text-sm font-extrabold text-contrast">{{ item.name }}</h3>
								<span class="shrink-0 rounded bg-[#f16436]/15 px-1.5 py-0.5 text-[10px] font-bold text-[#f16436]">
									CurseForge
								</span>
							</div>
							<span class="truncate text-xs text-secondary">by {{ item.author }}</span>
							<p class="m-0 line-clamp-2 text-xs leading-relaxed text-secondary">
								{{ item.summary }}
							</p>
						</div>
					</div>

					<div
						:class="[
							'flex items-center justify-between gap-2',
							viewMode === 'grid' ? 'border-0 border-t border-solid border-surface-4 pt-3' : 'shrink-0',
						]"
					>
						<div class="flex items-center gap-3 text-[11px] font-semibold text-secondary">
							<span class="inline-flex items-center gap-1">
								<DownloadIcon class="size-3.5" />
								{{ formatDownloads(item.downloads) }}
							</span>
							<span v-if="item.dateModified" class="inline-flex items-center gap-1">
								<UpdatedIcon class="size-3.5" />
								{{ dayjs(item.dateModified).fromNow() }}
							</span>
						</div>

						<div class="flex items-center gap-1.5">
							<button
								v-if="item.websiteUrl"
								type="button"
								title="Open on CurseForge"
								class="inline-flex size-8 cursor-pointer items-center justify-center rounded-lg border border-solid border-surface-4 bg-surface-3 text-secondary hover:text-contrast"
								@click="openUrl(item.websiteUrl)"
							>
								<ExternalIcon class="size-3.5" />
							</button>
							<Button
								:type="installedCurseForgeIds.has(item.id) ? 'outlined' : 'colored'"
								:color="installedCurseForgeIds.has(item.id) ? 'default' : 'orange'"
								size="sm"
								class="!rounded-lg !font-bold"
								:disabled="installingCurseForgeIds.has(item.id) || installedCurseForgeIds.has(item.id)"
								@click="handleInstallCurseForge(item)"
							>
								<SpinnerIcon v-if="installingCurseForgeIds.has(item.id)" class="size-3.5 animate-spin" />
								<CheckIcon v-else-if="installedCurseForgeIds.has(item.id)" class="size-3.5 text-emerald-400" />
								<DownloadIcon v-else class="size-3.5" />
								<span>
									{{
										installingCurseForgeIds.has(item.id)
											? 'Installing...'
											: installedCurseForgeIds.has(item.id)
												? 'Installed'
												: 'Install'
									}}
								</span>
							</Button>
						</div>
					</div>
				</article>
			</div>
		</template>

		<!-- Pagination -->
		<div v-if="totalPages > 1 && !loading" class="flex justify-center pt-2">
			<Pagination :page="page" :count="totalPages" @switch-page="(p: number) => (page = p)" />
		</div>
	</div>
</template>
