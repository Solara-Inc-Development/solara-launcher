<script setup lang="ts">
import {
	CheckIcon,
	RefreshCwIcon,
	SaveIcon,
	SearchIcon,
	SpinnerIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, nextTick, ref, watch } from 'vue'

import Button from '#ui/components/base/buttons/Button.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

import {
	type BackgroundId,
	backgroundOptions,
	DEFAULT_BACKGROUND_ID,
	DEFAULT_SYMBOL_ID,
	RANDOM_CONFIG_BLACKLIST,
	SYMBOL_CATEGORIES,
	type SymbolCategory,
	type SymbolId,
	type SymbolOption,
	symbolOptions,
} from './editor-catalog'
import GeneratedIcon from './GeneratedIcon.vue'
import {
	type GradientAngle,
	renderStudioCanvas,
	renderStudioIconDataUrl,
	type StudioBackdropMode,
	type StudioFont,
	type StudioFrame,
	type StudioMode,
	type StudioPattern,
	type StudioRenderOptions,
} from './render-icon'
import type { IconBackground, IconConfig } from './types'

const props = defineProps<{
	config?: IconConfig | null
	loadRecents?: () => Promise<IconConfig[]>
	save: (config: IconConfig, symbolAsset: string) => Promise<string | void>
}>()

const emit = defineEmits<{
	saved: [result: string | void, config: IconConfig]
}>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const recentConfigs = ref<IconConfig[]>([])
const saving = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)
const previewCanvasRef = ref<HTMLCanvasElement | null>(null)
const previewDataUrl = ref<string>('')

// Studio state
const activeTab = ref<StudioMode>('icon')
const selectedBackground = ref<BackgroundId | 'custom'>(DEFAULT_BACKGROUND_ID)
const backdropMode = ref<StudioBackdropMode>('gradient')
const primaryColor = ref('#0f172a')
const secondaryColor = ref('#1e293b')
const gradientAngle = ref<GradientAngle>(135)
const selectedPattern = ref<StudioPattern>('none')
const selectedFrame = ref<StudioFrame>('none')
const iconScale = ref(74) // percent 55..95
const dropShadow = ref(true)

// Icon tab state
const selectedSymbol = ref<SymbolId>(DEFAULT_SYMBOL_ID)
const activeCategory = ref<'all' | SymbolCategory>('all')
const symbolSearch = ref('')

// Monogram tab state
const monogramText = ref('SOL')
const monogramSubtext = ref('1.21')
const monogramFont = ref<StudioFont>('bold')
const monogramColor = ref('#ffffff')

// Custom upload tab state
const uploadedImageUrl = ref<string | null>(null)
const uploadedFileName = ref<string>('')

const PATTERNS: { id: StudioPattern; label: string }[] = [
	{ id: 'none', label: 'Clean' },
	{ id: 'dots', label: 'Dots' },
	{ id: 'diagonal', label: 'Stripes' },
	{ id: 'grid', label: 'Grid' },
]

const FRAMES: { id: StudioFrame; label: string }[] = [
	{ id: 'none', label: 'None' },
	{ id: 'border', label: 'Border' },
	{ id: 'corners', label: 'HUD' },
	{ id: 'ring', label: 'Ring' },
]

const ANGLES: { value: GradientAngle; label: string }[] = [
	{ value: 180, label: '↓' },
	{ value: 135, label: '↘' },
	{ value: 90, label: '→' },
	{ value: 45, label: '↗' },
]

const FONTS: { id: StudioFont; label: string }[] = [
	{ id: 'bold', label: 'Sans' },
	{ id: 'mono', label: 'Mono' },
	{ id: 'serif', label: 'Classic' },
]

const MONOGRAM_COLORS = ['#ffffff', '#0f172a', '#38bdf8', '#fbbf24', '#34d399', '#f472b6']

const selectedSymbolOption = computed(
	() => symbolOptions.find((option) => option.id === selectedSymbol.value) ?? symbolOptions[0],
)

