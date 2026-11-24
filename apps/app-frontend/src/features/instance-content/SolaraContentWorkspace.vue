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
	Combobox,
	type ContentItem,
	injectContentManager,
	TeleportOverflowMenu,
	type UpdateAllSelection,
} from '@modrinth/ui'
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'

const props = defineProps<{
	highlightedItemId?: string
}>()

const ctx = injectContentManager()

type ViewMode = 'grid' | 'list'
type FilterTab =
	| 'all'
	| 'mod'
	| 'resourcepack'
	| 'shader'
	| 'datapack'
	| 'enabled'
	| 'disabled'
	| 'updates'
type SortKey = 'name' | 'status' | 'type'

const SORT_OPTIONS: { value: SortKey; label: string }[] = [
	{ value: 'name', label: 'Name A–Z' },
	{ value: 'status', label: 'Active First' },
	{ value: 'type', label: 'Content Type' },
]

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
	if (presentTypes.value.has('resourcepack'))
		list.push({ id: 'resourcepack', label: 'Resource Packs' })
	if (presentTypes.value.has('shader')) list.push({ id: 'shader', label: 'Shaders' })
	if (presentTypes.value.has('datapack')) list.push({ id: 'datapack', label: 'Datapacks' })
	list.push({ id: 'enabled', label: 'Active', count: stats.value.enabled })
	if (stats.value.disabled > 0)
		list.push({ id: 'disabled', label: 'Off', count: stats.value.disabled })
	if (stats.value.updates > 0)
		list.push({ id: 'updates', label: 'Updates', count: stats.value.updates })
	return list
})

function itemTitle(item: ContentItem): string {
	return (
		item.project?.title ??
		item.embedded_metadata?.name ??
		item.file_name.replace(/\.disabled$/i, '')
	)
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
			return 'Pack'
		case 'shader':
			return 'Shader'
		case 'datapack':
			return 'Data'
		default:
			return 'Addon'
	}
}

