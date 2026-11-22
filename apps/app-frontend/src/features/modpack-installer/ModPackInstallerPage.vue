<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	DownloadIcon,
	GridIcon,
	ListIcon,
	PackageIcon,
	SearchIcon,
	SpinnerIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import { Button, Checkbox, Combobox, injectNotificationManager, Pagination } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get_search_results_v3 } from '@/helpers/cache.js'
import { list as listInstances } from '@/helpers/instance'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { instanceKeys } from '@/pages/instance/query-options'
import { injectContentInstall } from '@/providers/content-install'

import {
	CURSEFORGE_CATEGORY_MAP,
	type CurseForgeFile,
	type CurseForgeModpackSummary,
	type CurseForgeSort,
	installCurseForgeModpack,
	searchCurseForgeModpacks,
} from './curseforge-api'
import CurseForgePackModal from './CurseForgePackModal.vue'

dayjs.extend(relativeTime)

type ProviderSource = 'modrinth' | 'curseforge'
type LoaderFilter = 'all' | 'fabric' | 'forge' | 'neoforge' | 'quilt'
type SortOption = 'relevance' | 'downloads' | 'updated' | 'newest'
type ViewMode = 'grid' | 'list'

interface ModrinthV3Hit extends Labrinth.Search.v3.ResultSearchProject {
	mrpack_loaders?: string[]
	game_versions?: string[]
	featured_gallery?: string | null
	gallery?: string[]
	project_loader_fields?: {
		mrpack_loaders?: string[]
		game_versions?: string[]
	}
}

const props = defineProps<{
	gameVersions: Labrinth.Tags.v2.GameVersion[]
}>()

const route = useRoute()
const router = useRouter()
const queryClient = useQueryClient()
const appSettings = useAppSettings()
const { handleError, addNotification } = injectNotificationManager()
const { install: installModrinthVersion } = injectContentInstall()

function resolveInitialSource(): ProviderSource {
	const querySource = String(route.query.source ?? '')
	if (querySource === 'curseforge' || querySource === 'modrinth') {
		return querySource
	}
	try {
		const saved = sessionStorage.getItem('solara:modpack-source')
		if (saved === 'curseforge' || saved === 'modrinth') {
			return saved
		}
	} catch {
		// Ignore storage restrictions
	}
	return 'modrinth'
}

const source = ref<ProviderSource>(resolveInitialSource())
watch(source, (val) => {
	try {
		sessionStorage.setItem('solara:modpack-source', val)
	} catch {
		// Ignore storage restrictions
	}
})
const viewMode = ref<ViewMode>('grid')
const searchQuery = ref('')
const debouncedQuery = ref('')
const selectedLoader = ref<LoaderFilter>('all')
const selectedGameVersion = ref<string>('all')
const selectedCategory = ref<string>('all')
const selectedSort = ref<SortOption>('relevance')
const page = ref(1)
const pageSize = 18

const loading = ref(false)
const errorMessage = ref<string | null>(null)
const offline = ref(!navigator.onLine)

// Modrinth state
const modrinthHits = ref<ModrinthV3Hit[]>([])
const modrinthTotal = ref(0)
const installedProjectIds = ref<Set<string>>(new Set())
const installingModrinthIds = ref<Set<string>>(new Set())

// CurseForge state
const curseforgeConfigured = ref(true)
const curseforgeItems = ref<CurseForgeModpackSummary[]>([])
const curseforgeTotal = ref(0)
const activeCurseForgeModalPack = ref<CurseForgeModpackSummary | null>(null)
const installingCurseForgeKey = ref<string | null>(null)

const LOADERS: { id: LoaderFilter; label: string }[] = [
	{ id: 'all', label: 'All' },
	{ id: 'fabric', label: 'Fabric' },
	{ id: 'forge', label: 'Forge' },
	{ id: 'neoforge', label: 'NeoForge' },
	{ id: 'quilt', label: 'Quilt' },
]

const SORT_OPTIONS: { value: SortOption; label: string }[] = [
	{ value: 'relevance', label: 'Most popular' },
	{ value: 'downloads', label: 'Most downloads' },
	{ value: 'updated', label: 'Recently updated' },
	{ value: 'newest', label: 'Newest releases' },
]

const CATEGORY_OPTIONS: { value: string; label: string }[] = [
	{ value: 'all', label: 'All categories' },
	{ value: 'adventure', label: 'Adventure' },
	{ value: 'tech', label: 'Technology' },
	{ value: 'magic', label: 'Magic' },
	{ value: 'optimization', label: 'Optimization' },
	{ value: 'combat', label: 'Combat' },
	{ value: 'multiplayer', label: 'Multiplayer' },
	{ value: 'quests', label: 'Quests' },
	{ value: 'kitchen-sink', label: 'Kitchen Sink' },
	{ value: 'lightweight', label: 'Lightweight' },
	{ value: 'skyblock', label: 'Skyblock' },
	{ value: 'hardcore', label: 'Hardcore' },
]

const gameVersionOptions = computed(() => {
	const releases = (props.gameVersions ?? [])
		.filter((v) => v.version_type === 'release')
		.map((v) => v.version)
	const list =
		releases.length > 0
			? releases.slice(0, 48)
			: ['1.21.4', '1.21.1', '1.20.4', '1.20.1', '1.19.2', '1.18.2', '1.16.5', '1.12.2']
	return [
		{ value: 'all', label: 'All versions' },
		...list.map((ver) => ({ value: ver, label: `MC ${ver}` })),
	]
})

