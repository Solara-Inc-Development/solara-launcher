<script setup lang="ts">
import {
	LibraryIcon,
	PackageIcon,
	PlusIcon,
	SearchIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { Button, Combobox } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, inject, ref } from 'vue'
import { useRouter } from 'vue-router'

import Instance from '@/components/ui/Instance.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { useHiddenInstances } from '@/features/library/hidden-instances/use-hidden-instances'
import { instanceKeys, instanceListQueryOptions } from '@/pages/instance/query-options'
import { useRootBreadcrumb } from '@/shared/breadcrumbs'

defineOptions({
	name: 'LibraryPage',
})

useRootBreadcrumb({
	slot: 'root',
	id: 'library',
	label: 'Library',
	to: '/library',
	visual: { type: 'icon', component: LibraryIcon },
})

const router = useRouter()
const queryClient = useQueryClient()
const showCreationModal = inject<(() => void) | undefined>('showCreationModal', undefined)

const instancesQuery = useQuery(instanceListQueryOptions())
const { hiddenIds } = useHiddenInstances()

useAppEvent('instance', () => {
	void queryClient.invalidateQueries({ queryKey: instanceKeys.list() })
})

const searchQuery = ref('')
const selectedLoader = ref<string>('all')
const sortBy = ref<'recent' | 'name' | 'created'>('recent')
const SORT_OPTIONS: { value: 'recent' | 'name' | 'created'; label: string }[] = [
	{ value: 'recent', label: 'Recently played' },
	{ value: 'name', label: 'Name (A–Z)' },
	{ value: 'created', label: 'Date created' },
]

const visibleInstances = computed(() =>
	(instancesQuery.data.value ?? []).filter((instance) => !hiddenIds.value.has(instance.id)),
)

const availableLoaders = computed(() => {
	const set = new Set<string>()
	for (const inst of visibleInstances.value) {
		if (inst.loader) set.add(inst.loader.toLowerCase())
	}
	return ['all', ...Array.from(set).sort()]
})

const filteredInstances = computed(() => {
	const q = searchQuery.value.trim().toLowerCase()
	const loader = selectedLoader.value

	const list = visibleInstances.value.filter((inst) => {
		if (loader !== 'all' && inst.loader.toLowerCase() !== loader) {
			return false
		}
		if (!q) return true
		return (
			inst.name.toLowerCase().includes(q) ||
			inst.game_version.toLowerCase().includes(q) ||
			inst.loader.toLowerCase().includes(q)
		)
	})

	return [...list].sort((a, b) => {
		if (sortBy.value === 'name') {
			return a.name.localeCompare(b.name)
		}
		if (sortBy.value === 'created') {
			return dayjs(b.created).valueOf() - dayjs(a.created).valueOf()
		}
		const aTime = Math.max(
			dayjs(a.created).valueOf(),
			a.last_played ? dayjs(a.last_played).valueOf() : 0,
		)
		const bTime = Math.max(
			dayjs(b.created).valueOf(),
			b.last_played ? dayjs(b.last_played).valueOf() : 0,
		)
		if (aTime === bTime) return a.name.localeCompare(b.name)
		return bTime - aTime
	})
})

function formatLoaderLabel(loader: string): string {
	if (loader === 'all') return 'All'
	if (loader === 'neoforge') return 'NeoForge'
	return loader.charAt(0).toUpperCase() + loader.slice(1)
}

function clearFilters() {
	searchQuery.value = ''
	selectedLoader.value = 'all'
}

function openCreateInstance() {
	showCreationModal?.()
}

function openBrowseModpacks() {
	void router.push('/browse/modpack')
}
</script>

