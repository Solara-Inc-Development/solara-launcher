<script setup lang="ts">
import { CheckIcon, DownloadIcon, TrashIcon } from '@modrinth/assets'
import { defineMessages, injectPageContext, useFormatNumber, useScrollIndicator, useVIntl } from '@modrinth/ui'
import { useResizeObserver } from '@vueuse/core'
import { computed, ref, useTemplateRef, watch } from 'vue'

import DownloadManagerJobRow from './download-manager-job.vue'
import type { DownloadManagerJob } from './use-download-manager'

const props = defineProps<{
	activeJobs: DownloadManagerJob[]
	attentionJobs: DownloadManagerJob[]
	completedJobs: DownloadManagerJob[]
	rate: string
}>()
defineEmits<{
	retry: [id: string]
	cancel: [id: string]
	togglePause: [id: string]
	dismiss: [id: string]
	copyDetails: [id: string]
	clearCompleted: []
	open: []
}>()

const { formatMessage } = useVIntl()
const { showAds } = injectPageContext()
const formatNumber = useFormatNumber()

const messages = defineMessages({
	title: { id: 'app.download-manager.title', defaultMessage: 'Solara Downloads' },
	active: { id: 'app.download-manager.active', defaultMessage: 'Active' },
	attention: { id: 'app.download-manager.attention', defaultMessage: 'Needs attention' },
	clearAll: { id: 'app.download-manager.clear-all', defaultMessage: 'Clear history' },
	clearCompleted: {
		id: 'app.download-manager.clear-completed',
		defaultMessage: 'Clear all completed downloads',
	},
	rate: { id: 'app.download-manager.estimated-rate', defaultMessage: 'Current download speed' },
})

const panel = useTemplateRef('panel')
const scrollContainer = useTemplateRef<HTMLElement>('scroll-container')
const scrollContent = useTemplateRef<HTMLElement>('scroll-content')
const { showTopFade, showBottomFade, checkScrollState } = useScrollIndicator(scrollContainer)
useResizeObserver(scrollContent, checkScrollState)

const currentTab = ref<'active' | 'history'>(
	props.activeJobs.length + props.attentionJobs.length > 0 || props.completedJobs.length === 0
		? 'active'
		: 'history',
)

const activeQueueCount = computed(() => props.activeJobs.length + props.attentionJobs.length)

watch(
	() => props.activeJobs.length + props.attentionJobs.length,
	(nextCount, prevCount) => {
		if (nextCount > (prevCount ?? 0)) {
			currentTab.value = 'active'
		}
	},
)

const headerSubtitle = computed(() => {
	if (props.activeJobs.length > 0 && props.attentionJobs.length > 0) {
		return `${formatNumber(props.activeJobs.length)} running · ${formatNumber(props.attentionJobs.length)} attention`
	}
	if (props.activeJobs.length > 0) {
		return props.activeJobs.length === 1
			? '1 transfer in progress'
			: `${formatNumber(props.activeJobs.length)} transfers in progress`
	}
	if (props.attentionJobs.length > 0) {
		return props.attentionJobs.length === 1
			? '1 transfer needs attention'
			: `${formatNumber(props.attentionJobs.length)} transfers need attention`
	}
	if (props.completedJobs.length > 0) {
		return `${formatNumber(props.completedJobs.length)} saved in history`
	}
	return 'Ready for Java, modpacks & updates'
})

defineExpose({ focus: () => panel.value?.focus() })
</script>

