<script setup lang="ts">
import { DownloadIcon, RefreshCwIcon, XIcon } from '@modrinth/assets'
import { Button, NewModal } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { getVersionChangelog } from '@/helpers/version-history'
import { appUpdateState, installAvailableAppUpdate } from '@/providers/app-update'

export interface SolaraUpdateNotification {
	version: string
	currentVersion?: string
	notes?: string | null
	publishedAt?: string | null
}

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const updateDetails = ref<SolaraUpdateNotification | null>(null)

const displayVersion = computed(() => {
	const raw = updateDetails.value?.version ?? appUpdateState.availableUpdate.value?.version ?? ''
	if (!raw) return 'Latest'
	return raw.startsWith('V-') || raw.startsWith('v') ? raw : `V-${raw}`
})

const changelogData = computed(() => {
	const base = getVersionChangelog(displayVersion.value)
	const rawNotes = updateDetails.value?.notes
	if (!rawNotes || typeof rawNotes !== 'string') return base

	const rawLines = rawNotes.split('\n').map((l) => l.trim())
	const features: string[] = []
	const fixes: string[] = []
	const future: string[] = []

	let currentSection: 'features' | 'fixes' | 'future' | null = null

	for (const line of rawLines) {
		if (!line) continue
		const headingMatch = line.match(/^#{1,4}\s+(.+)$/)
		if (headingMatch) {
			const heading = headingMatch[1].toLowerCase()
			if (heading.includes('fix') || heading.includes('bug') || heading.includes('issue')) {
				currentSection = 'fixes'
			} else if (
				heading.includes('future') ||
				heading.includes('next') ||
				heading.includes('plan') ||
				heading.includes('upcoming')
			) {
				currentSection = 'future'
			} else if (
				heading.includes('feature') ||
				heading.includes('added') ||
				heading.includes('what') ||
				heading.includes('new') ||
				heading.includes('change')
			) {
				currentSection = 'features'
			} else {
				currentSection = null
			}
			continue
		}

		if (!line.startsWith('-') && !line.startsWith('*') && !line.startsWith('•')) continue
		const clean = line.replace(/^[-*•]\s*/, '').trim()
		if (!clean) continue

		if (currentSection === 'fixes') {
			fixes.push(clean)
		} else if (currentSection === 'future') {
			future.push(clean)
		} else if (currentSection === 'features') {
			features.push(clean)
		} else {
			const lower = clean.toLowerCase()
			if (
				lower.includes('fix') ||
				lower.includes('resolve') ||
				lower.includes('correct') ||
				lower.includes('patch') ||
				lower.includes('error')
			) {
				fixes.push(clean)
			} else if (
				lower.includes('future') ||
				lower.includes('plan') ||
				lower.includes('remain') ||
				lower.includes('coming') ||
				lower.includes('next')
			) {
				future.push(clean)
			} else {
				features.push(clean)
			}
		}
	}

	if (features.length === 0 && fixes.length === 0 && future.length === 0) return base

	return {
		...base,
		featuresAdded: features.length > 0 ? features : base.featuresAdded,
		issuesFixed: fixes.length > 0 ? fixes : base.issuesFixed,
		futureUpdates: future.length > 0 ? future : base.futureUpdates,
	}
})

const isDownloading = computed(() => appUpdateState.downloading.value)
const isDownloadedReady = computed(() => appUpdateState.finishedDownloading.value)
const isRestarting = computed(() => appUpdateState.restarting.value)
const isInstallQueued = computed(() => appUpdateState.installQueued.value)
const downloadPercent = computed(() => appUpdateState.downloadPercent.value)
const isBusy = computed(() => isRestarting.value || isInstallQueued.value)

function show(details?: SolaraUpdateNotification) {
	if (details) {
		updateDetails.value = details
	}
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

async function notifyForVersion(_version?: string, _pendingVersion?: string | null) {
	// Startup update check triggers show() when an update is available on GitHub Releases CDN.
}

async function handleUpdateNow() {
	await installAvailableAppUpdate()
	if (!appUpdateState.downloading.value) {
		hide()
	}
}

defineExpose({ show, hide, notifyForVersion })
</script>

<template>
	<NewModal ref="modal" header="New Version Available" max-width="36rem">
		<div class="flex flex-col gap-4">
			<div class="flex flex-col gap-1">
				<h3 class="text-xl font-extrabold text-contrast m-0 flex items-center gap-2">
					<span class="size-2.5 rounded-full bg-brand animate-pulse" />
					<span v-if="isRestarting">Installing Latest Version...</span>
					<span v-else-if="isDownloadedReady">Update Ready — {{ displayVersion }}</span>
					<span v-else>Update Available — {{ displayVersion }}</span>
				</h3>
				<div class="flex flex-wrap items-center gap-2">
					<span class="text-base text-brand font-semibold">{{ displayVersion }} from GitHub Releases</span>
					<span
						v-if="isDownloadedReady"
						class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-bold bg-emerald-500/15 text-emerald-400"
					>
						Ready — click Update Now when you want to restart
					</span>
					<span
						v-else-if="isDownloading"
						class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-bold bg-brand/15 text-brand tabular-nums"
					>
						Downloading in background... {{ downloadPercent }}%
					</span>
				</div>
				<div
					v-if="isDownloading"
					class="w-full h-1.5 rounded-full bg-surface-4 overflow-hidden mt-1.5"
				>
					<div
						class="h-full rounded-full bg-brand transition-all duration-150"
						:style="{ width: `${Math.max(4, downloadPercent)}%` }"
					/>
				</div>
			</div>

			<div class="flex flex-col gap-3 max-h-[20rem] overflow-y-auto pr-1">
				<div class="rounded-xl border border-solid border-surface-5 bg-surface-2 p-3.5">
					<h4 class="text-contrast font-bold text-sm mb-2 flex items-center gap-2 m-0">
						Features Added
					</h4>
					<ul class="text-sm text-primary space-y-1.5 pl-4 list-disc m-0 mt-2">
						<li v-for="item in changelogData.featuresAdded" :key="item">{{ item }}</li>
					</ul>
				</div>

				<div class="rounded-xl border border-solid border-surface-5 bg-surface-2 p-3.5">
					<h4 class="text-contrast font-bold text-sm mb-2 flex items-center gap-2 m-0">
						Issues Fixed
					</h4>
					<ul class="text-sm text-primary space-y-1.5 pl-4 list-disc m-0 mt-2">
						<li v-for="item in changelogData.issuesFixed" :key="item">{{ item }}</li>
					</ul>
				</div>

				<div class="rounded-xl border border-solid border-surface-5 bg-surface-2 p-3.5">
					<h4 class="text-contrast font-bold text-sm mb-2 flex items-center gap-2 m-0">
						Future Updates Implementation
					</h4>
					<ul class="text-sm text-primary space-y-1.5 pl-4 list-disc m-0 mt-2">
						<li v-for="item in changelogData.futureUpdates" :key="item">{{ item }}</li>
					</ul>
				</div>
			</div>

			<div class="flex items-center justify-end gap-2 pt-2 border-0 border-t border-solid border-surface-5">
				<Button type="outlined" :disabled="isRestarting" @click="hide">
					<XIcon />
					I'll do it later
				</Button>
				<Button color="brand" :disabled="isBusy" @click="handleUpdateNow">
					<RefreshCwIcon v-if="isBusy" class="animate-spin" />
					<DownloadIcon v-else />
					<span v-if="isRestarting">Installing &amp; Restarting...</span>
					<span v-else-if="isInstallQueued">Auto-installing at 100% ({{ downloadPercent }}%)...</span>
					<span v-else>Update Now</span>
				</Button>
			</div>
		</div>
	</NewModal>
</template>