function itemTypeClass(type?: string): string {
	switch (type) {
		case 'mod':
			return 'solara-type-mod'
		case 'shader':
			return 'solara-type-shader'
		case 'resourcepack':
			return 'solara-type-pack'
		case 'datapack':
			return 'solara-type-data'
		default:
			return 'bg-surface-4 text-secondary'
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
const selectedUpdatableItems = computed(() =>
	selectedItems.value.filter((item) => item.has_update && !item.locked && !!item.update_version_id),
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

function toUpdateSelections(items: ContentItem[]): UpdateAllSelection[] {
	return items
		.filter((item) => item.has_update && !item.locked && !!item.update_version_id)
		.map((item) => ({
			id: ctx.getItemId(item),
			projectId: item.project?.id ?? ctx.getItemId(item),
			version: {
				id: item.update_version_id!,
				version_number: item.version?.version_number ?? item.update_version_id!,
				date_published: '',
				loaders: [],
				game_versions: [],
				files: [],
			},
		}))
}

function handleUpdateSingle(item: ContentItem) {
	if (!ctx.updateItem || item.locked || item.installing || ctx.isBusy.value) return
	ctx.updateItem(ctx.getItemId(item))
}

async function handleUpdateAll() {
	if (!ctx.bulkUpdateSelections || updatingAll.value || ctx.isBusy.value) return
	const selections = toUpdateSelections(allItems.value)
	if (selections.length === 0) return
	updatingAll.value = true
	try {
		await ctx.bulkUpdateSelections(selections)
	} finally {
		updatingAll.value = false
	}
}

async function handleBulkUpdateSelected() {
	if (!ctx.bulkUpdateSelections || updatingAll.value || ctx.isBusy.value) return
	const selections = toUpdateSelections(selectedUpdatableItems.value)
	if (selections.length === 0) return
	updatingAll.value = true
	try {
		await ctx.bulkUpdateSelections(selections)
		clearSelection()
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
	<div class="flex flex-col gap-3.5 pb-12">
		<!-- Smooth Borderless Linked Modpack Strip -->
		<div
			v-if="ctx.managedContent.value"
			class="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-bg-raised px-4 py-3"
		>
			<div class="flex min-w-0 items-center gap-3">
				<Avatar
					:src="ctx.managedContent.value.card.manager.iconUrl"
					:alt="ctx.managedContent.value.card.manager.name"
					size="40px"
					class="rounded-xl"
				/>
				<div class="flex min-w-0 flex-col">
					<div class="flex flex-wrap items-center gap-2">
						<span class="text-[10.5px] font-bold uppercase tracking-wider text-brand">
							Linked Modpack
						</span>
						<span
							v-if="ctx.managedContent.value.card.manager.version"
							class="rounded-full bg-surface-4 px-2 py-0.5 text-[10.5px] font-semibold text-secondary"
						>
							{{ ctx.managedContent.value.card.manager.version }}
						</span>
					</div>
					<h3 class="m-0 truncate text-sm font-bold text-contrast">
						{{ ctx.managedContent.value.card.manager.name }}
					</h3>
				</div>
			</div>

			<div class="flex flex-wrap items-center gap-1.5">
				<Button
					v-if="ctx.viewManagedContent"
					type="base"
					size="sm"
					class="!rounded-xl"
					@click="ctx.viewManagedContent()"
				>
					<PackageIcon class="size-3.5" />
					<span>Pack Mods ({{ ctx.managedContent.value.items.length }})</span>
				</Button>
				<Button
					v-if="ctx.runManagedContentPrimaryAction && ctx.managedContent.value.card.hasUpdate"
					type="colored"
					color="brand"
					size="sm"
					class="!rounded-xl !font-bold"
					@click="ctx.runManagedContentPrimaryAction($event)"
				>
					<DownloadIcon class="size-3.5" />
					<span>Update Pack</span>
				</Button>
				<Button
					v-if="ctx.openManagedContentSettings"
					type="base"
					size="sm"
					class="!rounded-xl"
					@click="ctx.openManagedContentSettings()"
				>
					<SettingsIcon class="size-3.5" />
					<span>Settings</span>
				</Button>
				<Button
					v-if="ctx.unlinkModpack"
					type="quiet"
					color="red"
					size="sm"
					class="!rounded-xl"
					@click="pendingUnlink = true"
				>
					<UnlinkIcon class="size-3.5" />
					<span>Unlink</span>
				</Button>
			</div>
		</div>

		<!-- Loading State -->
		<div
			v-if="ctx.loading.value"
			class="flex flex-col items-center justify-center gap-3 rounded-2xl bg-bg-raised py-14 text-secondary"
		>
			<SpinnerIcon class="size-6 animate-spin text-brand" />
			<span class="text-sm font-medium">Loading installed mods &amp; packs...</span>
		</div>

		<!-- Empty State -->
		<div
			v-else-if="allItems.length === 0"
			class="flex flex-col items-center justify-center gap-3.5 rounded-3xl bg-bg-raised px-6 py-16 text-center"
		>
			<div
				class="solara-empty-icon flex size-14 items-center justify-center rounded-2xl text-brand"
			>
				<PackageIcon class="size-7" />
			</div>
			<div class="flex flex-col gap-1">
				<h3 class="m-0 text-base font-extrabold text-contrast">
					{{ ctx.managedContent.value ? 'No extra mods added yet' : 'No mods or packs installed yet' }}
				</h3>
				<p class="m-0 max-w-md text-xs leading-relaxed text-secondary">
					Browse both Modrinth &amp; CurseForge catalogs in one place, or drag and drop local .jar /
					.zip files right here.
				</p>
			</div>
			<div class="flex flex-wrap items-center justify-center gap-2 pt-1">
				<Button
					type="base"
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
					<span>Install Mods (Modrinth + CurseForge)</span>
				</Button>
			</div>
		</div>

		<!-- Populated Pure-OLED Smooth Mods Workspace -->
		<template v-else>
			<!-- Open Pure-OLED Controls (zero outer box cage!) -->
			<div class="flex flex-col gap-2.5">
				<!-- Row 1: Search + Sort + Clean Raised Actions directly on pure OLED bg -->
				<div class="flex flex-wrap items-center gap-2">
					<div
						class="solara-oled-search flex h-10 min-w-[200px] flex-1 items-center gap-2.5 rounded-xl bg-bg-raised px-3.5 transition-all"
					>
						<SearchIcon class="size-4 shrink-0 text-secondary" />
						<input
							v-model="searchQuery"
							type="text"
							:placeholder="`Search ${stats.total} installed mods, shaders, or packs...`"
							class="w-full border-none bg-transparent text-[13px] font-medium text-contrast outline-none placeholder:text-secondary"
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

					<div class="w-36 shrink-0">
						<Combobox
							v-model="sortBy"
							:options="SORT_OPTIONS"
							placeholder="Sort"
							trigger-size="sm"
						/>
					</div>

					<div class="flex flex-wrap items-center gap-1.5">
						<button
							type="button"
							:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
							class="inline-flex h-9 cursor-pointer items-center gap-1.5 rounded-xl border-0 bg-bg-raised px-3 text-xs font-semibold text-contrast transition-colors hover:bg-surface-4 disabled:cursor-not-allowed disabled:opacity-50"
							@click="ctx.uploadFiles"
						>
							<FolderOpenIcon class="size-3.5 text-secondary" />
							<span>Import</span>
						</button>
						<Button
							type="colored"
							color="brand"
							size="sm"
							class="!h-9 !rounded-xl !px-3.5 !font-bold"
							:disabled="ctx.isBusy.value || ctx.disableAddContent?.value"
							@click="ctx.browse"
						>
							<CompassIcon class="size-3.5" />
							<span>Install Mods</span>
						</Button>
						<button
							type="button"
							:disabled="refreshing"
							title="Reload installed content"
							class="inline-flex size-9 cursor-pointer items-center justify-center rounded-xl border-0 bg-bg-raised text-secondary transition-colors hover:bg-surface-4 hover:text-contrast"
							@click="handleRefresh"
						>
							<RefreshCwIcon :class="['size-3.5', refreshing ? 'animate-spin' : '']" />
						</button>
						<div class="inline-flex items-center rounded-xl bg-bg-raised p-1">
							<button
								type="button"
								title="2-column wide cards"
								:class="[
									'inline-flex size-7 cursor-pointer items-center justify-center rounded-lg border-0 transition-colors',
									viewMode === 'grid'
										? 'bg-surface-4 text-contrast'
										: 'bg-transparent text-secondary hover:text-contrast',
								]"
								@click="viewMode = 'grid'"
							>
								<GridIcon class="size-3.5" />
							</button>
							<button
								type="button"
								title="Full-width compact rows"
								:class="[
									'inline-flex size-7 cursor-pointer items-center justify-center rounded-lg border-0 transition-colors',
									viewMode === 'list'
										? 'bg-surface-4 text-contrast'
										: 'bg-transparent text-secondary hover:text-contrast',
								]"
								@click="viewMode = 'list'"
							>
								<ListIcon class="size-3.5" />
							</button>
						</div>
					</div>
				</div>

				<!-- Row 2: Smooth Borderless Filter Pills + Update All -->
				<div class="flex flex-wrap items-center justify-between gap-2">
					<div class="flex flex-wrap items-center gap-1.5">
						<button
							v-for="tab in filterTabs"
							:key="tab.id"
							type="button"
							:class="[
								'inline-flex cursor-pointer items-center gap-1.5 rounded-full border-0 px-3 py-1 text-xs font-semibold transition-all duration-150',
								activeFilter === tab.id
									? 'solara-pill-active text-contrast'
									: 'bg-bg-raised text-secondary hover:bg-surface-4 hover:text-contrast',
							]"
							@click="activeFilter = tab.id"
						>
							<span>{{ tab.label }}</span>
							<span v-if="typeof tab.count === 'number'" class="opacity-65 tabular-nums">
								{{ tab.count }}
							</span>
						</button>
					</div>

					<button
						v-if="stats.updates > 0"
						type="button"
						:disabled="updatingAll || ctx.isBusy.value"
						class="solara-update-pill inline-flex cursor-pointer items-center gap-1.5 rounded-full border-0 px-3 py-1 text-xs font-bold transition-all hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-50"
						@click="handleUpdateAll"
					>
						<SpinnerIcon v-if="updatingAll" class="size-3.5 animate-spin" />
						<DownloadIcon v-else class="size-3.5" />
						<span>{{ updatingAll ? 'Updating...' : `Update All (${stats.updates})` }}</span>
					</button>
				</div>
			</div>

			<!-- Filtered Empty -->
			<div
				v-if="filteredItems.length === 0"
				class="flex flex-col items-center justify-center gap-2.5 rounded-2xl bg-bg-raised py-12 text-center"
			>
				<p class="m-0 text-sm font-semibold text-contrast">No matching mods or packs found.</p>
				<Button
					size="sm"
					type="base"
					class="!rounded-xl"
					@click="
						() => {
							searchQuery = ''
							activeFilter = 'all'
						}
					"
				>
					Reset filters
				</Button>
			</div>

			<!-- SILKY PURE-OLED COMPACT WIDE CARDS (2-col on wide screens, 1-col in list mode) -->
			<div
				v-else
				:class="
					viewMode === 'grid'
						? 'grid grid-cols-1 gap-2 xl:grid-cols-2'
						: 'flex flex-col gap-1.5'
				"
			>
				<article
					v-for="(item, idx) in filteredItems"
					:key="ctx.getItemId(item)"
					:style="{ animationDelay: `${Math.min(idx, 16) * 16}ms` }"
					:class="[
						'solara-mod-card solara-card-animate group relative flex items-center justify-between gap-3 rounded-2xl bg-bg-raised px-3.5 py-2.5',
						selectedSet.has(ctx.getItemId(item))
							? 'solara-mod-card-selected'
							: item.file_path === props.highlightedItemId
								? 'solara-mod-card-highlight'
								: '',
						!item.enabled ? 'opacity-55 hover:opacity-90' : '',
					]"
				>
					<!-- Left: Checkbox + Smooth Borderless Icon + Title & Metadata -->
					<div class="flex min-w-0 flex-1 items-center gap-3">
						<Checkbox
							:model-value="selectedSet.has(ctx.getItemId(item))"
							aria-label="Select mod"
							class="shrink-0"
							@update:model-value="toggleSelect(item)"
						/>

						<div class="relative shrink-0">
							<Avatar
								:src="itemIcon(item)"
								:alt="itemTitle(item)"
								size="40px"
								class="!rounded-xl transition-transform duration-200 group-hover:scale-[1.03]"
							/>
							<span
								v-if="item.installing"
								class="absolute inset-0 flex items-center justify-center rounded-xl bg-black/65"
							>
								<SpinnerIcon class="size-4 animate-spin text-brand" />
							</span>
						</div>

						<div class="flex min-w-0 flex-1 flex-col gap-0.5">
							<div class="flex min-w-0 items-center gap-1.5">
								<RouterLink
									v-if="itemProjectLink(item)"
									:to="itemProjectLink(item)!"
									class="truncate text-[13.5px] font-bold leading-tight text-contrast no-underline transition-colors hover:text-brand"
								>
									{{ itemTitle(item) }}
								</RouterLink>
								<span v-else class="truncate text-[13.5px] font-bold leading-tight text-contrast">
									{{ itemTitle(item) }}
								</span>
								<LockIcon v-if="item.locked" class="size-3 shrink-0 text-orange" />
								<span
									:class="[
										'inline-flex shrink-0 items-center rounded-md px-1.5 py-[1px] text-[10px] font-bold leading-tight',
										itemTypeClass(item.project_type),
									]"
								>
									{{ itemTypeLabel(item.project_type) }}
								</span>
							</div>

							<div class="flex min-w-0 items-center gap-1.5 text-[11.5px] text-secondary">
								<span class="truncate">by {{ itemAuthor(item) }}</span>
								<span class="opacity-40">·</span>
								<span class="max-w-[160px] truncate font-mono text-[10.5px] opacity-80">
									{{ itemVersion(item) }}
								</span>
							</div>
						</div>
					</div>

					<!-- Right: Borderless Ghost Actions + Smooth Toggle -->
					<div class="flex shrink-0 items-center gap-1">
						<button
							v-if="item.has_update && !item.locked && ctx.updateItem"
							type="button"
							:disabled="item.installing || ctx.isBusy.value"
							title="Update to latest compatible version"
							class="solara-update-pill mr-0.5 inline-flex h-7 cursor-pointer items-center gap-1 rounded-lg border-0 px-2.5 text-[11px] font-bold transition-all hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-50"
							@click="handleUpdateSingle(item)"
						>
							<SpinnerIcon v-if="item.installing" class="size-3 animate-spin" />
							<DownloadIcon v-else class="size-3" />
							<span class="hidden sm:inline">{{ item.installing ? 'Updating' : 'Update' }}</span>
						</button>

						<button
							v-if="itemCanSwitchVersion(item)"
							type="button"
							title="Switch version"
							class="inline-flex size-7 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent text-secondary/75 transition-all hover:bg-surface-4 hover:text-contrast"
							@click="ctx.switchVersion?.(item)"
						>
							<UpdatedIcon class="size-3.5" />
						</button>

						<button
							v-if="canDelete(item)"
							type="button"
							title="Remove from profile"
							class="inline-flex size-7 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent text-secondary/75 transition-all hover:bg-red/15 hover:text-red"
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
							<MoreVerticalIcon class="size-3.5" />
						</TeleportOverflowMenu>

						<button
							type="button"
							role="switch"
							:aria-checked="item.enabled"
							:title="item.enabled ? 'Disable' : 'Enable'"
							:disabled="!canToggle(item) || ctx.isBusy.value || item.installing"
							class="ml-1 inline-flex cursor-pointer items-center border-0 bg-transparent p-0 disabled:cursor-not-allowed disabled:opacity-50"
							@click="handleToggle(item)"
						>
							<span
								:class="[
									'relative inline-flex h-[18px] w-8 shrink-0 items-center rounded-full transition-colors duration-200',
									item.enabled ? 'bg-brand' : 'bg-surface-5',
								]"
							>
								<span
									:class="[
										'inline-block size-3 rounded-full bg-white shadow transition-transform duration-200',
										item.enabled ? 'translate-x-[17px]' : 'translate-x-[3px]',
									]"
								/>
							</span>
						</button>
					</div>
				</article>
			</div>
		</template>

		<!-- Frosted Floating Bulk Action Dock -->
		<div
			v-if="selectedIds.length > 0"
			class="solara-bulk-dock fixed bottom-6 left-1/2 z-40 flex -translate-x-1/2 flex-wrap items-center gap-2 rounded-2xl px-4 py-2.5 shadow-2xl backdrop-blur-xl"
		>
			<span class="pr-2 text-xs font-bold text-contrast tabular-nums">
				{{ selectedIds.length }} selected
			</span>
			<Button
				v-if="selectedUpdatableItems.length > 0 && ctx.bulkUpdateSelections"
				type="colored"
				color="brand"
				size="sm"
				class="!rounded-xl !font-bold"
				:disabled="updatingAll || ctx.isBusy.value"
				@click="handleBulkUpdateSelected"
			>
				<SpinnerIcon v-if="updatingAll" class="size-3.5 animate-spin" />
				<DownloadIcon v-else class="size-3.5" />
				<span>Update ({{ selectedUpdatableItems.length }})</span>
			</Button>
			<Button type="base" size="sm" class="!rounded-xl" @click="handleBulkEnable">
				<CheckIcon class="size-3.5" />
				<span>Enable</span>
			</Button>
			<Button type="base" size="sm" class="!rounded-xl" @click="handleBulkDisable">
				<XIcon class="size-3.5" />
				<span>Disable</span>
			</Button>
			<Button type="colored" color="red" size="sm" class="!rounded-xl" @click="pendingBulkDelete = true">
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
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 backdrop-blur-sm"
			@click.self="pendingDeleteItem = null"
		>
			<div class="w-full max-w-md rounded-3xl bg-bg-raised p-6 shadow-2xl">
				<h3 class="m-0 text-lg font-extrabold text-contrast">
					Remove {{ itemTitle(pendingDeleteItem) }}?
				</h3>
				<p class="mt-2 mb-5 text-sm text-secondary">
					This deletes <code>{{ pendingDeleteItem.file_name }}</code> from this profile. You can
					reinstall it anytime.
				</p>
				<div class="flex justify-end gap-2">
					<Button type="base" class="!rounded-xl" @click="pendingDeleteItem = null">
						Cancel
					</Button>
					<Button type="colored" color="red" class="!rounded-xl" @click="confirmSingleDelete">
						Remove
					</Button>
				</div>
			</div>
		</div>

		<!-- Bulk Delete Confirmation -->
		<div
			v-if="pendingBulkDelete"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 backdrop-blur-sm"
			@click.self="pendingBulkDelete = false"
		>
			<div class="w-full max-w-md rounded-3xl bg-bg-raised p-6 shadow-2xl">
				<h3 class="m-0 text-lg font-extrabold text-contrast">
					Remove {{ selectedIds.length }} items?
				</h3>
				<p class="mt-2 mb-5 text-sm text-secondary">
					Selected files will be permanently removed from this profile.
				</p>
				<div class="flex justify-end gap-2">
					<Button type="base" class="!rounded-xl" @click="pendingBulkDelete = false">
						Cancel
					</Button>
					<Button type="colored" color="red" class="!rounded-xl" @click="confirmBulkDelete">
						Remove All
					</Button>
				</div>
			</div>
		</div>

		<!-- Unlink Modpack Confirmation -->
		<div
			v-if="pendingUnlink"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 backdrop-blur-sm"
			@click.self="pendingUnlink = false"
		>
			<div class="w-full max-w-md rounded-3xl bg-bg-raised p-6 shadow-2xl">
				<h3 class="m-0 text-lg font-extrabold text-contrast">Unlink Modpack?</h3>
				<p class="mt-2 mb-5 text-sm text-secondary">
					Your installed mods will stay, and you’ll be able to customize this profile freely without
					modpack restrictions.
				</p>
				<div class="flex justify-end gap-2">
					<Button type="base" class="!rounded-xl" @click="pendingUnlink = false">
						Cancel
					</Button>
					<Button type="colored" color="red" class="!rounded-xl" @click="handleConfirmUnlink">
						Unlink Pack
					</Button>
				</div>
			</div>
		</div>

		<slot name="modals" />
	</div>
