<template>
	<Transition name="splash-fade" @after-leave="onAfterLeave">
		<div v-if="!doneLoading" class="splash-screen" :class="`${theme.active}-mode`">
			<div class="splash-ambient" data-tauri-drag-region />
			<div class="splash-content" data-tauri-drag-region>
				<!-- Animated Solara Logo -->
				<div class="splash-logo-stage">
					<div class="splash-logo-glow" />
					<div class="splash-logo-float">
						<SolaraIcon class="size-[5.25rem] block" />
					</div>
				</div>

				<!-- Brand Title -->
				<div class="splash-brand-title">Solara-Inc</div>

				<!-- Smooth Progress Bar -->
				<div class="splash-progress-track" role="progressbar" :aria-valuenow="Math.round(progress)" aria-valuemin="0" aria-valuemax="100">
					<div class="splash-progress-fill" :style="{ width: `${Math.max(4, Math.min(100, progress))}%` }">
						<div class="splash-progress-shimmer" />
					</div>
				</div>

				<!-- Live Staged Status Text -->
				<p class="splash-status-text">
					{{ statusMessage }}
				</p>
			</div>
		</div>
	</Transition>
</template>

<script setup>
import { SolaraIcon } from '@modrinth/assets'
import { defineMessages, injectLoadingState, useVIntl } from '@modrinth/ui'
import { listen } from '@tauri-apps/api/event'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { useAppEvent } from '@/composables/use-app-event'
import { useTheme } from '@/composables/use-theme.ts'
import { debugStartup } from '@/helpers/startup-debug'

const theme = useTheme()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	movingAppDirectory: {
		id: 'app.splash.moving-app-directory',
		defaultMessage: 'Moving your app directory...',
	},
})

const doneLoading = ref(false)
const loadingProgress = ref(0)
const directoryMoveFraction = ref(null)
const installingUpdateVersion = ref(null)

let progressTimer = null
let unlistenUpdateStatus = null

const progress = computed(() => {
	if (installingUpdateVersion.value) return 100
	return directoryMoveFraction.value === null
		? Math.min(loadingProgress.value, 100)
		: directoryMoveFraction.value * 100
})

const statusMessage = computed(() => {
	if (installingUpdateVersion.value) {
		return `Installing Solara Launcher ${installingUpdateVersion.value}...`
	}
	if (directoryMoveFraction.value !== null) {
		return formatMessage(messages.movingAppDirectory)
	}
	const p = loadingProgress.value
	if (p < 18) return 'Starting Solara Launcher...'
	if (p < 38) return 'Checking for updates...'
	if (p < 60) return 'Verifying account...'
	if (p < 82) return 'Loading instances...'
	if (p < 96) return 'Preparing workspace...'
	return 'Ready!'
})

const MIN_DISPLAY_MS = 1150
const mountedAt = Date.now()

const loading = injectLoadingState()

onMounted(async () => {
	debugStartup('Splash mounted')
	try {
		unlistenUpdateStatus = await listen('app-update-status', (event) => {
			const payload = event.payload
			if (payload && (payload.stage === 'installing' || payload.state === 'installing')) {
				installingUpdateVersion.value =
					payload.publicVersion ?? payload.version ?? 'latest'
			}
		})
	} catch {
		// Non-Tauri preview context
	}
})

onBeforeUnmount(() => {
	if (progressTimer) {
		clearTimeout(progressTimer)
		progressTimer = null
	}
	if (unlistenUpdateStatus) {
		unlistenUpdateStatus()
		unlistenUpdateStatus = null
	}
})

function onAfterLeave() {
	debugStartup('Splash fade completed', { displayedMs: Date.now() - mountedAt })
	loading.setEnabled(true)
}

function stepLoadingProgress() {
	if (doneLoading.value) return
	if (loadingProgress.value < 92) {
		// Smooth ease-out climb across startup stages
		const increment = loadingProgress.value < 45 ? 3.5 : loadingProgress.value < 75 ? 2.4 : 1.2
		loadingProgress.value = Math.min(92, loadingProgress.value + increment)
		progressTimer = setTimeout(stepLoadingProgress, 38)
	}
}