const hideInstalledModpacks = computed({
	get: () => appSettings.hideInstalledModpacks,
	set: (val: boolean) => {
		appSettings.hideInstalledModpacks = val
		void getSettings()
			.then((s) => {
				s.hide_installed_modpacks = val
				return setSettings(s)
			})
			.catch(handleError)
	},
})

const hasActiveFilters = computed(
	() =>
		searchQuery.value.trim() !== '' ||
		selectedLoader.value !== 'all' ||
		selectedGameVersion.value !== 'all' ||
		selectedCategory.value !== 'all' ||
		selectedSort.value !== 'relevance',
)

const totalResults = computed(() =>
	source.value === 'modrinth' ? modrinthTotal.value : curseforgeTotal.value,
)

const totalPages = computed(() =>
	Math.max(1, Math.min(250, Math.ceil(totalResults.value / pageSize))),
)

let debounceTimer: ReturnType<typeof setTimeout> | null = null
watch(searchQuery, (val) => {
	if (debounceTimer) clearTimeout(debounceTimer)
	debounceTimer = setTimeout(() => {
		debouncedQuery.value = val.trim()
		page.value = 1
	}, 260)
})

watch(
	[source, selectedLoader, selectedGameVersion, selectedCategory, selectedSort, hideInstalledModpacks],
	() => {
		page.value = 1
	},
)

function switchPage(nextPage: number) {
	page.value = Math.max(1, Math.min(totalPages.value, nextPage))
	window.scrollTo({ top: 0, behavior: 'smooth' })
}

async function refreshInstalledIds() {
	try {
		const instances = await queryClient.fetchQuery({
			networkMode: 'always' as const,
			queryKey: [...instanceKeys.all, 'installed-project-ids'],
			queryFn: listInstances,
			staleTime: 0,
		})
		const ids = new Set<string>()
		for (const inst of instances ?? []) {
			if (inst.link?.project_id) ids.add(inst.link.project_id)
		}
		installedProjectIds.value = ids
	} catch {
		// Non-fatal
	}
}

function escapeFilterValue(value: string): string {
	return `"${value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`
}

let activeRequestToken = 0

async function fetchCatalog() {
	const token = ++activeRequestToken
	loading.value = true
	errorMessage.value = null

	try {
		if (source.value === 'modrinth') {
			const filters: string[] = ['project_types = "modpack"']
			if (selectedLoader.value !== 'all') {
				filters.push(`mrpack_loaders = ${escapeFilterValue(selectedLoader.value)}`)
			}
			if (selectedGameVersion.value && selectedGameVersion.value !== 'all') {
				filters.push(`game_versions = ${escapeFilterValue(selectedGameVersion.value)}`)
			}
			if (selectedCategory.value && selectedCategory.value !== 'all') {
				filters.push(`categories = ${escapeFilterValue(selectedCategory.value)}`)
			}
			if (hideInstalledModpacks.value && installedProjectIds.value.size > 0) {
				const quotedIds = [...installedProjectIds.value].map(escapeFilterValue).join(', ')
				filters.push(`project_id NOT IN [${quotedIds}]`)
			}

			const params = [
				`limit=${pageSize}`,
				`index=${selectedSort.value}`,
				`new_filters=${encodeURIComponent(filters.join(' AND '))}`,
			]
			if (debouncedQuery.value) {
				params.push(`query=${encodeURIComponent(debouncedQuery.value)}`)
			}
			if (page.value > 1) {
				params.push(`offset=${(page.value - 1) * pageSize}`)
			}

			const raw = (await get_search_results_v3(`?${params.join('&')}`, 'must_revalidate')) as {
				result?: Labrinth.Search.v3.SearchResults
			} | null

			if (token !== activeRequestToken) return
			modrinthHits.value = (raw?.result?.hits ?? []) as ModrinthV3Hit[]
			modrinthTotal.value = raw?.result?.total_hits ?? 0
		} else {
			const sortMap: Record<SortOption, CurseForgeSort> = {
				relevance: 'popularity',
				downloads: 'downloads',
				updated: 'updated',
				newest: 'newest',
			}
			const categoryId =
				selectedCategory.value && selectedCategory.value !== 'all'
					? CURSEFORGE_CATEGORY_MAP[selectedCategory.value]
					: undefined

			const res = await searchCurseForgeModpacks({
				q: debouncedQuery.value || undefined,
				gameVersion:
					selectedGameVersion.value && selectedGameVersion.value !== 'all'
						? selectedGameVersion.value
						: undefined,
				loader: selectedLoader.value === 'all' ? undefined : selectedLoader.value,
				categoryId,
				sort: sortMap[selectedSort.value],
				index: (page.value - 1) * pageSize,
				pageSize,
			})

			if (token !== activeRequestToken) return
			curseforgeConfigured.value = res.configured
			curseforgeItems.value = res.items ?? []
			curseforgeTotal.value = res.pagination?.totalCount ?? 0
		}
	} catch (err) {
		if (token !== activeRequestToken) return
		errorMessage.value =
			err instanceof Error ? err.message : 'Unable to load modpacks right now.'
	} finally {
		if (token === activeRequestToken) {
			loading.value = false
		}
	}
}

watch(
	[
		source,
		debouncedQuery,
		selectedLoader,
		selectedGameVersion,
		selectedCategory,
		selectedSort,
		page,
		hideInstalledModpacks,
	],
	() => {
		void fetchCatalog()
	},
	{ immediate: true },
)

function resetFilters() {
	searchQuery.value = ''
	debouncedQuery.value = ''
	selectedLoader.value = 'all'
	selectedGameVersion.value = 'all'
	selectedCategory.value = 'all'
	selectedSort.value = 'relevance'
	page.value = 1
}

