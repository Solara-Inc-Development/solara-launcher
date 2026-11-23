<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ArrowLeftIcon,
	CheckIcon,
	ChevronDownIcon,
	DownloadIcon,
	ExternalIcon,
	FilterIcon,
	GridIcon,
	ListIcon,
	PackageIcon,
	SearchIcon,
	SpinnerIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	Checkbox,
	Combobox,
	formatLoaderLabel,
	injectNotificationManager,
	Pagination,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
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
type EnvironmentFilter = 'client' | 'server'

interface CategoryOption {
	id: string
	label: string
	modrinthSlug: string
	curseforgeId: number
}

interface ModrinthV3Hit {
	project_id: string
	slug?: string
	name?: string
	title?: string
	summary?: string
	description?: string
	author?: string
	icon_url?: string | null
	featured_gallery?: string | null
	gallery?: string[]
	downloads: number
	date_modified?: string
	display_categories?: string[]
	categories?: string[]
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
const viewMode = ref<ViewMode>('list')
const searchQuery = ref('')
const debouncedQuery = ref('')
const selectedSort = ref<SortOption>('relevance')
const page = ref(1)
const pageSize = 18

// Sidebar filter states (work on both Modrinth & CurseForge)
const selectedCategories = ref<string[]>([])
const selectedEnvironments = ref<EnvironmentFilter[]>([])
const matchInstanceVersion = ref(true)
const matchInstanceLoader = ref(true)
const hideInstalled = ref(false)
const openSections = ref({
	categories: true,
	compatibility: true,
	environment: true,
})

const loading = ref(false)
const errorMessage = ref<string | null>(null)

// Modrinth state
const modrinthHits = ref<ModrinthV3Hit[]>([])
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

const SORT_OPTIONS: { value: SortOption; label: string }[] = [
	{ value: 'relevance', label: 'Most Popular' },
	{ value: 'downloads', label: 'Most Downloads' },
	{ value: 'updated', label: 'Recently Updated' },
	{ value: 'newest', label: 'Newest' },
]

const CATEGORIES_BY_KIND: Record<ContentKind, CategoryOption[]> = {
	mod: [
		{ id: 'optimization', label: 'Optimization', modrinthSlug: 'optimization', curseforgeId: 6814 },
		{ id: 'adventure', label: 'Adventure & RPG', modrinthSlug: 'adventure', curseforgeId: 422 },
		{ id: 'technology', label: 'Technology', modrinthSlug: 'technology', curseforgeId: 412 },
		{ id: 'magic', label: 'Magic', modrinthSlug: 'magic', curseforgeId: 419 },
		{ id: 'worldgen', label: 'World Gen', modrinthSlug: 'worldgen', curseforgeId: 406 },
		{ id: 'utility', label: 'Utility & QoL', modrinthSlug: 'utility', curseforgeId: 5191 },
		{ id: 'storage', label: 'Storage', modrinthSlug: 'storage', curseforgeId: 420 },
		{ id: 'equipment', label: 'Combat & Gear', modrinthSlug: 'equipment', curseforgeId: 434 },
		{ id: 'food', label: 'Food & Farming', modrinthSlug: 'food', curseforgeId: 436 },
		{ id: 'mobs', label: 'Mobs', modrinthSlug: 'mobs', curseforgeId: 411 },
		{ id: 'decoration', label: 'Cosmetic & Decor', modrinthSlug: 'decoration', curseforgeId: 424 },
		{ id: 'library', label: 'API & Library', modrinthSlug: 'library', curseforgeId: 421 },
	],
	resourcepack: [
		{ id: '16x', label: '16x Resolution', modrinthSlug: '16x', curseforgeId: 393 },
		{ id: '32x', label: '32x Resolution', modrinthSlug: '32x', curseforgeId: 394 },
		{ id: '64x', label: '64x Resolution', modrinthSlug: '64x', curseforgeId: 395 },
		{ id: '128x', label: '128x Resolution', modrinthSlug: '128x', curseforgeId: 396 },
		{ id: 'realistic', label: 'Realistic', modrinthSlug: 'realistic', curseforgeId: 400 },
		{ id: 'simplistic', label: 'Traditional / Clean', modrinthSlug: 'simplistic', curseforgeId: 403 },
		{ id: 'medieval', label: 'Medieval', modrinthSlug: 'medieval', curseforgeId: 402 },
		{ id: 'animated', label: 'Animated', modrinthSlug: 'tweaks', curseforgeId: 404 },
		{ id: 'modded', label: 'Mod Support', modrinthSlug: 'modded', curseforgeId: 4465 },
	],
	shader: [
		{ id: 'realistic', label: 'Realistic', modrinthSlug: 'realistic', curseforgeId: 6553 },
		{ id: 'fantasy', label: 'Fantasy', modrinthSlug: 'fantasy', curseforgeId: 6554 },
		{ id: 'vanilla-like', label: 'Vanilla-like', modrinthSlug: 'vanilla-like', curseforgeId: 6555 },
		{ id: 'atmosphere', label: 'Atmospheric', modrinthSlug: 'atmosphere', curseforgeId: 6553 },
		{ id: 'cartoon', label: 'Stylized / Cartoon', modrinthSlug: 'cartoon', curseforgeId: 6554 },
	],
	datapack: [
		{ id: 'adventure', label: 'Adventure', modrinthSlug: 'adventure', curseforgeId: 6948 },
		{ id: 'technology', label: 'Technology', modrinthSlug: 'technology', curseforgeId: 6951 },
		{ id: 'magic', label: 'Magic', modrinthSlug: 'magic', curseforgeId: 6952 },
		{ id: 'utility', label: 'Utility', modrinthSlug: 'utility', curseforgeId: 6953 },
		{ id: 'worldgen', label: 'Fantasy & World', modrinthSlug: 'worldgen', curseforgeId: 6949 },
		{ id: 'library', label: 'Library', modrinthSlug: 'library', curseforgeId: 6950 },
	],
}

const availableCategories = computed(() => CATEGORIES_BY_KIND[contentKind.value] ?? [])

const activeFilterCount = computed(() => {
	let count = selectedCategories.value.length
	if (source.value === 'modrinth') count += selectedEnvironments.value.length
	if (!matchInstanceVersion.value) count += 1
	if (contentKind.value === 'mod' && !isVanilla.value && !matchInstanceLoader.value) count += 1
	if (hideInstalled.value) count += 1
	return count
})

const contentKindPluralLabel = computed(() => {
	switch (contentKind.value) {
		case 'resourcepack':
			return 'resource packs'
		case 'shader':
			return 'shaders'
		case 'datapack':
			return 'datapacks'
		default:
			return 'mods'
	}
})

let searchTimer: ReturnType<typeof setTimeout> | null = null
watch(searchQuery, (val) => {
	if (searchTimer) clearTimeout(searchTimer)
	searchTimer = setTimeout(() => {
		debouncedQuery.value = val.trim()
		page.value = 1
	}, 260)
})

watch(contentKind, () => {
	selectedCategories.value = []
	page.value = 1
})

watch(source, () => {
	// CurseForge API filters by a single categoryId at a time
	if (source.value === 'curseforge' && selectedCategories.value.length > 1) {
		selectedCategories.value = [selectedCategories.value[0]]
	}
	page.value = 1
})

watch(
	[selectedSort, selectedCategories, selectedEnvironments, matchInstanceVersion, matchInstanceLoader, hideInstalled],
	() => {
		page.value = 1
	},
	{ deep: true },
)

function toggleCategory(catId: string) {
	if (source.value === 'curseforge') {
		selectedCategories.value = selectedCategories.value[0] === catId ? [] : [catId]
		return
	}
	if (selectedCategories.value.includes(catId)) {
		selectedCategories.value = selectedCategories.value.filter((c) => c !== catId)
	} else {
		selectedCategories.value = [...selectedCategories.value, catId]
	}
}

function toggleEnvironment(env: EnvironmentFilter) {
	if (selectedEnvironments.value.includes(env)) {
		selectedEnvironments.value = selectedEnvironments.value.filter((e) => e !== env)
	} else {
		selectedEnvironments.value = [...selectedEnvironments.value, env]
	}
}

function resetSidebarFilters() {
	selectedCategories.value = []
	selectedEnvironments.value = []
	matchInstanceVersion.value = true
	matchInstanceLoader.value = true
	hideInstalled.value = false
	searchQuery.value = ''
	page.value = 1
}

function formatDownloads(count: number): string {
	if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`
	if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K`
	return String(count)
}

function formatCategoryLabel(raw: string): string {
	return raw
		.split(/[-_\s]+/)
		.filter(Boolean)
		.map((part) => part.charAt(0).toUpperCase() + part.slice(1))
		.join(' ')
}

function modrinthName(hit: ModrinthV3Hit): string {
	return hit.name || hit.title || hit.slug || 'Untitled'
}

function modrinthSummary(hit: ModrinthV3Hit): string {
	return hit.summary || hit.description || ''
}

function modrinthBannerUrl(hit: ModrinthV3Hit): string | null {
	if (hit.featured_gallery) return hit.featured_gallery
	if (Array.isArray(hit.gallery) && hit.gallery.length > 0 && typeof hit.gallery[0] === 'string') {
		return hit.gallery[0]
	}
	return null
}

function modrinthTags(hit: ModrinthV3Hit): string[] {
	const raw = hit.display_categories?.length ? hit.display_categories : (hit.categories ?? [])
	const loaders = new Set(['fabric', 'forge', 'neoforge', 'quilt', 'liteloader', 'modloader', 'rift'])
	return raw.filter((cat) => !loaders.has(cat.toLowerCase())).slice(0, 2)
}

function escapeFilterValue(value: string): string {
	return `"${value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`
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

let activeRequestToken = 0

async function fetchCatalog() {
	const token = ++activeRequestToken
	loading.value = true
	errorMessage.value = null

	try {
		if (source.value === 'modrinth') {
			const filters: string[] = [`project_types = ${escapeFilterValue(contentKind.value)}`]
			if (matchInstanceVersion.value && props.instance.game_version) {
				filters.push(`game_versions = ${escapeFilterValue(props.instance.game_version)}`)
			}
			if (
				contentKind.value === 'mod' &&
				matchInstanceLoader.value &&
				props.instance.loader &&
				props.instance.loader !== 'vanilla'
			) {
				filters.push(`categories = ${escapeFilterValue(props.instance.loader)}`)
			}

			for (const catId of selectedCategories.value) {
				const found = availableCategories.value.find((c) => c.id === catId)
				if (found) {
					filters.push(`categories = ${escapeFilterValue(found.modrinthSlug)}`)
				}
			}

			if (selectedEnvironments.value.includes('client')) {
				filters.push('(client_side = "required" OR client_side = "optional")')
			}
			if (selectedEnvironments.value.includes('server')) {
				filters.push('(server_side = "required" OR server_side = "optional")')
			}

			if (hideInstalled.value && installedModrinthIds.value.size > 0) {
				const quotedIds = [...installedModrinthIds.value].map(escapeFilterValue).join(', ')
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
			modrinthHits.value = (raw?.result?.hits ?? []) as unknown as ModrinthV3Hit[]
			modrinthTotal.value = Number(raw?.result?.total_hits ?? 0)
		} else {
			const selectedCat =
				selectedCategories.value.length > 0
					? availableCategories.value.find((c) => c.id === selectedCategories.value[0])
					: undefined

			const res = await searchCurseForgeModpacks({
				q: debouncedQuery.value || undefined,
				gameVersion:
					matchInstanceVersion.value && props.instance.game_version
						? props.instance.game_version
						: undefined,
				loader:
					contentKind.value === 'mod' &&
					matchInstanceLoader.value &&
					props.instance.loader !== 'vanilla'
						? props.instance.loader
						: undefined,
				categoryId: selectedCat?.curseforgeId,
				contentType: contentKind.value,
				sort: mapCurseForgeSort(selectedSort.value),
				index: (page.value - 1) * pageSize,
				pageSize,
			})
			if (token !== activeRequestToken) return
			curseforgeConfigured.value = res.configured
			const rawItems = res.items ?? []
			curseforgeItems.value = hideInstalled.value
				? rawItems.filter((item) => !installedCurseForgeIds.value.has(item.id))
				: rawItems
			curseforgeTotal.value = res.pagination?.totalCount ?? 0
		}
	} catch (err) {
		if (token !== activeRequestToken) return
		errorMessage.value = err instanceof Error ? err.message : 'Failed to load catalog.'
	} finally {
		if (token === activeRequestToken) {
			loading.value = false
		}
	}
}

watch(
	[
		source,
		contentKind,
		debouncedQuery,
		selectedSort,
		page,
		selectedCategories,
		selectedEnvironments,
		matchInstanceVersion,
		matchInstanceLoader,
		hideInstalled,
	],
	() => {
		void fetchCatalog()
	},
	{ deep: true },
)

onMounted(() => {
	void loadInstalledModrinthIds()
	void fetchCatalog()
})

onBeforeUnmount(() => {
	if (searchTimer) clearTimeout(searchTimer)
})

const totalPages = computed(() => {
	const total = source.value === 'modrinth' ? modrinthTotal.value : curseforgeTotal.value
	return Math.max(1, Math.min(50, Math.ceil(total / pageSize)))
})

function switchPage(nextPage: number) {
	page.value = Math.max(1, Math.min(totalPages.value, nextPage))
	window.scrollTo({ top: 0, behavior: 'smooth' })
}

async function handleInstallModrinth(hit: ModrinthV3Hit) {
	const projectId = hit.project_id
	const displayName = modrinthName(hit)
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
			throw new Error(`No compatible version found for ${displayName}.`)
		}

		await installModrinthVersion(projectId, picked.id, props.instance.id, 'SolaraInstanceBrowser')

		const installedNext = new Set(installedModrinthIds.value)
		installedNext.add(projectId)
		installedModrinthIds.value = installedNext

		await queryClient.invalidateQueries({ queryKey: instanceKeys.content(props.instance.id) })
		addNotification({
			type: 'success',
			title: `Installed ${displayName}`,
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
	<div class="mx-auto flex w-full max-w-[1380px] flex-col gap-5 px-6 py-6">
		<!-- Pure-OLED Open Header + Dual Provider Switcher -->
		<header class="flex flex-wrap items-center justify-between gap-4">
			<div class="flex min-w-0 items-center gap-3.5">
				<button
					type="button"
					class="inline-flex h-10 cursor-pointer items-center gap-1.5 rounded-xl border-0 bg-bg-raised px-3.5 text-xs font-bold text-contrast transition-colors hover:bg-surface-4"
					@click="goBackToInstance"
				>
					<ArrowLeftIcon class="size-4" />
					<span>Back</span>
				</button>

				<Avatar
					:src="getInstanceIconUrl(instance.icon_path)"
					:alt="instance.name"
					size="44px"
					:tint-by="instance.id"
					class="rounded-xl"
				/>

				<div class="flex min-w-0 flex-col gap-0.5">
					<span class="text-[11px] font-bold uppercase tracking-wider text-secondary">
						Installing into
					</span>
					<div class="flex flex-wrap items-center gap-2">
						<h1 class="m-0 truncate text-lg font-extrabold tracking-tight text-contrast">
							{{ instance.name }}
						</h1>
						<span class="solara-brand-pill rounded-full px-2.5 py-0.5 text-[11px] font-bold">
							{{ formatLoaderLabel(instance.loader) }} · MC {{ instance.game_version }}
						</span>
					</div>
				</div>
			</div>

			<div class="flex flex-wrap items-center gap-2">
				<!-- Pure-OLED Modrinth / CurseForge Switcher -->
				<div
					class="inline-flex items-center rounded-xl bg-bg-raised p-1"
					role="tablist"
					aria-label="Content source"
				>
					<button
						type="button"
						role="tab"
						:aria-selected="source === 'modrinth'"
						class="flex cursor-pointer items-center gap-2 rounded-lg border-none px-4 py-1.5 text-sm font-semibold transition-all"
						:class="
							source === 'modrinth'
								? 'solara-source-modrinth-active text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="source = 'modrinth'"
					>
						<span
							class="size-2 rounded-full"
							:style="{
								backgroundColor: source === 'modrinth' ? '#1bd96a' : 'currentColor',
								opacity: source === 'modrinth' ? 1 : 0.45,
							}"
						/>
						<span>Modrinth</span>
					</button>
					<button
						type="button"
						role="tab"
						:aria-selected="source === 'curseforge'"
						class="flex cursor-pointer items-center gap-2 rounded-lg border-none px-4 py-1.5 text-sm font-semibold transition-all"
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

				<!-- List / Grid Toggle -->
				<div
					class="inline-flex items-center rounded-xl bg-bg-raised p-1"
					role="group"
					aria-label="Layout view"
				>
					<button
						type="button"
						title="Long cards view"
						aria-label="Long cards view"
						class="flex size-8 cursor-pointer items-center justify-center rounded-lg border-none p-0 transition-colors"
						:class="
							viewMode === 'list'
								? 'bg-surface-4 text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="viewMode = 'list'"
					>
						<ListIcon class="size-4" />
					</button>
					<button
						type="button"
						title="2-column card view"
						aria-label="2-column card view"
						class="flex size-8 cursor-pointer items-center justify-center rounded-lg border-none p-0 transition-colors"
						:class="
							viewMode === 'grid'
								? 'bg-surface-4 text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="viewMode = 'grid'"
					>
						<GridIcon class="size-4" />
					</button>
				</div>
			</div>
		</header>

		<!-- Main 2-Column Layout: Left Filter Sidebar + Right Long-Card Catalog -->
		<div class="grid grid-cols-1 items-start gap-5 lg:grid-cols-[248px_minmax(0,1fr)]">
			<!-- LEFT FILTER SIDEBAR (Silky Borderless Pure-OLED) -->
			<aside
				aria-label="Catalog filters"
				class="flex flex-col gap-4 rounded-2xl bg-bg-raised p-4 lg:sticky lg:top-6"
			>
				<!-- Sidebar Title + Reset -->
				<div class="solara-subtle-divider flex items-center justify-between gap-2 pb-3">
					<div class="flex items-center gap-2">
						<FilterIcon class="size-4 text-brand" />
						<h2 class="m-0 text-sm font-extrabold text-contrast">Filters</h2>
						<span
							v-if="activeFilterCount > 0"
							class="solara-brand-pill rounded-full px-2 py-0.5 text-[10.5px] font-bold tabular-nums"
						>
							{{ activeFilterCount }}
						</span>
					</div>
					<button
						v-if="activeFilterCount > 0 || searchQuery"
						type="button"
						class="cursor-pointer border-none bg-transparent p-0 text-xs font-semibold text-brand hover:underline"
						@click="resetSidebarFilters"
					>
						Reset
					</button>
				</div>

				<!-- Profile Compatibility Section -->
				<div class="solara-subtle-divider flex flex-col gap-2 pb-3.5">
					<button
						type="button"
						class="flex w-full cursor-pointer items-center justify-between border-none bg-transparent p-0 text-left text-[11px] font-bold uppercase tracking-wider text-secondary hover:text-contrast"
						@click="openSections.compatibility = !openSections.compatibility"
					>
						<span>Compatibility</span>
						<ChevronDownIcon
							:class="[
								'size-4 transition-transform',
								openSections.compatibility ? 'rotate-180' : '',
							]"
						/>
					</button>

					<div v-if="openSections.compatibility" class="flex flex-col gap-1.5">
						<label
							class="flex cursor-pointer items-center justify-between gap-2 rounded-xl bg-surface-2 px-3 py-2 text-xs font-medium text-contrast transition-colors hover:bg-surface-4"
						>
							<span>MC {{ instance.game_version }}</span>
							<Checkbox v-model="matchInstanceVersion" aria-label="Match Minecraft version" />
						</label>

						<label
							v-if="contentKind === 'mod' && !isVanilla"
							class="flex cursor-pointer items-center justify-between gap-2 rounded-xl bg-surface-2 px-3 py-2 text-xs font-medium text-contrast transition-colors hover:bg-surface-4"
						>
							<span>{{ formatLoaderLabel(instance.loader) }} Loader</span>
							<Checkbox v-model="matchInstanceLoader" aria-label="Match mod loader" />
						</label>

						<label
							class="flex cursor-pointer items-center justify-between gap-2 rounded-xl bg-surface-2 px-3 py-2 text-xs font-medium text-secondary transition-colors hover:bg-surface-4 hover:text-contrast"
						>
							<span>Hide installed</span>
							<Checkbox v-model="hideInstalled" aria-label="Hide installed items" />
						</label>
					</div>
				</div>

				<!-- Categories Section -->
				<div class="solara-subtle-divider flex flex-col gap-2 pb-3.5">
					<button
						type="button"
						class="flex w-full cursor-pointer items-center justify-between border-none bg-transparent p-0 text-left text-[11px] font-bold uppercase tracking-wider text-secondary hover:text-contrast"
						@click="openSections.categories = !openSections.categories"
					>
						<span>Categories</span>
						<ChevronDownIcon
							:class="[
								'size-4 transition-transform',
								openSections.categories ? 'rotate-180' : '',
							]"
						/>
					</button>

					<div v-if="openSections.categories" class="flex max-h-[300px] flex-col gap-1 overflow-y-auto pr-0.5">
						<button
							v-for="cat in availableCategories"
							:key="cat.id"
							type="button"
							:class="[
								'flex w-full cursor-pointer items-center justify-between gap-2 rounded-xl border-0 px-3 py-1.5 text-left text-xs font-semibold transition-all',
								selectedCategories.includes(cat.id)
									? 'solara-pill-active text-brand'
									: 'bg-transparent text-secondary hover:bg-surface-4 hover:text-contrast',
							]"
							@click="toggleCategory(cat.id)"
						>
							<span class="truncate">{{ cat.label }}</span>
							<CheckIcon v-if="selectedCategories.includes(cat.id)" class="size-3.5 shrink-0 text-brand" />
						</button>
					</div>
				</div>

