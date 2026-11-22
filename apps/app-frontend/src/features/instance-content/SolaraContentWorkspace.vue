<script setup lang="ts">
import {
	CheckIcon,
	CompassIcon,
	DownloadIcon,
	FolderOpenIcon,
	GridIcon,
	ListIcon,
	LockIcon,
	MoreVerticalIcon,
	PackageIcon,
	RefreshCwIcon,
	SearchIcon,
	SettingsIcon,
	SpinnerIcon,
	TrashIcon,
	UnlinkIcon,
	UpdatedIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	Checkbox,
	type ContentItem,
	injectContentManager,
	TeleportOverflowMenu,
} from '@modrinth/ui'
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'

const props = defineProps<{
	highlightedItemId?: string
}>()

const ctx = injectContentManager()

type ViewMode = 'grid' | 'list'
type FilterTab = 'all' | 'mod' | 'resourcepack' | 'shader' | 'datapack' | 'enabled' | 'disabled' | 'updates'
type SortKey = 'name' | 'status' | 'type'

const viewMode = ref<ViewMode>('grid')
const searchQuery = ref('')
const activeFilter = ref<FilterTab>('all')
const sortBy = ref<SortKey>('name')
const selectedIds = ref<string[]>([])
const refreshing = ref(false)
const updatingAll = ref(false)
const pendingDeleteItem = ref<ContentItem | null>(null)
const pendingBulkDelete = ref(false)
const pendingUnlink = ref(false)

const allItems = computed(() => ctx.items.value ?? [])

const stats = computed(() => {
	let enabled = 0
	let disabled = 0
	let updates = 0
	for (const item of allItems.value) {
		if (item.enabled) enabled++
		else disabled++
		if (item.has_update && !item.locked) updates++
	}
	return {
		total: allItems.value.length,
		enabled,
		disabled,
		updates,
	}
})

const presentTypes = computed(() => {
	const set = new Set<string>()
	for (const item of allItems.value) {
		if (item.project_type) set.add(item.project_type)
	}
	return set
})

const filterTabs = computed<{ id: FilterTab; label: string; count?: number }[]>(() => {
	const list: { id: FilterTab; label: string; count?: number }[] = [
		{ id: 'all', label: 'All', count: stats.value.total },
	]
	if (presentTypes.value.has('mod')) list.push({ id: 'mod', label: 'Mods' })
	if (presentTypes.value.has('resourcepack')) list.push({ id: 'resourcepack', label: 'Resource Packs' })
	if (presentTypes.value.has('shader')) list.push({ id: 'shader', label: 'Shaders' })
	if (presentTypes.value.has('datapack')) list.push({ id: 'datapack', label: 'Datapacks' })
	list.push({ id: 'enabled', label: 'Active', count: stats.value.enabled })
	if (stats.value.disabled > 0) list.push({ id: 'disabled', label: 'Disabled', count: stats.value.disabled })
	if (stats.value.updates > 0) list.push({ id: 'updates', label: 'Updates', count: stats.value.updates })
	return list
})

function itemTitle(item: ContentItem): string {
	return item.project?.title ?? item.embedded_metadata?.name ?? item.file_name.replace(/\.disabled$/i, '')
}

function itemAuthor(item: ContentItem): string {
	return item.owner?.name ?? item.embedded_metadata?.authors?.[0] ?? 'Local file'
}

function itemVersion(item: ContentItem): string {
	return item.version?.version_number ?? item.embedded_metadata?.version ?? item.file_name
}

function itemIcon(item: ContentItem): string | null {
	return item.project?.icon_url ?? item.embedded_metadata?.icon_url ?? null
}

function itemTypeLabel(type?: string): string {
	switch (type) {
		case 'mod':
			return 'Mod'
		case 'resourcepack':
			return 'Resource Pack'
		case 'shader':
			return 'Shader'
		case 'datapack':
			return 'Datapack'
		default:
			return 'Addon'
	}
}