function formatDownloads(n?: number): string {
	const value = n ?? 0
	if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`
	if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`
	return String(value)
}

function formatLoaderLabel(loader: string): string {
	const map: Record<string, string> = {
		fabric: 'Fabric',
		forge: 'Forge',
		neoforge: 'NeoForge',
		quilt: 'Quilt',
	}
	return map[loader.toLowerCase()] ?? loader
}

function formatCategoryLabel(cat: string): string {
	return cat
		.split('-')
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ')
}

function modrinthLoaders(hit: ModrinthV3Hit): string[] {
	const known = ['fabric', 'forge', 'neoforge', 'quilt']
	const raw = [
		...(hit.mrpack_loaders ?? []),
		...(hit.project_loader_fields?.mrpack_loaders ?? []),
		...(hit.categories ?? []),
		...(hit.display_categories ?? []),
	].map((c) => c.toLowerCase())
	return known.filter((l) => raw.includes(l))
}

function modrinthLatestMcVersion(hit: ModrinthV3Hit): string | null {
	if (hit.latest_version) return hit.latest_version
	const versions = hit.game_versions ?? hit.project_loader_fields?.game_versions ?? []
	if (versions.length === 0) return null
	const standard = versions.filter((v) => /^\d+\.\d+/.test(v))
	return standard.length > 0 ? standard[standard.length - 1] : versions[versions.length - 1]
}

function modrinthBannerUrl(hit: ModrinthV3Hit): string | null {
	return hit.featured_gallery || hit.gallery?.[0] || null
}

function modrinthTags(hit: ModrinthV3Hit): string[] {
	const loaders = new Set(['fabric', 'forge', 'neoforge', 'quilt', 'mrpack'])
	const raw = hit.display_categories ?? hit.categories ?? []
	const unique: string[] = []
	for (const c of raw) {
		const lower = c.toLowerCase()
		if (loaders.has(lower) || unique.includes(lower)) continue
		unique.push(lower)
		if (unique.length >= 2) break
	}
	return unique
}

function openModrinthProject(hit: ModrinthV3Hit) {
	void router.push({
		path: `/project/${hit.project_id ?? hit.slug}`,
		query: { b: '/browse/modpack' },
	})
}

function openCurseForgeProject(pack: CurseForgeModpackSummary, tab?: 'versions' | 'gallery') {
	void router.push({
		path: `/curseforge/modpack/${pack.id}`,
		query: tab ? { tab } : undefined,
	})
}

async function handleInstallModrinth(hit: ModrinthV3Hit) {
	const pid = hit.project_id
	if (!pid || installingModrinthIds.value.has(pid)) return
	const next = new Set(installingModrinthIds.value)
	next.add(pid)
	installingModrinthIds.value = next

	try {
		await installModrinthVersion(
			pid,
			null,
			null,
			'ModPackInstaller',
			(versionId) => {
				const updated = new Set(installingModrinthIds.value)
				updated.delete(pid)
				installingModrinthIds.value = updated
				if (versionId) {
					const inst = new Set(installedProjectIds.value)
					inst.add(pid)
					installedProjectIds.value = inst
				}
			},
			(profileId) => {
				void router.push(`/instance/${profileId}`)
			},
			{
				preferredLoader: selectedLoader.value === 'all' ? undefined : selectedLoader.value,
				preferredGameVersion:
					selectedGameVersion.value && selectedGameVersion.value !== 'all'
						? selectedGameVersion.value
						: undefined,
			},
		)
	} catch (err) {
		const updated = new Set(installingModrinthIds.value)
		updated.delete(pid)
		installingModrinthIds.value = updated
		handleError(err as Error)
	}
}

async function handleInstallCurseForge(
	pack: CurseForgeModpackSummary,
	explicitFile?: CurseForgeFile | null,
) {
	const targetFile = explicitFile ?? pack.latestFile
	if (!targetFile || targetFile.restricted || !targetFile.downloadUrl) {
		openCurseForgeProject(pack, 'versions')
		return
	}

	const key = `cf:${pack.id}:${targetFile.id}`
	if (installingCurseForgeKey.value) return
	installingCurseForgeKey.value = key

	try {
		const res = await installCurseForgeModpack({
			modId: pack.id,
			fileId: targetFile.id,
			name: pack.name,
			downloadUrl: targetFile.downloadUrl,
			iconUrl: pack.logoUrl,
			fileName: targetFile.fileName,
		})

		activeCurseForgeModalPack.value = null
		if (res.restrictedMods.length > 0) {
			addNotification({
				title: `${res.name} installed with manual mods`,
				text: `${res.installedMods} mods installed. ${res.restrictedMods.length} mod(s) restrict third-party downloads on CurseForge and can be added manually to the instance mods folder.`,
				type: 'warning',
			})
		} else {
			addNotification({
				title: `${res.name} installed`,
				text: `Ready to play on Minecraft ${res.gameVersion} (${formatLoaderLabel(res.loader)}).`,
				type: 'success',
			})
		}
		void router.push(`/instance/${encodeURIComponent(res.instanceId)}`)
	} catch (err) {
		handleError(err as Error)
	} finally {
		installingCurseForgeKey.value = null
	}
}

function handleOnline() {
	offline.value = false
	void fetchCatalog()
}

function handleOffline() {
	offline.value = true
}

useAppEvent('instance', async (event) => {
	if (event.event === 'created' || event.event === 'removed') {
		await refreshInstalledIds()
	}
})

onMounted(() => {
	window.addEventListener('online', handleOnline)
	window.addEventListener('offline', handleOffline)
	void refreshInstalledIds()
})