watch(
	[loading.barEnabled, loading.pending],
	([barEnabled, pending]) => {
		debugStartup('Splash loading state changed', { barEnabled, pending })
		if (barEnabled) {
			return
		}

		if (pending) {
			if (!progressTimer && loadingProgress.value < 92) {
				stepLoadingProgress()
			}
			return
		}

		if (progressTimer) {
			clearTimeout(progressTimer)
			progressTimer = null
		}

		const elapsed = Date.now() - mountedAt
		const delay = Math.max(0, MIN_DISPLAY_MS - elapsed)
		debugStartup('Splash dismissal scheduled', { delayMs: delay, displayedMs: elapsed })

		// Smoothly glide to 100% right before fading out
		const finishStep = () => {
			if (loadingProgress.value < 100) {
				loadingProgress.value = Math.min(100, loadingProgress.value + 4)
				progressTimer = setTimeout(finishStep, 28)
			}
		}
		finishStep()

		setTimeout(() => {
			if (loading.pending.value || installingUpdateVersion.value) {
				debugStartup('Splash dismissal deferred: active work or installing update')
				return
			}
			loadingProgress.value = 100
			doneLoading.value = true
			debugStartup('Splash fade started', { displayedMs: Date.now() - mountedAt })
		}, delay)
	},
	{ immediate: true },
)

useAppEvent('loading', (e) => {
	if (e.event.type === 'directory_move') {
		directoryMoveFraction.value = e.fraction ?? null
	}
})
</script>

<style scoped lang="scss">
.splash-screen {
	position: fixed;
	inset: 0;
	z-index: 10000;
	background: var(--color-bg);
	display: flex;
	align-items: center;
	justify-content: center;
	user-select: none;
	overflow: hidden;
}

.splash-ambient {
	position: absolute;
	inset: 0;
	pointer-events: none;
	background: radial-gradient(
		44% 44% at 50% 45%,
		color-mix(in srgb, var(--color-brand) 12%, transparent) 0%,
		transparent 72%
	);
}

.splash-content {
	position: relative;
	z-index: 2;
	display: flex;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: 1.15rem;
	padding: 2rem;
	width: 100%;
	max-width: 24rem;
}

.splash-logo-stage {
	position: relative;
	width: 6.75rem;
	height: 6.75rem;
	display: flex;
	align-items: center;
	justify-content: center;
}

.splash-logo-glow {
	position: absolute;
	inset: -0.5rem;
	border-radius: 9999px;
	background: radial-gradient(
		circle at center,
		rgba(61, 214, 245, 0.28) 0%,
		rgba(255, 181, 90, 0.14) 50%,
		transparent 74%
	);
	filter: blur(20px);
	animation: splash-glow-breathe 3.6s ease-in-out infinite;
}

.splash-logo-float {
	position: relative;
	z-index: 2;
	animation: splash-logo-bob 3.2s ease-in-out infinite;
	filter: drop-shadow(0 12px 26px rgba(61, 214, 245, 0.28));
}

.splash-brand-title {
	font-size: 1.5rem;
	font-weight: 800;
	letter-spacing: 0.12em;
	color: var(--color-contrast);
	line-height: 1.15;
}

.splash-progress-track {
	position: relative;
	width: 100%;
	max-width: 18rem;
	height: 0.375rem;
	border-radius: 9999px;
	background: color-mix(in srgb, var(--surface-4) 85%, transparent);
	overflow: hidden;
	margin-top: 0.25rem;
}

.splash-progress-fill {
	position: relative;
	height: 100%;
	border-radius: 9999px;
	background: linear-gradient(90deg, #3dd6f5 0%, #58e0ff 55%, #ffb55a 100%);
	transition: width 140ms ease-out;
	overflow: hidden;
}

.splash-progress-shimmer {
	position: absolute;
	inset: 0;
	background: linear-gradient(
		90deg,
		transparent 0%,
		rgba(255, 255, 255, 0.45) 50%,
		transparent 100%
	);
	animation: splash-shimmer 1.4s ease-in-out infinite;
}

.splash-status-text {
	margin: 0;
	min-height: 1.25rem;
	font-size: 0.8125rem;
	font-weight: 600;
	color: var(--color-secondary);
	letter-spacing: 0.01em;
	text-align: center;
}

.splash-fade-leave-active {
	transition: opacity 0.32s ease-in-out;
}

.splash-fade-leave-to {
	opacity: 0;
}

@keyframes splash-logo-bob {
	0%,
	100% {
		transform: translateY(0px) scale(1);
	}
	50% {
		transform: translateY(-6px) scale(1.03);
	}
}

@keyframes splash-glow-breathe {
	0%,
	100% {
		opacity: 0.72;
		transform: scale(0.95);
	}
	50% {
		opacity: 1;
		transform: scale(1.08);
	}
}

@keyframes splash-shimmer {
	0% {
		transform: translateX(-100%);
	}
	100% {
		transform: translateX(100%);
	}
}

@media (prefers-reduced-motion: reduce) {
	.splash-logo-glow,
	.splash-logo-float,
	.splash-progress-shimmer {
		animation: none;
	}
}
</style>
