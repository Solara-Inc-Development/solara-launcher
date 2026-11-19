<template>
	<NewModal ref="modal" max-width="50rem" :on-hide="handleModalHide">
		<template #title>
			<div class="flex items-center gap-2.5">
				<span
					class="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-extrabold uppercase tracking-[0.16em] bg-brand/15 text-brand"
				>
					<SparklesIcon class="size-3" />
					<span>Solara Studio</span>
				</span>
				<span class="text-lg font-extrabold text-contrast tracking-tight">
					{{ mode === 'edit' ? 'Customize Skin' : 'Import New Skin' }}
				</span>
			</div>
		</template>

		<div class="solara-skin-customizer flex flex-col-reverse md:flex-row gap-5">
			<!-- LEFT COLUMN: SOLARA STUDIO CONTROLS -->
			<div class="flex-1 flex flex-col gap-4 min-w-0">
				<!-- 1. Skin Texture Dropzone / Replace Panel -->
				<section
					v-if="mode === 'edit' && canEditTextureAndModel"
					class="solara-customizer-panel rounded-2xl bg-surface-3/55 p-4 flex flex-col gap-3"
				>
					<div class="flex items-center justify-between gap-2">
						<span class="text-xs font-extrabold uppercase tracking-wider text-secondary">
							Skin Texture
						</span>
						<span
							v-if="uploadedTextureUrl"
							class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/15 text-emerald-400"
						>
							<CheckIcon class="size-3" />
							New .PNG ready
						</span>
					</div>

					<div
						class="solara-texture-dropzone rounded-xl border border-dashed p-3.5 flex items-center justify-between gap-3 cursor-pointer transition-all"
						:class="
							isDraggingTexture
								? 'border-brand bg-brand/10'
								: 'border-surface-5/60 bg-surface-2/60 hover:border-brand/50 hover:bg-surface-2'
						"
						@click="openTextureFileBrowser"
						@dragenter.prevent="isDraggingTexture = true"
						@dragover.prevent="isDraggingTexture = true"
						@dragleave.prevent="isDraggingTexture = false"
						@drop.prevent="onTextureDrop"
					>
						<div class="flex items-center gap-3 min-w-0">
							<div
								class="size-9 rounded-xl bg-brand/15 text-brand flex items-center justify-center shrink-0"
							>
								<UploadIcon class="size-4" />
							</div>
							<div class="flex flex-col min-w-0">
								<span class="text-xs font-bold text-contrast truncate">
									{{ uploadedTextureUrl ? 'Replace selected texture' : 'Upload or drop a .PNG skin' }}
								</span>
								<span class="text-[11px] text-secondary truncate">
									64x64 or 64x32 Minecraft skin file
								</span>
							</div>
						</div>
						<Button type="outlined" size="sm" @click.stop="openTextureFileBrowser">
							Browse
						</Button>
					</div>

					<input
						ref="textureFileInput"
						type="file"
						accept="image/png"
						class="hidden"
						@change="onTextureFileInputChange"
					/>
				</section>

				<!-- 2. Body Model Selector (Classic vs Slim Cards) -->
				<section
					v-if="canEditTextureAndModel"
					class="solara-customizer-panel rounded-2xl bg-surface-3/55 p-4 flex flex-col gap-3"
				>
					<div class="flex items-center justify-between gap-2">
						<span class="text-xs font-extrabold uppercase tracking-wider text-secondary">
							Body Model
						</span>
						<span class="text-[11px] font-semibold text-secondary">
							{{ variant === 'SLIM' ? '3px arms' : '4px arms' }}
						</span>
					</div>

					<div class="grid grid-cols-2 gap-2.5">
						<button
							type="button"
							class="solara-model-card rounded-xl p-3 text-left flex flex-col gap-1 border-0 cursor-pointer transition-all"
							:class="
								variant === 'CLASSIC'
									? 'solara-selected-ring bg-brand/12 text-contrast'
									: 'bg-surface-2/80 text-secondary hover:bg-surface-2 hover:text-contrast'
							"
							@click="variant = 'CLASSIC'"
						>
							<div class="flex items-center justify-between gap-2">
								<span class="text-sm font-extrabold text-contrast">Classic</span>
								<span
									v-if="variant === 'CLASSIC'"
									class="size-4 rounded-full bg-brand text-[var(--color-accent-contrast)] flex items-center justify-center"
								>
									<CheckIcon class="size-2.5" />
								</span>
							</div>
							<span class="text-[11px] text-secondary">Wide 4px Steve arms</span>
						</button>

						<button
							type="button"
							class="solara-model-card rounded-xl p-3 text-left flex flex-col gap-1 border-0 cursor-pointer transition-all"
							:class="
								variant === 'SLIM'
									? 'solara-selected-ring bg-brand/12 text-contrast'
									: 'bg-surface-2/80 text-secondary hover:bg-surface-2 hover:text-contrast'
							"
							@click="variant = 'SLIM'"
						>
							<div class="flex items-center justify-between gap-2">
								<span class="text-sm font-extrabold text-contrast">Slim</span>
								<span
									v-if="variant === 'SLIM'"
									class="size-4 rounded-full bg-brand text-[var(--color-accent-contrast)] flex items-center justify-center"
								>
									<CheckIcon class="size-2.5" />
								</span>
							</div>
							<span class="text-[11px] text-secondary">Slender 3px Alex arms</span>
						</button>
					</div>
				</section>

				<!-- 3. Cape Selector Panel -->
				<section class="solara-customizer-panel rounded-2xl bg-surface-3/55 p-4 flex flex-col gap-3 flex-1">
					<div class="flex items-center justify-between gap-2">
						<span class="text-xs font-extrabold uppercase tracking-wider text-secondary">
							Cape
						</span>
						<span class="text-[11px] font-bold text-brand truncate max-w-[12rem]">
							{{ selectedCape ? selectedCape.name || 'Cape' : 'No Cape' }}
						</span>
					</div>

					<div class="relative w-full">
						<Transition
							enter-active-class="transition-all duration-200 ease-out"
							enter-from-class="opacity-0 max-h-0"
							enter-to-class="opacity-100 max-h-6"
							leave-active-class="transition-all duration-200 ease-in"
							leave-from-class="opacity-100 max-h-6"
							leave-to-class="opacity-0 max-h-0"
						>
							<div
								v-if="showCapeTopFade"
								class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-6 bg-gradient-to-b from-surface-3 to-transparent"
							/>
						</Transition>

						<div
							ref="capeListRef"
							class="solara-cape-grid flex flex-wrap gap-2.5 overflow-y-auto pr-1"
							:style="{ maxHeight: capeListMaxHeight }"
							@scroll="checkCapeScrollState"
						>
							<button
								type="button"
								class="solara-no-cape-tile w-[4.75rem] h-[6.1rem] rounded-xl border-0 flex flex-col items-center justify-center gap-1.5 cursor-pointer transition-all shrink-0"
								:class="
									!selectedCape
										? 'solara-selected-ring bg-brand/12 text-brand'
										: 'bg-surface-2/85 text-secondary hover:bg-surface-2 hover:text-contrast'
								"
								@click="selectCape(undefined)"
							>
								<XIcon class="size-4" />
								<span class="text-[11px] font-bold">No Cape</span>
							</button>

							<CapeButton
								v-for="cape in sortedCapes"
								:id="cape.id"
								:key="cape.id"
								:texture="cape.texture"
								:name="cape.name || formatMessage(messages.capeFallbackName)"
								:selected="selectedCape?.id === cape.id"
								@select="selectCape(cape)"
							/>
						</div>

						<p
							v-if="sortedCapes.length === 0"
							class="m-0 mt-2.5 text-[11px] text-secondary leading-relaxed"
						>
							No unlocked Minecraft capes found on this account yet.
						</p>

						<Transition
							enter-active-class="transition-all duration-200 ease-out"
							enter-from-class="opacity-0 max-h-0"
							enter-to-class="opacity-100 max-h-6"
							leave-active-class="transition-all duration-200 ease-in"
							leave-from-class="opacity-100 max-h-6"
							leave-to-class="opacity-0 max-h-0"
						>
							<div
								v-if="showCapeBottomFade"
								class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-6 bg-gradient-to-t from-surface-3 to-transparent"
							/>
						</Transition>
					</div>
				</section>
			</div>

			<!-- RIGHT COLUMN: ELEVATED 3D LIVE PREVIEW STAGE -->
			<div
				class="solara-customizer-stage relative w-full md:w-[17rem] md:min-w-[17rem] rounded-3xl bg-surface-3/60 p-4 flex flex-col items-center justify-between gap-3 overflow-hidden shrink-0"
			>
				<div class="solara-stage-glow pointer-events-none absolute inset-0" />

				<div class="relative z-10 w-full flex items-center justify-between text-[11px] font-bold text-secondary">
					<span class="inline-flex items-center gap-1.5 text-contrast">
						<span class="size-1.5 rounded-full bg-brand" />
						Live 3D Preview
					</span>
					<span class="px-2 py-0.5 rounded-full bg-surface-2/80 text-secondary">
						{{ variant === 'SLIM' ? 'Slim' : 'Classic' }}
					</span>
				</div>

				<div class="relative z-10 h-[19.5rem] w-full flex items-center justify-center">
					<SkinPreviewRenderer
						:variant="variant"
						:texture-src="previewSkin || ''"
						:ears-texture-src="uploadedTextureUrl?.original ?? currentSkin?.texture"
						:cape-src="selectedCapeTexture"
						framing="modal"
						:initial-rotation="-Math.PI / 10"
						:action-trigger="previewPoseTrigger"
						class="h-full w-full"
					/>
				</div>

				<!-- Quick Pose Preview Pills -->
				<div class="relative z-10 flex flex-wrap items-center justify-center gap-1.5 w-full">
					<button
						v-for="pose in previewPoses"
						:key="pose.id"
						type="button"
						class="px-2.5 py-1 rounded-full text-[11px] font-bold border-0 cursor-pointer transition-all"
						:class="
							previewPoseTrigger?.name === pose.id
								? 'bg-brand text-[var(--color-accent-contrast)]'
								: 'bg-surface-2/85 text-secondary hover:text-contrast hover:bg-surface-4'
						"
						@click="triggerPreviewPose(pose.id)"
					>
						{{ pose.label }}
					</button>
				</div>
			</div>
		</div>

		<template #actions>
			<div class="flex items-center justify-between gap-3 w-full">
				<span class="text-xs text-secondary hidden sm:inline">
					{{ hasEdits ? 'Unsaved changes ready' : 'Customize model, texture, or cape' }}
				</span>
				<div class="flex gap-2 justify-end ml-auto">
					<Button type="outlined" :disabled="isSaving" @click="hide">
						<XIcon />{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						v-tooltip="saveTooltip"
						type="colored"
						color="brand"
						:disabled="disableSave || isSaving"
						@click="save"
					>
						<SpinnerIcon v-if="isSaving" class="animate-spin" />
						<CheckIcon v-else-if="mode === 'new'" />
						<SaveIcon v-else />
						{{ mode === 'new' ? 'Add to Wardrobe' : 'Save Changes' }}
					</Button>
				</div>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import {
	CheckIcon,
	SaveIcon,
	SparklesIcon,
	SpinnerIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	CapeButton,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	NewModal,
	SkinPreviewRenderer,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { arrayBufferToBase64 } from '@modrinth/utils'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import defaultSteveSkin from '@/assets/skins/steve.png'
import {
	type Cape,
	determineModelType,
	equip_skin,
	get_normalized_skin_texture,
	normalize_skin_texture,
	save_custom_skin,
	type Skin,
	type SkinModel,
	type SkinTextureUrl,
} from '@/helpers/skins.ts'

const CAPE_LIST_MAX_HEIGHT = 240
const messages = defineMessages({
	capeFallbackName: {
		id: 'app.skins.modal.cape-fallback-name',
		defaultMessage: 'Cape',
	},
	savingTooltip: {
		id: 'app.skins.modal.saving-tooltip',
		defaultMessage: 'Saving...',
	},
	uploadSkinFirstTooltip: {
		id: 'app.skins.modal.upload-skin-first-tooltip',
		defaultMessage: 'Upload a skin first!',
	},
	makeEditFirstTooltip: {
		id: 'app.skins.modal.make-edit-first-tooltip',
		defaultMessage: 'Make an edit to the skin first!',
	},
	demoSaveTooltip: {
		id: 'app.skins.modal.demo-save-tooltip',
		defaultMessage: 'Sign in to save skins.',
	},
})

const previewPoses = [
	{ id: 'solara_wave', label: 'Wave' },
	{ id: 'solara_sit_stand', label: 'Sit' },
	{ id: 'solara_cheer_jump', label: 'Jump' },
	{ id: 'solara_salute', label: 'Salute' },
] as const

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const modal = useTemplateRef('modal')
const textureFileInput = useTemplateRef<HTMLInputElement>('textureFileInput')
const capeListRef = ref<HTMLElement | null>(null)
const capeListMaxHeight = ref(`${CAPE_LIST_MAX_HEIGHT}px`)
const mode = ref<'new' | 'edit'>('new')
const currentSkin = ref<Skin | null>(null)
const isSaving = ref(false)
const isDraggingTexture = ref(false)

const previewPoseTrigger = ref<{ name: string; seq: number } | undefined>(undefined)
let previewPoseSeq = 0
function triggerPreviewPose(id: string) {
	previewPoseTrigger.value = { name: id, seq: ++previewPoseSeq }
}

const uploadedTextureUrl = ref<SkinTextureUrl | null>(null)
const previewSkin = ref<string>('')

const variant = ref<SkinModel>('CLASSIC')
const selectedCape = ref<Cape | undefined>(undefined)
const props = defineProps<{ capes?: Cape[]; demo?: boolean }>()

const selectedCapeTexture = computed(() => selectedCape.value?.texture)
const canEditTextureAndModel = computed(() => currentSkin.value?.source !== 'default')
const {
	showTopFade: showCapeTopFade,
	showBottomFade: showCapeBottomFade,
	checkScrollState: checkCapeScrollState,
	forceCheck: forceCapeScrollCheck,
} = useScrollIndicator(capeListRef)

let capeListLayoutFrame: number | null = null
function updateCapeListLayout() {
	const capeList = capeListRef.value
	if (!capeList) {
		capeListMaxHeight.value = `${CAPE_LIST_MAX_HEIGHT}px`
		forceCapeScrollCheck()
		return
	}
	capeListMaxHeight.value = `${CAPE_LIST_MAX_HEIGHT}px`
	nextTick(() => forceCapeScrollCheck())
}

function refreshCapeListLayout() {
	if (capeListLayoutFrame !== null) {
		cancelAnimationFrame(capeListLayoutFrame)
	}

	capeListLayoutFrame = requestAnimationFrame(() => {
		capeListLayoutFrame = null
		updateCapeListLayout()
	})
}

const sortedCapes = computed(() => {
	return [...(props.capes || [])].sort((a, b) => {
		const nameA = (a.name || '').toLowerCase()
		const nameB = (b.name || '').toLowerCase()
		return nameA.localeCompare(nameB)
	})
})

async function loadPreviewSkin() {
	if (uploadedTextureUrl.value) {
		previewSkin.value = uploadedTextureUrl.value.normalized
	} else if (currentSkin.value) {
		try {
			previewSkin.value = await get_normalized_skin_texture(currentSkin.value)
		} catch (error) {
			console.error('Failed to load skin texture:', error)
			previewSkin.value = defaultSteveSkin
		}
	} else {
		previewSkin.value = defaultSteveSkin
	}
}

const hasEdits = computed(() => {
	if (mode.value !== 'edit') return true
	if (uploadedTextureUrl.value) return true
	if (!currentSkin.value) return false
	if (variant.value !== currentSkin.value.variant) return true
	if ((selectedCape.value?.id || null) !== (currentSkin.value.cape_id || null)) return true
	return false
})

const disableSave = computed(
	() =>
		props.demo ||
		(mode.value === 'new' && !uploadedTextureUrl.value) ||
		(mode.value === 'edit' && !hasEdits.value),
)

const saveTooltip = computed(() => {
	if (isSaving.value) return formatMessage(messages.savingTooltip)
	if (props.demo) return formatMessage(messages.demoSaveTooltip)
	if (mode.value === 'new' && !uploadedTextureUrl.value) {
		return formatMessage(messages.uploadSkinFirstTooltip)
	}
	if (mode.value === 'edit' && !hasEdits.value) {
		return formatMessage(messages.makeEditFirstTooltip)
	}
	return undefined
})

function resetState() {
	mode.value = 'new'
	currentSkin.value = null
	uploadedTextureUrl.value = null
	previewSkin.value = ''
	variant.value = 'CLASSIC'
	selectedCape.value = undefined
	isSaving.value = false
	isDraggingTexture.value = false
	previewPoseTrigger.value = undefined
}

function handleModalHide() {
	setTimeout(() => resetState(), 250)
}

async function show(e: MouseEvent, skin?: Skin) {
	mode.value = skin ? 'edit' : 'new'
	currentSkin.value = skin ?? null
	if (skin) {
		variant.value = skin.variant
		selectedCape.value = props.capes?.find((c) => c.id === skin.cape_id)
	} else {
		variant.value = 'CLASSIC'
		selectedCape.value = undefined
	}

	await loadPreviewSkin()

	modal.value?.show(e)
	nextTick(() => refreshCapeListLayout())
}

async function showNew(e: MouseEvent, skinTextureUrl: SkinTextureUrl) {
	mode.value = 'new'
	currentSkin.value = null
	uploadedTextureUrl.value = skinTextureUrl
	variant.value = await determineModelType(skinTextureUrl.original)
	selectedCape.value = undefined

	await loadPreviewSkin()

	modal.value?.show(e)
	nextTick(() => refreshCapeListLayout())
}

async function setUploadedTexture(skinTextureUrl: SkinTextureUrl) {
	uploadedTextureUrl.value = skinTextureUrl
	await loadPreviewSkin()
	nextTick(() => refreshCapeListLayout())
}

function hide() {
	modal.value?.hide()
}

function selectCape(cape: Cape | undefined) {
	selectedCape.value = cape
}

function openTextureFileBrowser() {
	textureFileInput.value?.click()
}

async function processTextureFile(file: File) {
	try {
		const originalSkinTexUrl = `data:image/png;base64,${arrayBufferToBase64(
			await file.arrayBuffer(),
		)}`
		const skinTextureNormalized = await normalize_skin_texture(originalSkinTexUrl)
		await setUploadedTexture({
			original: originalSkinTexUrl,
			normalized: `data:image/png;base64,${arrayBufferToBase64(skinTextureNormalized)}`,
		})
	} catch (error) {
		handleError(error)
	}
}

async function onTextureFileInputChange(e: Event) {
	const files = (e.target as HTMLInputElement).files
	const file = files?.[0]

	if (!file) {
		return
	}

	try {
		await processTextureFile(file)
	} finally {
		if (textureFileInput.value) {
			textureFileInput.value.value = ''
		}
	}
}

async function onTextureDrop(e: DragEvent) {
	isDraggingTexture.value = false
	const file = e.dataTransfer?.files?.[0]
	if (!file || !file.name.toLowerCase().endsWith('.png')) return
	await processTextureFile(file)
}

async function save() {
	if (props.demo) return

	isSaving.value = true

	try {
		let textureUrl: string

		if (uploadedTextureUrl.value) {
			textureUrl = uploadedTextureUrl.value.original
		} else {
			textureUrl = currentSkin.value!.texture
		}

		const bytes: Uint8Array = new Uint8Array(await (await fetch(textureUrl)).arrayBuffer())

		if (mode.value === 'new') {
			const addedSkin = await save_custom_skin(
				{
					texture_key: '',
					variant: variant.value,
					cape_id: selectedCape.value?.id,
					texture: textureUrl,
					source: 'custom',
					is_equipped: false,
				},
				bytes,
				variant.value,
				selectedCape.value,
				true,
			)
			emit('saved', {
				applied: false,
				skin: addedSkin,
			})
		} else {
			const updatedSkin = await save_custom_skin(
				currentSkin.value!,
				bytes,
				variant.value,
				selectedCape.value,
				!!uploadedTextureUrl.value && textureUrl !== currentSkin.value?.texture,
			)

			if (currentSkin.value?.is_equipped) {
				await equip_skin(updatedSkin)
			}

			emit('saved', {
				applied: !!currentSkin.value?.is_equipped,
				skin: updatedSkin,
				previousSkin: currentSkin.value!,
			})
		}

		hide()
	} catch (err) {
		handleError(err)
	} finally {
		isSaving.value = false
	}
}

watch([uploadedTextureUrl, currentSkin], async () => {
	await loadPreviewSkin()
	refreshCapeListLayout()
})

watch(
	() => props.capes,
	() => {
		nextTick(() => refreshCapeListLayout())
	},
	{ immediate: true },
)

watch(
	capeListRef,
	(capeList, _, onCleanup) => {
		if (!capeList) return

		const resizeObserver = new ResizeObserver(() => refreshCapeListLayout())
		resizeObserver.observe(capeList)

		window.addEventListener('resize', refreshCapeListLayout, { passive: true })
		refreshCapeListLayout()

		onCleanup(() => {
			resizeObserver.disconnect()
			window.removeEventListener('resize', refreshCapeListLayout)

			if (capeListLayoutFrame !== null) {
				cancelAnimationFrame(capeListLayoutFrame)
				capeListLayoutFrame = null
			}
		})
	},
	{ flush: 'post' },
)

const emit = defineEmits<{
	(event: 'saved', options: { applied: boolean; skin?: Skin; previousSkin?: Skin }): void
	(event: 'deleted', skin: Skin): void
}>()

defineExpose({
	show,
	showNew,
	hide,
})
</script>

<style scoped lang="scss">
.solara-stage-glow {
	background: radial-gradient(
		65% 58% at 50% 48%,
		color-mix(in srgb, var(--color-brand) 16%, transparent) 0%,
		transparent 74%
	);
}

.solara-selected-ring {
	box-shadow: inset 0 0 0 2px var(--color-brand);
}

.solara-cape-grid :deep(button) {
	border: none;
}
</style>