const filteredSymbols = computed(() => {
	const q = symbolSearch.value.trim().toLowerCase()
	return symbolOptions.filter((option) => {
		if (activeCategory.value !== 'all' && option.category !== activeCategory.value) return false
		if (!q) return true
		return (
			option.label.toLowerCase().includes(q) ||
			option.id.toLowerCase().includes(q) ||
			option.keywords.some((k) => k.includes(q))
		)
	})
})

const renderOptions = computed<StudioRenderOptions>(() => ({
	mode: activeTab.value,
	backdropMode: backdropMode.value,
	primaryColor: primaryColor.value,
	secondaryColor: secondaryColor.value,
	angle: gradientAngle.value,
	pattern: selectedPattern.value,
	frame: selectedFrame.value,
	scale: iconScale.value / 100,
	shadow: dropShadow.value,
	symbolAsset: selectedSymbolOption.value.asset,
	uploadAsset: uploadedImageUrl.value,
	monogramText: monogramText.value,
	monogramSubtext: monogramSubtext.value,
	monogramFont: monogramFont.value,
	monogramColor: monogramColor.value,
}))

const selectedConfig = computed<IconConfig>(() => {
	const symbolId =
		activeTab.value === 'monogram'
			? 'solara_monogram'
			: activeTab.value === 'upload'
				? 'solara_custom'
				: selectedSymbol.value
	return {
		background:
			backdropMode.value === 'solid'
				? { type: 'color', value: normalizeHex(primaryColor.value) }
				: {
						type: 'linear-top-down-gradient',
						top_color: normalizeHex(primaryColor.value),
						bottom_color: normalizeHex(secondaryColor.value),
					},
		symbol: symbolId,
	}
})

const visibleRecentConfigs = computed(() =>
	recentConfigs.value.filter(
		(config) => backgroundOption(config.background) && symbolOption(config.symbol),
	),
)