</template>

<style scoped>
.solara-empty-icon {
	background-color: color-mix(in srgb, var(--color-brand) 14%, transparent);
}

.solara-oled-search {
	border: 1px solid transparent;
}

.solara-oled-search:focus-within {
	border-color: color-mix(in srgb, var(--color-brand) 55%, transparent);
}

.solara-update-pill {
	background-color: color-mix(in srgb, var(--color-brand) 16%, transparent);
	color: var(--color-brand);
}

.solara-pill-active {
	background-color: color-mix(in srgb, var(--color-brand) 18%, var(--surface-3));
	color: var(--color-brand);
}

.solara-mod-card {
	border: 1px solid transparent;
	transition:
		background-color 180ms ease,
		border-color 180ms ease,
		transform 220ms cubic-bezier(0.22, 1, 0.36, 1),
		opacity 180ms ease;
}

.solara-mod-card:hover {
	background-color: color-mix(in srgb, var(--surface-4) 68%, var(--surface-3));
	transform: translateY(-1px);
}

.solara-mod-card-selected {
	border-color: color-mix(in srgb, var(--color-brand) 65%, transparent);
	background-color: color-mix(in srgb, var(--color-brand) 10%, var(--surface-3));
}

.solara-mod-card-highlight {
	border-color: color-mix(in srgb, var(--color-brand) 55%, transparent);
}

.solara-bulk-dock {
	background-color: color-mix(in srgb, var(--surface-3) 92%, transparent);
	border: 1px solid color-mix(in srgb, var(--color-brand) 38%, transparent);
}

.solara-type-mod {
	background-color: color-mix(in srgb, var(--color-brand) 14%, transparent);
	color: var(--color-brand);
}

.solara-type-shader {
	background-color: color-mix(in srgb, var(--color-orange) 15%, transparent);
	color: var(--color-orange);
}

.solara-type-pack {
	background-color: color-mix(in srgb, var(--color-blue) 15%, transparent);
	color: var(--color-blue);
}

.solara-type-data {
	background-color: color-mix(in srgb, var(--color-purple) 15%, transparent);
	color: var(--color-purple);
}
</style>
