<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	ChevronLeftIcon,
	ChevronRightIcon,
	DownloadIcon,
	PackageIcon,
	SearchIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { Button, injectNotificationManager } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

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

const props = defineProps<{
	gameVersions: Labrinth.Tags.v2.GameVersion[]
}>()

const router = useRouter()
const queryClient = useQueryClient()
const appSettings = useAppSettings()
const { handleError, addNotification } = injectNotificationManager()
const { install: installModrinthVersion } = injectContentInstall()

const source = ref<ProviderSource>('modrinth')
const searchQuery = ref('')
const debouncedQuery = ref('')
const selectedLoader = ref<LoaderFilter>('all')
const selectedGameVersion = ref('')
const selectedCategory = ref('')
const selectedSort = ref<SortOption>('relevance')
const page = ref(1)
const pageSize = 20

const loading = ref(false)
const errorMessage = ref<string | null>(null)
const offline = ref(!navigator.onLine)

// Modrinth state
const modrinthHits = ref<Labrinth.Search.v3.ResultSearchProject[]>([])
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
	{ id: 'all', label: 'All loaders' },
	{ id: 'fabric', label: 'Fabric' },
	{ id: 'forge', label: 'Forge' },
	{ id: 'neoforge', label: 'NeoForge' },
	{ id: 'quilt', label: 'Quilt' },
]

const CATEGORIES: { id: string; label: string }[] = [
	{ id: '', label: 'All categories' },
	{ id: 'adventure', label: 'Adventure' },
	{ id: 'tech', label: 'Technology' },
	{ id: 'magic', label: 'Magic' },
	{ id: 'optimization', label: 'Optimization' },
	{ id: 'combat', label: 'Combat' },
	{ id: 'multiplayer', label: 'Multiplayer' },
	{ id: 'quests', label: 'Quests' },
	{ id: 'kitchen-sink', label: 'Kitchen Sink' },
	{ id: 'lightweight', label: 'Lightweight' },
	{ id: 'skyblock', label: 'Skyblock' },
	{ id: 'hardcore', label: 'Hardcore' },
]

const SORT_OPTIONS: { id: SortOption; label: string }[] = [
	{ id: 'relevance', label: 'Most popular' },
	{ id: 'downloads', label: 'Most downloads' },
	{ id: 'updated', label: 'Recently updated' },
	{ id: 'newest', label: 'Newest' },
]

