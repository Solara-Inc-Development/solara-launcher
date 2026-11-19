<script setup lang="ts">
import {
	CheckIcon,
	ChevronDownIcon,
	ChevronLeftIcon,
	ChevronRightIcon,
	DropdownIcon,
	ExternalIcon,
	LogInIcon,
	LogOutIcon,
	PlayIcon,
	SpinnerIcon,
	TrashIcon,
	UserPlusIcon,
} from '@modrinth/assets'
import { Avatar, Button, NewModal, SkinPreviewRenderer } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, onBeforeUnmount, onMounted, ref, useTemplateRef } from 'vue'

import defaultSteveSkin from '@/assets/skins/steve.png'
import { useMinecraftAccounts } from '@/features/minecraft-accounts/use-minecraft-accounts'
import {
	bootstrapReleasesQueryOptions,
	newsArticlesQueryOptions,
	type ReleaseNews,
	type SolaraNewsItem,
} from '@/features/news/queries'
import { product } from '@/config'
import { VERIFIED_VERSION_HISTORY, type VersionChangelog } from '@/helpers/version-history'
import { useRootBreadcrumb } from '@/shared/breadcrumbs'

defineOptions({
	name: 'LibraryPage',
})

useRootBreadcrumb({
	slot: 'root',
	id: 'home',
	label: 'Home',
	to: '/',
	visual: { type: 'icon', component: PlayIcon },
})

const mc = useMinecraftAccounts()
const {
	accounts,
	loginDisabled,
	selectedAccount,
	username,
	equippedSkin,
	equippedSkinTexture,
	equippedCapeTexture,
	avatarUrl,
	getAccountAvatarUrl,
	setAccount,
	login,
	logout,
} = mc

// Account dropdown state (when logged in)
const accountMenuOpen = ref(false)
const accountMenuRef = ref<HTMLElement | null>(null)

function handleDocumentClick(event: MouseEvent) {
	if (
		accountMenuOpen.value &&
		accountMenuRef.value &&
		!accountMenuRef.value.contains(event.target as Node)
	) {
		accountMenuOpen.value = false
	}
}

function handleEscapeKey(event: KeyboardEvent) {
	if (event.key === 'Escape' && accountMenuOpen.value) {
		accountMenuOpen.value = false
	}
}

// Detect WebGL support so 3D skin viewer never crashes on software-only environments
const webglSupported = ref(true)
function checkWebGLSupport(): boolean {
	try {
		const canvas = document.createElement('canvas')
		const gl = canvas.getContext('webgl2') || canvas.getContext('webgl')
		return Boolean(gl)
	} catch {
		return false
	}
}

onMounted(() => {
	webglSupported.value = checkWebGLSupport()
	document.addEventListener('click', handleDocumentClick)
	document.addEventListener('keydown', handleEscapeKey)
})

onBeforeUnmount(() => {
	document.removeEventListener('click', handleDocumentClick)
	document.removeEventListener('keydown', handleEscapeKey)
})

// 3D animated skin props matching Skin Editor
const heroSkinTexture = computed(() => equippedSkinTexture.value || defaultSteveSkin)
const heroSkinVariant = computed<'CLASSIC' | 'SLIM' | 'UNKNOWN'>(
	() => equippedSkin.value?.variant ?? 'CLASSIC',
)
const heroCapeTexture = computed(() => equippedCapeTexture.value)
const heroAnimationConfig = {
	baseAnimation: 'idle',
	randomAnimations: ['idle_sub_1', 'idle_sub_2', 'idle_sub_3'],
	randomAnimationInterval: 7000,
	transitionDuration: 0.25,
}

async function handleSelectAccount(account: (typeof accounts.value)[number]) {
	accountMenuOpen.value = false
	await setAccount(account)
}

async function handleAddAccount() {
	accountMenuOpen.value = false
	await login()
}

async function handleLogoutCurrent() {
	if (!selectedAccount.value) return
	const id = selectedAccount.value.profile.id
	accountMenuOpen.value = false
	await logout(id)
}

async function handleRemoveAccount(id: string) {
	await logout(id)
	if (accounts.value.length === 0) {
		accountMenuOpen.value = false
	}
}

