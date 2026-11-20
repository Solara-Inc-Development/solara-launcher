<script setup>
import { GaugeIcon } from '@modrinth/assets'
import { defineMessages, injectNotificationManager, Slider, useVIntl } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { ref, watch } from 'vue'

import { appSettingsKeys, get, set } from '@/helpers/settings.ts'

import AppDirectorySettings from './app-directory/index.vue'
import ContentStorageSettings from './content-storage.vue'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const settings = ref(await get())

const messages = defineMessages({
	maximumConcurrentDownloadsTitle: {
		id: 'app.settings.resource-management.maximum-concurrent-downloads.title',
		defaultMessage: 'Max concurrent downloads',
	},
	maximumConcurrentDownloadsDescription: {
		id: 'app.settings.resource-management.maximum-concurrent-downloads.description',
		defaultMessage: 'Files downloaded at once. Lower this if your internet connection drops.',
	},
	maximumConcurrentWritesTitle: {
		id: 'app.settings.resource-management.maximum-concurrent-writes.title',
		defaultMessage: 'Max concurrent writes',
	},
	maximumConcurrentWritesDescription: {
		id: 'app.settings.resource-management.maximum-concurrent-writes.description',
		defaultMessage: 'Files written to disk at once. Lower this if you see disk I/O errors.',
	},
})

const limitsMutation = useMutation({
	networkMode: 'always',
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (limits) => set({ ...(await get()), ...limits }),
	onError: handleError,
	onSettled: () => queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
})

watch(
	() => ({
		max_concurrent_downloads: settings.value.max_concurrent_downloads,
		max_concurrent_writes: settings.value.max_concurrent_writes,
	}),
	(limits) => limitsMutation.mutate(limits),
)
</script>

<template>
	<div class="flex flex-col gap-5">
		<ContentStorageSettings />
		<AppDirectorySettings />

		<section class="solara-perf-card flex flex-col gap-5 rounded-2xl p-5">
			<div class="flex flex-wrap items-center justify-between gap-3">
				<div class="flex items-center gap-3">
					<div
						class="solara-perf-icon flex size-10 shrink-0 items-center justify-center rounded-xl"
						aria-hidden="true"
					>
						<GaugeIcon class="size-5 text-sky-400" />
					</div>
					<div class="flex flex-col gap-0.5">
						<h2 class="m-0 text-base font-bold text-contrast">Transfer &amp; disk limits</h2>
						<p class="m-0 text-xs font-medium text-secondary">
							Tune download concurrency and parallel disk writes for your connection &amp; SSD.
						</p>
					</div>
				</div>
				<span class="solara-restart-chip rounded-full px-2.5 py-1 text-[11px] font-semibold">
					Applies after restart
				</span>
			</div>

			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				<div class="solara-slider-box flex flex-col gap-2.5 rounded-xl p-4">
					<div class="flex items-center justify-between gap-2">
						<h3 class="m-0 text-sm font-semibold text-contrast">
							{{ formatMessage(messages.maximumConcurrentDownloadsTitle) }}
						</h3>
						<span class="solara-value-pill is-cyan rounded-full px-2.5 py-0.5 text-xs font-bold tabular-nums">
							{{ settings.max_concurrent_downloads }}
						</span>
					</div>
					<Slider
						id="max-downloads"
						v-model="settings.max_concurrent_downloads"
						:min="1"
						:max="10"
						:step="1"
					/>
					<p class="m-0 text-xs leading-relaxed text-secondary">
						{{ formatMessage(messages.maximumConcurrentDownloadsDescription) }}
					</p>
				</div>

				<div class="solara-slider-box flex flex-col gap-2.5 rounded-xl p-4">
					<div class="flex items-center justify-between gap-2">
						<h3 class="m-0 text-sm font-semibold text-contrast">
							{{ formatMessage(messages.maximumConcurrentWritesTitle) }}
						</h3>
						<span class="solara-value-pill is-amber rounded-full px-2.5 py-0.5 text-xs font-bold tabular-nums">
							{{ settings.max_concurrent_writes }}
						</span>
					</div>
					<Slider
						id="max-writes"
						v-model="settings.max_concurrent_writes"
						:min="1"
						:max="50"
						:step="1"
					/>
					<p class="m-0 text-xs leading-relaxed text-secondary">
						{{ formatMessage(messages.maximumConcurrentWritesDescription) }}
					</p>
				</div>
			</div>
		</section>
	</div>
</template>

<style scoped>
.solara-perf-card {
	background: color-mix(in srgb, var(--surface-3) 86%, transparent);
	box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--surface-5) 55%, transparent);
}

:global(.light-mode) .solara-perf-card {
	background: #ffffff;
	box-shadow:
		0 2px 10px rgba(15, 23, 42, 0.04),
		inset 0 0 0 1px rgba(15, 23, 42, 0.07);
}

.solara-perf-icon {
	background: linear-gradient(135deg, rgba(56, 189, 248, 0.18), rgba(251, 191, 36, 0.14));
}

.solara-restart-chip {
	background: color-mix(in srgb, var(--surface-2) 85%, transparent);
	color: var(--color-secondary);
}

:global(.light-mode) .solara-restart-chip {
	background: #f1f5f9;
}

.solara-slider-box {
	background: color-mix(in srgb, var(--surface-2) 72%, transparent);
}

:global(.light-mode) .solara-slider-box {
	background: #f4f7fb;
}

.solara-value-pill.is-cyan {
	background: rgba(56, 189, 248, 0.16);
	color: #38bdf8;
}

.solara-value-pill.is-amber {
	background: rgba(251, 191, 36, 0.16);
	color: #fbbf24;
}

:global(.light-mode) .solara-value-pill.is-cyan {
	background: rgba(2, 132, 199, 0.12);
	color: #0284c7;
}

:global(.light-mode) .solara-value-pill.is-amber {
	background: rgba(217, 119, 6, 0.12);
	color: #d97706;
}
</style>
