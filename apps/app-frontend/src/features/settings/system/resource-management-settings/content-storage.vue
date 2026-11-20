<script setup lang="ts">
import { DatabaseIcon, LoaderCircleIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	Input,
	ProgressBar,
	useFormatBytes,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'

type StoreUsage = {
	unique_bytes: number
	shared_bytes: number
	unused_cache_bytes: number
	private_copy_bytes: number
	object_count: number
	cache_limit_bytes: number
}

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()
const gibibyte = 1024 ** 3
const queryClient = useQueryClient()
const storeUsageKey = ['content-store', 'usage'] as const
const {
	data: storeUsage,
	isPending: calculating,
	error: storeUsageError,
} = useQuery({
	queryKey: storeUsageKey,
	networkMode: 'always',
	queryFn: async () => {
		const [usage] = await Promise.all([
			invoke<StoreUsage>('plugin:settings|store_usage'),
			new Promise<void>((resolve) => setTimeout(resolve, 400)),
		])
		return usage
	},
})
const cacheLimitGiB = ref<number | undefined>()
watch(
	() => storeUsage.value?.cache_limit_bytes,
	(bytes) => {
		cacheLimitGiB.value = (bytes ?? 5 * gibibyte) / gibibyte
	},
	{ immediate: true },
)
watch(storeUsageError, (error) => {
	if (error) handleError(error)
})
const busy = computed(
	() => calculating.value || clearMutation.isPending.value || cacheLimitMutation.isPending.value,
)
const clearing = computed(() => clearMutation.isPending.value)
const clearedBytes = computed(() => clearMutation.data.value ?? null)
const totalBytes = computed(
	() => (storeUsage.value?.unique_bytes ?? 0) + (storeUsage.value?.private_copy_bytes ?? 0),
)

const messages = defineMessages({
	title: {
		id: 'app.settings.resource-management.store.title',
		defaultMessage: 'Content storage',
	},
	description: {
		id: 'app.settings.resource-management.store.description',
		defaultMessage: 'Save disk space by sharing game files, mods, and packs across instances.',
	},
	calculating: {
		id: 'app.settings.resource-management.store.calculating',
		defaultMessage: 'Calculating…',
	},
	stored: {
		id: 'app.settings.resource-management.store.total',
		defaultMessage: '{size} used',
	},
	unique: {
		id: 'app.settings.resource-management.store.unique.label',
		defaultMessage: 'Unique',
	},
	shared: {
		id: 'app.settings.resource-management.store.shared.label',
		defaultMessage: 'Shared',
	},
	unused: {
		id: 'app.settings.resource-management.store.unused.label',
		defaultMessage: 'Orphaned',
	},
	clear: {
		id: 'app.settings.resource-management.store.clear',
		defaultMessage: 'Clear unused',
	},
	clearing: {
		id: 'app.settings.resource-management.store.clearing',
		defaultMessage: 'Clearing…',
	},
	clearDescription: {
		id: 'app.settings.resource-management.store.clear.description',
		defaultMessage: 'Delete unused downloaded content',
	},
	limit: {
		id: 'app.settings.resource-management.store.cache-limit.label',
		defaultMessage: 'Unused cache limit',
	},
	limitDescription: {
		id: 'app.settings.resource-management.store.cache-limit.description',
		defaultMessage:
			'Keep up to this much unused content so re-installing mods and packs stays fast.',
	},
	limitUnit: {
		id: 'app.settings.resource-management.store.cache-limit.unit',
		defaultMessage: 'GiB',
	},
	cleared: {
		id: 'app.settings.resource-management.store.cleared',
		defaultMessage: 'Freed {size} of disk space.',
	},
	nothingToClear: {
		id: 'app.settings.resource-management.store.nothing-to-clear',
		defaultMessage: 'No unused content to clear.',
	},
})

const categories = computed(() => {
	const unused = storeUsage.value?.unused_cache_bytes ?? 0
	const shared = storeUsage.value?.shared_bytes ?? 0
	return [
		{
			id: 'unique',
			label: messages.unique,
			bytes: Math.max(0, totalBytes.value - unused - shared),
			tone: 'tone-cyan',
		},
		{
			id: 'shared',
			label: messages.shared,
			bytes: shared,
			tone: 'tone-violet',
		},
		{
			id: 'unused',
			label: messages.unused,
			bytes: unused,
			tone: 'tone-amber',
		},
	]
})

const clearMutation = useMutation({
	networkMode: 'always',
	mutationFn: () => invoke<number>('plugin:settings|store_cleanup'),
	onSuccess: () => queryClient.invalidateQueries({ queryKey: storeUsageKey }),
	onError: handleError,
})

function clearUnused() {
	if (busy.value) return
	clearMutation.reset()
	clearMutation.mutate()
}

const cacheLimitMutation = useMutation({
	networkMode: 'always',
	mutationFn: (bytes: number) => invoke('plugin:settings|store_set_cache_limit', { bytes }),
	onSuccess: (_result, bytes) => {
		queryClient.setQueryData<StoreUsage>(storeUsageKey, (usage) =>
			usage ? { ...usage, cache_limit_bytes: bytes } : usage,
		)
	},
	onError: (error) => {
		cacheLimitGiB.value = (storeUsage.value?.cache_limit_bytes ?? 5 * gibibyte) / gibibyte
		handleError(error)
	},
})

function saveCacheLimit() {
	if (busy.value) return
	const bytes = Math.round(Number(cacheLimitGiB.value) * gibibyte)
	if (!Number.isSafeInteger(bytes) || bytes < 0) {
		cacheLimitGiB.value = (storeUsage.value?.cache_limit_bytes ?? 5 * gibibyte) / gibibyte
		return
	}
	cacheLimitMutation.mutate(bytes)
}
</script>

<template>
	<section
		v-if="calculating || storeUsage"
		class="solara-res-card @container flex flex-col gap-5 rounded-2xl p-5"
		aria-labelledby="content-storage-title"
	>
		<!-- Card Header -->
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div class="flex min-w-0 items-center gap-3">
				<div
					class="solara-res-icon is-cyan flex size-10 shrink-0 items-center justify-center rounded-xl"
					aria-hidden="true"
				>
					<DatabaseIcon class="size-5 text-sky-400" />
				</div>
				<div class="flex min-w-0 flex-col gap-0.5">
					<h2 id="content-storage-title" class="m-0 text-base font-bold text-contrast">
						{{ formatMessage(messages.title) }}
					</h2>
					<p class="m-0 text-xs font-medium text-secondary">
						{{ formatMessage(messages.description) }}
					</p>
				</div>
			</div>
			<span
				class="solara-res-total-pill shrink-0 rounded-full px-3 py-1 text-xs font-bold tabular-nums"
				aria-live="polite"
			>
				{{
					calculating
						? formatMessage(messages.calculating)
						: formatMessage(messages.stored, { size: formatBytes(totalBytes, 1) })
				}}
			</span>
		</div>

		<!-- Storage Bar + Stat Tiles -->
		<div class="flex flex-col gap-3">
			<ProgressBar
				v-if="calculating"
				:progress="0"
				waiting
				full-width
				class="[&>div:last-child]:h-2.5 [&>div:last-child]:bg-surface-4"
				aria-hidden="true"
			/>
			<div
				v-else
				aria-hidden="true"
				class="solara-storage-track flex h-2.5 gap-1 overflow-hidden rounded-full p-0.5"
			>
				<template v-for="category in categories" :key="category.id">
					<div
						v-if="category.bytes > 0"
						class="solara-storage-segment h-full min-w-[6px] rounded-full motion-safe:transition-[flex-grow] motion-safe:duration-300"
						:class="category.tone"
						:style="{ flexGrow: category.bytes, flexBasis: '0%' }"
					/>
				</template>
			</div>

			<dl v-if="storeUsage" class="m-0 grid grid-cols-1 gap-2.5 @md:grid-cols-3">
				<div
					v-for="category in categories"
					:key="category.id"
					class="solara-stat-tile flex items-center justify-between gap-2 rounded-xl px-3 py-2.5"
				>
					<dt class="flex items-center gap-2 text-xs font-semibold text-secondary">
						<span
							class="solara-storage-dot size-2.5 shrink-0 rounded-full"
							:class="category.tone"
							aria-hidden="true"
						/>
						{{ formatMessage(category.label) }}
					</dt>
					<dd class="m-0 text-sm font-bold tabular-nums text-contrast">
						{{ formatBytes(category.bytes, 1) }}
					</dd>
				</div>
			</dl>
		</div>

		<!-- Cache Limit & Cleanup Row -->
		<div
			class="solara-res-subrow flex flex-col gap-4 rounded-xl p-3.5 @lg:flex-row @lg:items-center @lg:justify-between"
		>
			<div class="flex min-w-0 flex-1 flex-col gap-0.5">
				<h3 class="m-0 text-sm font-semibold text-contrast">
					<label for="store-cache-limit">{{ formatMessage(messages.limit) }}</label>
				</h3>
				<p id="store-cache-limit-description" class="m-0 text-xs text-secondary">
					{{ formatMessage(messages.limitDescription) }}
				</p>
			</div>
			<div class="flex flex-wrap items-center gap-2.5">
				<Input
					id="store-cache-limit"
					v-model="cacheLimitGiB"
					type="number"
					:min="0"
					:step="0.5"
					aria-describedby="store-cache-limit-description"
					:disabled="busy"
					wrapper-class="w-28 shrink-0"
					@change="saveCacheLimit"
				>
					<template #trailing>
						<span class="text-xs font-semibold text-secondary">
							{{ formatMessage(messages.limitUnit) }}
						</span>
					</template>
				</Input>
				<Button
					v-tooltip="formatMessage(messages.clearDescription)"
					type="colored"
					color="red"
					size="md"
					:disabled="busy || !storeUsage || storeUsage.unused_cache_bytes === 0"
					:loading="clearing"
					@click="clearUnused"
				>
					<LoaderCircleIcon v-if="clearing" class="motion-safe:animate-spin" aria-hidden="true" />
					<TrashIcon v-else aria-hidden="true" />
					{{ formatMessage(clearing ? messages.clearing : messages.clear) }}
				</Button>
			</div>
		</div>

		<div aria-live="polite" class="empty:hidden">
			<p v-if="clearedBytes !== null" class="m-0 text-xs font-semibold text-emerald-400">
				{{
					clearedBytes > 0
						? formatMessage(messages.cleared, { size: formatBytes(clearedBytes, 1) })
						: formatMessage(messages.nothingToClear)
				}}
			</p>
		</div>
	</section>
</template>

<style scoped>
.solara-res-card {
	background: color-mix(in srgb, var(--surface-3) 86%, transparent);
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--surface-5) 55%, transparent);
}

