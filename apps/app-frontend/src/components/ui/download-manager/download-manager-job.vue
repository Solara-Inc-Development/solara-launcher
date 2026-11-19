<script setup lang="ts">
import {
	BoxIcon,
	CheckIcon,
	CopyIcon,
	DownloadIcon,
	MinecraftServerIcon,
	PauseIcon,
	PlayIcon,
	TrashIcon,
	UpdatedIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	BulletDivider,
	Button,
	commonMessages,
	defineMessages,
	IconButton,
	truncatedTooltip,
	useGeneratedIconSrc,
	useRelativeTime,
	useServerIcon,
	useVIntl,
} from '@modrinth/ui'
import { computed, useTemplateRef } from 'vue'
import { RouterLink } from 'vue-router'

import type { DownloadManagerJob } from './use-download-manager'

const props = defineProps<{ job: DownloadManagerJob }>()
const { config: serverIconConfig, image: serverIconImage } = useServerIcon(
	() => props.job.serverId ?? '',
)
const serverIconGeneratedSrc = useGeneratedIconSrc(serverIconConfig)
const serverIcon = computed(() => serverIconGeneratedSrc.value ?? serverIconImage.value)
const displayIcon = computed(() =>
	props.job.serverId ? (serverIcon.value ?? MinecraftServerIcon) : props.job.iconUrl,
)
defineEmits<{
	retry: [id: string]
	cancel: [id: string]
	togglePause: [id: string]
	dismiss: [id: string]
	copyDetails: [id: string]
	open: []
}>()

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime({ style: 'narrow', numeric: 'always' })
const titleRef = useTemplateRef('title')
const textRef = useTemplateRef('text')
const messages = defineMessages({
	pause: { id: 'app.download-manager.pause', defaultMessage: 'Pause installation' },
	resume: { id: 'app.download-manager.resume', defaultMessage: 'Resume installation' },
	retry: { id: 'app.action-bar.install.retry', defaultMessage: 'Retry' },
	copyDetails: { id: 'app.action-bar.install.copy-details', defaultMessage: 'Copy details' },
	copied: { id: 'app.action-bar.install.copied-details', defaultMessage: 'Copied' },
	dismiss: { id: 'app.action-bar.install.dismiss', defaultMessage: 'Dismiss' },
	cancel: { id: 'app.download-manager.cancel', defaultMessage: 'Cancel installation' },
	remove: { id: 'app.download-manager.remove', defaultMessage: 'Remove from history' },
	openInstance: { id: 'app.action-bar.install.open-instance', defaultMessage: 'Open instance' },
	eta: {
		id: 'app.download-manager.eta',
		defaultMessage: 'Estimated time remaining for this download',
	},
})

const needsAttention = computed(
	() => props.job.status === 'failed' || props.job.status === 'interrupted',
)
const complete = computed(() => props.job.status === 'succeeded' || props.job.status === 'canceled')
const cancelLabel = computed(() =>
	formatMessage(
		needsAttention.value
			? messages.dismiss
			: props.job.kind === 'external-file' || props.job.kind === 'debug-export'
				? commonMessages.cancelButton
				: messages.cancel,
	),
)
const instanceLink = computed(() =>
	props.job.status === 'succeeded' && props.job.instanceId
		? `/instance/${encodeURIComponent(props.job.instanceId)}`
		: undefined,
)

const statusBadge = computed(() => {
	if (props.job.status === 'running') {
		if (props.job.canceling) return { label: 'Canceling', tone: 'muted' }
		if (props.job.paused) return { label: 'Paused', tone: 'amber' }
		return { label: 'Downloading', tone: 'cyan' }
	}
	if (props.job.status === 'failed') return { label: 'Failed', tone: 'red' }
	if (props.job.status === 'interrupted') return { label: 'Interrupted', tone: 'amber' }
	if (props.job.status === 'succeeded') return { label: 'Done', tone: 'emerald' }
	return { label: 'Canceled', tone: 'muted' }
})

const clampedProgress = computed(() => Math.max(0, Math.min(1, props.job.progress || 0)))
const isWaiting = computed(
	() => props.job.waiting && !props.job.paused && !props.job.canceling,
)
</script>