function itemTypeClass(type?: string): string {
	switch (type) {
		case 'mod':
			return 'border-indigo-500/30 bg-indigo-500/12 text-indigo-300'
		case 'shader':
			return 'border-amber-500/30 bg-amber-500/12 text-amber-300'
		case 'resourcepack':
			return 'border-sky-500/30 bg-sky-500/12 text-sky-300'
		case 'datapack':
			return 'border-purple-500/30 bg-purple-500/12 text-purple-300'
		default:
			return 'border-surface-5 bg-surface-3 text-secondary'
	}
}

const filteredItems = computed(() => {
	const q = searchQuery.value.trim().toLowerCase()
	let list = allItems.value.filter((item) => {
		if (activeFilter.value === 'enabled' && !item.enabled) return false
		if (activeFilter.value === 'disabled' && item.enabled) return false
		if (activeFilter.value === 'updates' && !(item.has_update && !item.locked)) return false
		if (
			['mod', 'resourcepack', 'shader', 'datapack'].includes(activeFilter.value) &&
			item.project_type !== activeFilter.value
		) {
			return false
		}
		if (!q) return true
		return (
			itemTitle(item).toLowerCase().includes(q) ||
			itemAuthor(item).toLowerCase().includes(q) ||
			item.file_name.toLowerCase().includes(q)
		)
	})

	list = [...list].sort((a, b) => {
		if (sortBy.value === 'status') {
			if (a.enabled !== b.enabled) return a.enabled ? -1 : 1
		} else if (sortBy.value === 'type') {
			const cmp = (a.project_type ?? '').localeCompare(b.project_type ?? '')
			if (cmp !== 0) return cmp
		}
		return itemTitle(a).localeCompare(itemTitle(b))
	})

	return list
})

const selectedSet = computed(() => new Set(selectedIds.value))
const selectedItems = computed(() =>
	allItems.value.filter((item) => selectedSet.value.has(ctx.getItemId(item))),
)

function toggleSelect(item: ContentItem) {
	const id = ctx.getItemId(item)
	if (selectedSet.value.has(id)) {
		selectedIds.value = selectedIds.value.filter((x) => x !== id)
	} else {
		selectedIds.value = [...selectedIds.value, id]
	}
}

function clearSelection() {
	selectedIds.value = []
}

function canToggle(item: ContentItem): boolean {
	return ctx.canToggleItem ? ctx.canToggleItem(item) : true
}

function canDelete(item: ContentItem): boolean {
	return ctx.canDeleteItem ? ctx.canDeleteItem(item) : true
}

function handleToggle(item: ContentItem) {
	if (!canToggle(item) || ctx.isBusy.value || item.installing) return
	void ctx.toggleEnabled(item, !item.enabled)
}

async function handleRefresh() {
	refreshing.value = true
	try {
		await ctx.refresh()
	} finally {
		refreshing.value = false
	}
}

async function handleUpdateAll() {
	if (!ctx.bulkUpdateSelections || updatingAll.value) return
	const outdated = allItems.value.filter((item) => item.has_update && !item.locked && item.update_version_id)
	if (outdated.length === 0) return
	updatingAll.value = true
	try {
		await ctx.bulkUpdateSelections(
			outdated.map((item) => ({
				item,
				versionId: item.update_version_id!,
			})),
		)
	} finally {
		updatingAll.value = false
	}
}

async function confirmSingleDelete() {
	const target = pendingDeleteItem.value
	pendingDeleteItem.value = null
	if (!target) return
	await ctx.deleteItem(target)
	selectedIds.value = selectedIds.value.filter((id) => id !== ctx.getItemId(target))
}

async function confirmBulkDelete() {
	pendingBulkDelete.value = false
	const targets = selectedItems.value.filter((i) => canDelete(i))
	if (targets.length === 0) return
	await ctx.bulkDeleteItems(targets)
	clearSelection()
}

async function handleBulkEnable() {
	await ctx.bulkEnableItems(selectedItems.value)
	clearSelection()
}

