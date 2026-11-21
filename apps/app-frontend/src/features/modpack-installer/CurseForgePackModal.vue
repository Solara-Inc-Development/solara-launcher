<script setup lang="ts">
import {
	DownloadIcon,
	ExternalIcon,
	PackageIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ref, watch } from 'vue'

import {
	type CurseForgeFile,
	type CurseForgeModpackDetail,
	type CurseForgeModpackSummary,
	getCurseForgeModpackDetail,
} from './curseforge-api'

dayjs.extend(relativeTime)

const props = defineProps<{
	pack: CurseForgeModpackSummary | null
	installingKey: string | null
}>()

const emit = defineEmits<{
	close: []
	install: [pack: CurseForgeModpackSummary, file: CurseForgeFile]
}>()

const loading = ref(false)
const error = ref<string | null>(null)
const detail = ref<CurseForgeModpackDetail | null>(null)

watch(
	() => props.pack?.id,
	async (modId) => {
		if (!modId) {
			detail.value = null
			error.value = null
			return
		}
		loading.value = true
		error.value = null
		detail.value = null
		try {
			const res = await getCurseForgeModpackDetail(modId)
			detail.value = res.modpack
		} catch (err) {
			error.value = err instanceof Error ? err.message : 'Failed to load modpack versions.'
		} finally {
			loading.value = false
		}
	},
	{ immediate: true },
)

function formatCount(n: number): string {
	if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
	if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
	return String(n)
}

function formatSize(bytes: number): string {
	if (!bytes) return ''
	if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
	return `${Math.max(1, Math.round(bytes / 1024))} KB`
}

function formatLoader(loader: string): string {
	const map: Record<string, string> = {
		fabric: 'Fabric',
		forge: 'Forge',
		neoforge: 'NeoForge',
		quilt: 'Quilt',
	}
	return map[loader.toLowerCase()] ?? loader
}

function openWebsite(url: string) {
	if (!url) return
	void openUrl(url)
}
</script>