const releaseGameVersions = computed(() => {
	const releases = (props.gameVersions ?? [])
		.filter((v) => v.version_type === 'release')
		.map((v) => v.version)
	if (releases.length > 0) return releases.slice(0, 40)
	return ['1.21.4', '1.21.1', '1.20.4', '1.20.1', '1.19.2', '1.18.2', '1.16.5', '1.12.2']
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
		selectedGameVersion.value !== '' ||
		selectedCategory.value !== '' ||
		selectedSort.value !== 'relevance',
)

const totalResults = computed(() =>
	source.value === 'modrinth' ? modrinthTotal.value : curseforgeTotal.value,
)

const totalPages = computed(() =>
	Math.max(1, Math.min(500, Math.ceil(totalResults.value / pageSize))),
)

let debounceTimer: ReturnType<typeof setTimeout> | null = null
watch(searchQuery, (val) => {
	if (debounceTimer) clearTimeout(debounceTimer)
	debounceTimer = setTimeout(() => {
		debouncedQuery.value = val.trim()
		page.value = 1
	}, 260)
})

watch([source, selectedLoader, selectedGameVersion, selectedCategory, selectedSort, hideInstalledModpacks], () => {
	page.value = 1
})

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

let activeRequestToken = 0

async function fetchCatalog() {
	const token = ++activeRequestToken
	loading.value = true
	errorMessage.value = null

	try {
		if (source.value === 'modrinth') {
			const facets: string[][] = [['project_types:modpack']]
			if (selectedLoader.value !== 'all') {
				facets.push([`categories:${selectedLoader.value}`])
			}
			if (selectedGameVersion.value) {
				facets.push([`game_versions:${selectedGameVersion.value}`])
			}
			if (selectedCategory.value) {
				facets.push([`categories:${selectedCategory.value}`])
			}
			if (hideInstalledModpacks.value && installedProjectIds.value.size > 0) {
				for (const id of installedProjectIds.value) {
					facets.push([`project_id!=${id}`])
				}
			}

			const qs = new URLSearchParams()
			if (debouncedQuery.value) qs.set('query', debouncedQuery.value)
			qs.set('limit', String(pageSize))
			qs.set('offset', String((page.value - 1) * pageSize))
			qs.set('index', selectedSort.value)
			qs.set('facets', JSON.stringify(facets))

			const raw = (await get_search_results_v3(qs.toString(), 'must_revalidate')) as {
				result?: Labrinth.Search.v3.SearchResults
			} | null

			if (token !== activeRequestToken) return
			modrinthHits.value = raw?.result?.hits ?? []
			modrinthTotal.value = raw?.result?.total_hits ?? 0
		} else {
			const sortMap: Record<SortOption, CurseForgeSort> = {
				relevance: 'popularity',
				downloads: 'downloads',
				updated: 'updated',
				newest: 'newest',
			}
			const categoryId = selectedCategory.value
				? CURSEFORGE_CATEGORY_MAP[selectedCategory.value]
				: undefined

			const res = await searchCurseForgeModpacks({
				q: debouncedQuery.value || undefined,
				gameVersion: selectedGameVersion.value || undefined,
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
	[source, debouncedQuery, selectedLoader, selectedGameVersion, selectedCategory, selectedSort, page, hideInstalledModpacks],
	() => {
		void fetchCatalog()
	},
	{ immediate: true },
)

function resetFilters() {
	searchQuery.value = ''
	debouncedQuery.value = ''
	selectedLoader.value = 'all'
	selectedGameVersion.value = ''
	selectedCategory.value = ''
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

function modrinthLoaders(hit: Labrinth.Search.v3.ResultSearchProject): string[] {
	const known = ['fabric', 'forge', 'neoforge', 'quilt']
	const raw = [
		...(hit.categories ?? []),
		...(hit.display_categories ?? []),
	].map((c) => c.toLowerCase())
	return known.filter((l) => raw.includes(l))
}

function openModrinthProject(hit: Labrinth.Search.v3.ResultSearchProject) {
	void router.push({
		path: `/project/${hit.project_id ?? hit.slug}`,
		query: { b: '/browse/modpack' },
	})
}

async function handleInstallModrinth(hit: Labrinth.Search.v3.ResultSearchProject) {
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
				preferredGameVersion: selectedGameVersion.value || undefined,
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
		activeCurseForgeModalPack.value = pack
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
	<div class="mx-auto flex w-full max-w-[1260px] flex-col gap-5 px-6 py-6">
		<!-- Top Header & Source Switcher -->
		<header class="flex flex-wrap items-center justify-between gap-4">
			<div class="flex flex-col gap-1">
				<h1 class="m-0 text-2xl font-bold tracking-tight text-contrast">
					ModPackInstaller
				</h1>
				<p class="m-0 text-sm text-secondary">
					<template v-if="source === 'modrinth'">
						Browse and install Minecraft modpacks from Modrinth
						<span v-if="totalResults > 0" class="tabular-nums">
							· {{ totalResults.toLocaleString() }} packs
						</span>
					</template>
					<template v-else>
						Browse and install Minecraft modpacks from CurseForge
						<span v-if="curseforgeConfigured && totalResults > 0" class="tabular-nums">
							· {{ totalResults.toLocaleString() }} packs
						</span>
					</template>
				</p>
			</div>

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
					class="flex cursor-pointer items-center gap-2 rounded-lg border-none px-4 py-2 text-sm font-semibold transition-colors"
					:class="
						source === 'modrinth'
							? 'bg-surface-4 text-contrast shadow-sm'
							: 'bg-transparent text-secondary hover:text-contrast'
					"
					@click="source = 'modrinth'"
				>
					<span
						class="size-2 rounded-full"
						:class="source === 'modrinth' ? 'bg-emerald-400' : 'bg-surface-5'"
						aria-hidden="true"
					/>
					<span>Modrinth</span>
				</button>
				<button
					type="button"
					role="tab"
					:aria-selected="source === 'curseforge'"
					class="flex cursor-pointer items-center gap-2 rounded-lg border-none px-4 py-2 text-sm font-semibold transition-colors"
					:class="
						source === 'curseforge'
							? 'bg-surface-4 text-contrast shadow-sm'
							: 'bg-transparent text-secondary hover:text-contrast'
					"
					@click="source = 'curseforge'"
				>
					<span
						class="size-2 rounded-full"
						:class="source === 'curseforge' ? 'bg-orange-400' : 'bg-surface-5'"
						aria-hidden="true"
					/>
					<span>CurseForge</span>
				</button>
			</div>
		</header>

		<!-- Unified Search & Filter Panel -->
		<section class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4">
			<!-- Row 1: Search Input + Sort -->
			<div class="flex flex-wrap items-center gap-3">
				<div
					class="flex min-w-[240px] flex-1 items-center gap-2.5 rounded-xl border border-solid border-surface-5 bg-surface-2 px-3.5 py-2.5 focus-within:border-brand"
				>
					<SearchIcon class="size-4 shrink-0 text-secondary" aria-hidden="true" />
					<input
						v-model="searchQuery"
						type="text"
						:placeholder="
							source === 'modrinth'
								? 'Search Modrinth modpacks...'
								: 'Search CurseForge modpacks...'
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

				<select
					v-model="selectedSort"
					aria-label="Sort modpacks"
					class="h-10 cursor-pointer rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 text-sm font-medium text-contrast outline-none focus:border-brand"
				>
					<option v-for="opt in SORT_OPTIONS" :key="opt.id" :value="opt.id">
						{{ opt.label }}
					</option>
				</select>
			</div>

			<!-- Row 2: Loaders + MC Version + Category + Options -->
			<div class="flex flex-wrap items-center justify-between gap-3">
				<div class="flex flex-wrap items-center gap-1.5" role="group" aria-label="Filter by mod loader">
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

				<div class="flex flex-wrap items-center gap-2.5">
					<select
						v-model="selectedGameVersion"
						aria-label="Filter by Minecraft version"
						class="h-8 cursor-pointer rounded-lg border border-solid border-surface-5 bg-surface-2 px-2.5 text-xs font-medium text-contrast outline-none focus:border-brand"
					>
						<option value="">All versions</option>
						<option v-for="ver in releaseGameVersions" :key="ver" :value="ver">
							{{ ver }}
						</option>
					</select>

					<select
						v-model="selectedCategory"
						aria-label="Filter by category"
						class="h-8 cursor-pointer rounded-lg border border-solid border-surface-5 bg-surface-2 px-2.5 text-xs font-medium text-contrast outline-none focus:border-brand"
					>
						<option v-for="cat in CATEGORIES" :key="cat.id" :value="cat.id">
							{{ cat.label }}
						</option>
					</select>

					<label
						v-if="source === 'modrinth'"
						class="flex cursor-pointer select-none items-center gap-2 rounded-lg border border-solid border-surface-5 bg-surface-2 px-2.5 py-1.5 text-xs font-medium text-secondary transition-colors hover:text-contrast"
					>
						<input
							v-model="hideInstalledModpacks"
							type="checkbox"
							class="solara-checkbox m-0 cursor-pointer"
						/>
						<span>Hide installed</span>
					</label>

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

		<!-- Offline Banner -->
		<div
			v-if="offline"
			class="rounded-2xl border border-solid border-surface-5 bg-surface-3 p-6 text-center text-sm text-secondary"
		>
			You appear to be offline. Reconnect to the internet to browse and install modpacks.
		</div>

		<!-- Error State -->
		<div
			v-else-if="errorMessage"
			class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-8 text-center"
		>
			<p class="m-0 text-sm font-semibold text-contrast">{{ errorMessage }}</p>
			<Button size="sm" @click="fetchCatalog">Retry</Button>
		</div>

		<!-- CurseForge Not Yet Configured State (Honest Status) -->
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
				CurseForge modpack search is routed securely through Solara's private server so no API key is ever embedded in the app. Once the CurseForge API key is added on the server, modpacks will appear here automatically.
			</p>
			<Button size="sm" @click="source = 'modrinth'">Browse Modrinth Modpacks</Button>
		</div>

		<!-- Loading Skeleton Grid -->
		<div
			v-else-if="loading"
			class="grid grid-cols-1 gap-3.5 xl:grid-cols-2"
			aria-busy="true"
		>
			<div
				v-for="n in 8"
				:key="n"
				class="flex items-start gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
			>
				<div class="size-16 shrink-0 animate-pulse rounded-xl bg-surface-4" />
				<div class="flex flex-1 flex-col gap-2">
					<div class="h-4 w-44 animate-pulse rounded bg-surface-4" />
					<div class="h-3 w-full animate-pulse rounded bg-surface-4" />
					<div class="h-3 w-2/3 animate-pulse rounded bg-surface-4" />
				</div>
			</div>
		</div>

		<!-- Modrinth Results Grid -->
		<template v-else-if="source === 'modrinth'">
			<div
				v-if="modrinthHits.length === 0"
				class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-10 text-center"
			>
				<p class="m-0 text-sm font-semibold text-contrast">No Modrinth modpacks matched your filters.</p>
				<Button v-if="hasActiveFilters" size="sm" @click="resetFilters">Clear filters</Button>
			</div>

			<div v-else class="grid grid-cols-1 gap-3.5 xl:grid-cols-2">
				<article
					v-for="hit in modrinthHits"
					:key="hit.project_id"
					class="solara-mp-card group flex items-start gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4 transition-colors"
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

					<div class="flex min-w-0 flex-1 flex-col gap-1.5">
						<div class="flex items-start justify-between gap-3">
							<div class="min-w-0">
								<button
									type="button"
									class="m-0 max-w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-base font-bold text-contrast hover:text-brand"
									@click="openModrinthProject(hit)"
								>
									{{ hit.name }}
								</button>
								<p class="m-0 text-xs text-secondary">
									by {{ hit.author || 'Community' }}
								</p>
							</div>

							<Button
								:type="installedProjectIds.has(hit.project_id) ? 'outlined' : 'colored'"
								:color="installedProjectIds.has(hit.project_id) ? 'default' : 'brand'"
								size="sm"
								class="shrink-0"
								:disabled="
									installedProjectIds.has(hit.project_id) ||
									installingModrinthIds.has(hit.project_id)
								"
								@click="handleInstallModrinth(hit)"
							>
								<SpinnerIcon
									v-if="installingModrinthIds.has(hit.project_id)"
									class="size-4 animate-spin"
								/>
								<CheckIcon v-else-if="installedProjectIds.has(hit.project_id)" class="size-4" />
								<DownloadIcon v-else class="size-4" />
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

						<p class="m-0 line-clamp-2 text-xs leading-relaxed text-secondary">
							{{ hit.summary }}
						</p>

						<div class="mt-1 flex flex-wrap items-center gap-1.5 text-[11px] font-medium text-secondary">
							<span
								v-for="ldr in modrinthLoaders(hit)"
								:key="ldr"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-primary"
							>
								{{ formatLoaderLabel(ldr) }}
							</span>
							<span
								v-if="hit.latest_version"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 tabular-nums text-primary"
							>
								{{ hit.latest_version }}
							</span>
							<span class="ml-auto tabular-nums">
								{{ formatDownloads(hit.downloads) }} downloads
								<template v-if="hit.date_modified">
									· {{ dayjs(hit.date_modified).fromNow() }}
								</template>
							</span>
						</div>
					</div>
				</article>
			</div>
		</template>

		<!-- CurseForge Results Grid -->
		<template v-else>
			<div
				v-if="curseforgeItems.length === 0"
				class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-10 text-center"
			>
				<p class="m-0 text-sm font-semibold text-contrast">No CurseForge modpacks matched your filters.</p>
				<Button v-if="hasActiveFilters" size="sm" @click="resetFilters">Clear filters</Button>
			</div>

			<div v-else class="grid grid-cols-1 gap-3.5 xl:grid-cols-2">
				<article
					v-for="pack in curseforgeItems"
					:key="pack.id"
					class="solara-mp-card group flex items-start gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4 transition-colors"
				>
					<button
						type="button"
						class="flex size-16 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-2 p-0"
						@click="activeCurseForgeModalPack = pack"
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

					<div class="flex min-w-0 flex-1 flex-col gap-1.5">
						<div class="flex items-start justify-between gap-3">
							<div class="min-w-0">
								<button
									type="button"
									class="m-0 max-w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-base font-bold text-contrast hover:text-brand"
									@click="activeCurseForgeModalPack = pack"
								>
									{{ pack.name }}
								</button>
								<p class="m-0 text-xs text-secondary">
									by {{ pack.author }}
								</p>
							</div>

							<div class="flex shrink-0 items-center gap-1.5">
								<Button
									size="sm"
									@click="activeCurseForgeModalPack = pack"
								>
									Versions
								</Button>
								<Button
									type="colored"
									color="brand"
									size="sm"
									:disabled="installingCurseForgeKey !== null"
									@click="handleInstallCurseForge(pack)"
								>
									<SpinnerIcon
										v-if="installingCurseForgeKey?.startsWith(`cf:${pack.id}:`)"
										class="size-4 animate-spin"
									/>
									<DownloadIcon v-else class="size-4" />
									<span>
										{{
											installingCurseForgeKey?.startsWith(`cf:${pack.id}:`)
												? 'Installing'
												: 'Install'
										}}
									</span>
								</Button>
							</div>
						</div>

						<p class="m-0 line-clamp-2 text-xs leading-relaxed text-secondary">
							{{ pack.summary }}
						</p>

						<div class="mt-1 flex flex-wrap items-center gap-1.5 text-[11px] font-medium text-secondary">
							<span
								v-for="ldr in pack.loaders.slice(0, 3)"
								:key="ldr"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-primary"
							>
								{{ formatLoaderLabel(ldr) }}
							</span>
							<span
								v-if="pack.gameVersions.length > 0"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 tabular-nums text-primary"
							>
								{{ pack.gameVersions[0] }}
							</span>
							<span class="ml-auto tabular-nums">
								{{ formatDownloads(pack.downloads) }} downloads
								<template v-if="pack.dateModified">
									· {{ dayjs(pack.dateModified).fromNow() }}
								</template>
							</span>
						</div>
					</div>
				</article>
			</div>
		</template>

		<!-- Pagination Controls -->
		<nav
			v-if="!loading && !errorMessage && totalPages > 1 && (source === 'modrinth' || curseforgeConfigured)"
			class="flex items-center justify-center gap-2 pt-2"
			aria-label="Pagination"
		>
			<Button size="sm" :disabled="page <= 1" @click="page = Math.max(1, page - 1)">
				<ChevronLeftIcon class="size-4" />
				<span>Previous</span>
			</Button>
			<span class="px-3 text-xs font-semibold tabular-nums text-secondary">
				Page {{ page }} of {{ totalPages }}
			</span>
			<Button size="sm" :disabled="page >= totalPages" @click="page = Math.min(totalPages, page + 1)">
				<span>Next</span>
				<ChevronRightIcon class="size-4" />
			</Button>
		</nav>

		<!-- CurseForge Details & Versions Modal -->
		<CurseForgePackModal
			:pack="activeCurseForgeModalPack"
			:installing-key="installingCurseForgeKey"
			@close="activeCurseForgeModalPack = null"
			@install="(pack, file) => handleInstallCurseForge(pack, file)"
		/>
	</div>
</template>

<style scoped>
.solara-loader-active {
	border-color: var(--color-brand);
	background: color-mix(in srgb, var(--color-brand) 14%, transparent);
}

.solara-mp-card:hover {
	border-color: color-mix(in srgb, var(--color-brand) 40%, var(--surface-5));
	background: color-mix(in srgb, var(--surface-4) 55%, var(--surface-3));
}

.solara-checkbox {
	width: 14px;
	height: 14px;
	accent-color: var(--color-brand);
}
</style>