onBeforeUnmount(() => {
	window.removeEventListener('online', handleOnline)
	window.removeEventListener('offline', handleOffline)
	if (debounceTimer) clearTimeout(debounceTimer)
})
</script>

<template>
	<div class="mx-auto flex w-full max-w-[1320px] flex-col gap-5 px-6 py-6">
		<!-- Top Header & Source Switcher -->
		<header class="flex flex-wrap items-center justify-between gap-4">
			<div class="flex flex-col gap-1">
				<div class="flex items-center gap-2.5">
					<h1 class="m-0 text-2xl font-bold tracking-tight text-contrast">
						Modpacks
					</h1>
					<span
						v-if="!loading && totalResults > 0 && (source === 'modrinth' || curseforgeConfigured)"
						class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-0.5 text-xs font-semibold tabular-nums text-secondary"
					>
						{{ totalResults.toLocaleString() }}
					</span>
				</div>
				<p class="m-0 text-sm text-secondary">
					{{
						source === 'modrinth'
							? 'Install curated Minecraft modpacks directly from Modrinth.'
							: 'Install popular Minecraft modpacks directly from CurseForge.'
					}}
				</p>
			</div>

			<div class="flex flex-wrap items-center gap-2.5">
				<!-- Provider Segmented Switcher -->
				<div
					class="inline-flex items-center rounded-xl border border-solid border-surface-5 bg-surface-3 p-1"
					role="tablist"
					aria-label="Modpack source"
				>
					<button
						type="button"
						role="tab"
						:aria-selected="source === 'modrinth'"
						class="solara-source-tab flex cursor-pointer items-center gap-2 rounded-lg border-none px-4 py-2 text-sm font-semibold transition-all"
						:class="
							source === 'modrinth'
								? 'solara-source-modrinth-active text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="source = 'modrinth'"
					>
						<svg class="size-4 shrink-0" viewBox="0 0 24 24" fill="none" aria-hidden="true">
							<circle
								cx="12"
								cy="12"
								r="9.5"
								:stroke="source === 'modrinth' ? '#1bd96a' : 'currentColor'"
								stroke-width="2"
							/>
							<path
								d="M8.5 14.5L11 9.5L13.5 12L16 8.5"
								:stroke="source === 'modrinth' ? '#1bd96a' : 'currentColor'"
								stroke-width="2"
								stroke-linecap="round"
								stroke-linejoin="round"
							/>
						</svg>
						<span>Modrinth</span>
					</button>
					<button
						type="button"
						role="tab"
						:aria-selected="source === 'curseforge'"
						class="solara-source-tab flex cursor-pointer items-center gap-2 rounded-lg border-none px-4 py-2 text-sm font-semibold transition-all"
						:class="
							source === 'curseforge'
								? 'solara-source-cf-active text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="source = 'curseforge'"
					>
						<svg class="size-4 shrink-0" viewBox="0 0 24 24" fill="none" aria-hidden="true">
							<path
								d="M4 7H20L18.2 11.2C17.4 13 15.6 14.2 13.6 14.2H11.5L10.5 17H16.5V19H7.5V17L9 13.8C6.6 13.2 4.8 11.3 4.2 8.8L4 7Z"
								:fill="source === 'curseforge' ? '#f16436' : 'currentColor'"
							/>
						</svg>
						<span>CurseForge</span>
					</button>
				</div>

				<!-- Grid / List Toggle -->
				<div
					class="inline-flex items-center rounded-xl border border-solid border-surface-5 bg-surface-3 p-1"
					role="group"
					aria-label="Layout view"
				>
					<button
						type="button"
						class="flex size-9 cursor-pointer items-center justify-center rounded-lg border-none p-0 transition-colors"
						:class="
							viewMode === 'grid'
								? 'bg-surface-4 text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						title="Card view"
						aria-label="Card view"
						@click="viewMode = 'grid'"
					>
						<GridIcon class="size-4" />
					</button>
					<button
						type="button"
						class="flex size-9 cursor-pointer items-center justify-center rounded-lg border-none p-0 transition-colors"
						:class="
							viewMode === 'list'
								? 'bg-surface-4 text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						title="List view"
						aria-label="List view"
						@click="viewMode = 'list'"
					>
						<ListIcon class="size-4" />
					</button>
				</div>
			</div>
		</header>

		<!-- Search & Filter Bar -->
		<section class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-3.5">
			<!-- Row 1: Search + Sort Combobox -->
			<div class="flex flex-wrap items-center gap-2.5">
				<div
					class="flex h-10 min-w-[240px] flex-1 items-center gap-2.5 rounded-xl border border-solid border-surface-5 bg-surface-2 px-3.5 transition-colors focus-within:border-brand"
				>
					<SearchIcon class="size-4 shrink-0 text-secondary" aria-hidden="true" />
					<input
						v-model="searchQuery"
						type="text"
						:placeholder="
							source === 'modrinth'
								? 'Search Modrinth modpacks (e.g. Fabulously Optimized, Cobblemon)...'
								: 'Search CurseForge modpacks (e.g. All the Mods, RLCraft, Better MC)...'
						"
						class="w-full border-none bg-transparent text-sm text-contrast outline-none placeholder:text-secondary"
					/>
					<button
						v-if="searchQuery"
						type="button"
						class="flex size-5 cursor-pointer items-center justify-center rounded-md border-none bg-transparent p-0 text-secondary hover:text-contrast"
						aria-label="Clear search"
						@click="searchQuery = ''"
					>
						<XIcon class="size-3.5" />
					</button>
				</div>

				<div class="w-48 shrink-0">
					<Combobox
						v-model="selectedSort"
						:options="SORT_OPTIONS"
						placeholder="Sort by"
						trigger-size="md"
					/>
				</div>
			</div>

			<!-- Row 2: Loader Pills + Combobox Filters + Hide Installed -->
			<div class="flex flex-wrap items-center justify-between gap-2.5">
				<div
					class="flex flex-wrap items-center gap-1.5"
					role="group"
					aria-label="Filter by mod loader"
				>
					<button
						v-for="loader in LOADERS"
						:key="loader.id"
						type="button"
						class="cursor-pointer rounded-lg border border-solid px-3 py-1.5 text-xs font-semibold transition-colors"
						:class="
							selectedLoader === loader.id
								? 'solara-loader-active text-contrast'
								: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
						"
						@click="selectedLoader = loader.id"
					>
						{{ loader.label }}
					</button>
				</div>

				<div class="flex flex-wrap items-center gap-2">
					<div class="w-36">
						<Combobox
							v-model="selectedGameVersion"
							:options="gameVersionOptions"
							searchable
							sync-with-selection
							placeholder="MC Version"
							search-placeholder="Type version..."
							trigger-size="sm"
						/>
					</div>

					<div class="w-40">
						<Combobox
							v-model="selectedCategory"
							:options="CATEGORY_OPTIONS"
							placeholder="Category"
							trigger-size="sm"
						/>
					</div>

					<div
						v-if="source === 'modrinth'"
						class="flex h-8 items-center rounded-lg border border-solid border-surface-5 bg-surface-2 px-2.5"
					>
						<Checkbox
							v-model="hideInstalledModpacks"
							label="Hide installed"
							label-class="text-xs font-medium text-secondary"
						/>
					</div>

					<button
						v-if="hasActiveFilters"
						type="button"
						class="cursor-pointer rounded-lg border-none bg-transparent px-2 py-1 text-xs font-semibold text-brand hover:underline"
						@click="resetFilters"
					>
						Reset
					</button>
				</div>
			</div>
		</section>

		<!-- Offline State -->
		<div
			v-if="offline"
			class="rounded-2xl border border-solid border-surface-5 bg-surface-3 p-8 text-center text-sm text-secondary"
		>
			You appear to be offline. Reconnect to browse and install modpacks.
		</div>

		<!-- Error State -->
		<div
			v-else-if="errorMessage"
			class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-10 text-center"
		>
			<p class="m-0 text-sm font-semibold text-contrast">{{ errorMessage }}</p>
			<Button size="sm" @click="fetchCatalog">Retry</Button>
		</div>

		<!-- CurseForge Awaiting Key State -->
		<div
			v-else-if="source === 'curseforge' && !loading && !curseforgeConfigured"
			class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 px-6 py-12 text-center"
		>
			<div class="flex size-12 items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-2">
				<PackageIcon class="size-6 text-secondary" />
			</div>
			<h2 class="m-0 text-base font-bold text-contrast">
				CurseForge catalog is awaiting server API key
			</h2>
			<p class="m-0 max-w-md text-sm leading-relaxed text-secondary">
				CurseForge modpack search is routed securely through Solara's private server.
			</p>
			<Button size="sm" @click="source = 'modrinth'">Browse Modrinth Modpacks</Button>
		</div>

		<!-- Loading Skeletons -->
		<div
			v-else-if="loading"
			:class="
				viewMode === 'grid'
					? 'grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3'
					: 'flex flex-col gap-3'
			"
			aria-busy="true"
		>
			<div
				v-for="n in 9"
				:key="n"
				class="overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-3"
			>
				<div v-if="viewMode === 'grid'" class="h-28 w-full animate-pulse bg-surface-4" />
				<div class="flex flex-col gap-2.5 p-4">
					<div class="h-4 w-44 animate-pulse rounded bg-surface-4" />
					<div class="h-3 w-full animate-pulse rounded bg-surface-4" />
					<div class="h-3 w-2/3 animate-pulse rounded bg-surface-4" />
				</div>
			</div>
		</div>

		<!-- MODRINTH RESULTS -->
		<template v-else-if="source === 'modrinth'">
			<div
				v-if="modrinthHits.length === 0"
				class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-12 text-center"
			>
				<p class="m-0 text-sm font-semibold text-contrast">No Modrinth modpacks matched your filters.</p>
				<Button v-if="hasActiveFilters" size="sm" @click="resetFilters">Clear filters</Button>
			</div>

			<!-- Modrinth Grid Cards -->
			<div
				v-else-if="viewMode === 'grid'"
				class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3"
			>
				<article
					v-for="hit in modrinthHits"
					:key="hit.project_id"
					class="solara-mp-card group flex flex-col overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-3 transition-all"
				>
					<!-- Cover Header -->
					<div
						class="relative h-28 w-full cursor-pointer overflow-hidden bg-surface-2"
						@click="openModrinthProject(hit)"
					>
						<img
							v-if="modrinthBannerUrl(hit)"
							:src="modrinthBannerUrl(hit)!"
							:alt="hit.name"
							loading="lazy"
							class="size-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
						/>
						<img
							v-else-if="hit.icon_url"
							:src="hit.icon_url"
							alt=""
							loading="lazy"
							class="size-full scale-125 object-cover opacity-35 blur-xl"
						/>
						<div class="solara-card-scrim pointer-events-none absolute inset-0" />

						<!-- Top-right version & loader pills -->
						<div class="absolute right-2.5 top-2.5 flex items-center gap-1.5">
							<span
								v-if="modrinthLoaders(hit)[0]"
								class="solara-cover-pill rounded-md px-2 py-0.5 text-[11px] font-semibold"
							>
								{{ formatLoaderLabel(modrinthLoaders(hit)[0]) }}
							</span>
							<span
								v-if="modrinthLatestMcVersion(hit)"
								class="solara-cover-pill rounded-md px-2 py-0.5 text-[11px] font-semibold tabular-nums"
							>
								{{ modrinthLatestMcVersion(hit) }}
							</span>
						</div>
					</div>

					<!-- Body -->
					<div class="relative flex flex-1 flex-col gap-2.5 px-4 pb-4 pt-3">
						<div class="flex items-end gap-3">
							<button
								type="button"
								class="-mt-10 flex size-14 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-xl border-2 border-solid border-surface-3 bg-surface-2 p-0 shadow-md"
								@click="openModrinthProject(hit)"
							>
								<img
									v-if="hit.icon_url"
									:src="hit.icon_url"
									:alt="hit.name"
									loading="lazy"
									class="size-full object-cover"
								/>
								<PackageIcon v-else class="size-6 text-secondary" />
							</button>

							<div class="min-w-0 flex-1">
								<button
									type="button"
									class="m-0 block w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-[15px] font-bold leading-snug text-contrast transition-colors hover:text-brand"
									@click="openModrinthProject(hit)"
								>
									{{ hit.name }}
								</button>
								<p class="m-0 truncate text-xs text-secondary">
									by {{ hit.author || 'Community' }}
								</p>
							</div>
						</div>

						<p class="m-0 line-clamp-2 min-h-[2.4rem] text-xs leading-relaxed text-secondary">
							{{ hit.summary }}
						</p>

						<div v-if="modrinthTags(hit).length > 0" class="flex flex-wrap items-center gap-1.5">
							<span
								v-for="tag in modrinthTags(hit)"
								:key="tag"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-[11px] font-medium text-secondary"
							>
								{{ formatCategoryLabel(tag) }}
							</span>
						</div>

						<!-- Dedicated Footer -->
						<div
							class="mt-auto flex items-center justify-between gap-2 border-0 border-t border-solid border-surface-5 pt-3"
						>
							<div class="flex items-center gap-3 text-xs font-medium text-secondary tabular-nums">
								<span class="flex items-center gap-1" title="Total downloads">
									<DownloadIcon class="size-3.5" />
									{{ formatDownloads(hit.downloads) }}
								</span>
								<span
									v-if="hit.date_modified"
									class="flex items-center gap-1"
									title="Last updated"
								>
									<UpdatedIcon class="size-3.5" />
									{{ dayjs(hit.date_modified).fromNow() }}
								</span>
							</div>

							<div class="flex shrink-0 items-center gap-1.5">
								<Button size="sm" type="outlined" @click="openModrinthProject(hit)">
									Details
								</Button>
								<Button
									:type="installedProjectIds.has(hit.project_id) ? 'outlined' : 'colored'"
									:color="installedProjectIds.has(hit.project_id) ? 'default' : 'brand'"
									size="sm"
									:disabled="
										installedProjectIds.has(hit.project_id) ||
										installingModrinthIds.has(hit.project_id)
									"
									@click="handleInstallModrinth(hit)"
								>
									<SpinnerIcon
										v-if="installingModrinthIds.has(hit.project_id)"
										class="size-3.5 animate-spin"
									/>
									<CheckIcon v-else-if="installedProjectIds.has(hit.project_id)" class="size-3.5" />
									<DownloadIcon v-else class="size-3.5" />
									<span>
										{{
											installingModrinthIds.has(hit.project_id)
												? 'Installing'
												: installedProjectIds.has(hit.project_id)
													? 'Installed'
													: 'Install'
										}}
									</span>
								</Button>
							</div>
						</div>
					</div>
				</article>
			</div>

			<!-- Modrinth List Rows -->
			<div v-else class="flex flex-col gap-2.5">
				<article
					v-for="hit in modrinthHits"
					:key="hit.project_id"
					class="solara-mp-card flex flex-wrap items-center gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4 transition-all"
				>
					<button
						type="button"
						class="flex size-16 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-2 p-0"
						@click="openModrinthProject(hit)"
					>
						<img
							v-if="hit.icon_url"
							:src="hit.icon_url"
							:alt="hit.name"
							loading="lazy"
							class="size-full object-cover"
						/>
						<PackageIcon v-else class="size-7 text-secondary" />
					</button>

					<div class="flex min-w-0 flex-1 flex-col gap-1">
						<div class="flex flex-wrap items-baseline gap-2">
							<button
								type="button"
								class="m-0 max-w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-base font-bold text-contrast hover:text-brand"
								@click="openModrinthProject(hit)"
							>
								{{ hit.name }}
							</button>
							<span class="text-xs text-secondary">by {{ hit.author || 'Community' }}</span>
						</div>
						<p class="m-0 line-clamp-1 text-xs text-secondary">{{ hit.summary }}</p>
						<div class="mt-0.5 flex flex-wrap items-center gap-1.5 text-[11px] font-medium">
							<span
								v-for="ldr in modrinthLoaders(hit)"
								:key="ldr"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-primary"
							>
								{{ formatLoaderLabel(ldr) }}
							</span>
							<span
								v-if="modrinthLatestMcVersion(hit)"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 tabular-nums text-primary"
							>
								MC {{ modrinthLatestMcVersion(hit) }}
							</span>
							<span
								v-for="tag in modrinthTags(hit)"
								:key="tag"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-secondary"
							>
								{{ formatCategoryLabel(tag) }}
							</span>
						</div>
					</div>

					<div class="flex shrink-0 items-center gap-3">
						<div class="hidden flex-col items-end text-xs text-secondary tabular-nums sm:flex">
							<span class="font-semibold text-primary">{{ formatDownloads(hit.downloads) }}</span>
							<span v-if="hit.date_modified">{{ dayjs(hit.date_modified).fromNow() }}</span>
						</div>
						<Button size="sm" type="outlined" @click="openModrinthProject(hit)">Details</Button>
						<Button
							:type="installedProjectIds.has(hit.project_id) ? 'outlined' : 'colored'"
							:color="installedProjectIds.has(hit.project_id) ? 'default' : 'brand'"
							size="sm"
							:disabled="
								installedProjectIds.has(hit.project_id) ||
								installingModrinthIds.has(hit.project_id)
							"
							@click="handleInstallModrinth(hit)"
						>
							<SpinnerIcon
								v-if="installingModrinthIds.has(hit.project_id)"
								class="size-3.5 animate-spin"
							/>
							<CheckIcon v-else-if="installedProjectIds.has(hit.project_id)" class="size-3.5" />
							<DownloadIcon v-else class="size-3.5" />
							<span>
								{{
									installingModrinthIds.has(hit.project_id)
										? 'Installing'
										: installedProjectIds.has(hit.project_id)
											? 'Installed'
											: 'Install'
								}}
							</span>
						</Button>
					</div>
				</article>
			</div>
		</template>

		<!-- CURSEFORGE RESULTS -->
		<template v-else>
			<div
				v-if="curseforgeItems.length === 0"
				class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-12 text-center"
			>
				<p class="m-0 text-sm font-semibold text-contrast">No CurseForge modpacks matched your filters.</p>
				<Button v-if="hasActiveFilters" size="sm" @click="resetFilters">Clear filters</Button>
			</div>

			<!-- CurseForge Grid Cards -->
			<div
				v-else-if="viewMode === 'grid'"
				class="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3"
			>
				<article
					v-for="pack in curseforgeItems"
					:key="pack.id"
					class="solara-mp-card group flex flex-col overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-3 transition-all"
				>
					<!-- Cover Header -->
					<div
						class="relative h-28 w-full cursor-pointer overflow-hidden bg-surface-2"
						@click="openCurseForgeProject(pack)"
					>
						<img
							v-if="pack.bannerUrl"
							:src="pack.bannerUrl"
							:alt="pack.name"
							loading="lazy"
							class="size-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
						/>
						<img
							v-else-if="pack.logoUrl"
							:src="pack.logoUrl"
							alt=""
							loading="lazy"
							class="size-full scale-125 object-cover opacity-35 blur-xl"
						/>
						<div class="solara-card-scrim pointer-events-none absolute inset-0" />

						<div class="absolute right-2.5 top-2.5 flex items-center gap-1.5">
							<span
								v-if="pack.loaders[0]"
								class="solara-cover-pill rounded-md px-2 py-0.5 text-[11px] font-semibold"
							>
								{{ formatLoaderLabel(pack.loaders[0]) }}
							</span>
							<span
								v-if="pack.gameVersions[0]"
								class="solara-cover-pill rounded-md px-2 py-0.5 text-[11px] font-semibold tabular-nums"
							>
								{{ pack.gameVersions[0] }}
							</span>
						</div>
					</div>

					<!-- Body -->
					<div class="relative flex flex-1 flex-col gap-2.5 px-4 pb-4 pt-3">
						<div class="flex items-end gap-3">
							<button
								type="button"
								class="-mt-10 flex size-14 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-xl border-2 border-solid border-surface-3 bg-surface-2 p-0 shadow-md"
								@click="openCurseForgeProject(pack)"
							>
								<img
									v-if="pack.logoUrl"
									:src="pack.logoUrl"
									:alt="pack.name"
									loading="lazy"
									class="size-full object-cover"
								/>
								<PackageIcon v-else class="size-6 text-secondary" />
							</button>

							<div class="min-w-0 flex-1">
								<button
									type="button"
									class="m-0 block w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-[15px] font-bold leading-snug text-contrast transition-colors hover:text-brand"
									@click="openCurseForgeProject(pack)"
								>
									{{ pack.name }}
								</button>
								<p class="m-0 truncate text-xs text-secondary">by {{ pack.author }}</p>
							</div>
						</div>

						<p class="m-0 line-clamp-2 min-h-[2.4rem] text-xs leading-relaxed text-secondary">
							{{ pack.summary }}
						</p>

						<div v-if="pack.categories.length > 0" class="flex flex-wrap items-center gap-1.5">
							<span
								v-for="cat in pack.categories.slice(0, 2)"
								:key="cat.id"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-[11px] font-medium text-secondary"
							>
								{{ cat.name }}
							</span>
						</div>

						<!-- Dedicated Footer -->
						<div
							class="mt-auto flex items-center justify-between gap-2 border-0 border-t border-solid border-surface-5 pt-3"
						>
							<div class="flex items-center gap-3 text-xs font-medium text-secondary tabular-nums">
								<span class="flex items-center gap-1" title="Total downloads">
									<DownloadIcon class="size-3.5" />
									{{ formatDownloads(pack.downloads) }}
								</span>
								<span
									v-if="pack.dateModified"
									class="flex items-center gap-1"
									title="Last updated"
								>
									<UpdatedIcon class="size-3.5" />
									{{ dayjs(pack.dateModified).fromNow() }}
								</span>
							</div>

							<div class="flex shrink-0 items-center gap-1.5">
								<Button size="sm" type="outlined" @click="openCurseForgeProject(pack)">
									Details
								</Button>
								<Button
									type="colored"
									color="brand"
									size="sm"
									:disabled="installingCurseForgeKey !== null"
									@click="handleInstallCurseForge(pack)"
								>
									<SpinnerIcon
										v-if="
											pack.latestFile &&
											installingCurseForgeKey === `cf:${pack.id}:${pack.latestFile.id}`
										"
										class="size-3.5 animate-spin"
									/>
									<DownloadIcon v-else class="size-3.5" />
									<span>
										{{
											pack.latestFile &&
											installingCurseForgeKey === `cf:${pack.id}:${pack.latestFile.id}`
												? 'Installing'
												: 'Install'
										}}
									</span>
								</Button>
							</div>
						</div>
					</div>
				</article>
			</div>

			<!-- CurseForge List Rows -->
			<div v-else class="flex flex-col gap-2.5">
				<article
					v-for="pack in curseforgeItems"
					:key="pack.id"
					class="solara-mp-card flex flex-wrap items-center gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4 transition-all"
				>
					<button
						type="button"
						class="flex size-16 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-2 p-0"
						@click="openCurseForgeProject(pack)"
					>
						<img
							v-if="pack.logoUrl"
							:src="pack.logoUrl"
							:alt="pack.name"
							loading="lazy"
							class="size-full object-cover"
						/>
						<PackageIcon v-else class="size-7 text-secondary" />
					</button>

					<div class="flex min-w-0 flex-1 flex-col gap-1">
						<div class="flex flex-wrap items-baseline gap-2">
							<button
								type="button"
								class="m-0 max-w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-base font-bold text-contrast hover:text-brand"
								@click="openCurseForgeProject(pack)"
							>
								{{ pack.name }}
							</button>
							<span class="text-xs text-secondary">by {{ pack.author }}</span>
						</div>
						<p class="m-0 line-clamp-1 text-xs text-secondary">{{ pack.summary }}</p>
						<div class="mt-0.5 flex flex-wrap items-center gap-1.5 text-[11px] font-medium">
							<span
								v-for="ldr in pack.loaders.slice(0, 2)"
								:key="ldr"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-primary"
							>
								{{ formatLoaderLabel(ldr) }}
							</span>
							<span
								v-if="pack.gameVersions[0]"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 tabular-nums text-primary"
							>
								MC {{ pack.gameVersions[0] }}
							</span>
							<span
								v-for="cat in pack.categories.slice(0, 2)"
								:key="cat.id"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-secondary"
							>
								{{ cat.name }}
							</span>
						</div>
					</div>

					<div class="flex shrink-0 items-center gap-3">
						<div class="hidden flex-col items-end text-xs text-secondary tabular-nums sm:flex">
							<span class="font-semibold text-primary">{{ formatDownloads(pack.downloads) }}</span>
							<span v-if="pack.dateModified">{{ dayjs(pack.dateModified).fromNow() }}</span>
						</div>
						<Button size="sm" type="outlined" @click="openCurseForgeProject(pack)">
							Details
						</Button>
						<Button
							type="colored"
							color="brand"
							size="sm"
							:disabled="installingCurseForgeKey !== null"
							@click="handleInstallCurseForge(pack)"
						>
							<SpinnerIcon
								v-if="
									pack.latestFile &&
									installingCurseForgeKey === `cf:${pack.id}:${pack.latestFile.id}`
								"
								class="size-3.5 animate-spin"
							/>
							<DownloadIcon v-else class="size-3.5" />
							<span>
								{{
									pack.latestFile &&
									installingCurseForgeKey === `cf:${pack.id}:${pack.latestFile.id}`
										? 'Installing'
										: 'Install'
								}}
							</span>
						</Button>
					</div>
				</article>
			</div>
		</template>

		<!-- Pagination -->
		<footer
			v-if="!loading && !errorMessage && totalPages > 1"
			class="flex items-center justify-center pt-2"
		>
			<Pagination :page="page" :count="totalPages" @switch-page="switchPage" />
		</footer>

		<!-- CurseForge Version Picker Modal -->
		<CurseForgePackModal
			:pack="activeCurseForgeModalPack"
			:installing-key="installingCurseForgeKey"
			@close="activeCurseForgeModalPack = null"
			@install="(pack, file) => handleInstallCurseForge(pack, file)"
		/>
	</div>
