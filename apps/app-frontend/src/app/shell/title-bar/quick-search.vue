<script setup lang="ts">
import {
	HomeIcon,
	LibraryIcon,
	PackageIcon,
	PaletteIcon,
	PlayIcon,
	PlusIcon,
	SearchIcon,
	SettingsIcon,
	StopCircleIcon,
	XIcon,
} from '@modrinth/assets'
import { Avatar, injectNotificationManager } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { useHiddenInstances } from '@/features/library/hidden-instances/use-hidden-instances'
import { getInstanceIconUrl, kill, run } from '@/helpers/instance'
import { get_all } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'
import { instanceListQueryOptions } from '@/pages/instance/query-options'

const emit = defineEmits<{
	'create-instance': []
	'open-settings': []
}>()

const router = useRouter()
const { handleError } = injectNotificationManager()
const instancesQuery = useQuery(instanceListQueryOptions())
const { hiddenIds } = useHiddenInstances()

const rootRef = ref<HTMLElement | null>(null)
const inputRef = ref<HTMLInputElement | null>(null)
const open = ref(false)
const query = ref('')
const runningIds = ref<string[]>([])

const visibleInstances = computed<GameInstance[]>(() =>
	((instancesQuery.data.value ?? []) as GameInstance[]).filter(
		(inst) => !hiddenIds.value.has(inst.id),
	),
)

const matchedInstances = computed(() => {
	const q = query.value.trim().toLowerCase()
	if (!q) {
		return visibleInstances.value.slice(0, 3)
	}
	return visibleInstances.value
		.filter(
			(inst) =>
				inst.name.toLowerCase().includes(q) ||
				inst.loader.toLowerCase().includes(q) ||
				inst.game_version.toLowerCase().includes(q),
		)
		.slice(0, 4)
})

const quickNavItems = computed(() => {
	const items = [
		{ id: 'new', label: 'Create New Instance', icon: PlusIcon, action: () => emit('create-instance') },
		{ id: 'modpacks', label: 'Browse Modpacks', icon: PackageIcon, action: () => router.push('/browse/modpack') },
		{ id: 'library', label: 'Open Library', icon: LibraryIcon, action: () => router.push('/library') },
		{ id: 'skins', label: 'Open Skin Studio', icon: PaletteIcon, action: () => router.push('/skins') },
		{ id: 'home', label: 'Go to Home', icon: HomeIcon, action: () => router.push('/') },
		{ id: 'settings', label: 'Open Settings', icon: SettingsIcon, action: () => emit('open-settings') },
	]
	const q = query.value.trim().toLowerCase()
	if (!q) return items.slice(0, 4)
	return items.filter((item) => item.label.toLowerCase().includes(q)).slice(0, 4)
})

function closeDropdown() {
	open.value = false
	inputRef.value?.blur()
}

function focusSearch() {
	open.value = true
	nextTick(() => {
		inputRef.value?.focus()
		inputRef.value?.select()
	})
}

function searchCatalog(source: 'modrinth' | 'curseforge') {
	const q = query.value.trim()
	void router.push({
		path: '/browse/modpack',
		query: q ? { source, q } : { source },
	})
	closeDropdown()
}

function handleEnter() {
	const q = query.value.trim()
	if (q) {
		searchCatalog('modrinth')
		return
	}
	if (matchedInstances.value.length > 0) {
		openInstance(matchedInstances.value[0].id)
	} else {
		searchCatalog('modrinth')
	}
}

function openInstance(id: string) {
	void router.push(`/instance/${encodeURIComponent(id)}`)
	closeDropdown()
}

function runNavItem(action: () => unknown) {
	void action()
	closeDropdown()
}

async function refreshProcesses() {
	const list = ((await get_all().catch(handleError)) ?? []) as Array<{ instance_id: string }>
	runningIds.value = list.map((p) => p.instance_id)
}

async function toggleInstancePlay(instance: GameInstance) {
	if (runningIds.value.includes(instance.id)) {
		await kill(instance.id).catch(handleError)
	} else if (!instance.launch_blocked && instance.install_stage === 'installed') {
		await run(instance.id).catch((err) => handleSevereError(err, { instanceId: instance.id }))
	}
	await refreshProcesses()
}

