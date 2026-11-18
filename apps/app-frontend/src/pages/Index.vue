<script setup lang="ts">
import {
	CheckIcon,
	CompassIcon,
	DownloadIcon,
	ExternalIcon,
	LogInIcon,
	PlayIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	RefreshCwIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, inject, ref } from 'vue'
import { useRouter } from 'vue-router'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { useMinecraftAccounts } from '@/features/minecraft-accounts/use-minecraft-accounts'
import { newsArticlesQueryOptions } from '@/features/news/queries'
import { getInstanceIconUrl, run as runInstance } from '@/helpers/instance'
import { VERIFIED_VERSION_HISTORY } from '@/helpers/version-history'
import { instanceListQueryOptions } from '@/pages/instance/query-options'
import { useRootBreadcrumb } from '@/shared/breadcrumbs'

defineOptions({
	name: 'LibraryPage',
})

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const router = useRouter()
const showCreationModal = inject<() => void>('showCreationModal')
const appSettings = useAppSettings()

const mc = useMinecraftAccounts()
const {
	accounts,
	loginDisabled,
	selectedAccount,
	username,
	avatarUrl,
	getAccountAvatarUrl,
	setAccount,
	login,
	logout,
} = mc

const isSwitchingAccounts = ref(false)
const launchingInstanceId = ref<string | null>(null)

useRootBreadcrumb({
	slot: 'root',
	id: 'home',
	label: 'Home',
	to: '/',
	visual: { type: 'icon', component: PlayIcon },
})

const instancesQuery = useQuery(instanceListQueryOptions())
const instances = computed(() => instancesQuery.data.value ?? [])

const recentInstances = computed(() =>
	[...instances.value].sort((a, b) =>
		dayjs(b.last_played ?? b.created).diff(dayjs(a.last_played ?? a.created)),
	),
)

const newsQuery = useQuery({
	...newsArticlesQueryOptions(),
	select: (articles) => articles.slice(0, 4),
})
const newsList = computed(() => newsQuery.data.value ?? [])

async function handleLaunch(instance: any) {
	if (launchingInstanceId.value) return
	launchingInstanceId.value = instance.id
	try {
		await runInstance(instance.id)
	} catch (error) {
		handleError(error)
	} finally {
		launchingInstanceId.value = null
	}
}

function formatDate(iso?: string | null) {
	if (!iso) return ''
	return dayjs(iso).format('MMMM D, YYYY')
}

const messages = defineMessages({
	home: {
		id: 'app.navigation.home',
		defaultMessage: 'Home',
	},
})
</script>