:global(.light-mode) .solara-res-card {
	background: #ffffff;
	box-shadow:
		0 2px 10px rgba(15, 23, 42, 0.04),
		inset 0 0 0 1px rgba(15, 23, 42, 0.07);
}

.solara-res-icon.is-cyan {
	background: linear-gradient(135deg, rgba(56, 189, 248, 0.18), rgba(96, 165, 250, 0.12));
}

.solara-res-total-pill {
	background: linear-gradient(135deg, rgba(56, 189, 248, 0.15), rgba(251, 191, 36, 0.12));
	color: #38bdf8;
}

:global(.light-mode) .solara-res-total-pill {
	background: rgba(2, 132, 199, 0.1);
	color: #0284c7;
}

.solara-storage-track {
	background: color-mix(in srgb, var(--surface-2) 90%, transparent);
}

.solara-storage-segment.tone-cyan,
.solara-storage-dot.tone-cyan {
	background: #38bdf8;
}

.solara-storage-segment.tone-violet,
.solara-storage-dot.tone-violet {
	background: #a78bfa;
}

.solara-storage-segment.tone-amber,
.solara-storage-dot.tone-amber {
	background: #fbbf24;
}

.solara-stat-tile,
.solara-res-subrow {
	background: color-mix(in srgb, var(--surface-2) 72%, transparent);
}

:global(.light-mode) .solara-stat-tile,
:global(.light-mode) .solara-res-subrow {
	background: #f4f7fb;
}
</style>