<template>
	<div
		class="solara-dl-card flex flex-col gap-2.5 rounded-2xl p-3 transition-colors duration-150"
		:class="{ 'is-complete': complete, 'is-attention': needsAttention }"
		:aria-busy="job.busy || undefined"
	>
		<div class="flex items-start justify-between gap-2.5">
			<component
				:is="instanceLink ? RouterLink : 'div'"
				:to="instanceLink"
				:aria-label="
					instanceLink ? `${formatMessage(messages.openInstance)}: ${job.title}` : undefined
				"
				class="flex min-w-0 flex-1 items-center gap-3 rounded-xl text-inherit no-underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-sky-400"
				@click="instanceLink && $emit('open')"
			>
				<Avatar
					v-if="displayIcon"
					:src="displayIcon"
					size="40px"
					no-shadow
					class="!rounded-xl shrink-0"
				/>
				<div
					v-else
					class="solara-dl-card-icon flex size-10 shrink-0 items-center justify-center rounded-xl"
				>
					<component
						:is="
							job.kind === 'shared-instance-publish'
								? UploadIcon
								: job.kind === 'external-file' || job.kind === 'debug-export'
									? DownloadIcon
									: BoxIcon
						"
						class="size-5 text-sky-400"
						aria-hidden="true"
					/>
				</div>

				<div class="flex min-w-0 flex-1 flex-col gap-1">
					<div class="flex min-w-0 items-center gap-1.5">
						<span
							ref="title"
							v-tooltip="truncatedTooltip(titleRef, job.title)"
							class="truncate text-sm font-semibold leading-5 text-contrast"
						>
							{{ job.title }}
						</span>
						<span
							class="solara-dl-status-pill shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-bold leading-none"
							:class="`tone-${statusBadge.tone}`"
						>
							{{ statusBadge.label }}
						</span>
					</div>

					<div
						class="flex min-w-0 items-center gap-1 text-xs font-medium leading-4"
						:class="
							needsAttention
								? job.status === 'failed'
									? 'text-red'
									: 'text-amber-400'
								: 'text-secondary'
						"
					>
						<template v-if="job.taskType && job.taskType !== job.text">
							<span class="shrink-0 truncate text-secondary">{{ job.taskType }}</span>
							<BulletDivider class="shrink-0 text-secondary" />
						</template>
						<template v-if="complete && job.finishedAt">
							<time :datetime="job.finishedAt" class="shrink-0">
								{{ formatRelativeTime(job.finishedAt) }}
							</time>
							<BulletDivider class="shrink-0" />
						</template>
						<span ref="text" v-tooltip="truncatedTooltip(textRef, job.text)" class="truncate">
							{{ job.text }}
						</span>
					</div>
				</div>
			</component>

			<div class="flex shrink-0 items-center gap-0.5 pt-0.5">
				<IconButton
					v-if="job.canPause"
					v-tooltip="formatMessage(job.paused ? messages.resume : messages.pause)"
					:label="formatMessage(job.paused ? messages.resume : messages.pause)"
					type="quiet"
					size="sm"
					:disabled="job.busy"
					@click="$emit('togglePause', job.id)"
				>
					<PlayIcon v-if="job.paused" />
					<PauseIcon v-else />
				</IconButton>
				<IconButton
					v-if="job.canCancel || job.canceling || needsAttention"
					v-tooltip="cancelLabel"
					:label="cancelLabel"
					type="quiet"
					size="sm"
					:disabled="job.busy || job.canceling"
					@click="needsAttention ? $emit('dismiss', job.id) : $emit('cancel', job.id)"
				>
					<XIcon class="text-secondary" />
				</IconButton>
				<IconButton
					v-else-if="complete"
					v-tooltip="formatMessage(messages.remove)"
					:label="formatMessage(messages.remove)"
					type="quiet"
					size="sm"
					:disabled="job.busy"
					@click="$emit('dismiss', job.id)"
				>
					<TrashIcon />
				</IconButton>
			</div>
		</div>

		<!-- Solara Gradient Progress Track -->
		<div v-if="job.status === 'running'" class="flex flex-col gap-1.5">
			<div
				role="progressbar"
				:aria-label="`${job.title}: ${job.text}`"
				:aria-valuenow="Math.round(clampedProgress * 100)"
				aria-valuemin="0"
				aria-valuemax="100"
				class="solara-dl-progress-track relative h-1.5 w-full overflow-hidden rounded-full"
			>
				<div
					v-if="isWaiting"
					class="solara-dl-progress-indeterminate absolute inset-y-0 w-2/5 rounded-full"
				/>
				<div
					v-else
					class="solara-dl-progress-fill h-full rounded-full transition-all duration-300 ease-out"
					:class="{ 'is-paused': job.paused }"
					:style="{ width: `${Math.max(4, clampedProgress * 100)}%` }"
				/>
			</div>
			<div
				v-if="job.progressLabel || job.eta"
				class="flex items-center justify-between gap-2 text-[11px] font-semibold leading-4 text-secondary tabular-nums"
			>
				<span>{{ job.progressLabel }}</span>
				<span v-if="job.eta" v-tooltip="formatMessage(messages.eta)">
					{{ job.eta }}
				</span>
			</div>
		</div>

		<div v-if="job.canRetry || job.canCopyDetails" class="flex flex-wrap items-center gap-1.5 pt-0.5">
			<Button
				v-if="job.canCopyDetails"
				type="outlined"
				size="xs"
				:disabled="job.busy"
				@click="$emit('copyDetails', job.id)"
			>
				<CheckIcon v-if="job.copied" />
				<CopyIcon v-else />
				{{ formatMessage(job.copied ? messages.copied : messages.copyDetails) }}
			</Button>
			<Button
				v-if="job.canRetry"
				type="colored"
				color="brand"
				size="xs"
				:disabled="job.busy"
				@click="$emit('retry', job.id)"
			>
				<UpdatedIcon />
				{{ formatMessage(messages.retry) }}
			</Button>
		</div>
	</div>