async function handleBulkDisable() {
	await ctx.bulkDisableItems(selectedItems.value)
	clearSelection()
}

async function handleConfirmUnlink() {
	pendingUnlink.value = false
	await ctx.unlinkModpack?.()
}

function getCardOverflow(item: ContentItem) {
	return ctx.getOverflowOptions?.(item) ?? []
}

function itemProjectLink(item: ContentItem) {
	const mapped = ctx.mapToTableItem(item)
	return mapped.projectLink
}

function itemCanSwitchVersion(item: ContentItem) {
	const mapped = ctx.mapToTableItem(item)
	return !mapped.hideSwitchVersion && !!ctx.switchVersion
}
</script>

<template>
	<div class="flex flex-col gap-4 pb-10">
		<!-- Linked Modpack Banner (Custom Solara Card) -->
		<div
			v-if="ctx.managedContent.value"
			class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-solid border-brand/30 bg-surface-2 px-5 py-4"
		>
			<div class="flex min-w-0 items-center gap-3.5">
				<Avatar
					:src="ctx.managedContent.value.card.manager.iconUrl"
					:alt="ctx.managedContent.value.card.manager.name"
					size="48px"
					class="rounded-xl ring-1 ring-surface-5"
				/>
				<div class="flex min-w-0 flex-col gap-0.5">
					<div class="flex flex-wrap items-center gap-2">
						<span class="text-xs font-bold uppercase tracking-wider text-brand">Linked Modpack</span>
						<span
							v-if="ctx.managedContent.value.card.manager.version"
							class="rounded-md bg-surface-3 px-2 py-0.5 text-xs font-semibold text-secondary"
						>
							{{ ctx.managedContent.value.card.manager.version }}
						</span>
					</div>
					<h3 class="m-0 truncate text-base font-bold text-contrast">
						{{ ctx.managedContent.value.card.manager.name }}
					</h3>
				</div>
			</div>

			<div class="flex flex-wrap items-center gap-2">
				<Button
					v-if="ctx.viewManagedContent"
					type="outlined"
					size="md"
					class="!rounded-xl"
					@click="ctx.viewManagedContent()"
				>
					<PackageIcon class="size-4" />
					<span>Pack Mods ({{ ctx.managedContent.value.items.length }})</span>
				</Button>
				<Button
					v-if="ctx.runManagedContentPrimaryAction && ctx.managedContent.value.card.hasUpdate"
					type="colored"
					color="green"
					size="md"
					class="!rounded-xl !font-bold"
					@click="ctx.runManagedContentPrimaryAction($event)"
				>
					<DownloadIcon class="size-4" />
					<span>Update Pack</span>
				</Button>
				<Button
					v-if="ctx.openManagedContentSettings"
					type="outlined"
					size="md"
					class="!rounded-xl"
					@click="ctx.openManagedContentSettings()"
				>
					<SettingsIcon class="size-4" />
					<span>Pack Settings</span>
				</Button>
				<Button
					v-if="ctx.unlinkModpack"
					type="quiet"
					color="red"
					size="md"
					class="!rounded-xl"
					@click="pendingUnlink = true"
				>
					<UnlinkIcon class="size-4" />
					<span>Unlink</span>
				</Button>
			</div>
		</div>

		<!-- Loading State -->
		<div
			v-if="ctx.loading.value"
			class="flex flex-col items-center justify-center gap-3 rounded-2xl border border-solid border-surface-4 bg-surface-2 py-16 text-secondary"
		>
			<SpinnerIcon class="size-7 animate-spin text-brand" />
			<span class="text-sm font-medium">Loading installed mods &amp; packs...</span>
		</div>

		<!-- Empty State -->
		<div
			v-else-if="allItems.length === 0"
			class="flex flex-col items-center justify-center gap-4 rounded-2xl border border-dashed border-surface-5 bg-surface-2 px-6 py-16 text-center"
		>
			<div class="flex size-14 items-center justify-center rounded-2xl bg-brand/12 text-brand">
				<PackageIcon class="size-7" />
			</div>
			<div class="flex flex-col gap-1">
				<h3 class="m-0 text-lg font-extrabold text-contrast">
					{{ ctx.managedContent.value ? 'No extra mods added yet' : 'No mods or packs installed yet' }}
				</h3>
				<p class="m-0 max-w-md text-sm text-secondary">
					Browse both Modrinth &amp; CurseForge catalogs in one place, or drag and drop local .jar / .zip files right here.
				</p>
			</div>
			<div class="flex flex-wrap items-center justify-center gap-2.5 pt-1">
				<Button
					type="outlined"
					size="lg"
					class="!rounded-xl"
					:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
					@click="ctx.uploadFiles"
				>
					<FolderOpenIcon class="size-5" />
					<span>Import .jar</span>
				</Button>
				<Button
					type="colored"
					color="brand"
					size="lg"
					class="!rounded-xl !font-bold"
					:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
					@click="ctx.browse"
				>
					<CompassIcon class="size-5" />
					<span>Install Mods (Modrinth + CurseForge)</span>
				</Button>
			</div>
		</div>

		<!-- Populated Solara Mods Workspace -->
		<template v-else>
			<!-- Top Compact Stats + Control Deck -->
			<div class="flex flex-col gap-3 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-4">
				<!-- Row 1: Stats Pills + Main Action Buttons -->
				<div class="flex flex-wrap items-center justify-between gap-3">
					<div class="flex flex-wrap items-center gap-2 text-xs font-semibold">
						<span class="rounded-lg border border-solid border-surface-4 bg-surface-3 px-2.5 py-1 text-contrast">
							{{ stats.total }} Installed
						</span>
						<span class="rounded-lg border border-solid border-emerald-500/25 bg-emerald-500/10 px-2.5 py-1 text-emerald-400">
							{{ stats.enabled }} Active
						</span>
						<span
							v-if="stats.disabled > 0"
							class="rounded-lg border border-solid border-surface-4 bg-surface-3 px-2.5 py-1 text-secondary"
						>
							{{ stats.disabled }} Disabled
						</span>
						<button
							v-if="stats.updates > 0"
							type="button"
							:disabled="updatingAll || ctx.isBusy.value"
							class="inline-flex cursor-pointer items-center gap-1.5 rounded-lg border border-solid border-emerald-500/40 bg-emerald-500/15 px-2.5 py-1 text-emerald-300 transition-colors hover:bg-emerald-500/25 disabled:opacity-50"
							@click="handleUpdateAll"
						>
							<DownloadIcon class="size-3.5" />
							<span>{{ updatingAll ? 'Updating...' : `Update All (${stats.updates})` }}</span>
						</button>
					</div>

					<div class="flex flex-wrap items-center gap-2">
						<Button
							type="outlined"
							size="md"
							class="!rounded-xl"
							:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
							@click="ctx.uploadFiles"
						>
							<FolderOpenIcon class="size-4" />
							<span>Import .jar</span>
						</Button>
						<Button
							type="colored"
							color="brand"
							size="md"
							class="!rounded-xl !font-bold"
							:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
							@click="ctx.browse"
						>
							<CompassIcon class="size-4" />
							<span>Install Mods</span>
						</Button>
						<button
							type="button"
							:disabled="refreshing"
							title="Reload installed content"
							class="inline-flex size-9 cursor-pointer items-center justify-center rounded-xl border border-solid border-surface-4 bg-surface-3 text-secondary transition-colors hover:text-contrast"
							@click="handleRefresh"
						>
							<RefreshCwIcon :class="['size-4', refreshing ? 'animate-spin' : '']" />
						</button>
						<div class="inline-flex items-center rounded-xl border border-solid border-surface-4 bg-surface-3 p-0.5">
							<button
								type="button"
								title="Card grid view"
								:class="[
									'inline-flex size-8 cursor-pointer items-center justify-center rounded-lg border-0 transition-colors',
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
								title="Compact list view"
								:class="[
									'inline-flex size-8 cursor-pointer items-center justify-center rounded-lg border-0 transition-colors',
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

				<!-- Row 2: Search + Filter Chips + Sort -->
				<div class="flex flex-wrap items-center gap-2.5">
					<div class="relative min-w-[220px] flex-1">
						<SearchIcon class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-secondary" />
						<input
							v-model="searchQuery"
							type="text"
							placeholder="Search installed mods, shaders, or packs..."
							class="h-9 w-full rounded-xl border border-solid border-surface-4 bg-surface-1 pl-9 pr-8 text-sm text-contrast outline-none transition-colors focus:border-brand"
						/>
						<button
							v-if="searchQuery"
							type="button"
							class="absolute right-2.5 top-1/2 -translate-y-1/2 cursor-pointer border-0 bg-transparent p-0 text-secondary hover:text-contrast"
							@click="searchQuery = ''"
						>
							<XIcon class="size-4" />
						</button>
					</div>

					<select
						v-model="sortBy"
						aria-label="Sort installed mods"
						class="h-9 cursor-pointer rounded-xl border border-solid border-surface-4 bg-surface-1 px-3 text-xs font-semibold text-contrast outline-none focus:border-brand"
					>
						<option value="name">Sort: Name A–Z</option>
						<option value="status">Sort: Active First</option>
						<option value="type">Sort: Content Type</option>
					</select>
				</div>

				<div class="flex flex-wrap items-center gap-1.5">
					<button
						v-for="tab in filterTabs"
						:key="tab.id"
						type="button"
						:class="[
							'inline-flex cursor-pointer items-center gap-1.5 rounded-lg border border-solid px-2.5 py-1 text-xs font-semibold transition-colors',
							activeFilter === tab.id
								? 'border-brand/50 bg-brand/15 text-brand'
								: 'border-surface-4 bg-surface-3 text-secondary hover:text-contrast',
						]"
						@click="activeFilter = tab.id"
					>
						<span>{{ tab.label }}</span>
						<span v-if="typeof tab.count === 'number'" class="opacity-70">{{ tab.count }}</span>
					</button>
				</div>
			</div>

			<!-- Filtered Empty -->
			<div
				v-if="filteredItems.length === 0"
				class="flex flex-col items-center justify-center gap-2 rounded-2xl border border-solid border-surface-4 bg-surface-2 py-12 text-center"
			>
				<p class="m-0 text-sm font-semibold text-contrast">No matching mods or packs found.</p>
				<button
					type="button"
					class="cursor-pointer rounded-lg border border-solid border-surface-5 bg-surface-3 px-3 py-1.5 text-xs font-semibold text-secondary hover:text-contrast"
					@click="
						() => {
							searchQuery = ''
							activeFilter = 'all'
						}
					"
				>
					Reset filters
				</button>
			</div>

			<!-- GRID VIEW (Signature Solara Cards!) -->
			<div
				v-else-if="viewMode === 'grid'"
				class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3"
			>
				<article
					v-for="item in filteredItems"
					:key="ctx.getItemId(item)"
					:class="[
						'group relative flex flex-col justify-between gap-3 rounded-2xl border border-solid p-4 transition-all',
						selectedSet.has(ctx.getItemId(item))
							? 'border-brand bg-brand/8'
							: item.file_path === props.highlightedItemId
								? 'border-brand bg-surface-2'
								: 'border-surface-4 bg-surface-2 hover:border-surface-5',
						!item.enabled ? 'opacity-65 hover:opacity-90' : '',
					]"
				>
					<!-- Card Header -->
					<div class="flex items-start gap-3">
						<div class="relative shrink-0">
							<Avatar
								:src="itemIcon(item)"
								:alt="itemTitle(item)"
								size="48px"
								class="rounded-xl ring-1 ring-surface-5"
							/>
							<span
								v-if="item.installing"
								class="absolute inset-0 flex items-center justify-center rounded-xl bg-black/65"
							>
								<SpinnerIcon class="size-5 animate-spin text-brand" />
							</span>
						</div>

						<div class="flex min-w-0 flex-1 flex-col gap-0.5">
							<div class="flex items-center gap-1.5">
								<RouterLink
									v-if="itemProjectLink(item)"
									:to="itemProjectLink(item)!"
									class="truncate text-sm font-bold text-contrast no-underline hover:text-brand"
								>
									{{ itemTitle(item) }}
								</RouterLink>
								<span v-else class="truncate text-sm font-bold text-contrast">
									{{ itemTitle(item) }}
								</span>
								<LockIcon v-if="item.locked" class="size-3.5 shrink-0 text-amber-400" />
							</div>
							<span class="truncate text-xs text-secondary">by {{ itemAuthor(item) }}</span>
							<div class="mt-1 flex flex-wrap items-center gap-1.5">
								<span
									:class="[
										'inline-flex items-center rounded-md border border-solid px-1.5 py-0.5 text-[10.5px] font-bold',
										itemTypeClass(item.project_type),
									]"
								>
									{{ itemTypeLabel(item.project_type) }}
								</span>
								<span class="max-w-[140px] truncate rounded-md bg-surface-3 px-1.5 py-0.5 font-mono text-[10.5px] text-secondary">
									{{ itemVersion(item) }}
								</span>
							</div>
						</div>

						<!-- Checkbox -->
						<Checkbox
							:model-value="selectedSet.has(ctx.getItemId(item))"
							aria-label="Select mod"
							class="shrink-0"
							@update:model-value="toggleSelect(item)"
						/>
					</div>

					<!-- Card Footer: Custom Pill Switch + Quick Actions -->
					<div class="flex items-center justify-between gap-2 border-0 border-t border-solid border-surface-4 pt-3">
						<!-- Custom Solara Toggle Switch -->
						<button
							type="button"
							role="switch"
							:aria-checked="item.enabled"
							:disabled="!canToggle(item) || ctx.isBusy.value || item.installing"
							:class="[
								'inline-flex cursor-pointer items-center gap-2 border-0 bg-transparent p-0 text-xs font-semibold disabled:cursor-not-allowed disabled:opacity-50',
								item.enabled ? 'text-contrast' : 'text-secondary',
							]"
							@click="handleToggle(item)"
						>
							<span
								:class="[
									'relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors',
									item.enabled ? 'bg-brand' : 'bg-surface-5',
								]"
							>
								<span
									:class="[
										'inline-block size-3.5 rounded-full bg-white shadow transition-transform',
										item.enabled ? 'translate-x-[18px]' : 'translate-x-[3px]',
									]"
								/>
							</span>
							<span>{{ item.enabled ? 'Active' : 'Off' }}</span>
						</button>

						<div class="flex items-center gap-1">
							<button
								v-if="item.has_update && !item.locked && ctx.updateItem"
								type="button"
								title="Update to latest compatible version"
								class="inline-flex h-7 cursor-pointer items-center gap-1 rounded-lg border border-solid border-emerald-500/40 bg-emerald-500/15 px-2 text-xs font-bold text-emerald-300 hover:bg-emerald-500/25"
								@click="ctx.updateItem(item)"
							>
								<DownloadIcon class="size-3.5" />
								<span>Update</span>
							</button>

							<button
								v-if="itemCanSwitchVersion(item)"
								type="button"
								title="Switch version"
								class="inline-flex size-7 cursor-pointer items-center justify-center rounded-lg border border-solid border-surface-4 bg-surface-3 text-secondary hover:text-contrast"
								@click="ctx.switchVersion?.(item)"
							>
								<UpdatedIcon class="size-3.5" />
							</button>

							<button
								v-if="canDelete(item)"
								type="button"
								title="Remove from instance"
								class="inline-flex size-7 cursor-pointer items-center justify-center rounded-lg border border-solid border-surface-4 bg-surface-3 text-secondary hover:border-red/40 hover:bg-red/15 hover:text-red"
								@click="pendingDeleteItem = item"
							>
								<TrashIcon class="size-3.5" />
							</button>

							<TeleportOverflowMenu
								v-if="getCardOverflow(item).length > 0"
								type="quiet"
								size="sm"
								label="More options"
								:options="getCardOverflow(item)"
							>
								<MoreVerticalIcon class="size-4" />
							</TeleportOverflowMenu>
						</div>
					</div>
				</article>
			</div>

			<!-- COMPACT LIST VIEW -->
			<div v-else class="flex flex-col gap-2">
				<div
					v-for="item in filteredItems"
					:key="ctx.getItemId(item)"
					:class="[
						'flex flex-wrap items-center justify-between gap-3 rounded-xl border border-solid px-4 py-2.5 transition-colors',
						selectedSet.has(ctx.getItemId(item))
							? 'border-brand bg-brand/8'
							: 'border-surface-4 bg-surface-2 hover:border-surface-5',
						!item.enabled ? 'opacity-65' : '',
					]"
				>
					<div class="flex min-w-0 flex-1 items-center gap-3">
						<Checkbox
							:model-value="selectedSet.has(ctx.getItemId(item))"
							aria-label="Select mod"
							@update:model-value="toggleSelect(item)"
						/>
						<Avatar
							:src="itemIcon(item)"
							:alt="itemTitle(item)"
							size="36px"
							class="rounded-lg ring-1 ring-surface-5"
						/>
						<div class="flex min-w-0 flex-col">
							<div class="flex items-center gap-2">
								<RouterLink
									v-if="itemProjectLink(item)"
									:to="itemProjectLink(item)!"
									class="truncate text-sm font-bold text-contrast no-underline hover:text-brand"
								>
									{{ itemTitle(item) }}
								</RouterLink>
								<span v-else class="truncate text-sm font-bold text-contrast">
									{{ itemTitle(item) }}
								</span>
								<span
									:class="[
										'inline-flex items-center rounded-md border border-solid px-1.5 py-0.5 text-[10px] font-bold',
										itemTypeClass(item.project_type),
									]"
								>
									{{ itemTypeLabel(item.project_type) }}
								</span>
							</div>
							<span class="truncate text-xs text-secondary">
								{{ itemAuthor(item) }} · {{ itemVersion(item) }}
							</span>
						</div>
					</div>

					<div class="flex items-center gap-2">
						<button
							v-if="item.has_update && !item.locked && ctx.updateItem"
							type="button"
							class="inline-flex h-7 cursor-pointer items-center gap-1 rounded-lg border border-solid border-emerald-500/40 bg-emerald-500/15 px-2.5 text-xs font-bold text-emerald-300"
							@click="ctx.updateItem(item)"
						>
							<DownloadIcon class="size-3.5" />
							<span>Update</span>
						</button>

						<button
							type="button"
							role="switch"
							:aria-checked="item.enabled"
							:disabled="!canToggle(item) || ctx.isBusy.value || item.installing"
							class="inline-flex cursor-pointer items-center border-0 bg-transparent p-0"
							@click="handleToggle(item)"
						>
							<span
								:class="[
									'relative inline-flex h-5 w-9 items-center rounded-full transition-colors',
									item.enabled ? 'bg-brand' : 'bg-surface-5',
								]"
							>
								<span
									:class="[
										'inline-block size-3.5 rounded-full bg-white shadow transition-transform',
										item.enabled ? 'translate-x-[18px]' : 'translate-x-[3px]',
									]"
								/>
							</span>
						</button>

						<button
							v-if="canDelete(item)"
							type="button"
							title="Remove"
							class="inline-flex size-7 cursor-pointer items-center justify-center rounded-lg border border-solid border-surface-4 bg-surface-3 text-secondary hover:bg-red/15 hover:text-red"
							@click="pendingDeleteItem = item"
						>
							<TrashIcon class="size-3.5" />
						</button>
					</div>
				</div>
			</div>
		</template>

		<!-- Floating Bulk Action Bar -->
		<div
			v-if="selectedIds.length > 0"
			class="fixed bottom-6 left-1/2 z-40 flex -translate-x-1/2 flex-wrap items-center gap-2 rounded-2xl border border-solid border-brand/40 bg-surface-2 px-4 py-2.5 shadow-2xl"
		>
			<span class="pr-2 text-xs font-bold text-contrast">{{ selectedIds.length }} selected</span>
			<Button type="outlined" size="sm" class="!rounded-lg" @click="handleBulkEnable">
				<CheckIcon class="size-3.5" />
				<span>Enable</span>
			</Button>
			<Button type="outlined" size="sm" class="!rounded-lg" @click="handleBulkDisable">
				<XIcon class="size-3.5" />
				<span>Disable</span>
			</Button>
			<Button type="colored" color="red" size="sm" class="!rounded-lg" @click="pendingBulkDelete = true">
				<TrashIcon class="size-3.5" />
				<span>Remove</span>
			</Button>
			<button
				type="button"
				class="ml-1 cursor-pointer border-0 bg-transparent text-xs font-semibold text-secondary hover:text-contrast"
				@click="clearSelection"
			>
				Clear
			</button>
		</div>

		<!-- Single Delete Confirmation -->
		<div
			v-if="pendingDeleteItem"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
			@click.self="pendingDeleteItem = null"
		>
			<div class="w-full max-w-md rounded-2xl border border-solid border-surface-5 bg-surface-2 p-5 shadow-2xl">
				<h3 class="m-0 text-lg font-extrabold text-contrast">Remove {{ itemTitle(pendingDeleteItem) }}?</h3>
				<p class="mt-2 mb-5 text-sm text-secondary">
					This deletes <code>{{ pendingDeleteItem.file_name }}</code> from this instance. You can reinstall it anytime.
				</p>
				<div class="flex justify-end gap-2">
					<Button type="outlined" @click="pendingDeleteItem = null">Cancel</Button>
					<Button type="colored" color="red" @click="confirmSingleDelete">Remove</Button>
				</div>
			</div>
		</div>

		<!-- Bulk Delete Confirmation -->
		<div
			v-if="pendingBulkDelete"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
			@click.self="pendingBulkDelete = false"
		>
			<div class="w-full max-w-md rounded-2xl border border-solid border-surface-5 bg-surface-2 p-5 shadow-2xl">
				<h3 class="m-0 text-lg font-extrabold text-contrast">Remove {{ selectedIds.length }} items?</h3>
				<p class="mt-2 mb-5 text-sm text-secondary">
					Selected files will be permanently removed from this instance.
				</p>
				<div class="flex justify-end gap-2">
					<Button type="outlined" @click="pendingBulkDelete = false">Cancel</Button>
					<Button type="colored" color="red" @click="confirmBulkDelete">Remove All</Button>
				</div>
			</div>
		</div>

		<!-- Unlink Modpack Confirmation -->
		<div
			v-if="pendingUnlink"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4"
			@click.self="pendingUnlink = false"
		>
			<div class="w-full max-w-md rounded-2xl border border-solid border-surface-5 bg-surface-2 p-5 shadow-2xl">
				<h3 class="m-0 text-lg font-extrabold text-contrast">Unlink Modpack?</h3>
				<p class="mt-2 mb-5 text-sm text-secondary">
					Your installed mods will stay, and you’ll be able to customize this profile freely without modpack restrictions.
				</p>
				<div class="flex justify-end gap-2">
					<Button type="outlined" @click="pendingUnlink = false">Cancel</Button>
					<Button type="colored" color="red" @click="handleConfirmUnlink">Unlink Pack</Button>
				</div>
			</div>
		</div>

		<slot name="modals" />
	</div>
</template>