<template>
	<Teleport to="body">
		<div
			v-if="pack"
			class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm"
			@click.self="emit('close')"
		>
			<div
				class="flex max-h-[84vh] w-full max-w-3xl flex-col overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-2 shadow-2xl"
				role="dialog"
				aria-modal="true"
				:aria-label="pack.name"
			>
				<!-- Modal Header -->
				<div class="flex items-start gap-4 border-0 border-b border-solid border-surface-5 bg-surface-3 p-5">
					<div
						class="flex size-16 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-4"
					>
						<img
							v-if="pack.logoUrl"
							:src="pack.logoUrl"
							:alt="pack.name"
							class="size-full object-cover"
						/>
						<PackageIcon v-else class="size-7 text-secondary" />
					</div>
					<div class="min-w-0 flex-1">
						<div class="flex items-center gap-2">
							<h2 class="m-0 truncate text-xl font-bold text-contrast">
								{{ pack.name }}
							</h2>
							<span
								class="shrink-0 rounded-md border border-solid border-surface-5 bg-surface-2 px-2 py-0.5 text-[11px] font-semibold text-secondary"
							>
								CurseForge
							</span>
						</div>
						<p class="m-0 mt-1 text-xs font-medium text-secondary">
							by {{ pack.author }} · {{ formatCount(pack.downloads) }} downloads · Updated
							{{ pack.dateModified ? dayjs(pack.dateModified).fromNow() : 'recently' }}
						</p>
						<p class="m-0 mt-2 text-sm leading-snug text-primary">
							{{ pack.summary }}
						</p>
					</div>
					<div class="flex shrink-0 items-center gap-2">
						<button
							v-if="pack.websiteUrl"
							type="button"
							class="flex size-9 cursor-pointer items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-2 text-secondary transition-colors hover:bg-surface-4 hover:text-contrast"
							title="Open on CurseForge"
							@click="openWebsite(pack.websiteUrl)"
						>
							<ExternalIcon class="size-4" />
						</button>
						<button
							type="button"
							class="flex size-9 cursor-pointer items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-2 text-secondary transition-colors hover:bg-surface-4 hover:text-contrast"
							aria-label="Close"
							@click="emit('close')"
						>
							<XIcon class="size-4" />
						</button>
					</div>
				</div>

				<!-- Modal Content -->
				<div class="flex-1 overflow-y-auto p-5">
					<div v-if="loading" class="flex items-center justify-center gap-2.5 py-12 text-sm text-secondary">
						<SpinnerIcon class="size-5 animate-spin" />
						<span>Loading CurseForge versions…</span>
					</div>

					<div
						v-else-if="error"
						class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-4 text-sm text-secondary"
					>
						{{ error }}
					</div>

					<div v-else-if="detail" class="flex flex-col gap-5">
						<!-- Screenshots -->
						<div v-if="detail.screenshots.length > 0" class="flex flex-col gap-2">
							<h3 class="m-0 text-xs font-bold uppercase tracking-wider text-secondary">
								Gallery
							</h3>
							<div class="flex gap-2.5 overflow-x-auto pb-1">
								<img
									v-for="(shot, i) in detail.screenshots.slice(0, 5)"
									:key="i"
									:src="shot.thumbnailUrl || shot.url"
									:alt="shot.title || pack.name"
									class="h-28 w-48 shrink-0 rounded-xl border border-solid border-surface-5 object-cover bg-surface-3"
								/>
							</div>
						</div>

						<!-- Versions List -->
						<div class="flex flex-col gap-2.5">
							<div class="flex items-center justify-between">
								<h3 class="m-0 text-xs font-bold uppercase tracking-wider text-secondary">
									Available Versions ({{ detail.files.length }})
								</h3>
							</div>

							<div v-if="detail.files.length === 0" class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-4 text-sm text-secondary">
								No downloadable modpack files were returned for this project.
							</div>

							<div v-else class="flex flex-col gap-2">
								<div
									v-for="file in detail.files"
									:key="file.id"
									class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-solid border-surface-5 bg-surface-3 px-4 py-3"
								>
									<div class="min-w-0 flex-1">
										<div class="flex flex-wrap items-center gap-2">
											<span class="truncate text-sm font-semibold text-contrast">
												{{ file.displayName || file.fileName }}
											</span>
											<span
												class="rounded-md px-1.5 py-0.5 text-[11px] font-semibold capitalize"
												:class="
													file.releaseType === 'release'
														? 'bg-surface-4 text-contrast'
														: 'bg-surface-2 text-secondary'
												"
											>
												{{ file.releaseType }}
											</span>
										</div>
										<div class="mt-1 flex flex-wrap items-center gap-2 text-xs text-secondary">
											<span v-if="file.gameVersions.length > 0">
												MC {{ file.gameVersions.slice(0, 3).join(', ') }}
											</span>
											<span v-if="file.loaders.length > 0">
												· {{ file.loaders.map(formatLoader).join(', ') }}
											</span>
											<span v-if="file.fileLength > 0">
												· {{ formatSize(file.fileLength) }}
											</span>
											<span v-if="file.fileDate">
												· {{ dayjs(file.fileDate).fromNow() }}
											</span>
										</div>
									</div>

									<Button
										type="colored"
										color="brand"
										size="sm"
										:disabled="file.restricted || !file.downloadUrl || installingKey !== null"
										@click="emit('install', detail, file)"
									>
										<SpinnerIcon
											v-if="installingKey === `cf:${pack.id}:${file.id}`"
											class="size-4 animate-spin"
										/>
										<DownloadIcon v-else class="size-4" />
										<span>
											{{
												file.restricted || !file.downloadUrl
													? 'Restricted'
													: installingKey === `cf:${pack.id}:${file.id}`
														? 'Installing…'
														: 'Install'
											}}
										</span>
									</Button>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	</Teleport>
</template>