<template>
	<div class="solara-library-page w-full max-w-6xl mx-auto px-6 md:px-10 py-7 flex flex-col gap-6">
		<!-- Header -->
		<header class="flex flex-wrap items-center justify-between gap-4">
			<div class="flex items-center gap-3.5">
				<div
					class="size-11 rounded-2xl border border-solid border-brand/30 bg-brand/10 text-brand flex items-center justify-center shrink-0"
				>
					<LibraryIcon class="size-5" />
				</div>
				<div class="flex flex-col gap-0.5">
					<div class="flex items-center gap-2.5">
						<h1 class="text-2xl font-extrabold text-contrast tracking-tight m-0">Library</h1>
						<span
							class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-bold bg-bg-raised text-secondary tabular-nums"
						>
							{{ visibleInstances.length }}
							{{ visibleInstances.length === 1 ? 'Instance' : 'Instances' }}
						</span>
					</div>
					<p class="text-sm text-secondary m-0">
						Launch, manage, and organize your Minecraft instances.
					</p>
				</div>
			</div>

			<div class="flex items-center gap-2.5">
				<Button type="outlined" @click="openBrowseModpacks">
					<PackageIcon class="size-4" />
					<span>Browse Modpacks</span>
				</Button>
				<Button color="brand" @click="openCreateInstance">
					<PlusIcon class="size-4" />
					<span>New Instance</span>
				</Button>
			</div>
		</header>

		<!-- Pure-OLED Open Search & Filter Strip -->
		<section
			v-if="visibleInstances.length > 0"
			class="flex flex-wrap items-center justify-between gap-3"
		>
			<div class="relative flex-1 min-w-[14rem] max-w-md">
				<SearchIcon
					class="pointer-events-none absolute left-3.5 top-1/2 -translate-y-1/2 size-4 text-secondary"
				/>
				<input
					v-model="searchQuery"
					type="text"
					placeholder="Search instances, versions, or loaders..."
					aria-label="Search instances"
					class="solara-oled-input w-full rounded-xl bg-bg-raised py-2.5 pl-10 pr-8 text-sm text-contrast placeholder:text-secondary focus:outline-none transition-all"
				/>
				<button
					v-if="searchQuery"
					type="button"
					aria-label="Clear search"
					class="absolute right-2.5 top-1/2 -translate-y-1/2 p-0.5 rounded bg-transparent border-0 text-secondary hover:text-contrast cursor-pointer flex"
					@click="searchQuery = ''"
				>
					<XIcon class="size-3.5" />
				</button>
			</div>

			<div class="flex flex-wrap items-center gap-2">
				<div class="flex flex-wrap items-center gap-1.5">
					<button
						v-for="loader in availableLoaders"
						:key="loader"
						type="button"
						class="px-3 py-1.5 rounded-xl text-xs font-semibold border-0 cursor-pointer transition-all duration-150"
						:class="
							selectedLoader === loader
								? 'bg-brand text-[var(--color-accent-contrast)] shadow-sm'
								: 'bg-bg-raised text-secondary hover:bg-surface-4 hover:text-contrast'
						"
						@click="selectedLoader = loader"
					>
						{{ formatLoaderLabel(loader) }}
					</button>
				</div>

				<div class="w-40 shrink-0">
					<Combobox
						v-model="sortBy"
						:options="SORT_OPTIONS"
						placeholder="Sort"
						trigger-size="sm"
					/>
				</div>
			</div>
		</section>

		<!-- Loading state -->
		<div
			v-if="instancesQuery.isPending.value && visibleInstances.length === 0"
			class="flex flex-col items-center justify-center gap-3 py-20 text-secondary"
		>
			<SpinnerIcon class="size-7 animate-spin text-brand" />
			<span class="text-sm font-medium">Loading your instances...</span>
		</div>

		<!-- Empty library state -->
		<div
			v-else-if="visibleInstances.length === 0"
			class="rounded-3xl bg-bg-raised px-6 py-16 flex flex-col items-center justify-center text-center gap-4"
		>
			<div
				class="size-16 rounded-2xl bg-brand/12 text-brand flex items-center justify-center"
			>
				<LibraryIcon class="size-8" />
			</div>
			<div class="flex flex-col gap-1.5 max-w-md">
				<h2 class="text-xl font-extrabold text-contrast m-0">No instances yet</h2>
				<p class="text-sm text-secondary m-0 leading-relaxed">
					Create your first Minecraft instance or install a modpack from ModPackInstaller to see it
					in your Library.
				</p>
			</div>
			<div class="flex flex-wrap items-center justify-center gap-3 mt-1">
				<Button color="brand" @click="openCreateInstance">
					<PlusIcon class="size-4" />
					<span>Create Instance</span>
				</Button>
				<Button type="outlined" @click="openBrowseModpacks">
					<PackageIcon class="size-4" />
					<span>Browse Modpacks</span>
				</Button>
			</div>
		</div>

		<!-- No search matches -->
		<div
			v-else-if="filteredInstances.length === 0"
			class="rounded-2xl bg-bg-raised px-6 py-12 flex flex-col items-center justify-center text-center gap-3"
		>
			<p class="text-base font-bold text-contrast m-0">No matching instances</p>
			<p class="text-sm text-secondary m-0">
				Try a different search query or reset your loader filter.
			</p>
			<Button type="outlined" size="small" @click="clearFilters">Clear filters</Button>
		</div>

		<!-- Smooth Pure-OLED Instances Grid -->
		<section
			v-else
			class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3.5"
		>
			<div
				v-for="(instance, idx) in filteredInstances"
				:key="instance.id"
				:style="{ animationDelay: `${Math.min(idx, 12) * 24}ms` }"
				class="solara-library-card solara-card-animate rounded-2xl overflow-hidden"
			>
				<Instance :instance="instance" />
			</div>
		</section>
	</div>
</template>

<style scoped>
.solara-oled-input {
	border: 1px solid transparent;
}

.solara-oled-input:focus {
	border-color: color-mix(in srgb, var(--color-brand) 60%, transparent);
}

.solara-library-card {
	transition:
		transform 0.22s cubic-bezier(0.22, 1, 0.36, 1),
		box-shadow 0.22s ease;
}

.solara-library-card:hover {
	transform: translateY(-2px);
	box-shadow: 0 12px 28px -16px rgba(0, 0, 0, 0.75);
}
</style>