				<!-- Environment Section (Modrinth Client/Server) -->
				<div v-if="source === 'modrinth'" class="flex flex-col gap-2">
					<button
						type="button"
						class="flex w-full cursor-pointer items-center justify-between border-none bg-transparent p-0 text-left text-[11px] font-bold uppercase tracking-wider text-secondary hover:text-contrast"
						@click="openSections.environment = !openSections.environment"
					>
						<span>Environment</span>
						<ChevronDownIcon
							:class="[
								'size-4 transition-transform',
								openSections.environment ? 'rotate-180' : '',
							]"
						/>
					</button>

					<div v-if="openSections.environment" class="flex flex-col gap-1.5">
						<button
							type="button"
							:class="[
								'flex w-full cursor-pointer items-center justify-between gap-2 rounded-xl border-0 px-3 py-1.5 text-left text-xs font-semibold transition-all',
								selectedEnvironments.includes('client')
									? 'solara-pill-active text-brand'
									: 'bg-surface-2 text-secondary hover:bg-surface-4 hover:text-contrast',
							]"
							@click="toggleEnvironment('client')"
						>
							<span>Client</span>
							<CheckIcon v-if="selectedEnvironments.includes('client')" class="size-3.5 text-brand" />
						</button>
						<button
							type="button"
							:class="[
								'flex w-full cursor-pointer items-center justify-between gap-2 rounded-xl border-0 px-3 py-1.5 text-left text-xs font-semibold transition-all',
								selectedEnvironments.includes('server')
									? 'solara-pill-active text-brand'
									: 'bg-surface-2 text-secondary hover:bg-surface-4 hover:text-contrast',
							]"
							@click="toggleEnvironment('server')"
						>
							<span>Server</span>
							<CheckIcon v-if="selectedEnvironments.includes('server')" class="size-3.5 text-brand" />
						</button>
					</div>
				</div>

				<!-- CurseForge Provider Hint -->
				<div
					v-else
					class="rounded-xl bg-surface-2 px-3 py-2.5 text-[11px] leading-relaxed text-secondary"
				>
					Browsing live <strong class="text-contrast">CurseForge</strong> catalog with 1-click
					profile install.
				</div>
			</aside>

			<!-- RIGHT CONTENT COLUMN: Open Pure-OLED Search + Modrinth-Style Long Cards -->
			<div class="flex min-w-0 flex-col gap-4">
				<!-- Open Search, Content Type Pills & Sort Bar -->
				<section class="flex flex-col gap-2.5">
					<div class="flex flex-wrap items-center gap-2.5">
						<div
							class="solara-oled-search flex h-10 min-w-[220px] flex-1 items-center gap-2.5 rounded-xl bg-bg-raised px-3.5 transition-all"
						>
							<SearchIcon class="size-4 shrink-0 text-secondary" aria-hidden="true" />
							<input
								v-model="searchQuery"
								type="text"
								:placeholder="`Search ${source === 'modrinth' ? 'Modrinth' : 'CurseForge'} ${contentKindPluralLabel}...`"
								class="w-full border-none bg-transparent text-sm text-contrast outline-none placeholder:text-secondary"
							/>
							<button
								v-if="searchQuery"
								type="button"
								aria-label="Clear search"
								class="flex size-5 cursor-pointer items-center justify-center rounded-full border-none bg-transparent p-0 text-secondary hover:text-contrast"
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

					<div class="flex flex-wrap items-center justify-between gap-2.5">
						<div class="flex flex-wrap items-center gap-1.5" role="group" aria-label="Content type">
							<button
								v-for="tab in CONTENT_TABS"
								:key="tab.id"
								type="button"
								:disabled="tab.disabled"
								:title="
									tab.disabled
										? 'Switch profile to Fabric, Forge, NeoForge, or Quilt to install mods'
										: undefined
								"
								class="cursor-pointer rounded-full border-0 px-3.5 py-1.5 text-xs font-semibold transition-all disabled:cursor-not-allowed disabled:opacity-40"
								:class="
									contentKind === tab.id
										? 'solara-pill-active text-brand'
										: 'bg-bg-raised text-secondary hover:bg-surface-4 hover:text-contrast'
								"
								@click="contentKind = tab.id"
							>
								{{ tab.label }}
							</button>
						</div>

						<span v-if="!loading && !errorMessage" class="text-xs font-medium text-secondary tabular-nums">
							{{ (source === 'modrinth' ? modrinthTotal : curseforgeTotal).toLocaleString() }}
							{{ contentKindPluralLabel }}
						</span>
					</div>
				</section>

				<!-- Error State -->
				<div
					v-if="errorMessage"
					class="flex flex-col items-center justify-center gap-3 rounded-2xl bg-bg-raised p-10 text-center"
				>
					<p class="m-0 text-sm font-semibold text-contrast">{{ errorMessage }}</p>
					<Button size="sm" class="!rounded-xl" @click="fetchCatalog">Retry</Button>
				</div>

				<!-- Loading Skeletons -->
				<div
					v-else-if="loading"
					:class="
						viewMode === 'grid'
							? 'grid grid-cols-1 gap-3.5 md:grid-cols-2'
							: 'flex flex-col gap-2.5'
					"
					aria-busy="true"
				>
					<div
						v-for="n in 8"
						:key="n"
						class="overflow-hidden rounded-2xl bg-bg-raised"
					>
						<div v-if="viewMode === 'grid'" class="h-24 w-full animate-pulse bg-surface-4" />
						<div class="flex items-center gap-4 p-4">
							<div v-if="viewMode === 'list'" class="size-16 shrink-0 animate-pulse rounded-2xl bg-surface-4" />
							<div class="flex flex-1 flex-col gap-2">
								<div class="h-4 w-48 animate-pulse rounded-lg bg-surface-4" />
								<div class="h-3 w-full animate-pulse rounded-lg bg-surface-4" />
								<div class="h-3 w-2/3 animate-pulse rounded-lg bg-surface-4" />
							</div>
						</div>
					</div>
				</div>

				<!-- MODRINTH RESULTS -->
				<template v-else-if="source === 'modrinth'">
					<div
						v-if="modrinthHits.length === 0"
						class="flex flex-col items-center justify-center gap-3 rounded-2xl bg-bg-raised p-12 text-center"
					>
						<div class="flex size-12 items-center justify-center rounded-2xl bg-surface-4 text-secondary">
							<PackageIcon class="size-6" />
						</div>
						<p class="m-0 text-sm font-semibold text-contrast">
							No matching Modrinth {{ contentKindPluralLabel }} found for your filters.
						</p>
						<Button
							v-if="activeFilterCount > 0 || searchQuery"
							size="sm"
							type="base"
							class="!rounded-xl"
							@click="resetSidebarFilters"
						>
							Reset filters
						</Button>
					</div>

					<!-- Modrinth Long Horizontal Cards (DEFAULT — Modrinth Pure-OLED Style) -->
					<div v-else-if="viewMode === 'list'" class="flex flex-col gap-2.5">
						<article
							v-for="(hit, idx) in modrinthHits"
							:key="hit.project_id"
							:style="{ animationDelay: `${Math.min(idx, 12) * 18}ms` }"
							class="solara-mp-card solara-card-animate group flex items-center gap-4 rounded-2xl bg-bg-raised px-4 py-3.5"
						>
							<button
								type="button"
								class="flex size-[68px] shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-2xl border-0 bg-surface-2 p-0 transition-transform duration-200 group-hover:scale-[1.03]"
								@click="router.push({ path: `/project/${hit.project_id}`, query: { i: instance.id } })"
							>
								<img
									v-if="hit.icon_url"
									:src="hit.icon_url"
									:alt="modrinthName(hit)"
									loading="lazy"
									class="size-full object-cover"
								/>
								<PackageIcon v-else class="size-7 text-secondary" />
							</button>

							<div class="flex min-w-0 flex-1 flex-col gap-1.5">
								<div class="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
									<button
										type="button"
										class="m-0 max-w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-[15.5px] font-bold leading-snug text-contrast transition-colors hover:text-brand"
										@click="router.push({ path: `/project/${hit.project_id}`, query: { i: instance.id } })"
									>
										{{ modrinthName(hit) }}
									</button>
									<span class="text-xs font-medium text-secondary">
										by {{ hit.author || 'Community' }}
									</span>
								</div>

								<p class="m-0 line-clamp-2 text-[12.5px] leading-relaxed text-secondary">
									{{ modrinthSummary(hit) }}
								</p>

								<div class="mt-0.5 flex flex-wrap items-center gap-1.5">
									<span class="inline-flex items-center gap-1 rounded-md bg-[#1bd96a]/12 px-2 py-0.5 text-[10.5px] font-semibold text-[#1bd96a]">
										<span class="size-1.5 rounded-full bg-[#1bd96a]" />
										Modrinth
									</span>
									<span
										v-for="tag in modrinthTags(hit)"
										:key="tag"
										class="rounded-md bg-surface-4 px-2 py-0.5 text-[10.5px] font-medium text-secondary"
									>
										{{ formatCategoryLabel(tag) }}
									</span>
								</div>
							</div>

							<div class="flex shrink-0 flex-col items-end justify-between gap-2.5 self-stretch pl-2">
								<div class="flex flex-col items-end gap-0.5 text-xs text-secondary tabular-nums">
									<span class="flex items-center gap-1 font-bold text-contrast">
										<DownloadIcon class="size-3.5 text-secondary" />
										{{ formatDownloads(hit.downloads) }}
									</span>
									<span v-if="hit.date_modified" class="flex items-center gap-1 text-[11px]">
										<UpdatedIcon class="size-3" />
										{{ dayjs(hit.date_modified).fromNow() }}
									</span>
								</div>

								<div class="flex items-center gap-1.5">
									<Button
										:type="installedModrinthIds.has(hit.project_id) ? 'base' : 'colored'"
										:color="installedModrinthIds.has(hit.project_id) ? 'default' : 'brand'"
										size="sm"
										class="!rounded-xl !px-3.5 !font-bold"
										:disabled="
											installingModrinthIds.has(hit.project_id) ||
											installedModrinthIds.has(hit.project_id)
										"
										@click="handleInstallModrinth(hit)"
									>
										<SpinnerIcon
											v-if="installingModrinthIds.has(hit.project_id)"
											class="size-3.5 animate-spin"
										/>
										<CheckIcon v-else-if="installedModrinthIds.has(hit.project_id)" class="size-3.5" />
										<DownloadIcon v-else class="size-3.5" />
										<span>
											{{
												installingModrinthIds.has(hit.project_id)
													? 'Installing'
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

					<!-- Modrinth 2-Column Spacious Cards (zero button overflow!) -->
					<div v-else class="grid grid-cols-1 gap-3.5 md:grid-cols-2">
						<article
							v-for="(hit, idx) in modrinthHits"
							:key="hit.project_id"
							:style="{ animationDelay: `${Math.min(idx, 11) * 20}ms` }"
							class="solara-mp-card solara-card-animate group flex flex-col overflow-hidden rounded-2xl bg-bg-raised"
						>
							<div
								class="relative h-24 w-full cursor-pointer overflow-hidden bg-surface-2"
								@click="router.push({ path: `/project/${hit.project_id}`, query: { i: instance.id } })"
							>
								<img
									v-if="modrinthBannerUrl(hit)"
									:src="modrinthBannerUrl(hit)!"
									:alt="modrinthName(hit)"
									loading="lazy"
									class="size-full object-cover transition-transform duration-300 group-hover:scale-[1.04]"
								/>
								<img
									v-else-if="hit.icon_url"
									:src="hit.icon_url"
									alt=""
									loading="lazy"
									class="size-full scale-125 object-cover opacity-30 blur-xl"
								/>
								<div class="solara-card-scrim pointer-events-none absolute inset-0" />
								<div class="absolute left-2.5 top-2.5">
									<span class="solara-cover-pill inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-[11px] font-semibold">
										<span class="size-1.5 rounded-full bg-[#1bd96a]" />
										Modrinth
									</span>
								</div>
							</div>

							<div class="relative flex min-w-0 flex-1 flex-col gap-2.5 px-4 pb-4 pt-3">
								<div class="flex min-w-0 items-end gap-3">
									<button
										type="button"
										class="-mt-9 flex size-13 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-2xl border-2 border-solid border-surface-3 bg-surface-2 p-0 shadow-md"
										@click="router.push({ path: `/project/${hit.project_id}`, query: { i: instance.id } })"
									>
										<img
											v-if="hit.icon_url"
											:src="hit.icon_url"
											:alt="modrinthName(hit)"
											loading="lazy"
											class="size-full object-cover"
										/>
										<PackageIcon v-else class="size-6 text-secondary" />
									</button>

									<div class="min-w-0 flex-1">
										<button
											type="button"
											class="m-0 block w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-[15px] font-bold leading-snug text-contrast transition-colors hover:text-brand"
											@click="router.push({ path: `/project/${hit.project_id}`, query: { i: instance.id } })"
										>
											{{ modrinthName(hit) }}
										</button>
										<p class="m-0 truncate text-xs text-secondary">
											by {{ hit.author || 'Community' }}
										</p>
									</div>
								</div>

								<p class="m-0 line-clamp-2 min-h-[2.3rem] text-xs leading-relaxed text-secondary">
									{{ modrinthSummary(hit) }}
								</p>

								<div class="solara-subtle-top mt-auto flex flex-wrap items-center justify-between gap-2 pt-3">
									<div class="flex min-w-0 items-center gap-2.5 text-xs font-medium text-secondary tabular-nums">
										<span class="flex items-center gap-1 font-semibold text-contrast">
											<DownloadIcon class="size-3.5 text-secondary" />
											{{ formatDownloads(hit.downloads) }}
										</span>
										<span v-if="hit.date_modified" class="truncate text-[11px]">
											{{ dayjs(hit.date_modified).fromNow() }}
										</span>
									</div>

									<Button
										:type="installedModrinthIds.has(hit.project_id) ? 'base' : 'colored'"
										:color="installedModrinthIds.has(hit.project_id) ? 'default' : 'brand'"
										size="sm"
										class="shrink-0 !rounded-xl !px-3.5 !font-bold"
										:disabled="
											installingModrinthIds.has(hit.project_id) ||
											installedModrinthIds.has(hit.project_id)
										"
										@click="handleInstallModrinth(hit)"
									>
										<SpinnerIcon
											v-if="installingModrinthIds.has(hit.project_id)"
											class="size-3.5 animate-spin"
										/>
										<CheckIcon v-else-if="installedModrinthIds.has(hit.project_id)" class="size-3.5" />
										<DownloadIcon v-else class="size-3.5" />
										<span>
											{{
												installingModrinthIds.has(hit.project_id)
													? 'Installing'
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
						class="flex flex-col items-center justify-center gap-3 rounded-2xl bg-bg-raised p-12 text-center"
					>
						<div class="flex size-12 items-center justify-center rounded-2xl bg-surface-4 text-secondary">
							<PackageIcon class="size-6" />
						</div>
						<p class="m-0 text-sm font-semibold text-contrast">
							{{
								!curseforgeConfigured
									? 'CurseForge catalog is temporarily unavailable.'
									: `No matching CurseForge ${contentKindPluralLabel} found for your filters.`
							}}
						</p>
						<Button
							v-if="activeFilterCount > 0 || searchQuery"
							size="sm"
							type="base"
							class="!rounded-xl"
							@click="resetSidebarFilters"
						>
							Reset filters
						</Button>
					</div>

					<!-- CurseForge Long Horizontal Cards (DEFAULT — Modrinth Pure-OLED Style) -->
					<div v-else-if="viewMode === 'list'" class="flex flex-col gap-2.5">
						<article
							v-for="(item, idx) in curseforgeItems"
							:key="item.id"
							:style="{ animationDelay: `${Math.min(idx, 12) * 18}ms` }"
							class="solara-mp-card solara-card-animate group flex items-center gap-4 rounded-2xl bg-bg-raised px-4 py-3.5"
						>
							<button
								type="button"
								class="flex size-[68px] shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-2xl border-0 bg-surface-2 p-0 transition-transform duration-200 group-hover:scale-[1.03]"
								@click="item.websiteUrl && openUrl(item.websiteUrl)"
							>
								<img
									v-if="item.logoUrl"
									:src="item.logoUrl"
									:alt="item.name"
									loading="lazy"
									class="size-full object-cover"
								/>
								<PackageIcon v-else class="size-7 text-secondary" />
							</button>

							<div class="flex min-w-0 flex-1 flex-col gap-1.5">
								<div class="flex flex-wrap items-baseline gap-x-2 gap-y-0.5">
									<button
										type="button"
										class="m-0 max-w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-[15.5px] font-bold leading-snug text-contrast transition-colors hover:text-brand"
										@click="item.websiteUrl && openUrl(item.websiteUrl)"
									>
										{{ item.name }}
									</button>
									<span class="text-xs font-medium text-secondary">by {{ item.author }}</span>
								</div>

								<p class="m-0 line-clamp-2 text-[12.5px] leading-relaxed text-secondary">
									{{ item.summary }}
								</p>

								<div class="mt-0.5 flex flex-wrap items-center gap-1.5">
									<span class="inline-flex items-center gap-1 rounded-md bg-[#f16436]/14 px-2 py-0.5 text-[10.5px] font-semibold text-[#f16436]">
										<span class="size-1.5 rounded-full bg-[#f16436]" />
										CurseForge
									</span>
									<span
										v-for="cat in (item.categories ?? []).slice(0, 2)"
										:key="cat.id"
										class="rounded-md bg-surface-4 px-2 py-0.5 text-[10.5px] font-medium text-secondary"
									>
										{{ cat.name }}
									</span>
								</div>
							</div>

							<div class="flex shrink-0 flex-col items-end justify-between gap-2.5 self-stretch pl-2">
								<div class="flex flex-col items-end gap-0.5 text-xs text-secondary tabular-nums">
									<span class="flex items-center gap-1 font-bold text-contrast">
										<DownloadIcon class="size-3.5 text-secondary" />
										{{ formatDownloads(item.downloads) }}
									</span>
									<span v-if="item.dateModified" class="flex items-center gap-1 text-[11px]">
										<UpdatedIcon class="size-3" />
										{{ dayjs(item.dateModified).fromNow() }}
									</span>
								</div>

								<div class="flex items-center gap-1.5">
									<button
										v-if="item.websiteUrl"
										type="button"
										title="Open on CurseForge"
										class="inline-flex size-8 cursor-pointer items-center justify-center rounded-xl border-0 bg-surface-4 text-secondary transition-colors hover:text-contrast"
										@click="openUrl(item.websiteUrl)"
									>
										<ExternalIcon class="size-3.5" />
									</button>
									<Button
										:type="installedCurseForgeIds.has(item.id) ? 'base' : 'colored'"
										:color="installedCurseForgeIds.has(item.id) ? 'default' : 'brand'"
										size="sm"
										class="!rounded-xl !px-3.5 !font-bold"
										:disabled="
											installingCurseForgeIds.has(item.id) || installedCurseForgeIds.has(item.id)
										"
										@click="handleInstallCurseForge(item)"
									>
										<SpinnerIcon
											v-if="installingCurseForgeIds.has(item.id)"
											class="size-3.5 animate-spin"
										/>
										<CheckIcon v-else-if="installedCurseForgeIds.has(item.id)" class="size-3.5" />
										<DownloadIcon v-else class="size-3.5" />
										<span>
											{{
												installingCurseForgeIds.has(item.id)
													? 'Installing'
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

					<!-- CurseForge 2-Column Spacious Cards (zero button overflow!) -->
					<div v-else class="grid grid-cols-1 gap-3.5 md:grid-cols-2">
						<article
							v-for="(item, idx) in curseforgeItems"
							:key="item.id"
							:style="{ animationDelay: `${Math.min(idx, 11) * 20}ms` }"
							class="solara-mp-card solara-card-animate group flex flex-col overflow-hidden rounded-2xl bg-bg-raised"
						>
							<div
								class="relative h-24 w-full cursor-pointer overflow-hidden bg-surface-2"
								@click="item.websiteUrl && openUrl(item.websiteUrl)"
							>
								<img
									v-if="item.bannerUrl"
									:src="item.bannerUrl"
									:alt="item.name"
									loading="lazy"
									class="size-full object-cover transition-transform duration-300 group-hover:scale-[1.04]"
								/>
								<img
									v-else-if="item.logoUrl"
									:src="item.logoUrl"
									alt=""
									loading="lazy"
									class="size-full scale-125 object-cover opacity-30 blur-xl"
								/>
								<div class="solara-card-scrim pointer-events-none absolute inset-0" />
								<div class="absolute left-2.5 top-2.5">
									<span class="solara-cover-pill inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-[11px] font-semibold">
										<span class="size-1.5 rounded-full bg-[#f16436]" />
										CurseForge
									</span>
								</div>
							</div>

							<div class="relative flex min-w-0 flex-1 flex-col gap-2.5 px-4 pb-4 pt-3">
								<div class="flex min-w-0 items-end gap-3">
									<button
										type="button"
										class="-mt-9 flex size-13 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-2xl border-2 border-solid border-surface-3 bg-surface-2 p-0 shadow-md"
										@click="item.websiteUrl && openUrl(item.websiteUrl)"
									>
										<img
											v-if="item.logoUrl"
											:src="item.logoUrl"
											:alt="item.name"
											loading="lazy"
											class="size-full object-cover"
										/>
										<PackageIcon v-else class="size-6 text-secondary" />
									</button>

									<div class="min-w-0 flex-1">
										<button
											type="button"
											class="m-0 block w-full cursor-pointer truncate border-none bg-transparent p-0 text-left text-[15px] font-bold leading-snug text-contrast transition-colors hover:text-brand"
											@click="item.websiteUrl && openUrl(item.websiteUrl)"
										>
											{{ item.name }}
										</button>
										<p class="m-0 truncate text-xs text-secondary">by {{ item.author }}</p>
									</div>
								</div>

								<p class="m-0 line-clamp-2 min-h-[2.3rem] text-xs leading-relaxed text-secondary">
									{{ item.summary }}
								</p>

								<div class="solara-subtle-top mt-auto flex flex-wrap items-center justify-between gap-2 pt-3">
									<div class="flex min-w-0 items-center gap-2.5 text-xs font-medium text-secondary tabular-nums">
										<span class="flex items-center gap-1 font-semibold text-contrast">
											<DownloadIcon class="size-3.5 text-secondary" />
											{{ formatDownloads(item.downloads) }}
										</span>
										<span v-if="item.dateModified" class="truncate text-[11px]">
											{{ dayjs(item.dateModified).fromNow() }}
										</span>
									</div>

									<div class="flex shrink-0 items-center gap-1.5">
										<button
											v-if="item.websiteUrl"
											type="button"
											title="Open on CurseForge"
											class="inline-flex size-8 cursor-pointer items-center justify-center rounded-xl border-0 bg-surface-4 text-secondary transition-colors hover:text-contrast"
											@click="openUrl(item.websiteUrl)"
										>
											<ExternalIcon class="size-3.5" />
										</button>
										<Button
											:type="installedCurseForgeIds.has(item.id) ? 'base' : 'colored'"
											:color="installedCurseForgeIds.has(item.id) ? 'default' : 'brand'"
											size="sm"
											class="!rounded-xl !px-3.5 !font-bold"
											:disabled="
												installingCurseForgeIds.has(item.id) ||
												installedCurseForgeIds.has(item.id)
											"
											@click="handleInstallCurseForge(item)"
										>
											<SpinnerIcon
												v-if="installingCurseForgeIds.has(item.id)"
												class="size-3.5 animate-spin"
											/>
											<CheckIcon v-else-if="installedCurseForgeIds.has(item.id)" class="size-3.5" />
											<DownloadIcon v-else class="size-3.5" />
											<span>
												{{
													installingCurseForgeIds.has(item.id)
														? 'Installing'
														: installedCurseForgeIds.has(item.id)
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
				</template>

				<!-- Pagination -->
				<footer
					v-if="!loading && !errorMessage && totalPages > 1"
					class="flex items-center justify-center pt-2"
				>
					<Pagination :page="page" :count="totalPages" @switch-page="switchPage" />
				</footer>
			</div>
		</div>
	</div>
</template>

<style scoped>
.solara-brand-pill {
	background-color: color-mix(in srgb, var(--color-brand) 15%, transparent);
	color: var(--color-brand);
}

.solara-oled-search {
	border: 1px solid transparent;
}

.solara-oled-search:focus-within {
	border-color: color-mix(in srgb, var(--color-brand) 55%, transparent);
}

.solara-subtle-divider {
	border-bottom: 1px solid color-mix(in srgb, var(--surface-5) 35%, transparent);
}

.solara-subtle-top {
	border-top: 1px solid color-mix(in srgb, var(--surface-5) 35%, transparent);
}

.solara-source-modrinth-active {
	background-color: color-mix(in srgb, #1bd96a 16%, var(--surface-4));
}

.solara-source-cf-active {
	background-color: color-mix(in srgb, #f16436 16%, var(--surface-4));
}

.solara-pill-active {
	background-color: color-mix(in srgb, var(--color-brand) 18%, var(--surface-3));
}

.solara-mp-card {
	border: 1px solid transparent;
	transition:
		background-color 180ms ease,
		transform 220ms cubic-bezier(0.22, 1, 0.36, 1);
}

.solara-mp-card:hover {
	background-color: color-mix(in srgb, var(--surface-4) 68%, var(--surface-3));
	transform: translateY(-1px);
}

.solara-card-scrim {
	background: linear-gradient(
		to bottom,
		rgba(0, 0, 0, 0.08) 0%,
		rgba(0, 0, 0, 0.24) 60%,
		color-mix(in srgb, var(--surface-3) 96%, transparent) 100%
	);
}

.solara-cover-pill {
	background-color: rgba(10, 14, 22, 0.78);
	color: #f8fafc;
	backdrop-filter: blur(6px);
}
</style>