<template>
	<div
		ref="panel"
		role="dialog"
		:aria-label="formatMessage(messages.title)"
		tabindex="-1"
		class="solara-dl-panel flex w-[392px] max-w-[calc(100vw-2rem)] flex-col overflow-hidden rounded-[1.25rem] outline-none"
	>
		<!-- Solara Drawer Header -->
		<div class="solara-dl-header flex shrink-0 items-center justify-between gap-3 px-4 pb-3 pt-4">
			<div class="flex min-w-0 items-center gap-3">
				<div
					class="solara-dl-brand-icon flex size-9 shrink-0 items-center justify-center rounded-xl"
					aria-hidden="true"
				>
					<DownloadIcon class="size-4.5 text-sky-400" />
				</div>
				<div class="flex min-w-0 flex-col">
					<span class="truncate text-[15px] font-bold tracking-tight text-contrast">
						Solara Downloads
					</span>
					<span class="truncate text-xs font-medium text-secondary">
						{{ headerSubtitle }}
					</span>
				</div>
			</div>
			<span
				v-if="rate && activeJobs.length > 0"
				v-tooltip="formatMessage(messages.rate)"
				class="solara-dl-speed-chip shrink-0 rounded-full px-2.5 py-1 text-xs font-bold tabular-nums"
			>
				{{ rate }}
			</span>
		</div>

		<!-- Segmented Solara Tabs -->
		<div class="flex shrink-0 items-center justify-between gap-2 px-4 pb-3">
			<div
				class="solara-dl-tabs grid flex-1 grid-cols-2 gap-1 rounded-xl p-1"
				role="tablist"
				aria-label="Download sections"
			>
				<button
					type="button"
					role="tab"
					:aria-selected="currentTab === 'active'"
					class="solara-dl-tab flex cursor-pointer items-center justify-center gap-1.5 rounded-lg border-0 px-3 py-1.5 text-xs font-semibold transition-all duration-150"
					:class="{ 'is-active': currentTab === 'active' }"
					@click="currentTab = 'active'"
				>
					<span>Active</span>
					<span
						v-if="activeQueueCount > 0"
						class="solara-dl-tab-count rounded-full px-1.5 py-0.5 text-[10px] font-bold leading-none tabular-nums"
					>
						{{ formatNumber(activeQueueCount) }}
					</span>
				</button>
				<button
					type="button"
					role="tab"
					:aria-selected="currentTab === 'history'"
					class="solara-dl-tab flex cursor-pointer items-center justify-center gap-1.5 rounded-lg border-0 px-3 py-1.5 text-xs font-semibold transition-all duration-150"
					:class="{ 'is-active': currentTab === 'history' }"
					@click="currentTab = 'history'"
				>
					<span>History</span>
					<span
						v-if="completedJobs.length > 0"
						class="solara-dl-tab-count rounded-full px-1.5 py-0.5 text-[10px] font-bold leading-none tabular-nums"
					>
						{{ formatNumber(completedJobs.length) }}
					</span>
				</button>
			</div>

			<button
				v-if="currentTab === 'history' && completedJobs.length > 0"
				v-tooltip="formatMessage(messages.clearCompleted)"
				type="button"
				class="solara-dl-clear-btn flex h-8 shrink-0 cursor-pointer items-center gap-1 rounded-xl border-0 px-2.5 text-xs font-semibold transition-colors"
				:disabled="completedJobs.some((job) => job.busy)"
				@click="$emit('clearCompleted')"
			>
				<TrashIcon class="size-3.5" aria-hidden="true" />
				<span>Clear</span>
			</button>
		</div>

		<!-- Task List -->
		<div class="relative min-h-0">
			<div
				class="pointer-events-none absolute inset-x-0 top-0 z-10 h-5 bg-gradient-to-b from-[var(--solara-dl-bg)] to-transparent transition-opacity duration-200"
				:class="showTopFade ? 'opacity-100' : 'opacity-0'"
				aria-hidden="true"
			/>
			<div
				ref="scroll-container"
				class="min-h-0 overflow-y-auto overscroll-contain"
				:style="{
					maxHeight: `min(28rem, max(10rem, calc(100dvh - 10rem - ${showAds ? 250 : 0}px)))`,
				}"
			>
				<div ref="scroll-content" class="flex flex-col gap-2.5 px-4 pb-4 pt-0.5">
					<template v-if="currentTab === 'active'">
						<template v-if="activeJobs.length > 0 || attentionJobs.length > 0">
							<DownloadManagerJobRow
								v-for="job in activeJobs"
								:key="job.id"
								:job="job"
								@retry="$emit('retry', $event)"
								@cancel="$emit('cancel', $event)"
								@toggle-pause="$emit('togglePause', $event)"
								@dismiss="$emit('dismiss', $event)"
								@copy-details="$emit('copyDetails', $event)"
								@open="$emit('open')"
							/>
							<div v-if="attentionJobs.length > 0" class="flex flex-col gap-2">
								<span
									class="px-1 pt-1 text-[11px] font-bold uppercase tracking-wider text-amber-400"
								>
									{{ formatMessage(messages.attention) }}
								</span>
								<DownloadManagerJobRow
									v-for="job in attentionJobs"
									:key="job.id"
									:job="job"
									@retry="$emit('retry', $event)"
									@cancel="$emit('cancel', $event)"
									@toggle-pause="$emit('togglePause', $event)"
									@dismiss="$emit('dismiss', $event)"
									@copy-details="$emit('copyDetails', $event)"
									@open="$emit('open')"
								/>
							</div>
						</template>
						<div
							v-else
							class="solara-dl-empty flex flex-col items-center justify-center gap-2 rounded-2xl px-6 py-8 text-center"
						>
							<div
								class="solara-dl-empty-icon flex size-11 items-center justify-center rounded-2xl"
								aria-hidden="true"
							>
								<CheckIcon class="size-5 text-sky-400" />
							</div>
							<span class="text-sm font-semibold text-contrast">No active downloads</span>
							<p class="m-0 max-w-[16rem] text-xs leading-relaxed text-secondary">
								Java runtimes, modpack installs, and launcher updates will show live progress here.
							</p>
						</div>
					</template>

					<template v-else>
						<template v-if="completedJobs.length > 0">
							<DownloadManagerJobRow
								v-for="job in completedJobs"
								:key="job.id"
								:job="job"
								@retry="$emit('retry', $event)"
								@cancel="$emit('cancel', $event)"
								@toggle-pause="$emit('togglePause', $event)"
								@dismiss="$emit('dismiss', $event)"
								@copy-details="$emit('copyDetails', $event)"
								@open="$emit('open')"
							/>
						</template>
						<div
							v-else
							class="solara-dl-empty flex flex-col items-center justify-center gap-2 rounded-2xl px-6 py-8 text-center"
						>
							<div
								class="solara-dl-empty-icon flex size-11 items-center justify-center rounded-2xl"
								aria-hidden="true"
							>
								<DownloadIcon class="size-5 text-secondary" />
							</div>
							<span class="text-sm font-semibold text-contrast">No recent history</span>
							<p class="m-0 max-w-[16rem] text-xs leading-relaxed text-secondary">
								Finished installs and runtime downloads will stay listed here for quick access.
							</p>
						</div>
					</template>
				</div>
			</div>
			<div
				class="pointer-events-none absolute inset-x-0 bottom-0 z-10 h-5 bg-gradient-to-t from-[var(--solara-dl-bg)] to-transparent transition-opacity duration-200"
				:class="showBottomFade ? 'opacity-100' : 'opacity-0'"
				aria-hidden="true"
			/>
		</div>
	</div>
