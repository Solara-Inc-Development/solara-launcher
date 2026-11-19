<script setup lang="ts">
import {
	CheckIcon,
	CircleAlertIcon,
	DownloadIcon,
	LoaderSpinnerIcon,
	PauseIcon,
} from '@modrinth/assets'
import { defineMessages, truncatedTooltip, useFormatNumber, useVIntl } from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'

import type { DownloadManagerJob } from './use-download-manager'

const props = defineProps<{
	selectedJob: DownloadManagerJob | null
	activeCount: number
	progress: number
	hasAttention: boolean
	rate: string
	completing: boolean
	animated: boolean
	expanded: boolean
	panelId: string
}>()
defineEmits<{ close: [] }>()

const messages = defineMessages({
	title: { id: 'app.download-manager.title', defaultMessage: 'Solara Downloads' },
	show: { id: 'app.download-manager.show-tasks', defaultMessage: 'Show downloads' },
	hide: { id: 'app.download-manager.hide-tasks', defaultMessage: 'Hide downloads' },
	attention: { id: 'app.download-manager.attention', defaultMessage: 'Needs attention' },
})

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const trigger = useTemplateRef('trigger')
const titleRef = useTemplateRef('title')

const isFailed = computed(
	() => props.selectedJob?.status === 'failed' || props.selectedJob?.status === 'interrupted',
)
const clampedProgress = computed(() => Math.max(0, Math.min(1, props.progress || 0)))
const progressPercent = computed(() => Math.round(clampedProgress.value * 100))

const statusState = computed<'complete' | 'failed' | 'paused' | 'running' | 'idle'>(() => {
	if (props.completing) return 'complete'
	if (isFailed.value || (props.hasAttention && !props.activeCount)) return 'failed'
	if (props.activeCount > 0) return props.selectedJob?.paused ? 'paused' : 'running'
	return 'idle'
})

defineExpose({ focus: () => trigger.value?.focus() })
</script>

<template>
	<div class="solara-dl-trigger-wrap relative flex shrink-0 items-center">
		<button
			ref="trigger"
			v-tooltip="formatMessage(messages.title)"
			type="button"
			class="solara-dl-trigger relative isolate flex h-8 max-w-[20rem] cursor-pointer items-center gap-2 overflow-hidden rounded-full border-0 px-3 text-sm font-medium leading-5 transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
			:class="{
				'is-expanded': expanded,
				'is-running': statusState === 'running',
				'is-paused': statusState === 'paused',
				'is-failed': statusState === 'failed',
				'is-complete': statusState === 'complete',
			}"
			:aria-label="formatMessage(expanded ? messages.hide : messages.show)"
			:aria-expanded="expanded"
			:aria-controls="expanded ? panelId : undefined"
			aria-haspopup="dialog"
			@keydown.esc.stop.prevent="$emit('close')"
		>
			<span class="solara-dl-icon-wrap relative flex size-4 shrink-0 items-center justify-center">
				<CheckIcon
					v-if="statusState === 'complete'"
					class="size-4 text-emerald-400"
					aria-hidden="true"
				/>
				<CircleAlertIcon
					v-else-if="statusState === 'failed'"
					class="size-4 text-amber-400"
					aria-hidden="true"
				/>
				<LoaderSpinnerIcon
					v-else-if="statusState === 'running'"
					class="size-4 text-sky-400 motion-safe:animate-spin"
					aria-hidden="true"
				/>
				<PauseIcon
					v-else-if="statusState === 'paused'"
					class="size-3.5 text-amber-400"
					aria-hidden="true"
				/>
				<DownloadIcon v-else class="size-3.5 solara-dl-idle-icon" aria-hidden="true" />
			</span>

			<template v-if="selectedJob && statusState !== 'idle'">
				<span
					ref="title"
					v-tooltip="truncatedTooltip(titleRef, selectedJob.title)"
					class="max-w-[9.5rem] truncate text-xs font-semibold text-contrast"
				>
					{{ selectedJob.title }}
				</span>
				<span
					v-if="rate && statusState === 'running'"
					class="hidden text-[11px] font-semibold tabular-nums solara-dl-rate sm:inline"
				>
					{{ rate }}
				</span>
				<span
					v-else-if="statusState === 'running' && progressPercent > 0"
					class="text-[11px] font-semibold tabular-nums solara-dl-rate"
				>
					{{ progressPercent }}%
				</span>
			</template>
			<template v-else>
				<span class="text-xs font-semibold tracking-tight solara-dl-label">Downloads</span>
			</template>

			<span
				v-if="activeCount > 0"
				class="solara-dl-count-badge flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[10px] font-bold leading-none tabular-nums"
			>
				{{ formatNumber(activeCount) }}
			</span>

			<CircleAlertIcon
				v-if="hasAttention && statusState !== 'failed'"
				v-tooltip="formatMessage(messages.attention)"
				class="size-3.5 shrink-0 text-amber-400"
				:aria-label="formatMessage(messages.attention)"
			/>

			<span
				v-if="statusState === 'running' || statusState === 'paused' || statusState === 'complete'"
				class="solara-dl-rail pointer-events-none absolute inset-x-0 bottom-0 h-[2.5px]"
				aria-hidden="true"
			>
				<span
					class="solara-dl-rail-fill block h-full origin-left transition-transform duration-300 ease-out"
					:class="{
						'is-complete': statusState === 'complete',
						'is-paused': statusState === 'paused',
					}"
					:style="{
						transform: `scaleX(${statusState === 'complete' ? 1 : clampedProgress})`,
					}"
				/>
			</span>
		</button>
	</div>