</template>

<style scoped>
.solara-source-modrinth-active {
	background-color: color-mix(in srgb, #1bd96a 14%, var(--surface-4));
	box-shadow: inset 0 0 0 1px color-mix(in srgb, #1bd96a 34%, transparent);
}

.solara-source-cf-active {
	background-color: color-mix(in srgb, #f16436 14%, var(--surface-4));
	box-shadow: inset 0 0 0 1px color-mix(in srgb, #f16436 34%, transparent);
}

.solara-loader-active {
	border-color: var(--color-brand);
	background-color: color-mix(in srgb, var(--color-brand) 16%, var(--surface-2));
}

.solara-mp-card:hover {
	border-color: color-mix(in srgb, var(--color-brand) 48%, var(--surface-5));
	transform: translateY(-1px);
}

.solara-card-scrim {
	background: linear-gradient(
		to bottom,
		rgba(0, 0, 0, 0.08) 0%,
		rgba(0, 0, 0, 0.25) 60%,
		color-mix(in srgb, var(--surface-3) 96%, transparent) 100%
	);
}

.solara-cover-pill {
	background-color: rgba(10, 14, 22, 0.76);
	color: #f8fafc;
	border: 1px solid rgba(255, 255, 255, 0.16);
	backdrop-filter: blur(6px);
}
</style>