function handleGlobalKeydown(event: KeyboardEvent) {
	if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
		event.preventDefault()
		if (open.value) {
			closeDropdown()
		} else {
			focusSearch()
		}
	} else if (event.key === 'Escape' && open.value) {
		event.preventDefault()
		closeDropdown()
	}
}

function handleOutsideClick(event: MouseEvent) {
	if (open.value && rootRef.value && !rootRef.value.contains(event.target as Node)) {
		open.value = false
	}
}

useAppEvent('process', refreshProcesses)

onMounted(() => {
	void refreshProcesses()
	window.addEventListener('keydown', handleGlobalKeydown)
	document.addEventListener('mousedown', handleOutsideClick)
})

onBeforeUnmount(() => {
	window.removeEventListener('keydown', handleGlobalKeydown)
	document.removeEventListener('mousedown', handleOutsideClick)
})
</script>

<template>
	<div
		ref="rootRef"
		data-tauri-drag-region-exclude
		class="relative w-full max-w-[24rem] xl:max-w-[28rem]"
	>
		<div
			class="solara-quick-search-pill flex h-8.5 w-full items-center gap-2 rounded-xl px-3 transition-all duration-200"
			:class="{ 'is-focused': open }"
			@click="open = true"
		>
			<SearchIcon class="size-3.5 shrink-0 text-brand" />
			<input
				ref="inputRef"
				v-model="query"
				type="text"
				placeholder="Quick search modpacks, instances, actions..."
				aria-label="Quick search"
				class="min-w-0 flex-1 border-0 bg-transparent p-0 text-xs font-medium text-contrast placeholder:text-secondary focus:outline-none"
				@focus="open = true"
				@keydown.enter.prevent="handleEnter"
			/>
			<button
				v-if="query"
				type="button"
				aria-label="Clear search"
				class="flex size-4 shrink-0 cursor-pointer items-center justify-center rounded border-0 bg-transparent p-0 text-secondary hover:text-contrast"
				@click.stop="query = ''"
			>
				<XIcon class="size-3" />
			</button>
			<kbd
				v-else
				class="solara-kbd hidden sm:inline-flex shrink-0 items-center rounded-md px-1.5 py-0.5 text-[10px] font-bold tracking-wide text-secondary select-none"
			>
				Ctrl K
			</kbd>
		</div>

		<Transition
			enter-active-class="transition duration-150 ease-out"
			enter-from-class="opacity-0 -translate-y-1 scale-[0.98]"
			enter-to-class="opacity-100 translate-y-0 scale-100"
			leave-active-class="transition duration-100 ease-in"
			leave-from-class="opacity-100 translate-y-0 scale-100"
			leave-to-class="opacity-0 -translate-y-1 scale-[0.98]"
		>
			<div
				v-if="open"
				class="solara-search-dropdown absolute left-0 right-0 top-[calc(100%+0.45rem)] z-[10020] flex max-h-[24rem] flex-col gap-2 overflow-y-auto rounded-2xl p-2.5 shadow-2xl"
			>
				<!-- Search Modpack Catalogs -->
				<div class="flex flex-col gap-1">
					<div class="px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-secondary">
						Modpack Catalog
					</div>
					<button
						type="button"
						class="flex w-full cursor-pointer items-center justify-between gap-2 rounded-xl border-0 bg-transparent px-2.5 py-2 text-left transition-colors hover:bg-brand/12"
						@click="searchCatalog('modrinth')"
					>
						<span class="flex items-center gap-2 text-xs font-semibold text-contrast min-w-0">
							<SearchIcon class="size-3.5 shrink-0 text-brand" />
							<span class="truncate">
								{{ query.trim() ? `Search "${query.trim()}" in Modpacks` : 'Browse Popular Modpacks' }}
							</span>
						</span>
						<span class="rounded-md bg-brand/15 px-1.5 py-0.5 text-[10px] font-bold text-brand shrink-0">
							Enter
						</span>
					</button>
					<button
						v-if="query.trim()"
						type="button"
						class="flex w-full cursor-pointer items-center justify-between gap-2 rounded-xl border-0 bg-transparent px-2.5 py-2 text-left transition-colors hover:bg-surface-4"
						@click="searchCatalog('curseforge')"
					>
						<span class="flex items-center gap-2 text-xs font-semibold text-contrast min-w-0">
							<PackageIcon class="size-3.5 shrink-0 text-orange" />
							<span class="truncate">Search "{{ query.trim() }}" on CurseForge</span>
						</span>
						<span class="text-[10px] font-semibold text-secondary shrink-0">CurseForge</span>
					</button>
				</div>

				<!-- Matching Installed Instances -->
				<div v-if="matchedInstances.length > 0" class="flex flex-col gap-1 border-0 border-t border-solid border-surface-5/60 pt-2">
					<div class="px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-secondary">
						Your Instances
					</div>
					<div
						v-for="inst in matchedInstances"
						:key="inst.id"
						class="flex items-center justify-between gap-2 rounded-xl px-2 py-1.5 transition-colors hover:bg-surface-4"
					>
						<button
							type="button"
							class="flex min-w-0 flex-1 cursor-pointer items-center gap-2.5 border-0 bg-transparent p-0 text-left"
							@click="openInstance(inst.id)"
						>
							<Avatar
								:src="getInstanceIconUrl(inst.icon_path)"
								size="24px"
								:tint-by="inst.id"
								class="shrink-0 rounded-md"
								pad-transparent-corners
							/>
							<span class="flex min-w-0 flex-col">
								<span class="truncate text-xs font-bold text-contrast">{{ inst.name }}</span>
								<span class="truncate text-[11px] capitalize text-secondary">
									{{ inst.loader }} {{ inst.game_version }}
								</span>
							</span>
						</button>
						<button
							type="button"
							:title="runningIds.includes(inst.id) ? 'Stop' : 'Play'"
							class="flex size-7 shrink-0 cursor-pointer items-center justify-center rounded-lg border-0 transition-colors"
							:class="
								runningIds.includes(inst.id)
									? 'bg-red/15 text-red hover:bg-red/25'
									: 'bg-brand/15 text-brand hover:bg-brand hover:text-[var(--color-accent-contrast)]'
							"
							@click.stop="toggleInstancePlay(inst)"
						>
							<StopCircleIcon v-if="runningIds.includes(inst.id)" class="size-4" />
							<PlayIcon v-else class="size-3.5" />
						</button>
					</div>
				</div>

				<!-- Quick Actions -->
				<div v-if="quickNavItems.length > 0" class="flex flex-col gap-1 border-0 border-t border-solid border-surface-5/60 pt-2">
					<div class="px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider text-secondary">
						Quick Actions
					</div>
					<div class="grid grid-cols-2 gap-1">
						<button
							v-for="nav in quickNavItems"
							:key="nav.id"
							type="button"
							class="flex cursor-pointer items-center gap-2 rounded-xl border-0 bg-transparent px-2.5 py-1.5 text-left text-xs font-semibold text-primary transition-colors hover:bg-surface-4 hover:text-contrast"
							@click="runNavItem(nav.action)"
						>
							<component :is="nav.icon" class="size-3.5 shrink-0 text-brand" />
							<span class="truncate">{{ nav.label }}</span>
						</button>
					</div>
				</div>
			</div>
		</Transition>
	</div>
</template>

<style scoped>
.h-8\.5 {
	height: 2.125rem;
}

.solara-quick-search-pill {
	background-color: color-mix(in srgb, var(--color-bg) 76%, transparent);
	border: 1px solid var(--surface-4);
	box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
}

.solara-quick-search-pill:hover {
	border-color: var(--surface-5);
}

.solara-quick-search-pill.is-focused {
	border-color: color-mix(in srgb, var(--color-brand) 55%, transparent);
	box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-brand) 16%, transparent);
}

.solara-kbd {
	background-color: color-mix(in srgb, var(--color-raised-bg) 88%, transparent);
	border: 1px solid var(--surface-4);
}

.solara-search-dropdown {
	background-color: color-mix(in srgb, var(--color-bg) 95%, var(--color-raised-bg));
	border: 1px solid var(--surface-5);
	backdrop-filter: blur(18px);
}
</style>