function normalizeHex(hex: string): string {
	const clean = hex.trim()
	if (/^#[0-9a-fA-F]{6}$/.test(clean)) return clean.toLowerCase()
	return '#0f172a'
}

function backgroundOption(background?: IconBackground) {
	if (background?.type !== 'linear-top-down-gradient') return undefined
	return backgroundOptions.find(
		(option) =>
			option.background.top_color.toLowerCase() === background.top_color.toLowerCase() &&
			option.background.bottom_color.toLowerCase() === background.bottom_color.toLowerCase(),
	)
}

function backgroundKey(background: IconBackground) {
	if (background.type === 'color') return `${background.type}-${background.value}`
	return `${background.type}-${background.top_color}-${background.bottom_color}`
}

function symbolOption(symbol: string) {
	return symbolOptions.find((option) => option.id === symbol)
}

function applyPresetBackground(option: (typeof backgroundOptions)[number]) {
	selectedBackground.value = option.id
	primaryColor.value = option.background.top_color
	secondaryColor.value = option.background.bottom_color
	if (backdropMode.value === 'solid') {
		backdropMode.value = 'gradient'
	}
}

function onCustomColorInput(which: 'primary' | 'secondary', event: Event) {
	const target = event.target as HTMLInputElement
	selectedBackground.value = 'custom'
	if (which === 'primary') primaryColor.value = target.value
	else secondaryColor.value = target.value
}

function selectRecent(config: IconConfig) {
	const background = backgroundOption(config.background)
	const symbol = symbolOption(config.symbol)
	if (!background || !symbol) return
	activeTab.value = 'icon'
	applyPresetBackground(background)
	selectedSymbol.value = symbol.id
}

function pickRandomLook() {
	const candidates = backgroundOptions.flatMap((background) =>
		symbolOptions
			.filter(
				(symbol: SymbolOption) =>
					!symbol.excludeFromRandomization &&
					background.id !== selectedBackground.value &&
					symbol.id !== selectedSymbol.value &&
					!RANDOM_CONFIG_BLACKLIST.some(
						(cfg) => cfg.background === background.id && cfg.symbol === symbol.id,
					),
			)
			.map((symbol) => ({ background, symbol })),
	)
	const picked = candidates[Math.floor(Math.random() * candidates.length)]
	if (!picked) return null

	const randomPatterns: StudioPattern[] = ['none', 'none', 'dots', 'diagonal', 'grid']
	const randomFrames: StudioFrame[] = ['none', 'none', 'border', 'corners', 'ring']
	const randomAngles: GradientAngle[] = [180, 135, 135, 45]

	return {
		background: picked.background,
		symbol: picked.symbol,
		pattern: randomPatterns[Math.floor(Math.random() * randomPatterns.length)],
		frame: randomFrames[Math.floor(Math.random() * randomFrames.length)],
		angle: randomAngles[Math.floor(Math.random() * randomAngles.length)],
	}
}

function surpriseMe() {
	const picked = pickRandomLook()
	if (!picked) return
	activeTab.value = 'icon'
	applyPresetBackground(picked.background)
	selectedSymbol.value = picked.symbol.id
	selectedPattern.value = picked.pattern
	selectedFrame.value = picked.frame
	gradientAngle.value = picked.angle
	iconScale.value = 74
	dropShadow.value = true
}

function handleFileSelect(event: Event) {
	const input = event.target as HTMLInputElement
	const file = input.files?.[0]
	if (!file) return
	const reader = new FileReader()
	reader.onload = () => {
		if (typeof reader.result === 'string') {
			uploadedImageUrl.value = reader.result
			uploadedFileName.value = file.name
			iconScale.value = 84
		}
	}
	reader.readAsDataURL(file)
	input.value = ''
}

let renderSeq = 0
async function refreshPreview() {
	const seq = ++renderSeq
	const canvas = previewCanvasRef.value
	if (!canvas) return
	try {
		await renderStudioCanvas(canvas, renderOptions.value)
		if (seq !== renderSeq) return
		previewDataUrl.value = canvas.toDataURL('image/png')
	} catch {
		// Ignore preview render error
	}
}

watch(
	renderOptions,
	() => {
		void refreshPreview()
	},
	{ deep: true },
)

async function refreshRecentConfigs() {
	try {
		recentConfigs.value = props.loadRecents ? await props.loadRecents() : []
	} catch (error) {
		handleError(error instanceof Error ? error : new Error(String(error)))
	}
}

function show() {
	activeTab.value = 'icon'
	const matchedBg = backgroundOption(props.config?.background)
	if (matchedBg) {
		applyPresetBackground(matchedBg)
	} else if (props.config?.background?.type === 'color') {
		selectedBackground.value = 'custom'
		backdropMode.value = 'solid'
		primaryColor.value = props.config.background.value
	} else if (props.config?.background?.type === 'linear-top-down-gradient') {
		selectedBackground.value = 'custom'
		backdropMode.value = 'gradient'
		primaryColor.value = props.config.background.top_color
		secondaryColor.value = props.config.background.bottom_color
	} else {
		const def = backgroundOptions.find((b) => b.id === DEFAULT_BACKGROUND_ID) ?? backgroundOptions[0]
		applyPresetBackground(def)
	}

	selectedSymbol.value = symbolOption(props.config?.symbol ?? '')?.id ?? DEFAULT_SYMBOL_ID
	modal.value?.show()
	void refreshRecentConfigs()
	void nextTick(() => refreshPreview())
}

function hide() {
	modal.value?.hide()
}

async function saveIcon() {
	if (saving.value) return
	if (activeTab.value === 'upload' && !uploadedImageUrl.value) {
		handleError(new Error('Please choose an image file first.'))
		return
	}

	saving.value = true
	try {
		const dataUrl = await renderStudioIconDataUrl(renderOptions.value, 512)
		const config = selectedConfig.value
		const result = await props.save(config, dataUrl)
		emit('saved', result, config)
		saving.value = false
		await nextTick()
		hide()
	} catch (error) {
		handleError(error instanceof Error ? error : new Error(String(error)))
	} finally {
		saving.value = false
	}
}

async function randomize() {
	const picked = pickRandomLook()
	if (picked) {
		activeTab.value = 'icon'
		applyPresetBackground(picked.background)
		selectedSymbol.value = picked.symbol.id
		selectedPattern.value = picked.pattern
		selectedFrame.value = picked.frame
		gradientAngle.value = picked.angle
		iconScale.value = 74
		dropShadow.value = true
	}
	const dataUrl = await renderStudioIconDataUrl(renderOptions.value, 512)
	return { config: selectedConfig.value, symbolAsset: dataUrl }
}

defineExpose({ show, hide, randomize })

const messages = defineMessages({
	title: {
		id: 'instance.icon-editor.title',
		defaultMessage: 'Solara Icon Studio',
	},
	saveIcon: {
		id: 'instance.icon-editor.save',
		defaultMessage: 'Apply icon',
	},
})
</script>

<template>
	<NewModal
		ref="modal"
		header="Solara Icon Studio"
		width="960px"
		max-width="calc(100vw - 2rem)"
		no-padding
		actions-divider
		:disable-close="saving"
	>
		<div class="flex h-[588px] max-h-[calc(100vh-160px)] min-h-0 flex-col md:flex-row">
			<!-- Left Live Preview & Quick Controls -->
			<aside
				class="flex w-full shrink-0 flex-col gap-4 overflow-y-auto border-0 border-b border-solid border-surface-5 bg-surface-2 p-5 md:w-[264px] md:border-b-0 md:border-r"
			>
				<div
					class="flex flex-col items-center gap-3.5 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4"
				>
					<div class="relative overflow-hidden rounded-2xl border border-solid border-surface-5 shadow-md">
						<canvas
							ref="previewCanvasRef"
							width="256"
							height="256"
							class="block size-[148px]"
						/>
					</div>

					<!-- Multi-size live strip -->
					<div class="flex items-center gap-2.5">
						<img
							v-if="previewDataUrl"
							:src="previewDataUrl"
							alt="Medium preview"
							class="size-10 rounded-xl border border-solid border-surface-5 object-cover"
						/>
						<img
							v-if="previewDataUrl"
							:src="previewDataUrl"
							alt="Small preview"
							class="size-7 rounded-lg border border-solid border-surface-5 object-cover"
						/>
						<img
							v-if="previewDataUrl"
							:src="previewDataUrl"
							alt="Compact preview"
							class="size-5 rounded-md border border-solid border-surface-5 object-cover"
						/>
					</div>
				</div>

				<Button class="w-full" size="sm" @click="surpriseMe">
					<RefreshCwIcon class="size-4" />
					<span>Shuffle look</span>
				</Button>

				<!-- Scale & Shadow Controls -->
				<div class="flex flex-col gap-3 rounded-xl border border-solid border-surface-5 bg-surface-3 p-3.5">
					<div class="flex flex-col gap-1.5">
						<div class="flex items-center justify-between text-xs font-semibold">
							<span class="text-secondary">Artwork size</span>
							<span class="tabular-nums text-contrast">{{ iconScale }}%</span>
						</div>
						<input
							v-model.number="iconScale"
							type="range"
							min="52"
							max="96"
							step="2"
							class="solara-range w-full cursor-pointer"
						/>
					</div>

					<label class="flex cursor-pointer items-center justify-between text-xs font-semibold text-secondary">
						<span>Depth shadow</span>
						<button
							type="button"
							role="switch"
							:aria-checked="dropShadow"
							class="relative h-5 w-9 cursor-pointer rounded-full border border-solid border-surface-5 p-0 transition-colors"
							:class="dropShadow ? 'bg-brand' : 'bg-surface-2'"
							@click="dropShadow = !dropShadow"
						>
							<span
								class="block size-3.5 rounded-full bg-white shadow transition-transform"
								:class="dropShadow ? 'translate-x-[18px]' : 'translate-x-[2px]'"
							/>
						</button>
					</label>
				</div>

				<!-- Recents -->
				<div v-if="visibleRecentConfigs.length" class="flex flex-col gap-2">
					<span class="text-xs font-semibold uppercase tracking-wide text-secondary">Recent</span>
					<div class="grid grid-cols-4 gap-2">
						<button
							v-for="(recentConfig, index) in visibleRecentConfigs.slice(0, 8)"
							:key="`${backgroundKey(recentConfig.background)}-${recentConfig.symbol}`"
							type="button"
							class="relative size-11 cursor-pointer overflow-hidden rounded-xl border border-solid border-surface-5 p-0 transition-transform hover:scale-105"
							:aria-label="`Recent icon ${index + 1}`"
							@click="selectRecent(recentConfig)"
						>
							<GeneratedIcon :config="recentConfig" class="size-full" />
						</button>
					</div>
				</div>
			</aside>

			<!-- Right Studio Workbench -->
			<div class="flex min-w-0 flex-1 flex-col overflow-hidden bg-surface-2">
				<!-- Mode Switcher Bar -->
				<div
					class="flex items-center justify-between gap-3 border-0 border-b border-solid border-surface-5 bg-surface-3 px-4 py-3"
				>
					<div
						class="inline-flex items-center rounded-xl border border-solid border-surface-5 bg-surface-2 p-1"
						role="tablist"
					>
						<button
							type="button"
							role="tab"
							:aria-selected="activeTab === 'icon'"
							class="cursor-pointer rounded-lg border-none px-3.5 py-1.5 text-xs font-semibold transition-colors"
							:class="
								activeTab === 'icon'
									? 'bg-surface-4 text-contrast shadow-sm'
									: 'bg-transparent text-secondary hover:text-contrast'
							"
							@click="activeTab = 'icon'"
						>
							3D Icons
						</button>
						<button
							type="button"
							role="tab"
							:aria-selected="activeTab === 'monogram'"
							class="cursor-pointer rounded-lg border-none px-3.5 py-1.5 text-xs font-semibold transition-colors"
							:class="
								activeTab === 'monogram'
									? 'bg-surface-4 text-contrast shadow-sm'
									: 'bg-transparent text-secondary hover:text-contrast'
							"
							@click="activeTab = 'monogram'"
						>
							Monogram Badge
						</button>
						<button
							type="button"
							role="tab"
							:aria-selected="activeTab === 'upload'"
							class="cursor-pointer rounded-lg border-none px-3.5 py-1.5 text-xs font-semibold transition-colors"
							:class="
								activeTab === 'upload'
									? 'bg-surface-4 text-contrast shadow-sm'
									: 'bg-transparent text-secondary hover:text-contrast'
							"
							@click="activeTab = 'upload'"
						>
							Framed Upload
						</button>
					</div>

					<span class="hidden text-xs text-secondary sm:inline">
						512 × 512 PNG
					</span>
				</div>

				<!-- Scrollable Controls Area -->
				<div class="flex-1 space-y-5 overflow-y-auto p-4">
					<!-- TAB 1: 3D ICONS -->
					<section v-if="activeTab === 'icon'" class="flex flex-col gap-3">
						<div class="flex flex-wrap items-center justify-between gap-2">
							<div class="flex flex-wrap items-center gap-1.5">
								<button
									v-for="cat in SYMBOL_CATEGORIES"
									:key="cat.id"
									type="button"
									class="cursor-pointer rounded-lg border border-solid px-2.5 py-1 text-xs font-semibold transition-colors"
									:class="
										activeCategory === cat.id
											? 'solara-chip-active text-contrast'
											: 'border-surface-5 bg-surface-3 text-secondary hover:text-contrast'
									"
									@click="activeCategory = cat.id"
								>
									{{ cat.label }}
								</button>
							</div>

							<div
								class="flex items-center gap-2 rounded-lg border border-solid border-surface-5 bg-surface-3 px-2.5 py-1.5 focus-within:border-brand"
							>
								<SearchIcon class="size-3.5 text-secondary" />
								<input
									v-model="symbolSearch"
									type="text"
									placeholder="Filter icons..."
									class="w-32 border-none bg-transparent text-xs text-contrast outline-none placeholder:text-secondary"
								/>
							</div>
						</div>

						<div class="grid grid-cols-5 gap-2 sm:grid-cols-6">
							<button
								v-for="option in filteredSymbols"
								:key="option.id"
								v-tooltip="{ content: option.label, delay: { show: 350, hide: 0 } }"
								type="button"
								class="group relative flex aspect-square cursor-pointer items-center justify-center rounded-xl border border-solid bg-surface-3 p-2.5 transition-all"
								:class="
									selectedSymbol === option.id
										? 'border-brand bg-surface-4 shadow-sm'
										: 'border-surface-5 hover:border-surface-4 hover:bg-surface-4'
								"
								:aria-label="option.label"
								:aria-pressed="selectedSymbol === option.id"
								@click="selectedSymbol = option.id"
							>
								<img
									:src="option.asset"
									:alt="option.label"
									class="size-full object-contain transition-transform duration-150 group-hover:scale-105"
								/>
								<span
									v-if="selectedSymbol === option.id"
									class="absolute right-1 top-1 flex size-4 items-center justify-center rounded-full bg-brand text-brand-inverted"
								>
									<CheckIcon class="size-2.5" />
								</span>
							</button>
						</div>
					</section>

					<!-- TAB 2: MONOGRAM BADGE -->
					<section
						v-else-if="activeTab === 'monogram'"
						class="grid grid-cols-1 gap-4 rounded-xl border border-solid border-surface-5 bg-surface-3 p-4 sm:grid-cols-2"
					>
						<div class="flex flex-col gap-1.5">
							<label class="text-xs font-semibold text-secondary">Main badge text (1–5 chars)</label>
							<input
								v-model="monogramText"
								type="text"
								maxlength="5"
								placeholder="SMP"
								class="rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-sm font-bold uppercase tracking-wider text-contrast outline-none focus:border-brand"
							/>
						</div>

						<div class="flex flex-col gap-1.5">
							<label class="text-xs font-semibold text-secondary">Subtitle tag (optional)</label>
							<input
								v-model="monogramSubtext"
								type="text"
								maxlength="12"
								placeholder="1.21 / FABRIC"
								class="rounded-xl border border-solid border-surface-5 bg-surface-2 px-3 py-2 text-sm font-semibold uppercase tracking-wide text-contrast outline-none focus:border-brand"
							/>
						</div>

						<div class="flex flex-col gap-1.5">
							<span class="text-xs font-semibold text-secondary">Typography</span>
							<div class="flex items-center gap-1.5">
								<button
									v-for="f in FONTS"
									:key="f.id"
									type="button"
									class="flex-1 cursor-pointer rounded-lg border border-solid py-1.5 text-xs font-semibold transition-colors"
									:class="
										monogramFont === f.id
											? 'solara-chip-active text-contrast'
											: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
									"
									@click="monogramFont = f.id"
								>
									{{ f.label }}
								</button>
							</div>
						</div>

						<div class="flex flex-col gap-1.5">
							<span class="text-xs font-semibold text-secondary">Text color</span>
							<div class="flex items-center gap-2">
								<button
									v-for="c in MONOGRAM_COLORS"
									:key="c"
									type="button"
									class="size-7 cursor-pointer rounded-full border-2 border-solid transition-transform hover:scale-105"
									:class="monogramColor === c ? 'border-brand scale-105' : 'border-surface-5'"
									:style="{ backgroundColor: c }"
									:aria-label="`Text color ${c}`"
									@click="monogramColor = c"
								/>
							</div>
						</div>
					</section>

					<!-- TAB 3: FRAMED UPLOAD -->
					<section
						v-else
						class="flex flex-col items-center justify-center gap-3 rounded-xl border border-dashed border-surface-5 bg-surface-3 px-6 py-8 text-center"
					>
						<input
							ref="fileInputRef"
							type="file"
							accept="image/png,image/jpeg,image/webp,image/gif"
							class="hidden"
							@change="handleFileSelect"
						/>
						<div class="flex size-12 items-center justify-center rounded-xl border border-solid border-surface-5 bg-surface-2">
							<UploadIcon class="size-5 text-secondary" />
						</div>
						<div class="flex flex-col gap-1">
							<span class="text-sm font-semibold text-contrast">
								{{ uploadedFileName || 'Upload custom artwork' }}
							</span>
							<span class="text-xs text-secondary">
								Combine your own PNG, JPG, or WebP with Solara backdrops, scale, and HUD frames.
							</span>
						</div>
						<div class="flex items-center gap-2">
							<Button size="sm" @click="fileInputRef?.click()">
								<UploadIcon class="size-4" />
								<span>{{ uploadedImageUrl ? 'Replace image' : 'Choose image' }}</span>
							</Button>
							<Button
								v-if="uploadedImageUrl"
								size="sm"
								type="outlined"
								@click="
									uploadedImageUrl = null;
									uploadedFileName = ''
								"
							>
								Clear
							</Button>
						</div>
					</section>

					<!-- SHARED BACKDROP & FINISH STUDIO -->
					<section class="flex flex-col gap-3.5 rounded-xl border border-solid border-surface-5 bg-surface-3 p-4">
						<div class="flex flex-wrap items-center justify-between gap-2">
							<span class="text-xs font-bold uppercase tracking-wider text-secondary">
								Backdrop & Finish
							</span>
							<div class="flex items-center gap-1.5">
								<button
									type="button"
									class="cursor-pointer rounded-lg border border-solid px-2.5 py-1 text-xs font-semibold transition-colors"
									:class="
										backdropMode === 'gradient'
											? 'solara-chip-active text-contrast'
											: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
									"
									@click="backdropMode = 'gradient'"
								>
									Gradient
								</button>
								<button
									type="button"
									class="cursor-pointer rounded-lg border border-solid px-2.5 py-1 text-xs font-semibold transition-colors"
									:class="
										backdropMode === 'solid'
											? 'solara-chip-active text-contrast'
											: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
									"
									@click="backdropMode = 'solid'"
								>
									Solid
								</button>
								<template v-if="backdropMode === 'gradient'">
									<span class="mx-1 h-4 w-px bg-surface-5" />
									<button
										v-for="ang in ANGLES"
										:key="ang.value"
										type="button"
										class="flex size-6 cursor-pointer items-center justify-center rounded-md border border-solid p-0 text-xs font-bold transition-colors"
										:class="
											gradientAngle === ang.value
												? 'solara-chip-active text-contrast'
												: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
										"
										:title="`Gradient angle ${ang.value}°`"
										@click="gradientAngle = ang.value"
									>
										{{ ang.label }}
									</button>
								</template>
							</div>
						</div>

						<!-- Palette Swatches + Custom Color Pickers -->
						<div class="flex flex-wrap items-center gap-2">
							<button
								v-for="option in backgroundOptions"
								:key="option.id"
								v-tooltip="{ content: formatMessage(option.name), delay: { show: 350, hide: 0 } }"
								type="button"
								class="relative size-8 cursor-pointer rounded-lg border-2 border-solid p-0 transition-transform hover:scale-105"
								:class="selectedBackground === option.id ? 'border-white scale-105 shadow' : 'border-transparent'"
								:style="{
									backgroundImage: `linear-gradient(135deg, ${option.background.top_color}, ${option.background.bottom_color})`,
								}"
								:aria-label="formatMessage(option.name)"
								@click="applyPresetBackground(option)"
							>
								<span
									v-if="selectedBackground === option.id"
									class="flex size-full items-center justify-center text-white drop-shadow"
								>
									<CheckIcon class="size-3.5" />
								</span>
							</button>

							<span class="mx-1 h-6 w-px bg-surface-5" />

							<label
								class="flex cursor-pointer items-center gap-1.5 rounded-lg border border-solid border-surface-5 bg-surface-2 px-2 py-1 text-xs font-medium text-secondary hover:text-contrast"
								title="Custom primary color"
							>
								<input
									type="color"
									:value="primaryColor"
									class="solara-color-input cursor-pointer"
									@input="onCustomColorInput('primary', $event)"
								/>
								<span>Color 1</span>
							</label>

							<label
								v-if="backdropMode === 'gradient'"
								class="flex cursor-pointer items-center gap-1.5 rounded-lg border border-solid border-surface-5 bg-surface-2 px-2 py-1 text-xs font-medium text-secondary hover:text-contrast"
								title="Custom secondary color"
							>
								<input
									type="color"
									:value="secondaryColor"
									class="solara-color-input cursor-pointer"
									@input="onCustomColorInput('secondary', $event)"
								/>
								<span>Color 2</span>
							</label>
						</div>

						<!-- Pattern & Frame Row -->
						<div class="grid grid-cols-1 gap-3 pt-1 sm:grid-cols-2">
							<div class="flex flex-col gap-1.5">
								<span class="text-xs font-semibold text-secondary">Texture pattern</span>
								<div class="flex flex-wrap items-center gap-1.5">
									<button
										v-for="pat in PATTERNS"
										:key="pat.id"
										type="button"
										class="cursor-pointer rounded-lg border border-solid px-2.5 py-1 text-xs font-semibold transition-colors"
										:class="
											selectedPattern === pat.id
												? 'solara-chip-active text-contrast'
												: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
										"
										@click="selectedPattern = pat.id"
									>
										{{ pat.label }}
									</button>
								</div>
							</div>

							<div class="flex flex-col gap-1.5">
								<span class="text-xs font-semibold text-secondary">Frame overlay</span>
								<div class="flex flex-wrap items-center gap-1.5">
									<button
										v-for="fr in FRAMES"
										:key="fr.id"
										type="button"
										class="cursor-pointer rounded-lg border border-solid px-2.5 py-1 text-xs font-semibold transition-colors"
										:class="
											selectedFrame === fr.id
												? 'solara-chip-active text-contrast'
												: 'border-surface-5 bg-surface-2 text-secondary hover:text-contrast'
										"
										@click="selectedFrame = fr.id"
									>
										{{ fr.label }}
									</button>
								</div>
							</div>
						</div>
					</section>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex items-center justify-between gap-4 px-2">
				<span class="text-xs text-secondary">
					3D icon assets via Microsoft Fluent Emoji (MIT) · Custom badges rendered locally
				</span>
				<div class="flex shrink-0 items-center gap-2">
					<Button :disabled="saving" type="outlined" @click="hide">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button type="colored" color="brand" :disabled="saving" @click="saveIcon">
						<SpinnerIcon v-if="saving" class="animate-spin" />
						<SaveIcon v-else />
						{{ formatMessage(messages.saveIcon) }}
					</Button>
				</div>
			</div>
		</template>
	</NewModal>
</template>

<style scoped>
.solara-chip-active {
	border-color: var(--color-brand);
	background-color: color-mix(in srgb, var(--color-brand) 16%, var(--surface-3));
}

.solara-range {
	accent-color: var(--color-brand);
	background: transparent;
	height: 20px;
	padding: 0;
	margin: 0;
}

.solara-color-input {
	width: 18px;
	height: 18px;
	padding: 0;
	border: none;
	border-radius: 4px;
	background: transparent;
	overflow: hidden;
}

.solara-color-input::-webkit-color-swatch-wrapper {
	padding: 0;
}

.solara-color-input::-webkit-color-swatch {
	border: none;
	border-radius: 4px;
}
</style>