</template>

<style scoped>
.solara-dl-trigger {
	background: color-mix(in srgb, var(--surface-4) 78%, transparent);
	color: var(--color-base);
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--surface-5) 80%, transparent);
}

.solara-dl-trigger:hover,
.solara-dl-trigger.is-expanded {
	background: color-mix(in srgb, var(--surface-4) 96%, rgba(56, 189, 248, 0.14));
	box-shadow: inset 0 0 0 1px rgba(56, 189, 248, 0.35);
}

.solara-dl-trigger.is-running {
	background: linear-gradient(
		135deg,
		color-mix(in srgb, var(--surface-4) 88%, rgba(56, 189, 248, 0.18)),
		color-mix(in srgb, var(--surface-4) 88%, rgba(251, 191, 36, 0.12))
	);
	box-shadow: inset 0 0 0 1px rgba(56, 189, 248, 0.38);
}

.solara-dl-trigger.is-failed {
	background: color-mix(in srgb, var(--surface-4) 85%, rgba(245, 158, 11, 0.18));
	box-shadow: inset 0 0 0 1px rgba(245, 158, 11, 0.45);
}

.solara-dl-trigger.is-complete {
	background: color-mix(in srgb, var(--surface-4) 85%, rgba(16, 185, 129, 0.18));
	box-shadow: inset 0 0 0 1px rgba(16, 185, 129, 0.45);
}

.solara-dl-idle-icon {
	color: #38bdf8;
}

.solara-dl-label {
	color: var(--color-contrast);
}

.solara-dl-rate {
	color: #38bdf8;
}

.solara-dl-count-badge {
	background: linear-gradient(135deg, #38bdf8 0%, #fbbf24 100%);
	color: #071019;
}

.solara-dl-rail {
	background: color-mix(in srgb, var(--surface-5) 55%, transparent);
}

.solara-dl-rail-fill {
	background: linear-gradient(90deg, #38bdf8 0%, #60a5fa 50%, #fbbf24 100%);
}

.solara-dl-rail-fill.is-paused {
	background: #f59e0b;
}

.solara-dl-rail-fill.is-complete {
	background: #10b981;
}

:global(.light-mode) .solara-dl-idle-icon,
:global(.light-mode) .solara-dl-rate {
	color: #0284c7;
}
</style>