function scrollToBelowFold() {
	const el = document.getElementById('solara-below-fold')
	if (el) {
		el.scrollIntoView({ behavior: 'smooth', block: 'start' })
	}
}

// --- News (last 30 max) ---
const newsQuery = useQuery(newsArticlesQueryOptions())
const bootstrapQuery = useQuery(bootstrapReleasesQueryOptions())

const fallbackNewsItems = computed<SolaraNewsItem[]>(() =>
	VERIFIED_VERSION_HISTORY.slice(0, 30).map((entry) => ({
		id: entry.version,
		publicVersion: entry.version,
		channel: entry.channel,
		path: `${product.source}/releases/tag/${entry.version}`,
		thumbnail: '',
		title: entry.title,
		summary: entry.featuresAdded[0] ?? entry.issuesFixed[0] ?? '',
		date: entry.date,
		notes: [
			...entry.featuresAdded.map((item) => `- ${item}`),
			...entry.issuesFixed.map((item) => `- ${item}`),
		].join('\n'),
		bullets: [...entry.featuresAdded, ...entry.issuesFixed],
	})),
)

const allNewsItems = computed<SolaraNewsItem[]>(() => {
	const live = newsQuery.data.value ?? []
	const source = live.length > 0 ? live : fallbackNewsItems.value
	return source.slice(0, 30)
})

const featuredNewsItems = computed(() => allNewsItems.value.slice(0, 5))