<template>
	<div class="home-page-container p-6 md:p-10 max-w-5xl mx-auto flex flex-col gap-10">
		<!-- 1. Cool Welcome Header -->
		<section class="welcome-header flex flex-col gap-2 pb-6 border-0 border-b border-solid border-surface-5">
			<div class="flex items-center gap-2">
				<span
					class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-brand/10 text-brand border border-solid border-brand/25"
				>
					<span class="size-2 rounded-full bg-brand animate-pulse" />
					Solara Launcher
				</span>
			</div>
			<h1 class="text-3xl md:text-5xl font-black text-contrast tracking-tight m-0">
				Welcome Back, {{ username || 'Player' }}
			</h1>
			<p class="text-base md:text-xl text-secondary font-medium m-0 flex items-center gap-2">
				Good To See You back :]
			</p>
		</section>

		<!-- 2. Minecraft Account / Playing As Card -->
		<section class="account-section flex flex-col gap-3">
			<div class="flex items-center justify-between">
				<h2 class="text-xs font-bold text-secondary uppercase tracking-widest m-0">
					Playing As / Minecraft Profile
				</h2>
				<button
					v-if="accounts.length > 1"
					type="button"
					class="text-xs font-semibold text-brand hover:underline bg-transparent border-0 cursor-pointer p-0"
					@click="isSwitchingAccounts = !isSwitchingAccounts"
				>
					{{ isSwitchingAccounts ? 'Done' : 'Switch Account' }}
				</button>
			</div>

			<div
				v-if="accounts.length === 0"
				class="rounded-2xl border border-solid border-surface-5 bg-surface-2 p-6 flex flex-col sm:flex-row sm:items-center justify-between gap-4 shadow-sm"
			>
				<div class="flex items-center gap-4">
					<Avatar size="48px" src="https://launcher-files.modrinth.com/assets/steve_head.png" />
					<div class="flex flex-col">
						<span class="text-base font-bold text-contrast">Not Signed In</span>
						<span class="text-sm text-secondary">
							Sign in with your Microsoft account to launch licensed Minecraft and sync skins.
						</span>
					</div>
				</div>
				<Button color="brand" :disabled="loginDisabled" @click="login()">
					<LogInIcon v-if="!loginDisabled" />
					<SpinnerIcon v-else class="animate-spin" />
					Sign in to Minecraft
				</Button>
			</div>

			<div
				v-else
				class="rounded-2xl border border-solid border-surface-5 bg-surface-2 overflow-hidden shadow-sm"
			>
				<div class="p-5 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
					<div class="flex items-center gap-4 min-w-0">
						<div class="relative shrink-0">
							<Avatar size="52px" :src="avatarUrl" class="rounded-xl border border-surface-5" />
							<span
								class="absolute -bottom-1 -right-1 size-3.5 rounded-full bg-emerald-500 border-2 border-surface-2"
								title="Active Profile"
							/>
						</div>
						<div class="flex flex-col min-w-0">
							<div class="flex items-center gap-2">
								<span class="text-xl font-bold text-contrast truncate">
									{{ selectedAccount ? selectedAccount.profile.name : 'Unknown Account' }}
								</span>
								<span
									class="inline-flex px-2 py-0.5 rounded text-[11px] font-semibold bg-surface-4 text-brand"
								>
									Microsoft
								</span>
							</div>
							<span class="text-xs text-secondary">Ready to play</span>
						</div>
					</div>

					<div class="flex items-center gap-2 shrink-0">
						<Button
							type="outlined"
							size="sm"
							@click="isSwitchingAccounts = !isSwitchingAccounts"
						>
							{{ isSwitchingAccounts ? 'Close' : 'Manage Accounts' }}
						</Button>
						<Button
							color="brand"
							size="sm"
							:disabled="loginDisabled"
							@click="login()"
						>
							<PlusIcon />
							Add Account
						</Button>
					</div>
				</div>

				<!-- Switch / Manage Accounts Accordion List -->
				<div
					v-if="isSwitchingAccounts"
					class="p-4 bg-surface-3 border-0 border-t border-solid border-surface-5 flex flex-col gap-2"
				>
					<span class="text-xs font-bold text-secondary uppercase tracking-wider mb-1">
						Select Active Account
					</span>
					<div
						v-for="account in accounts"
						:key="account.profile.id"
						class="flex items-center justify-between p-2.5 rounded-xl bg-surface-2 border border-surface-5 hover:border-brand/40 transition-colors"
					>
						<button
							type="button"
							class="flex items-center gap-3 bg-transparent border-0 cursor-pointer flex-grow text-left p-0 min-w-0"
							@click="setAccount(account)"
						>
							<RadioButtonCheckedIcon
								v-if="selectedAccount && selectedAccount.profile.id === account.profile.id"
								class="size-5 text-brand shrink-0"
							/>
							<RadioButtonIcon v-else class="size-5 text-secondary shrink-0" />
							<Avatar :src="getAccountAvatarUrl(account)" size="30px" class="rounded" />
							<span
								class="truncate text-sm font-semibold"
								:class="
									selectedAccount && selectedAccount.profile.id === account.profile.id
										? 'text-contrast'
										: 'text-primary'
								"
							>
								{{ account.profile.name }}
							</span>
						</button>
						<IconButton
							type="quiet"
							color="red"
							label="Remove account"
							size="xs"
							class="text-secondary hover:text-red shrink-0"
							@click="logout(account.profile.id)"
						>
							<TrashIcon class="size-4" />
						</IconButton>
					</div>
				</div>
			</div>
		</section>

		<!-- 3. Installed Instances / Quick Launch -->
		<section class="instances-section flex flex-col gap-3">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<h2 class="text-xs font-bold text-secondary uppercase tracking-widest m-0">
						Installed Instances
					</h2>
					<span class="text-xs text-secondary font-medium">({{ instances.length }})</span>
				</div>
				<div class="flex items-center gap-2">
					<Button
						type="outlined"
						size="xs"
						to="/browse/modpack"
					>
						<CompassIcon class="size-3.5" />
						ModPackInstaller
					</Button>
					<Button
						color="brand"
						size="xs"
						@click="showCreationModal?.()"
					>
						<PlusIcon class="size-3.5" />
						New Instance
					</Button>
				</div>
			</div>

			<div v-if="instances.length === 0" class="rounded-2xl border border-surface-5 bg-surface-2 p-8 text-center flex flex-col items-center gap-4">
				<p class="text-base text-secondary m-0">
					No instances installed yet. Install a modpack from ModPackInstaller or create your custom instance!
				</p>
				<div class="flex items-center gap-3">
					<Button color="brand" to="/browse/modpack">
						<CompassIcon />
						Explore Modpacks
					</Button>
					<Button type="outlined" @click="showCreationModal?.()">
						<PlusIcon />
						Create Instance
					</Button>
				</div>
			</div>

			<div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
				<div
					v-for="instance in recentInstances.slice(0, 6)"
					:key="instance.id"
					class="group relative rounded-2xl border border-solid border-surface-5 bg-surface-2 p-4 flex items-center justify-between gap-3 hover:border-brand/50 hover:bg-surface-3 transition-all cursor-pointer shadow-sm"
					@click="router.push(`/instance/${encodeURIComponent(instance.id)}`)"
				>
					<div class="flex items-center gap-3 min-w-0">
						<img
							:src="getInstanceIconUrl(instance.icon_path)"
							alt=""
							class="size-11 rounded-xl object-cover bg-surface-4 shrink-0 border border-surface-5"
						/>
						<div class="flex flex-col min-w-0">
							<span class="text-sm font-bold text-contrast truncate group-hover:text-brand transition-colors">
								{{ instance.name }}
							</span>
							<span class="text-xs text-secondary capitalize">
								{{ instance.loader }} {{ instance.game_version }}
							</span>
						</div>
					</div>

					<Button
						color="brand"
						size="sm"
						class="shrink-0 !px-3"
						:disabled="launchingInstanceId === instance.id"
						@click.stop="handleLaunch(instance)"
					>
						<SpinnerIcon v-if="launchingInstanceId === instance.id" class="animate-spin" />
						<PlayIcon v-else class="size-4" />
						Play
					</Button>
				</div>
			</div>
		</section>

		<!-- 4. Solara News Section -->
		<section v-if="newsList.length > 0" class="news-section flex flex-col gap-3">
			<div class="flex items-center justify-between">
				<h2 class="text-xs font-bold text-secondary uppercase tracking-widest m-0">
					Solara News & Announcements
				</h2>
			</div>

			<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
				<a
					v-for="article in newsList"
					:key="article.path"
					:href="article.path"
					target="_blank"
					rel="noreferrer"
					class="rounded-2xl border border-solid border-surface-5 bg-surface-2 p-4 flex gap-4 hover:border-brand/50 hover:bg-surface-3 transition-all no-underline shadow-sm group"
				>
					<img
						v-if="article.thumbnail"
						:src="article.thumbnail"
						alt=""
						class="size-16 rounded-xl object-cover bg-surface-4 shrink-0 border border-surface-5"
					/>
					<div class="flex flex-col justify-center min-w-0">
						<span class="text-xs text-secondary font-medium mb-1">
							{{ formatDate(article.date) }}
						</span>
						<span class="text-sm font-bold text-contrast group-hover:text-brand transition-colors truncate">
							{{ article.title }}
						</span>
						<p class="text-xs text-secondary m-0 mt-1 line-clamp-2">
							{{ article.summary }}
						</p>
					</div>
				</a>
			</div>
		</section>

		<!-- 5. Version History & Changelogs Section -->
		<section class="version-history-section flex flex-col gap-4">
			<div class="flex items-center justify-between">
				<h2 class="text-xs font-bold text-secondary uppercase tracking-widest m-0">
					Version History & Detailed Changelogs
				</h2>
			</div>

			<div class="flex flex-col gap-4">
				<div
					v-for="release in VERIFIED_VERSION_HISTORY"
					:key="release.version"
					class="rounded-2xl border border-solid border-surface-5 bg-surface-2 p-5 flex flex-col gap-4 shadow-sm"
				>
					<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-0 border-b border-solid border-surface-5 pb-3">
						<div class="flex items-center gap-2.5">
							<span
								class="px-2.5 py-1 rounded-lg text-xs font-black"
								:class="release.isLatest ? 'bg-brand text-[var(--color-accent-contrast)]' : 'bg-surface-4 text-contrast'"
							>
								{{ release.version }}
							</span>
							<span class="text-base font-bold text-contrast">{{ release.title }}</span>
						</div>
						<div class="flex items-center gap-2 text-xs text-secondary">
							<span>{{ release.date }}</span>
							<span>•</span>
							<span class="capitalize">{{ release.channel }}</span>
						</div>
					</div>

					<div class="grid grid-cols-1 md:grid-cols-3 gap-3">
						<!-- Features Added -->
						<div class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-3.5">
							<h3 class="text-contrast font-bold text-xs uppercase tracking-wider mb-2 m-0">
								Features Added
							</h3>
							<ul class="text-xs text-primary space-y-1.5 pl-4 list-disc m-0">
								<li v-for="feat in release.featuresAdded" :key="feat">{{ feat }}</li>
							</ul>
						</div>

						<!-- Issues Fixed -->
						<div class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-3.5">
							<h3 class="text-contrast font-bold text-xs uppercase tracking-wider mb-2 m-0">
								Issues Fixed
							</h3>
							<ul class="text-xs text-primary space-y-1.5 pl-4 list-disc m-0">
								<li v-for="fix in release.issuesFixed" :key="fix">{{ fix }}</li>
							</ul>
						</div>

						<!-- Future Updates Implementation -->
						<div class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-3.5">
							<h3 class="text-contrast font-bold text-xs uppercase tracking-wider mb-2 m-0">
								Future Updates Implementation
							</h3>
							<ul class="text-xs text-primary space-y-1.5 pl-4 list-disc m-0">
								<li v-for="fut in release.futureUpdates" :key="fut">{{ fut }}</li>
							</ul>
						</div>
					</div>
				</div>
			</div>
		</section>
	</div>
</template>
