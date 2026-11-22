<script setup lang="ts">
import {
	CalendarIcon,
	ChevronLeftIcon,
	ChevronRightIcon,
	DownloadIcon,
	ExternalIcon,
	LeftArrowIcon,
	PackageIcon,
	SpinnerIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import { Button, injectNotificationManager } from '@modrinth/ui'
import { configuredXss } from '@modrinth/utils'
import { openUrl } from '@tauri-apps/plugin-opener'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import {
	type CurseForgeFile,
	type CurseForgeModpackDetail,
	getCurseForgeModpackDetail,
	installCurseForgeModpack,
} from './curseforge-api'

dayjs.extend(relativeTime)

type DetailTab = 'description' | 'versions' | 'gallery'

const route = useRoute()
const router = useRouter()
const { handleError, addNotification } = injectNotificationManager()

const loading = ref(true)
const error = ref<string | null>(null)
const detail = ref<CurseForgeModpackDetail | null>(null)
const activeTab = ref<DetailTab>('description')
const selectedVersionFilter = ref<string>('all')
const selectedLoaderFilter = ref<string>('all')
const installingFileId = ref<number | null>(null)
const lightboxIndex = ref<number | null>(null)

const modId = computed(() => {
	const raw = Number(route.params.id)
	return Number.isFinite(raw) && raw > 0 ? Math.floor(raw) : 0
})

const sanitizedDescriptionHtml = computed(() => {
	const raw = detail.value?.descriptionHtml?.trim()
	if (!raw) return ''
	const preCleaned = raw.replace(
		/<img\b[^>]*\bsrc=["'][^"']*youtube\.com\/watch[^"']*["'][^>]*>/gi,
		'',
	)
	return configuredXss.process(preCleaned)
})

const availableMcVersions = computed(() => {
	const set = new Set<string>()
	for (const f of detail.value?.files ?? []) {
		for (const gv of f.gameVersions) set.add(gv)
	}
	return [...set]
})

const availableLoaders = computed(() => {
	const set = new Set<string>()
	for (const f of detail.value?.files ?? []) {
		for (const l of f.loaders) set.add(l)
	}
	return [...set]
})

const filteredFiles = computed(() => {
	const files = detail.value?.files ?? []
	return files.filter((f) => {
		if (selectedVersionFilter.value !== 'all' && !f.gameVersions.includes(selectedVersionFilter.value)) {
			return false
		}
		if (selectedLoaderFilter.value !== 'all' && !f.loaders.includes(selectedLoaderFilter.value)) {
			return false
		}
		return true
	})
})

const primaryDownloadableFile = computed<CurseForgeFile | null>(() => {
	const files = detail.value?.files ?? []
	const firstAllowed = files.find((f) => !f.restricted && Boolean(f.downloadUrl))
	if (firstAllowed) return firstAllowed
	const latest = detail.value?.latestFile
	if (latest && !latest.restricted && latest.downloadUrl) return latest
	return null
})

const externalLinks = computed(() => {
	const d = detail.value
	if (!d) return []
	const links: { label: string; url: string }[] = []
	const cfUrl = d.links?.websiteUrl || d.websiteUrl
	if (cfUrl) links.push({ label: 'CurseForge Page', url: cfUrl })
	if (d.links?.wikiUrl) links.push({ label: 'Wiki', url: d.links.wikiUrl })
	if (d.links?.sourceUrl) links.push({ label: 'Source Code', url: d.links.sourceUrl })
	if (d.links?.issuesUrl) links.push({ label: 'Issue Tracker', url: d.links.issuesUrl })
	return links
})

async function loadDetail(id: number) {
	if (!id) {
		loading.value = false
		error.value = 'Invalid CurseForge modpack ID.'
		detail.value = null
		return
	}
	loading.value = true
	error.value = null
	detail.value = null
	selectedVersionFilter.value = 'all'
	selectedLoaderFilter.value = 'all'
	lightboxIndex.value = null

	const requestedTab = String(route.query.tab ?? '')
	if (requestedTab === 'versions' || requestedTab === 'gallery') {
		activeTab.value = requestedTab
	} else {
		activeTab.value = 'description'
	}

	try {
		const res = await getCurseForgeModpackDetail(id)
		detail.value = res.modpack
		if (activeTab.value === 'gallery' && (res.modpack.screenshots?.length ?? 0) === 0) {
			activeTab.value = 'description'
		}
	} catch (err) {
		error.value = err instanceof Error ? err.message : 'Unable to load CurseForge modpack details.'
	} finally {
		loading.value = false
	}
}

watch(modId, (id) => {
	void loadDetail(id)
}, { immediate: true })

function goBack() {
	if (window.history.length > 1) {
		router.back()
	} else {
		void router.push({ path: '/browse/modpack', query: { source: 'curseforge' } })
	}
}

function formatCount(n: number): string {
	if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
	if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
	return String(n)
}

function formatSize(bytes: number): string {
	if (!bytes) return ''
	if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
	return `${Math.max(1, Math.round(bytes / 1024))} KB`
}

function formatLoader(loader: string): string {
	const map: Record<string, string> = {
		fabric: 'Fabric',
		forge: 'Forge',
		neoforge: 'NeoForge',
		quilt: 'Quilt',
	}
	return map[loader.toLowerCase()] ?? loader
}

function openExternal(url: string) {
	if (!url || !/^https?:\/\//i.test(url)) return
	void openUrl(url)
}

function handleDescriptionClick(event: MouseEvent) {
	const target = event.target as HTMLElement | null
	const anchor = target?.closest('a')
	if (!anchor) return
	const href = anchor.getAttribute('href')
	if (!href) return
	event.preventDefault()

	let resolved = href
	if (href.includes('linkout?remoteUrl=')) {
		try {
			const parsed = new URL(href, 'https://www.curseforge.com')
			const remote = parsed.searchParams.get('remoteUrl')
			if (remote) {
				let decoded = decodeURIComponent(remote)
				if (decoded.startsWith('http%')) decoded = decodeURIComponent(decoded)
				resolved = decoded
			}
		} catch {
			// Fall back to original URL if parsing fails
		}
	}

	if (/^https?:\/\//i.test(resolved)) {
		void openUrl(resolved)
	}
}

async function handleInstall(file: CurseForgeFile | null) {
	const pack = detail.value
	if (!pack) return
	if (!file || file.restricted || !file.downloadUrl) {
		activeTab.value = 'versions'
		return
	}
	if (installingFileId.value !== null) return
	installingFileId.value = file.id

	try {
		const res = await installCurseForgeModpack({
			modId: pack.id,
			fileId: file.id,
			name: pack.name,
			downloadUrl: file.downloadUrl,
			iconUrl: pack.logoUrl,
			fileName: file.fileName,
		})

		if (res.restrictedMods.length > 0) {
			addNotification({
				title: `${res.name} installed with manual mods`,
				text: `${res.installedMods} mods installed. ${res.restrictedMods.length} mod(s) restrict third-party downloads on CurseForge and can be added manually to the instance mods folder.`,
				type: 'warning',
			})
		} else {
			addNotification({
				title: `${res.name} installed`,
				text: `Ready to play on Minecraft ${res.gameVersion} (${formatLoader(res.loader)}).`,
				type: 'success',
			})
		}
		void router.push(`/instance/${encodeURIComponent(res.instanceId)}`)
	} catch (err) {
		handleError(err as Error)
	} finally {
		installingFileId.value = null
	}
}

function stepLightbox(delta: number) {
	const total = detail.value?.screenshots.length ?? 0
	if (lightboxIndex.value === null || total === 0) return
	lightboxIndex.value = (lightboxIndex.value + delta + total) % total
}

function onKeyDown(event: KeyboardEvent) {
	if (lightboxIndex.value === null) return
	if (event.key === 'Escape') {
		lightboxIndex.value = null
	} else if (event.key === 'ArrowRight') {
		stepLightbox(1)
	} else if (event.key === 'ArrowLeft') {
		stepLightbox(-1)
	}
}

onMounted(() => {
	window.addEventListener('keydown', onKeyDown)
})

onBeforeUnmount(() => {
	window.removeEventListener('keydown', onKeyDown)
})
</script>

<template>
	<div class="mx-auto flex w-full max-w-[1320px] flex-col gap-5 px-6 py-6">
		<!-- Back navigation -->
		<div class="flex items-center justify-between gap-3">
			<button
				type="button"
				class="inline-flex cursor-pointer items-center gap-2 rounded-xl border border-solid border-surface-5 bg-surface-3 px-3.5 py-2 text-xs font-semibold text-secondary transition-colors hover:bg-surface-4 hover:text-contrast"
				@click="goBack"
			>
				<LeftArrowIcon class="size-4" />
				<span>Back to Modpacks</span>
			</button>

			<button
				v-if="detail && (detail.links?.websiteUrl || detail.websiteUrl)"
				type="button"
				class="inline-flex cursor-pointer items-center gap-1.5 rounded-xl border border-solid border-surface-5 bg-surface-3 px-3.5 py-2 text-xs font-semibold text-secondary transition-colors hover:bg-surface-4 hover:text-contrast"
				@click="openExternal(detail.links?.websiteUrl || detail.websiteUrl)"
			>
				<span>Open on CurseForge</span>
				<ExternalIcon class="size-3.5" />
			</button>
		</div>

		<!-- Loading -->
		<div
			v-if="loading"
			class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 py-20 text-sm text-secondary"
		>
			<SpinnerIcon class="size-6 animate-spin text-brand" />
			<span>Loading CurseForge modpack details…</span>
		</div>

		<!-- Error -->
		<div
			v-else-if="error"
			class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 py-16 text-center"
		>
			<p class="m-0 text-sm font-semibold text-contrast">{{ error }}</p>
			<div class="flex items-center gap-2">
				<Button size="sm" type="outlined" @click="goBack">Back</Button>
				<Button size="sm" @click="loadDetail(modId)">Retry</Button>
			</div>
		</div>

		<!-- Main Content -->
		<div
			v-else-if="detail"
			class="grid grid-cols-1 items-start gap-6 xl:grid-cols-[minmax(0,1fr)_19rem]"
		>
			<!-- Left column -->
			<div class="flex min-w-0 flex-col gap-5">
				<!-- Hero Banner + Header Card -->
				<section class="overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-3">
					<div
						v-if="detail.bannerUrl || detail.screenshots[0]"
						class="relative h-44 w-full overflow-hidden bg-surface-2"
					>
						<img
							:src="detail.screenshots[0]?.url || detail.bannerUrl!"
							:alt="detail.name"
							class="size-full object-cover"
						/>
						<div class="solara-cf-hero-scrim pointer-events-none absolute inset-0" />
					</div>

					<div class="flex flex-wrap items-start justify-between gap-4 p-5">
						<div class="flex min-w-0 flex-1 items-start gap-4">
							<div
								class="flex size-20 shrink-0 items-center justify-center overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-2 shadow-md"
								:class="{ '-mt-12 relative z-10': detail.bannerUrl || detail.screenshots[0] }"
							>
								<img
									v-if="detail.logoUrl"
									:src="detail.logoUrl"
									:alt="detail.name"
									class="size-full object-cover"
								/>
								<PackageIcon v-else class="size-8 text-secondary" />
							</div>

							<div class="min-w-0 flex-1">
								<div class="flex flex-wrap items-center gap-2">
									<h1 class="m-0 text-2xl font-bold tracking-tight text-contrast">
										{{ detail.name }}
									</h1>
									<span class="solara-cf-badge rounded-md px-2 py-0.5 text-[11px] font-semibold">
										CurseForge
									</span>
								</div>
								<p class="m-0 mt-1 text-xs font-medium text-secondary">
									by {{ detail.authors.length > 0 ? detail.authors.join(', ') : detail.author }}
								</p>
								<p class="m-0 mt-2 text-sm leading-relaxed text-primary">
									{{ detail.summary }}
								</p>
							</div>
						</div>

						<div class="flex shrink-0 items-center gap-2">
							<Button
								type="colored"
								color="brand"
								size="lg"
								:disabled="installingFileId !== null"
								@click="handleInstall(primaryDownloadableFile)"
							>
								<SpinnerIcon v-if="installingFileId !== null" class="size-4 animate-spin" />
								<DownloadIcon v-else class="size-4" />
								<span>
									{{
										installingFileId !== null
											? 'Installing…'
											: primaryDownloadableFile
												? 'Install'
												: 'View Versions'
									}}
								</span>
							</Button>
						</div>
					</div>
				</section>

				<!-- Tabs -->
				<div
					class="inline-flex w-fit items-center gap-1 rounded-xl border border-solid border-surface-5 bg-surface-3 p-1"
					role="tablist"
				>
					<button
						type="button"
						role="tab"
						:aria-selected="activeTab === 'description'"
						class="cursor-pointer rounded-lg border-none px-4 py-2 text-sm font-semibold transition-colors"
						:class="
							activeTab === 'description'
								? 'bg-surface-4 text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="activeTab = 'description'"
					>
						Description
					</button>
					<button
						type="button"
						role="tab"
						:aria-selected="activeTab === 'versions'"
						class="cursor-pointer rounded-lg border-none px-4 py-2 text-sm font-semibold transition-colors"
						:class="
							activeTab === 'versions'
								? 'bg-surface-4 text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="activeTab = 'versions'"
					>
						Versions ({{ detail.files.length }})
					</button>
					<button
						v-if="detail.screenshots.length > 0"
						type="button"
						role="tab"
						:aria-selected="activeTab === 'gallery'"
						class="cursor-pointer rounded-lg border-none px-4 py-2 text-sm font-semibold transition-colors"
						:class="
							activeTab === 'gallery'
								? 'bg-surface-4 text-contrast'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="activeTab = 'gallery'"
					>
						Gallery ({{ detail.screenshots.length }})
					</button>
				</div>

				<!-- Description Tab -->
				<section
					v-if="activeTab === 'description'"
					class="rounded-2xl border border-solid border-surface-5 bg-surface-3 p-6"
				>
					<div
						v-if="sanitizedDescriptionHtml"
						class="markdown-body solara-cf-description"
						@click="handleDescriptionClick"
						v-html="sanitizedDescriptionHtml"
					/>
					<div v-else class="text-sm leading-relaxed text-primary">
						<p class="m-0">{{ detail.summary || 'No detailed description provided by the author.' }}</p>
					</div>
				</section>

				<!-- Versions Tab -->
				<section
					v-else-if="activeTab === 'versions'"
					class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-5"
				>
					<div class="flex flex-wrap items-center justify-between gap-3">
						<h2 class="m-0 text-sm font-bold uppercase tracking-wider text-secondary">
							Available Files ({{ filteredFiles.length }})
						</h2>

						<div class="flex flex-wrap items-center gap-2">
							<div v-if="availableLoaders.length > 1" class="flex flex-wrap items-center gap-1">
								<button
									type="button"
									class="cursor-pointer rounded-md border border-solid px-2.5 py-1 text-xs font-semibold transition-colors"
									:class="
										selectedLoaderFilter === 'all'
											? 'solara-pill-active text-contrast'
											: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
									"
									@click="selectedLoaderFilter = 'all'"
								>
									All loaders
								</button>
								<button
									v-for="ldr in availableLoaders"
									:key="ldr"
									type="button"
									class="cursor-pointer rounded-md border border-solid px-2.5 py-1 text-xs font-semibold transition-colors"
									:class="
										selectedLoaderFilter === ldr
											? 'solara-pill-active text-contrast'
											: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
									"
									@click="selectedLoaderFilter = ldr"
								>
									{{ formatLoader(ldr) }}
								</button>
							</div>

							<div v-if="availableMcVersions.length > 1" class="flex flex-wrap items-center gap-1">
								<button
									type="button"
									class="cursor-pointer rounded-md border border-solid px-2.5 py-1 text-xs font-semibold transition-colors"
									:class="
										selectedVersionFilter === 'all'
											? 'solara-pill-active text-contrast'
											: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
									"
									@click="selectedVersionFilter = 'all'"
								>
									All MC
								</button>
								<button
									v-for="ver in availableMcVersions.slice(0, 8)"
									:key="ver"
									type="button"
									class="cursor-pointer rounded-md border border-solid px-2.5 py-1 text-xs font-semibold tabular-nums transition-colors"
									:class="
										selectedVersionFilter === ver
											? 'solara-pill-active text-contrast'
											: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
									"
									@click="selectedVersionFilter = ver"
								>
									{{ ver }}
								</button>
							</div>
						</div>
					</div>

					<div
						v-if="filteredFiles.length === 0"
						class="rounded-xl border border-solid border-surface-5 bg-surface-2 p-6 text-center text-sm text-secondary"
					>
						No modpack files matched your filter.
					</div>

					<div v-else class="flex flex-col gap-2.5">
						<div
							v-for="file in filteredFiles"
							:key="file.id"
							class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-solid border-surface-5 bg-surface-2 px-4 py-3.5"
						>
							<div class="min-w-0 flex-1">
								<div class="flex flex-wrap items-center gap-2">
									<span class="truncate text-sm font-semibold text-contrast">
										{{ file.displayName || file.fileName }}
									</span>
									<span
										class="rounded-md px-2 py-0.5 text-[11px] font-semibold capitalize"
										:class="
											file.releaseType === 'release'
												? 'bg-surface-4 text-contrast'
												: 'bg-surface-3 text-secondary'
										"
									>
										{{ file.releaseType }}
									</span>
								</div>
								<div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-secondary">
									<span v-if="file.gameVersions.length > 0">
										MC {{ file.gameVersions.slice(0, 4).join(', ') }}
									</span>
									<span v-if="file.loaders.length > 0">
										· {{ file.loaders.map(formatLoader).join(', ') }}
									</span>
									<span v-if="file.fileLength > 0">
										· {{ formatSize(file.fileLength) }}
									</span>
									<span v-if="file.fileDate">
										· {{ dayjs(file.fileDate).fromNow() }}
									</span>
								</div>
							</div>

							<Button
								type="colored"
								color="brand"
								size="sm"
								:disabled="file.restricted || !file.downloadUrl || installingFileId !== null"
								@click="handleInstall(file)"
							>
								<SpinnerIcon
									v-if="installingFileId === file.id"
									class="size-4 animate-spin"
								/>
								<DownloadIcon v-else class="size-4" />
								<span>
									{{
										file.restricted || !file.downloadUrl
											? 'Restricted'
											: installingFileId === file.id
												? 'Installing…'
												: 'Install'
									}}
								</span>
							</Button>
						</div>
					</div>
				</section>

				<!-- Gallery Tab -->
				<section
					v-else-if="activeTab === 'gallery'"
					class="grid grid-cols-1 gap-4 sm:grid-cols-2"
				>
					<button
						v-for="(shot, i) in detail.screenshots"
						:key="shot.url + i"
						type="button"
						class="group flex cursor-pointer flex-col overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-3 p-0 text-left transition-all hover:border-brand"
						@click="lightboxIndex = i"
					>
						<div class="aspect-video w-full overflow-hidden bg-surface-2">
							<img
								:src="shot.url || shot.thumbnailUrl"
								:alt="shot.title || detail.name"
								loading="lazy"
								class="size-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
							/>
						</div>
						<div v-if="shot.title" class="px-4 py-2.5 text-xs font-medium text-primary">
							{{ shot.title }}
						</div>
					</button>
				</section>
			</div>

			<!-- Right Sidebar -->
			<aside class="flex flex-col gap-4">
				<!-- Stats & Dates -->
				<div class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4">
					<h2 class="m-0 text-xs font-bold uppercase tracking-wider text-secondary">
						Details
					</h2>
					<div class="flex flex-col gap-2.5 text-sm">
						<div class="flex items-center justify-between gap-2">
							<span class="flex items-center gap-2 text-secondary">
								<DownloadIcon class="size-4" />
								Downloads
							</span>
							<strong class="tabular-nums text-contrast">
								{{ formatCount(detail.downloads) }}
							</strong>
						</div>
						<div v-if="detail.dateModified" class="flex items-center justify-between gap-2">
							<span class="flex items-center gap-2 text-secondary">
								<UpdatedIcon class="size-4" />
								Updated
							</span>
							<span class="text-contrast">{{ dayjs(detail.dateModified).fromNow() }}</span>
						</div>
						<div v-if="detail.dateCreated" class="flex items-center justify-between gap-2">
							<span class="flex items-center gap-2 text-secondary">
								<CalendarIcon class="size-4" />
								Created
							</span>
							<span class="text-contrast">{{ dayjs(detail.dateCreated).format('MMM D, YYYY') }}</span>
						</div>
					</div>
				</div>

				<!-- Loaders & MC Versions -->
				<div
					v-if="detail.loaders.length > 0 || detail.gameVersions.length > 0"
					class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
				>
					<div v-if="detail.loaders.length > 0" class="flex flex-col gap-2">
						<h3 class="m-0 text-xs font-bold uppercase tracking-wider text-secondary">
							Loaders
						</h3>
						<div class="flex flex-wrap gap-1.5">
							<span
								v-for="ldr in detail.loaders"
								:key="ldr"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2.5 py-1 text-xs font-semibold text-contrast"
							>
								{{ formatLoader(ldr) }}
							</span>
						</div>
					</div>

					<div v-if="detail.gameVersions.length > 0" class="flex flex-col gap-2">
						<h3 class="m-0 text-xs font-bold uppercase tracking-wider text-secondary">
							Minecraft Versions
						</h3>
						<div class="flex flex-wrap gap-1.5">
							<span
								v-for="ver in detail.gameVersions.slice(0, 12)"
								:key="ver"
								class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-xs font-medium tabular-nums text-primary"
							>
								{{ ver }}
							</span>
						</div>
					</div>
				</div>

				<!-- Categories -->
				<div
					v-if="detail.categories.length > 0"
					class="flex flex-col gap-2 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
				>
					<h3 class="m-0 text-xs font-bold uppercase tracking-wider text-secondary">
						Categories
					</h3>
					<div class="flex flex-wrap gap-1.5">
						<span
							v-for="cat in detail.categories"
							:key="cat.id"
							class="rounded-md border border-solid border-surface-5 bg-surface-2 px-2.5 py-1 text-xs font-medium text-secondary"
						>
							{{ cat.name }}
						</span>
					</div>
				</div>

				<!-- Links -->
				<div
					v-if="externalLinks.length > 0"
					class="flex flex-col gap-2 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
				>
					<h3 class="m-0 text-xs font-bold uppercase tracking-wider text-secondary">
						Links
					</h3>
					<div class="flex flex-col gap-1.5">
						<button
							v-for="link in externalLinks"
							:key="link.label"
							type="button"
							class="flex cursor-pointer items-center justify-between rounded-lg border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-xs font-semibold text-primary transition-colors hover:bg-surface-4 hover:text-contrast"
							@click="openExternal(link.url)"
						>
							<span>{{ link.label }}</span>
							<ExternalIcon class="size-3.5 text-secondary" />
						</button>
					</div>
				</div>
			</aside>
		</div>

		<!-- Screenshot Lightbox -->
		<Teleport to="body">
			<div
				v-if="detail && lightboxIndex !== null && detail.screenshots[lightboxIndex]"
				class="fixed inset-0 z-50 flex items-center justify-center bg-black/85 p-6 backdrop-blur-sm"
				@click.self="lightboxIndex = null"
			>
				<button
					type="button"
					class="fixed right-5 top-5 flex size-10 cursor-pointer items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-3 text-contrast hover:bg-surface-4"
					aria-label="Close lightbox"
					@click="lightboxIndex = null"
				>
					<XIcon class="size-5" />
				</button>

				<button
					v-if="detail.screenshots.length > 1"
					type="button"
					class="mr-3 flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-3 text-contrast hover:bg-surface-4"
					aria-label="Previous screenshot"
					@click="stepLightbox(-1)"
				>
					<ChevronLeftIcon class="size-5" />
				</button>

				<div class="flex max-h-[88vh] max-w-[82vw] flex-col items-center gap-3">
					<img
						:src="detail.screenshots[lightboxIndex].url"
						:alt="detail.screenshots[lightboxIndex].title || detail.name"
						class="max-h-[80vh] max-w-full rounded-2xl border border-solid border-surface-5 object-contain shadow-2xl"
					/>
					<p
						v-if="detail.screenshots[lightboxIndex].title"
						class="m-0 text-sm font-medium text-contrast"
					>
						{{ detail.screenshots[lightboxIndex].title }}
					</p>
				</div>

				<button
					v-if="detail.screenshots.length > 1"
					type="button"
					class="ml-3 flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-3 text-contrast hover:bg-surface-4"
					aria-label="Next screenshot"
					@click="stepLightbox(1)"
				>
					<ChevronRightIcon class="size-5" />
				</button>
			</div>
		</Teleport>
	</div>
</template>

<style scoped>
.solara-cf-badge {
	background-color: color-mix(in srgb, #f16436 16%, var(--surface-2));
	color: #f16436;
	border: 1px solid color-mix(in srgb, #f16436 32%, transparent);
}

.solara-cf-hero-scrim {
	background: linear-gradient(180deg, rgba(0, 0, 0, 0.08) 0%, rgba(0, 0, 0, 0.65) 100%);
}

.solara-pill-active {
	border-color: var(--color-brand);
	background-color: color-mix(in srgb, var(--color-brand) 16%, var(--surface-3));
}

.solara-cf-description :deep(img) {
	max-width: 100%;
	height: auto;
	border-radius: 0.75rem;
	margin: 0.5rem 0;
}

.solara-cf-description :deep(a) {
	color: var(--color-brand);
	text-decoration: underline;
}

.solara-cf-description :deep(iframe) {
	max-width: 100%;
	border-radius: 0.75rem;
}
</style>