// --- Version History / Updates (last 30 max) ---
function parseBootstrapReleaseToChangelog(release: ReleaseNews, isLatest: boolean): VersionChangelog {
	const raw = (release.notes ?? '').trim()
	const lines = raw
		.split('\n')
		.map((line) => line.trim())
		.filter((line) => line.length > 0 && !line.startsWith('Updater-signed Windows testing preview'))

	let title = `Solara Launcher ${release.publicVersion}`
	const featuresAdded: string[] = []
	const issuesFixed: string[] = []
	const futureUpdates: string[] = []

	for (const line of lines) {
		if (line.startsWith('# Changelog')) continue
		if (line.startsWith('- ') || line.startsWith('* ')) {
			const clean = line
				.replace(/^[-*]\s+/, '')
				.replace(/\*\*(.*?)\*\*/g, '$1')
				.replace(/`([^`]+)`/g, '$1')
				.trim()
			if (!clean) continue
			if (/\b(fix|fixed|resolv|bug|crash|guard|prevent|black screen|strip|clean)\b/i.test(clean)) {
				issuesFixed.push(clean)
			} else if (/\b(future|upcoming|planned|pending|next)\b/i.test(clean)) {
				futureUpdates.push(clean)
			} else {
				featuresAdded.push(clean)
			}
			continue
		}
		if (title === `Solara Launcher ${release.publicVersion}`) {
			const cleanHeading = line.replace(/^#{1,6}\s+/, '').trim()
			if (cleanHeading.length >= 6 && cleanHeading.length <= 96) {
				title = cleanHeading
			}
		}
	}

	if (featuresAdded.length === 0 && issuesFixed.length === 0) {
		featuresAdded.push(
			release.channel === 'stable'
				? `Published verified Solara Launcher ${release.publicVersion} release.`
				: `Published signed Windows testing preview ${release.publicVersion}.`,
		)
	}
	if (issuesFixed.length === 0) {
		issuesFixed.push('General stability, packaging, and launcher reliability improvements.')
	}
	if (futureUpdates.length === 0) {
		futureUpdates.push('Continued independent launcher core development and UI polish.')
	}

	return {
		version: release.publicVersion,
		title,
		date: formatDate(release.publishedAt || release.verifiedAt),
		channel: release.channel === 'stable' ? 'Stable' : 'Testing Preview',
		isLatest,
		featuresAdded: featuresAdded.slice(0, 6),
		issuesFixed: issuesFixed.slice(0, 6),
		futureUpdates: futureUpdates.slice(0, 4),
	}
}

const allVersionUpdates = computed<VersionChangelog[]>(() => {
	const curatedByVersion = new Map<string, VersionChangelog>()
	for (const item of VERIFIED_VERSION_HISTORY) {
		curatedByVersion.set(item.version, item)
	}

	const liveReleases = bootstrapQuery.data.value ?? []
	const merged: VersionChangelog[] = []
	const seen = new Set<string>()

	// Always keep top curated entries first if live hasn't synced the newest tag yet
	for (const curated of VERIFIED_VERSION_HISTORY) {
		if (!seen.has(curated.version)) {
			seen.add(curated.version)
			merged.push(curated)
		}
	}

	for (const release of liveReleases) {
		if (seen.has(release.publicVersion)) continue
		seen.add(release.publicVersion)
		const curated = curatedByVersion.get(release.publicVersion)
		merged.push(curated ?? parseBootstrapReleaseToChangelog(release, merged.length === 0))
	}

	return merged.slice(0, 30).map((entry, idx) => ({
		...entry,
		isLatest: idx === 0,
	}))
})

// Horizontal scroll controls for Updates strip
const updatesStripRef = ref<HTMLElement | null>(null)
function scrollUpdatesStrip(direction: -1 | 1) {
	const el = updatesStripRef.value
	if (!el) return
	el.scrollBy({ left: direction * 340, behavior: 'smooth' })
}

// Modals
const newsDetailModal = useTemplateRef<InstanceType<typeof NewModal>>('newsDetailModal')
const allNewsModal = useTemplateRef<InstanceType<typeof NewModal>>('allNewsModal')
const updateDetailModal = useTemplateRef<InstanceType<typeof NewModal>>('updateDetailModal')
const allVersionsModal = useTemplateRef<InstanceType<typeof NewModal>>('allVersionsModal')

const activeNewsItem = ref<SolaraNewsItem | null>(null)
const activeVersionEntry = ref<VersionChangelog | null>(null)

function openNewsDetail(item: SolaraNewsItem) {
	activeNewsItem.value = item
	allNewsModal.value?.hide()
	newsDetailModal.value?.show()
}

function openAllNews() {
	allNewsModal.value?.show()
}

function openUpdateDetail(entry: VersionChangelog) {
	activeVersionEntry.value = entry
	allVersionsModal.value?.hide()
	updateDetailModal.value?.show()
}

function openAllVersions() {
	allVersionsModal.value?.show()
}

function formatDate(iso?: string | null) {
	if (!iso) return ''
	const parsed = dayjs(iso)
	return parsed.isValid() ? parsed.format('MMM D, YYYY') : iso
}
</script>

<template>
	<div class="solara-home-root w-full max-w-6xl mx-auto px-6 md:px-10 pb-16 flex flex-col">
		<!-- ============================================================
		     1. ABOVE-THE-FOLD HERO (Full Initial Screen)
		     Matches owner sketch:
		     Left: Welcome Back, [Player Name] / Good To See You Back :] + Sign In / Account Dropdown Pill
		     Right: 3D Animated Player Skin (like Skin Editor)
		     ============================================================ -->
		<section
			class="solara-hero relative min-h-[calc(100vh-5.25rem)] flex flex-col justify-between py-8 md:py-10"
		>
			<div
				class="flex-1 grid grid-cols-1 lg:grid-cols-12 items-center gap-8 lg:gap-6 my-auto"
			>
				<!-- Left Column: Greeting + Account Pill -->
				<div class="lg:col-span-7 flex flex-col items-start justify-center gap-7">
					<div class="flex flex-col gap-3 w-full">
						<h1
							class="text-4xl sm:text-5xl md:text-6xl font-black text-contrast tracking-tight leading-[1.08] m-0"
						>
							<span class="block">Welcome Back,</span>
							<span class="block pl-6 sm:pl-12 md:pl-16 mt-1 text-brand truncate">
								{{ username || 'Player' }}
							</span>
						</h1>

						<p
							class="text-lg sm:text-xl md:text-2xl text-secondary font-medium tracking-tight m-0 mt-1"
						>
							Good To See You Back :]
						</p>
					</div>

					<!-- Sign In Pill (logged out) OR Player Name + Dropdown Pill (logged in) -->
					<div ref="accountMenuRef" class="relative mt-2 pl-2 sm:pl-8">
						<!-- Not Logged In -->
						<button
							v-if="!selectedAccount"
							type="button"
							:disabled="loginDisabled"
							class="solara-signin-pill inline-flex items-center justify-center gap-3 px-9 py-3.5 rounded-full text-base font-bold bg-surface-2 hover:bg-surface-3 text-contrast border-2 border-solid border-brand/60 hover:border-brand cursor-pointer transition-all shadow-sm disabled:opacity-60 disabled:cursor-not-allowed"
							@click="login()"
						>
							<SpinnerIcon v-if="loginDisabled" class="size-5 animate-spin text-brand" />
							<LogInIcon v-else class="size-5 text-brand" />
							<span>{{ loginDisabled ? 'Signing In...' : 'Sign In' }}</span>
						</button>

						<!-- Logged In: Player Name Pill + Dropdown for Switch / Logout / Add More Accounts -->
						<div v-else class="relative">
							<button
								type="button"
								class="solara-account-pill inline-flex items-center gap-3 pl-3 pr-5 py-2.5 rounded-full text-base font-bold bg-surface-2 hover:bg-surface-3 text-contrast border-2 border-solid border-brand/50 hover:border-brand cursor-pointer transition-all shadow-sm"
								@click="accountMenuOpen = !accountMenuOpen"
							>
								<div class="relative shrink-0">
									<Avatar
										size="34px"
										:src="avatarUrl"
										class="rounded-full border border-surface-5"
									/>
									<span
										class="absolute -bottom-0.5 -right-0.5 size-2.5 rounded-full bg-emerald-500 border-2 border-surface-2"
									/>
								</div>
								<span class="max-w-[14rem] truncate">{{ selectedAccount.profile.name }}</span>
								<DropdownIcon
									class="size-4 text-secondary transition-transform duration-200"
									:class="{ 'rotate-180': accountMenuOpen }"
								/>
							</button>

							<!-- Account Dropdown Menu -->
							<Transition
								enter-active-class="transition duration-150 ease-out"
								enter-from-class="opacity-0 -translate-y-1 scale-95"
								enter-to-class="opacity-100 translate-y-0 scale-100"
								leave-active-class="transition duration-100 ease-in"
								leave-from-class="opacity-100 translate-y-0 scale-100"
								leave-to-class="opacity-0 -translate-y-1 scale-95"
							>
								<div
									v-if="accountMenuOpen"
									class="absolute left-0 top-[calc(100%+0.6rem)] z-30 w-72 rounded-2xl border border-solid border-surface-5 bg-surface-2 p-2.5 shadow-xl flex flex-col gap-2"
								>
									<div class="px-2 pt-1 pb-0.5 text-[11px] font-bold uppercase tracking-wider text-secondary">
										Minecraft Accounts ({{ accounts.length }})
									</div>

									<div class="flex flex-col gap-1 max-h-48 overflow-y-auto">
										<div
											v-for="account in accounts"
											:key="account.profile.id"
											class="flex items-center justify-between gap-2 rounded-xl px-2.5 py-2 transition-colors"
											:class="
												selectedAccount.profile.id === account.profile.id
													? 'bg-brand/15 border border-solid border-brand/30'
													: 'hover:bg-surface-3 border border-solid border-transparent'
											"
										>
											<button
												type="button"
												class="flex items-center gap-2.5 bg-transparent border-0 p-0 cursor-pointer flex-1 min-w-0 text-left"
												@click="handleSelectAccount(account)"
											>
												<Avatar
													:src="getAccountAvatarUrl(account)"
													size="26px"
													class="rounded-md shrink-0"
												/>
												<span class="text-sm font-semibold text-contrast truncate">
													{{ account.profile.name }}
												</span>
												<CheckIcon
													v-if="selectedAccount.profile.id === account.profile.id"
													class="size-4 text-brand ml-auto shrink-0"
												/>
											</button>
											<button
												type="button"
												title="Remove account"
												class="p-1 rounded-lg bg-transparent border-0 text-secondary hover:text-red hover:bg-surface-4 cursor-pointer shrink-0 flex"
												@click.stop="handleRemoveAccount(account.profile.id)"
											>
												<TrashIcon class="size-3.5" />
											</button>
										</div>
									</div>

									<div class="h-px bg-surface-5 my-0.5" />

									<button
										type="button"
										:disabled="loginDisabled"
										class="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl bg-transparent hover:bg-surface-3 border-0 text-sm font-semibold text-contrast cursor-pointer transition-colors disabled:opacity-50"
										@click="handleAddAccount"
									>
										<SpinnerIcon v-if="loginDisabled" class="size-4 animate-spin text-brand" />
										<UserPlusIcon v-else class="size-4 text-brand" />
										<span>Add more accounts</span>
									</button>

									<button
										type="button"
										class="w-full flex items-center gap-2.5 px-3 py-2 rounded-xl bg-transparent hover:bg-red/10 border-0 text-sm font-semibold text-red cursor-pointer transition-colors"
										@click="handleLogoutCurrent"
									>
										<LogOutIcon class="size-4" />
										<span>Logout</span>
									</button>
								</div>
							</Transition>
						</div>
					</div>
				</div>

				<!-- Right Column: 3D Animated Player Skin Showcase -->
				<div class="lg:col-span-5 flex flex-col items-center justify-center relative">
					<div
						class="solara-skin-stage relative w-full max-w-[21rem] h-[25rem] md:h-[29rem] flex items-center justify-center"
					>
						<div class="solara-skin-glow pointer-events-none absolute inset-6 rounded-full" />

						<SkinPreviewRenderer
							v-if="webglSupported"
							:texture-src="heroSkinTexture"
							:ears-texture-src="equippedSkin?.texture"
							:cape-src="heroCapeTexture"
							:variant="heroSkinVariant"
							:nametag="username || undefined"
							framing="modal"
							:initial-rotation="Math.PI / 9"
							:animation-config="heroAnimationConfig"
							class="h-full w-full relative z-10"
						/>

						<div
							v-else
							class="relative z-10 flex flex-col items-center justify-center gap-3 p-6 rounded-3xl border border-solid border-surface-5 bg-surface-2/80"
						>
							<Avatar :src="avatarUrl" size="112px" class="rounded-2xl border border-surface-5" />
							<span class="text-sm font-bold text-contrast">{{ username || 'Player' }}</span>
						</div>
					</div>
				</div>
			</div>

			<!-- Scroll Cue at bottom of first screen -->
			<div class="flex justify-center pt-2">
				<button
					type="button"
					class="inline-flex items-center gap-2 px-4 py-1.5 rounded-full bg-transparent hover:bg-surface-2 border border-solid border-transparent hover:border-surface-5 text-xs font-semibold text-secondary hover:text-contrast cursor-pointer transition-all"
					@click="scrollToBelowFold"
				>
					<span>Scroll down for News &amp; Updates</span>
					<ChevronDownIcon class="size-4 text-brand animate-bounce" />
				</button>
			</div>
		</section>

		<!-- ============================================================
		     2. BELOW THE FOLD (# After player or user scrolls down)
		     ============================================================ -->
		<div
			id="solara-below-fold"
			class="flex flex-col gap-12 pt-10 border-0 border-t border-solid border-surface-5"
		>
			<!-- Section A: All News from Solara (5 vertical cards + See all news -> last 30 only) -->
			<section class="flex flex-col gap-4">
				<div class="flex items-center justify-between gap-4">
					<h2 class="text-xl md:text-2xl font-extrabold text-contrast tracking-tight m-0">
						All News from Solara
					</h2>
					<button
						type="button"
						class="text-sm font-semibold text-brand hover:underline bg-transparent border-0 cursor-pointer p-0 shrink-0"
						@click="openAllNews"
					>
						See all news
					</button>
				</div>

				<div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4">
					<article
						v-for="item in featuredNewsItems"
						:key="item.id"
						class="solara-news-card group rounded-2xl border border-solid border-surface-5 bg-surface-2 hover:border-brand/55 hover:bg-surface-3 transition-all p-4 flex flex-col justify-between min-h-[14.5rem] cursor-pointer shadow-sm"
						@click="openNewsDetail(item)"
					>
						<div class="flex flex-col gap-2.5">
							<div class="flex items-center justify-between gap-2">
								<span
									class="inline-flex px-2 py-0.5 rounded-md text-[11px] font-bold bg-brand/10 text-brand border border-solid border-brand/20"
								>
									{{ item.publicVersion }}
								</span>
								<span class="text-[11px] text-secondary truncate">
									{{ formatDate(item.date) }}
								</span>
							</div>

							<h3
								class="text-sm font-bold text-contrast group-hover:text-brand transition-colors m-0 line-clamp-2 leading-snug"
							>
								{{ item.title }}
							</h3>

							<p class="text-xs text-secondary m-0 line-clamp-4 leading-relaxed">
								{{ item.summary }}
							</p>
						</div>

						<div class="pt-3 mt-2 border-0 border-t border-solid border-surface-5/60 flex items-center justify-end">
							<button
								type="button"
								class="inline-flex items-center gap-1 text-xs font-bold text-brand group-hover:underline bg-transparent border-0 p-0 cursor-pointer"
								@click.stop="openNewsDetail(item)"
							>
								<span>... Read more</span>
							</button>
						</div>
					</article>
				</div>
			</section>

			<!-- Section B: All updates of solara launcher (Horizontal boxed row + All version history -> last 30 only) -->
			<section class="flex flex-col gap-4">
				<div class="flex items-center justify-between gap-4">
					<h2 class="text-xl md:text-2xl font-extrabold text-contrast tracking-tight m-0">
						All updates of solara launcher
					</h2>
					<div class="flex items-center gap-3 shrink-0">
						<div class="hidden sm:flex items-center gap-1">
							<button
								type="button"
								aria-label="Scroll updates left"
								class="size-7 rounded-lg border border-solid border-surface-5 bg-surface-2 hover:bg-surface-3 text-secondary hover:text-contrast flex items-center justify-center cursor-pointer transition-colors"
								@click="scrollUpdatesStrip(-1)"
							>
								<ChevronLeftIcon class="size-4" />
							</button>
							<button
								type="button"
								aria-label="Scroll updates right"
								class="size-7 rounded-lg border border-solid border-surface-5 bg-surface-2 hover:bg-surface-3 text-secondary hover:text-contrast flex items-center justify-center cursor-pointer transition-colors"
								@click="scrollUpdatesStrip(1)"
							>
								<ChevronRightIcon class="size-4" />
							</button>
						</div>
						<button
							type="button"
							class="text-sm font-semibold text-brand hover:underline bg-transparent border-0 cursor-pointer p-0"
							@click="openAllVersions"
						>
							All version history
						</button>
					</div>
				</div>

				<!-- Framed Horizontal Row matching sketch -->
				<div
					class="rounded-2xl border border-solid border-surface-5 bg-surface-2/60 p-4 shadow-sm"
				>
					<div
						ref="updatesStripRef"
						class="solara-updates-strip flex gap-3.5 overflow-x-auto pb-1 snap-x snap-mandatory"
					>
						<article
							v-for="release in allVersionUpdates"
							:key="release.version"
							class="group snap-start shrink-0 w-[16.5rem] sm:w-[17.5rem] rounded-xl border border-solid border-surface-5 bg-surface-2 hover:border-brand/55 hover:bg-surface-3 transition-all p-4 flex flex-col justify-between min-h-[12rem] cursor-pointer"
							@click="openUpdateDetail(release)"
						>
							<div class="flex flex-col gap-2">
								<div class="flex items-center justify-between gap-2">
									<span
										class="px-2.5 py-0.5 rounded-md text-xs font-black"
										:class="
											release.isLatest
												? 'bg-brand text-[var(--color-accent-contrast)]'
												: 'bg-surface-4 text-contrast'
										"
									>
										{{ release.version }}
									</span>
									<span class="text-[11px] text-secondary truncate">
										{{ release.date }}
									</span>
								</div>

								<h3
									class="text-sm font-bold text-contrast group-hover:text-brand transition-colors m-0 line-clamp-2 leading-snug"
								>
									{{ release.title }}
								</h3>

								<p class="text-xs text-secondary m-0 line-clamp-3 leading-relaxed">
									{{ release.featuresAdded[0] || release.issuesFixed[0] }}
								</p>
							</div>

							<div class="pt-2.5 mt-2 flex items-center justify-between text-xs">
								<span class="text-[11px] text-secondary">{{ release.channel }}</span>
								<button
									type="button"
									class="font-bold text-brand group-hover:underline bg-transparent border-0 p-0 cursor-pointer"
									@click.stop="openUpdateDetail(release)"
								>
									... Read more
								</button>
							</div>
						</article>
					</div>
				</div>
			</section>
		</div>

		<!-- ============================================================
		     MODALS: News Detail, See All News (Last 30), Update Detail, All Version History (Last 30)
		     ============================================================ -->

		<!-- News Detail Modal -->
		<NewModal ref="newsDetailModal" :header="activeNewsItem?.title || 'Solara News'" max-width="640px">
			<div v-if="activeNewsItem" class="flex flex-col gap-4 w-full max-w-[36rem]">
				<div class="flex items-center gap-2 text-xs text-secondary">
					<span class="px-2 py-0.5 rounded bg-brand/15 text-brand font-bold">
						{{ activeNewsItem.publicVersion }}
					</span>
					<span>{{ formatDate(activeNewsItem.date) }}</span>
					<span>•</span>
					<span>{{ activeNewsItem.channel }}</span>
				</div>

				<div
					class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-4 max-h-[50vh] overflow-y-auto"
				>
					<ul v-if="activeNewsItem.bullets.length > 0" class="m-0 pl-5 space-y-2 text-sm text-primary">
						<li v-for="(bullet, idx) in activeNewsItem.bullets" :key="idx">{{ bullet }}</li>
					</ul>
					<p v-else class="m-0 text-sm text-primary whitespace-pre-wrap leading-relaxed">
						{{ activeNewsItem.notes }}
					</p>
				</div>

				<div class="flex items-center justify-end gap-2 pt-1">
					<a
						v-if="activeNewsItem.path"
						:href="activeNewsItem.path"
						target="_blank"
						rel="noreferrer"
						class="inline-flex items-center gap-1.5 px-4 py-2 rounded-xl border border-solid border-surface-5 bg-surface-3 hover:bg-surface-4 text-sm font-semibold text-contrast no-underline transition-colors"
					>
						<ExternalIcon class="size-4" />
						<span>Open on GitHub</span>
					</a>
					<Button color="brand" @click="newsDetailModal?.hide()">Close</Button>
				</div>
			</div>
		</NewModal>

		<!-- See All News Modal (Last 30 Only) -->
		<NewModal ref="allNewsModal" header="All News from Solara" max-width="760px">
			<div class="flex flex-col gap-3 w-full max-w-[44rem]">
				<p class="text-xs text-secondary m-0">
					Showing the latest {{ allNewsItems.length }} news updates (last 30 only).
				</p>
				<div class="flex flex-col gap-2.5 max-h-[60vh] overflow-y-auto pr-1">
					<div
						v-for="item in allNewsItems"
						:key="item.id"
						class="rounded-xl border border-solid border-surface-5 bg-surface-3 hover:border-brand/50 p-3.5 flex items-center justify-between gap-4 cursor-pointer transition-colors"
						@click="openNewsDetail(item)"
					>
						<div class="flex flex-col gap-1 min-w-0">
							<div class="flex items-center gap-2">
								<span class="px-2 py-0.5 rounded text-[11px] font-bold bg-brand/15 text-brand">
									{{ item.publicVersion }}
								</span>
								<span class="text-sm font-bold text-contrast truncate">{{ item.title }}</span>
							</div>
							<p class="text-xs text-secondary m-0 line-clamp-2">{{ item.summary }}</p>
						</div>
						<div class="flex flex-col items-end gap-1 shrink-0">
							<span class="text-[11px] text-secondary">{{ formatDate(item.date) }}</span>
							<span class="text-xs font-bold text-brand">Read more</span>
						</div>
					</div>
				</div>
				<div class="flex justify-end pt-1">
					<Button type="outlined" @click="allNewsModal?.hide()">Close</Button>
				</div>
			</div>
		</NewModal>

		<!-- Single Update Changelog Detail Modal -->
		<NewModal
			ref="updateDetailModal"
			:header="activeVersionEntry ? `Solara Launcher ${activeVersionEntry.version}` : 'Update Changelog'"
			max-width="680px"
		>
			<div v-if="activeVersionEntry" class="flex flex-col gap-4 w-full max-w-[40rem]">
				<div class="flex flex-wrap items-center gap-2">
					<span class="px-2.5 py-0.5 rounded-md text-xs font-black bg-brand text-[var(--color-accent-contrast)]">
						{{ activeVersionEntry.version }}
					</span>
					<span class="text-base font-bold text-contrast">{{ activeVersionEntry.title }}</span>
					<span class="text-xs text-secondary ml-auto">
						{{ activeVersionEntry.date }} • {{ activeVersionEntry.channel }}
					</span>
				</div>

				<div class="flex flex-col gap-3 max-h-[55vh] overflow-y-auto pr-1">
					<div class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-3.5">
						<h4 class="text-xs font-bold uppercase tracking-wider text-contrast m-0 mb-2">
							Features Added
						</h4>
						<ul class="m-0 pl-4 space-y-1.5 text-xs text-primary list-disc">
							<li v-for="(feat, i) in activeVersionEntry.featuresAdded" :key="i">{{ feat }}</li>
						</ul>
					</div>

					<div class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-3.5">
						<h4 class="text-xs font-bold uppercase tracking-wider text-contrast m-0 mb-2">
							Issues Fixed
						</h4>
						<ul class="m-0 pl-4 space-y-1.5 text-xs text-primary list-disc">
							<li v-for="(fix, i) in activeVersionEntry.issuesFixed" :key="i">{{ fix }}</li>
						</ul>
					</div>

					<div class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-3.5">
						<h4 class="text-xs font-bold uppercase tracking-wider text-contrast m-0 mb-2">
							Future Updates Implementation
						</h4>
						<ul class="m-0 pl-4 space-y-1.5 text-xs text-primary list-disc">
							<li v-for="(fut, i) in activeVersionEntry.futureUpdates" :key="i">{{ fut }}</li>
						</ul>
					</div>
				</div>

				<div class="flex justify-end pt-1">
					<Button color="brand" @click="updateDetailModal?.hide()">Close</Button>
				</div>
			</div>
		</NewModal>

		<!-- All Version History Modal (Last 30 Only) -->
		<NewModal ref="allVersionsModal" header="All Version History" max-width="780px">
			<div class="flex flex-col gap-3 w-full max-w-[46rem]">
				<p class="text-xs text-secondary m-0">
					Showing the latest {{ allVersionUpdates.length }} releases (last 30 only).
				</p>
				<div class="flex flex-col gap-2.5 max-h-[60vh] overflow-y-auto pr-1">
					<div
						v-for="release in allVersionUpdates"
						:key="release.version"
						class="rounded-xl border border-solid border-surface-5 bg-surface-3 hover:border-brand/50 p-3.5 flex items-center justify-between gap-4 cursor-pointer transition-colors"
						@click="openUpdateDetail(release)"
					>
						<div class="flex flex-col gap-1 min-w-0">
							<div class="flex items-center gap-2">
								<span
									class="px-2 py-0.5 rounded text-xs font-black"
									:class="
										release.isLatest
											? 'bg-brand text-[var(--color-accent-contrast)]'
											: 'bg-surface-4 text-contrast'
									"
								>
									{{ release.version }}
								</span>
								<span class="text-sm font-bold text-contrast truncate">{{ release.title }}</span>
							</div>
							<p class="text-xs text-secondary m-0 line-clamp-1">
								{{ release.featuresAdded[0] || release.issuesFixed[0] }}
							</p>
						</div>
						<div class="flex flex-col items-end gap-1 shrink-0">
							<span class="text-[11px] text-secondary">{{ release.date }}</span>
							<span class="text-xs font-bold text-brand">View changelog</span>
						</div>
					</div>
				</div>
				<div class="flex justify-end pt-1">
					<Button type="outlined" @click="allVersionsModal?.hide()">Close</Button>
				</div>
			</div>
		</NewModal>
	</div>
</template>

<style scoped lang="scss">
.solara-skin-glow {
	background: radial-gradient(
		circle at center,
		color-mix(in srgb, var(--color-brand) 22%, transparent) 0%,
		transparent 70%
	);
	filter: blur(22px);
}

.solara-signin-pill,
.solara-account-pill {
	border: 2px solid color-mix(in srgb, var(--color-brand) 65%, transparent);
	transition:
		border-color 0.2s ease,
		background-color 0.2s ease,
		box-shadow 0.2s ease,
		transform 0.15s ease;
}

.solara-signin-pill:hover,
.solara-account-pill:hover {
	border-color: var(--color-brand);
	box-shadow: 0 0 26px -6px color-mix(in srgb, var(--color-brand) 50%, transparent);
}

.solara-updates-strip {
	scrollbar-width: thin;
	scrollbar-color: var(--color-button-bg) transparent;
}
</style>