</template>

<style scoped>
.solara-dl-panel {
	--solara-dl-bg: color-mix(in srgb, var(--surface-3) 96%, #050a12);
	background: var(--solara-dl-bg);
	box-shadow:
		0 20px 44px rgba(0, 0, 0, 0.36),
		0 0 0 1px color-mix(in srgb, var(--surface-5) 65%, rgba(56, 189, 248, 0.16));
}

:global(.light-mode) .solara-dl-panel {
	--solara-dl-bg: #ffffff;
	box-shadow:
		0 20px 44px rgba(15, 23, 42, 0.14),
		0 0 0 1px rgba(15, 23, 42, 0.08);
}

.solara-dl-brand-icon {
	background: linear-gradient(135deg, rgba(56, 189, 248, 0.18), rgba(251, 191, 36, 0.14));
}

.solara-dl-speed-chip {
	background: rgba(56, 189, 248, 0.14);
	color: #38bdf8;
}

:global(.light-mode) .solara-dl-speed-chip {
	background: rgba(2, 132, 199, 0.1);
	color: #0284c7;
}

.solara-dl-tabs {
	background: color-mix(in srgb, var(--surface-2) 85%, transparent);
}

.solara-dl-tab {
	background: transparent;
	color: var(--color-secondary);
}

.solara-dl-tab:hover {
	color: var(--color-contrast);
}

.solara-dl-tab.is-active {
	background: var(--surface-4);
	color: var(--color-contrast);
	box-shadow: 0 2px 8px rgba(0, 0, 0, 0.16);
}

:global(.light-mode) .solara-dl-tab.is-active {
	background: #ffffff;
	box-shadow: 0 1px 4px rgba(15, 23, 42, 0.1);
}

.solara-dl-tab-count {
	background: rgba(56, 189, 248, 0.16);
	color: #38bdf8;
}

:global(.light-mode) .solara-dl-tab-count {
	background: rgba(2, 132, 199, 0.12);
	color: #0284c7;
}

.solara-dl-clear-btn {
	background: color-mix(in srgb, var(--surface-4) 75%, transparent);
	color: var(--color-secondary);
}

.solara-dl-clear-btn:hover:not(:disabled) {
	background: rgba(239, 68, 68, 0.14);
	color: #f87171;
}

.solara-dl-clear-btn:disabled {
	opacity: 0.45;
	cursor: not-allowed;
}

.solara-dl-empty {
	background: color-mix(in srgb, var(--surface-2) 65%, transparent);
}

.solara-dl-empty-icon {
	background: linear-gradient(135deg, rgba(56, 189, 248, 0.14), rgba(251, 191, 36, 0.1));
}
</style>