</template>

<style scoped>
.solara-dl-card {
	background: color-mix(in srgb, var(--surface-4) 72%, transparent);
}

.solara-dl-card:hover {
	background: color-mix(in srgb, var(--surface-4) 95%, transparent);
}

.solara-dl-card.is-attention {
	background: color-mix(in srgb, var(--surface-4) 78%, rgba(245, 158, 11, 0.1));
}

:global(.light-mode) .solara-dl-card {
	background: #f4f7fb;
}

:global(.light-mode) .solara-dl-card:hover {
	background: #ebf1f8;
}

.solara-dl-card-icon {
	background: linear-gradient(135deg, rgba(56, 189, 248, 0.16), rgba(251, 191, 36, 0.12));
}

.solara-dl-status-pill.tone-cyan {
	background: rgba(56, 189, 248, 0.15);
	color: #38bdf8;
}

.solara-dl-status-pill.tone-emerald {
	background: rgba(16, 185, 129, 0.15);
	color: #34d399;
}

.solara-dl-status-pill.tone-amber {
	background: rgba(245, 158, 11, 0.16);
	color: #fbbf24;
}

.solara-dl-status-pill.tone-red {
	background: rgba(239, 68, 68, 0.16);
	color: #f87171;
}

.solara-dl-status-pill.tone-muted {
	background: color-mix(in srgb, var(--surface-5) 70%, transparent);
	color: var(--color-secondary);
}

:global(.light-mode) .solara-dl-status-pill.tone-cyan {
	color: #0284c7;
}

:global(.light-mode) .solara-dl-status-pill.tone-emerald {
	color: #059669;
}

:global(.light-mode) .solara-dl-status-pill.tone-amber {
	color: #d97706;
}

.solara-dl-progress-track {
	background: color-mix(in srgb, var(--surface-2) 90%, transparent);
}

.solara-dl-progress-fill {
	background: linear-gradient(90deg, #38bdf8 0%, #60a5fa 50%, #fbbf24 100%);
}

.solara-dl-progress-fill.is-paused {
	background: #f59e0b;
}

.solara-dl-progress-indeterminate {
	background: linear-gradient(90deg, transparent 0%, #38bdf8 45%, #fbbf24 75%, transparent 100%);
	animation: solara-dl-slide 1.3s ease-in-out infinite;
}

@keyframes solara-dl-slide {
	0% {
		left: -40%;
	}
	100% {
		left: 100%;
	}
}
</style>
