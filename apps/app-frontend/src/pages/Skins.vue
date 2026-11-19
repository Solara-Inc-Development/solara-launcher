<script setup lang="ts">
import {
	CheckIcon,
	EditIcon,
	EyeIcon,
	EyeOffIcon,
	InfoIcon,
	LayersIcon,
	LogInIcon,
	RotateCounterClockwiseIcon,
	ShirtIcon,
	SparklesIcon,
	SpinnerIcon,
	TrashIcon,
	UploadIcon,
	WindowsIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	CapeButton,
	commonMessages,
	ConfirmModal,
	defineMessages,
	injectAuth,
	injectModrinthClient,
	injectNotificationManager,
	SkinPreviewRenderer,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { arrayBufferToBase64 } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computedAsync } from '@vueuse/core'
import { computed, onMounted, onUnmounted, ref, useTemplateRef, watch } from 'vue'

import EarsModIcon from '@/assets/skins/ears-mod.png'
import defaultSteveSkin from '@/assets/skins/steve.png'
import BakedSkinButton from '@/components/ui/skin/BakedSkinButton.vue'
import EditSkinModal from '@/components/ui/skin/EditSkinModal.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { handleSevereError } from '@/composables/use-error.js'
import { injectMinecraftAccounts } from '@/features/minecraft-accounts/context'
import { trackEvent } from '@/helpers/analytics'
import { check_reachable, get_default_user, login as login_flow, users } from '@/helpers/auth'
import { listenNativeDragDrop, type NativeDragDropEvent } from '@/helpers/native-drag-drop'
import { cleanupUnusedPreviews } from '@/helpers/rendering/skin-previews'
import type { Cape, Skin, SkinTextureUrl } from '@/helpers/skins.ts'
import {
	equip_skin,
	filterDefaultSkins,
	filterSavedSkins,
	flush_pending_skin_change,
	flush_pending_skin_change_for_profile,
	get_available_capes,
	get_available_skins,
	get_dragged_skin_data,
	get_normalized_skin_texture,
	normalize_skin_texture,
	remove_custom_skin,
	save_custom_skin,
	set_custom_skin_order,
} from '@/helpers/skins.ts'
import { useRootBreadcrumb } from '@/shared/breadcrumbs'
import { appMessages } from '@/utils/app-messages'

useRootBreadcrumb({
	slot: 'root',
	id: 'skins',
	label: 'Skin Studio',
	to: '/skins',
	visual: { type: 'icon', component: ShirtIcon },
})

type UnlistenFn = () => void
type StudioTab = 'my-skins' | 'presets' | 'capes'

const PENDING_SKIN_REFRESH_DELAY_MS = 11_000
const DEFAULT_SKIN_SECTION_SORT_ORDER = ['Default skins']
const EARS_NOTICE_PLACEHOLDER = '__EARS_MOD_NAME__'
const messages = defineMessages({
	modrinthSection: {
		id: 'app.skins.section.modrinth',
		defaultMessage: 'Solara',
	},
	defaultSkinsSection: {
		id: 'app.skins.section.default-skins',
		defaultMessage: 'Default skins',
	},
	mineconEarth2017Section: {
		id: 'app.skins.section.minecon-earth-2017',
		defaultMessage: 'MINECON Earth 2017',
	},
	buildersAndBiomesSection: {
		id: 'app.skins.section.builders-and-biomes',
		defaultMessage: 'Builders & Biomes',
	},
	stridingHeroSection: {
		id: 'app.skins.section.striding-hero',
		defaultMessage: 'Striding Hero',
	},
	theGardenAwakensSection: {
		id: 'app.skins.section.the-garden-awakens',
		defaultMessage: 'The Garden Awakens',
	},
	chaseTheSkiesSection: {
		id: 'app.skins.section.chase-the-skies',
		defaultMessage: 'Chase the Skies',
	},
	theCopperAgeSection: {
		id: 'app.skins.section.the-copper-age',
		defaultMessage: 'The Copper Age',
	},
	mountsOfMayhemSection: {
		id: 'app.skins.section.mounts-of-mayhem',
		defaultMessage: 'Mounts of Mayhem',
	},
	tinyTakeoverSection: {
		id: 'app.skins.section.tiny-takeover',
		defaultMessage: 'Tiny Takeover',
	},
	chaosCubedSection: {
		id: 'app.skins.section.chaos-cubed',
		defaultMessage: 'Chaos Cubed',
	},
	dungeonsHeroSection: {
		id: 'app.skins.section.dungeons-hero',
		defaultMessage: 'Dungeons Hero',
	},
	dungeonsIIHeroSection: {
		id: 'app.skins.section.dungeons-ii-hero',
		defaultMessage: 'Dungeons II Hero',
	},
	wildernessBoundSection: {
		id: 'app.skins.section.wilderness-bound',
		defaultMessage: 'Wilderness Bound',
	},
	rateLimitTitle: {
		id: 'app.skins.rate-limit.title',
		defaultMessage: 'Slow down!',
	},
	rateLimitText: {
		id: 'app.skins.rate-limit.text',
		defaultMessage:
			"You're changing your skin too frequently. Mojang's servers have temporarily blocked further requests. Please wait a moment before trying again.",
	},
	droppedFileErrorTitle: {
		id: 'app.skins.dropped-file-error.title',
		defaultMessage: 'Error processing file',
	},
	droppedFileErrorText: {
		id: 'app.skins.dropped-file-error.text',
		defaultMessage: 'Failed to read the dropped file.',
	},
	reorderSkinErrorTitle: {
		id: 'app.skins.reorder-error.title',
		defaultMessage: 'Failed to reorder skins',
	},
	reorderSkinErrorText: {
		id: 'app.skins.reorder-error.text',
		defaultMessage: 'Your skin order could not be saved.',
	},
	deleteSkinTitle: {
		id: 'app.skins.delete-modal.title',
		defaultMessage: 'Are you sure you want to delete this skin?',
	},
	deleteSkinDescription: {
		id: 'app.skins.delete-modal.description',
		defaultMessage: 'This will permanently delete the selected skin. This action cannot be undone.',
	},
	previewingBadge: {
		id: 'app.skins.previewing-badge',
		defaultMessage: 'Previewing',
	},
	applyButton: {
		id: 'app.skins.apply-button',
		defaultMessage: 'Apply',
	},
	demoApplyTooltip: {
		id: 'app.skins.demo.apply-tooltip',
		defaultMessage: 'Sign in to apply skins.',
	},
	editSkinButton: {
		id: 'app.skins.preview.edit-button',
		defaultMessage: 'Edit skin',
	},
	earsFeatureNotice: {
		id: 'app.skins.ears-feature-notice',
		defaultMessage: 'This skin uses features from the {ears} mod',
	},
	toggleEarsFeaturesOff: {
		id: 'app.skins.toggle-ears-features-off',
		defaultMessage: 'Toggle off',
	},
	toggleEarsFeaturesOn: {
		id: 'app.skins.toggle-ears-features-on',
		defaultMessage: 'Toggle on',
	},
	demoTitle: {
		id: 'app.skins.demo.title',
		defaultMessage: 'Editing with a demo account',
	},
	demoDescription: {
		id: 'app.skins.demo.description',
		defaultMessage: 'Sign in to your Minecraft account to save and apply skins!',
	},
	signInButton: {
		id: 'app.skins.sign-in.button',
		defaultMessage: 'Sign in to Microsoft',
	},
})

const editSkinModal = useTemplateRef('editSkinModal')
const addSkinFileInput = useTemplateRef<HTMLInputElement>('addSkinFileInput')
const studioDropzoneEl = ref<HTMLElement | null>(null)
const studioTab = ref<StudioTab>('my-skins')
const selectedPresetSection = ref<string>('all')
const showNametagOverride = ref<boolean>(true)

const { formatMessage } = useVIntl()
const notifications = injectNotificationManager()
const { addNotification, handleError } = notifications
const auth = injectAuth()
const client = injectModrinthClient()

const appSettings = useAppSettings()
const skins = ref<Skin[]>([])
const capes = ref<Cape[]>([])
const offline = ref(!navigator.onLine)

const minecraftAccounts = injectMinecraftAccounts()
const currentUser = ref(undefined)
const currentUserId = ref<string | undefined>(undefined)

const username = computed(() => currentUser.value?.profile?.name ?? undefined)
const selectedSkin = ref<Skin | null>(null)
const isApplyingSkin = ref(false)
const earsFeaturesEnabled = ref(true)
const selectedSkinHasEarsFeatures = ref(false)

const originalSelectedSkin = ref<Skin | null>(null)
const earsFeatureNoticeParts = computed(() => {
	const notice = formatMessage(messages.earsFeatureNotice, {
		ears: EARS_NOTICE_PLACEHOLDER,
	})
	const placeholderIndex = notice.indexOf(EARS_NOTICE_PLACEHOLDER)

	if (placeholderIndex === -1) {
		return {
			before: notice,
			after: '',
			hasEarsLink: false,
		}
	}

	return {
		before: notice.slice(0, placeholderIndex),
		after: notice.slice(placeholderIndex + EARS_NOTICE_PLACEHOLDER.length),
		hasEarsLink: true,
	}
})

const savedSkins = computed(() => {
	try {
		return filterSavedSkins(skins.value)
	} catch (error) {
		handleError(error as Error)
		return []
	}
})
const authServerQuery = useQuery({
	queryKey: ['authServerReachability'],
	queryFn: async () => {
		await check_reachable()
		return true
	},
	refetchInterval: 5 * 60 * 1000,
	retry: false,
	refetchOnWindowFocus: false,
})
const defaultSkins = computed(() => filterDefaultSkins(skins.value))
const defaultSkinSections = computed(() => {
	const sections = new Map<string, Skin[]>()

	for (const skin of defaultSkins.value) {
		const section = skin.section ?? 'Default skins'
		const sectionSkins = sections.get(section)

		if (sectionSkins) {
			sectionSkins.push(skin)
		} else {
			sections.set(section, [skin])
		}
	}

	return Array.from(sections, ([section, skins]) => ({
		section,
		title: getDefaultSkinSectionTitle(section),
		infoTooltip: getDefaultSkinSectionInfoTooltip(section),
		skins,
	})).sort(
		(a, b) => getDefaultSkinSectionSortIndex(a.section) - getDefaultSkinSectionSortIndex(b.section),
	)
})

const currentCape = computed(() => {
	if (selectedSkin.value?.cape_id) {
		const overrideCape = capes.value.find((c) => c.id === selectedSkin.value?.cape_id)
		if (overrideCape) {
			return overrideCape
		}
	}
	return undefined
})

const skinTexture = computedAsync(async () => {
	const skin = selectedSkin.value
	if (skin?.texture) {
		try {
			return await get_normalized_skin_texture(skin)
		} catch (error) {
			if (skin.texture.startsWith('data:image/')) {
				return skin.texture
			}

			handleError(error as Error)
			return ''
		}
	} else {
		return ''
	}
})
const capeTexture = computed(() => currentCape.value?.texture)
const skinVariant = computed(() => selectedSkin.value?.variant ?? 'CLASSIC')
const skinNametag = computed(() =>
	!showNametagOverride.value || appSettings.hideNametagSkinsPage ? undefined : username.value,
)
const activeStudioTexture = computed(() => skinTexture.value || defaultSteveSkin)
const filteredPresetSections = computed(() => {
	if (selectedPresetSection.value === 'all') {
		return defaultSkinSections.value
	}
	return defaultSkinSections.value.filter(
		(section) => section.section === selectedPresetSection.value,
	)
})
const canQuickToggleVariant = computed(
	() => Boolean(selectedSkin.value) && selectedSkin.value?.source !== 'default',
)

function selectCapeForCurrentSkin(cape?: Cape) {
	if (!selectedSkin.value || isSkinManagementReadOnly.value) return
	selectedSkin.value = {
		...selectedSkin.value,
		cape_id: cape?.id,
	}
}

function setQuickArmVariant(variant: 'CLASSIC' | 'SLIM') {
	if (!selectedSkin.value || !canQuickToggleVariant.value || isSkinManagementReadOnly.value) return
	selectedSkin.value = {
		...selectedSkin.value,
		variant,
	}
}
const isSkinManagementReadOnly = computed(
	() =>
		!!currentUser.value &&
		(offline.value || (authServerQuery.isError.value && !authServerQuery.isLoading.value)),
)
const hasPendingSkinChange = computed(
	() => !skinsMatch(selectedSkin.value, originalSelectedSkin.value),
)

let userCheckInterval: number | null = null
let pendingSkinRefreshTimeout: number | null = null
let unlistenAddSkinDragDrop: UnlistenFn | null = null
let isUnmounted = false

const isDraggingSkinFile = ref(false)
const isAddSkinButtonDragActive = ref(false)

const deleteSkinModal = ref()
const skinToDelete = ref<Skin | null>(null)

function confirmDeleteSkin(skin: Skin) {
	if (isSkinManagementReadOnly.value) return

	skinToDelete.value = skin
	deleteSkinModal.value?.show()
}

async function deleteSkin() {
	if (isSkinManagementReadOnly.value) return

	const deletedSkin = skinToDelete.value
	if (!deletedSkin) return

	try {
		await remove_custom_skin(deletedSkin)
		removeLocalSkin(deletedSkin)
	} catch (error) {
		handleError(error as Error)
	} finally {
		skinToDelete.value = null
	}
}

async function loadCapes() {
	try {
		capes.value = (await get_available_capes()) ?? []
	} catch (error) {
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

async function loadSkins() {
	try {
		const loadedSkins = (await get_available_skins()) ?? []
		const loadedEquippedSkin = loadedSkins.find((s) => s.is_equipped)
		const locallyKnownEquippedSkin =
			originalSelectedSkin.value &&
			(loadedSkins.find((skin) => skinsMatch(skin, originalSelectedSkin.value)) ??
				(originalSelectedSkin.value.texture.startsWith('data:image/')
					? originalSelectedSkin.value
					: undefined))
		const shouldPreserveKnownEquippedSkin =
			isSkinManagementReadOnly.value &&
			locallyKnownEquippedSkin &&
			!skinsMatch(loadedEquippedSkin, locallyKnownEquippedSkin)

		skins.value =
			shouldPreserveKnownEquippedSkin && locallyKnownEquippedSkin
				? mergeEquippedSkin(loadedSkins, locallyKnownEquippedSkin)
				: loadedSkins
		void cleanupUnusedPreviews(skins.value).catch((error) =>
			console.warn('Could not clean skin previews', error),
		)
		selectedSkin.value = skins.value.find((s) => s.is_equipped) ?? null
		originalSelectedSkin.value = selectedSkin.value
	} catch (error) {
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

function mergeEquippedSkin(list: Skin[], equippedSkin: Skin) {
	let foundEquippedSkin = false
	const mergedSkins = list.map((skin) => {
		const isEquipped = skinsMatch(skin, equippedSkin)
		foundEquippedSkin ||= isEquipped

		return {
			...skin,
			is_equipped: isEquipped,
		}
	})

	if (!foundEquippedSkin) {
		mergedSkins.unshift({
			...equippedSkin,
			is_equipped: true,
		})
	}

	return mergedSkins
}

function skinsMatch(a?: Skin | null, b?: Skin | null) {
	return (
		a?.source === b?.source &&
		a?.texture_key === b?.texture_key &&
		a?.variant === b?.variant &&
		(a?.cape_id ?? null) === (b?.cape_id ?? null)
	)
}

function skinsMatchIgnoringSource(a?: Skin | null, b?: Skin | null) {
	return (
		a?.texture_key === b?.texture_key &&
		a?.variant === b?.variant &&
		(a?.cape_id ?? null) === (b?.cape_id ?? null)
	)
}

function isSkinSelected(skin: Skin) {
	return skinsMatch(selectedSkin.value, skin)
}

function isSkinActive(skin: Skin) {
	return hasPendingSkinChange.value && skinsMatch(originalSelectedSkin.value, skin)
}

function getErrorMessage(error: unknown) {
	if (error instanceof Error) return error.message
	if (
		typeof error === 'object' &&
		error !== null &&
		'message' in error &&
		typeof error.message === 'string'
	) {
		return error.message
	}
	return String(error)
}

function isMinecraftSkinRateLimitError(error: unknown) {
	const message = getErrorMessage(error)
	return message.includes('429 Too Many Requests') || message.includes('client error (429')
}

function getDefaultSkinSectionTitle(section?: string) {
	switch (section) {
		case 'Modrinth':
			return formatMessage(messages.modrinthSection)
		case 'MINECON Earth 2017':
			return formatMessage(messages.mineconEarth2017Section)
		case 'Builders & Biomes':
			return formatMessage(messages.buildersAndBiomesSection)
		case 'Striding Hero':
			return formatMessage(messages.stridingHeroSection)
		case 'The Garden Awakens':
			return formatMessage(messages.theGardenAwakensSection)
		case 'Chase the Skies':
			return formatMessage(messages.chaseTheSkiesSection)
		case 'The Copper Age':
			return formatMessage(messages.theCopperAgeSection)
		case 'Mounts of Mayhem':
			return formatMessage(messages.mountsOfMayhemSection)
		case 'Tiny Takeover':
			return formatMessage(messages.tinyTakeoverSection)
		case 'Chaos Cubed':
			return formatMessage(messages.chaosCubedSection)
		case 'Dungeons Hero':
			return formatMessage(messages.dungeonsHeroSection)
		case 'Dungeons II Hero':
			return formatMessage(messages.dungeonsIIHeroSection)
		case 'Wilderness Bound':
			return formatMessage(messages.wildernessBoundSection)
		case 'Default skins':
			return formatMessage(messages.defaultSkinsSection)
		default:
			return section ?? formatMessage(messages.defaultSkinsSection)
	}
}

function getDefaultSkinSectionInfoTooltip(_section: string) {
	return undefined
}

function getDefaultSkinSectionSortIndex(section: string) {
	const index = DEFAULT_SKIN_SECTION_SORT_ORDER.indexOf(section)
	return index === -1 ? DEFAULT_SKIN_SECTION_SORT_ORDER.length : index
}

function changeSkin(newSkin: Skin) {
	if (isSkinManagementReadOnly.value) return

	selectedSkin.value = newSkin
}

function resetSelectedSkin() {
	selectedSkin.value =
		skins.value.find((skin) => skinsMatch(skin, originalSelectedSkin.value)) ??
		originalSelectedSkin.value
}

function removeLocalSkin(deletedSkin: Skin) {
	const nextSkins = skins.value.filter((skin) => !skinsMatch(skin, deletedSkin))
	skins.value = nextSkins

	if (selectedSkin.value && skinsMatch(selectedSkin.value, deletedSkin)) {
		selectedSkin.value =
			nextSkins.find((skin) => skinsMatch(skin, originalSelectedSkin.value)) ??
			nextSkins.find((skin) => skin.is_equipped) ??
			null
	}

	if (originalSelectedSkin.value && skinsMatch(originalSelectedSkin.value, deletedSkin)) {
		originalSelectedSkin.value = nextSkins.find((skin) => skin.is_equipped) ?? null
	}

	void cleanupUnusedPreviews(skins.value).catch((error) =>
		console.warn('Could not clean skin previews', error),
	)
}

function setLocallyEquippedSkin(skinToApply: Skin) {
	skins.value = skins.value.map((skin) => ({
		...skin,
		is_equipped: skinsMatch(skin, skinToApply),
	}))
	originalSelectedSkin.value =
		skins.value.find((skin) => skinsMatch(skin, skinToApply)) ?? skinToApply
	selectedSkin.value = originalSelectedSkin.value
	void minecraftAccounts.value?.setEquippedSkin(originalSelectedSkin.value)
}

function insertLocalSkin(savedSkin: Skin) {
	const firstNonCustomSkinIndex = skins.value.findIndex((skin) => skin.source !== 'custom')

	if (firstNonCustomSkinIndex === -1) {
		skins.value = [...skins.value, savedSkin]
		return
	}

	const nextSkins = [...skins.value]
	nextSkins.splice(firstNonCustomSkinIndex, 0, savedSkin)
	skins.value = nextSkins
}

function updateLocalSkin(savedSkin: Skin, applied: boolean, previousSkin?: Skin) {
	let foundSkin = false
	const replacesSelectedSkin =
		selectedSkin.value?.texture_key === savedSkin.texture_key ||
		(previousSkin ? skinsMatch(selectedSkin.value, previousSkin) : false)
	const replacesOriginalSkin =
		originalSelectedSkin.value?.texture_key === savedSkin.texture_key ||
		(previousSkin ? skinsMatch(originalSelectedSkin.value, previousSkin) : false)

	skins.value = skins.value.map((skin) => {
		const isUpdatedSkin = skin.texture_key === savedSkin.texture_key
		const isPreviousSkin = previousSkin && skinsMatch(skin, previousSkin)

		if (isUpdatedSkin || isPreviousSkin) {
			foundSkin = true
			return {
				...savedSkin,
				is_equipped: applied || savedSkin.is_equipped,
			}
		}

		return {
			...skin,
			is_equipped: applied ? false : skin.is_equipped,
		}
	})

	if (!foundSkin) {
		insertLocalSkin({
			...savedSkin,
			is_equipped: applied || savedSkin.is_equipped,
		})
	}

	if (applied) {
		const locallyEquippedSkin =
			skins.value.find((skin) => skin.texture_key === savedSkin.texture_key) ?? savedSkin

		originalSelectedSkin.value = locallyEquippedSkin
		selectedSkin.value = locallyEquippedSkin
		void minecraftAccounts.value?.setEquippedSkin(locallyEquippedSkin)
	} else {
		const locallySavedSkin =
			skins.value.find((skin) => skin.texture_key === savedSkin.texture_key) ?? savedSkin

		if (replacesSelectedSkin) {
			selectedSkin.value = locallySavedSkin
		}

		if (replacesOriginalSkin) {
			originalSelectedSkin.value = locallySavedSkin
		}
	}

	void cleanupUnusedPreviews(skins.value).catch((error) =>
		console.warn('Could not clean skin previews', error),
	)
}

async function reorderSavedSkins(orderedSkins: Skin[]) {
	const previousSkins = skins.value
	const previousSelectedSkin = selectedSkin.value
	const previousOriginalSelectedSkin = originalSelectedSkin.value
	const orderedTextureKeys = orderedSkins.map((skin) => skin.texture_key)
	const orderedTextureKeySet = new Set(orderedTextureKeys)
	const remainingSavedSkins = previousSkins.filter(
		(skin) => skin.source !== 'default' && !orderedTextureKeySet.has(skin.texture_key),
	)
	const defaultSkins = previousSkins.filter((skin) => skin.source === 'default')
	const nextSavedSkins = [...orderedSkins, ...remainingSavedSkins]

	skins.value = [...nextSavedSkins, ...defaultSkins]
	void cleanupUnusedPreviews(skins.value).catch((error) =>
		console.warn('Could not clean skin previews', error),
	)

	try {
		const persistedSavedSkins = await preserveExternalSkins(nextSavedSkins)

		if (persistedSavedSkins.some((skin, index) => skin !== nextSavedSkins[index])) {
			skins.value = [...persistedSavedSkins, ...defaultSkins]
			void cleanupUnusedPreviews(skins.value).catch((error) =>
				console.warn('Could not clean skin previews', error),
			)
		}

		await set_custom_skin_order(
			persistedSavedSkins
				.filter((skin) => skin.source === 'custom')
				.map((skin) => skin.texture_key),
		)
	} catch (error) {
		skins.value = previousSkins
		selectedSkin.value = previousSelectedSkin
		originalSelectedSkin.value = previousOriginalSelectedSkin
		void cleanupUnusedPreviews(skins.value).catch((error) =>
			console.warn('Could not clean skin previews', error),
		)
		addNotification({
			type: 'error',
			title: formatMessage(messages.reorderSkinErrorTitle),
			text: error instanceof Error ? error.message : formatMessage(messages.reorderSkinErrorText),
		})
		await loadSkins()
	}
}

async function preserveExternalSkins(skinsToPersist: Skin[]) {
	const preservedSkins: Skin[] = []

	for (const skin of skinsToPersist) {
		if (skin.source !== 'custom_external') {
			preservedSkins.push(skin)
			continue
		}

		const textureBlob = await normalize_skin_texture(skin.texture)
		const capeId = skin.cape_id ? capes.value.find((cape) => cape.id === skin.cape_id) : undefined
		const savedSkin = await save_custom_skin(skin, textureBlob, skin.variant, capeId, false)
		const preservedSkin: Skin = {
			...savedSkin,
			source: 'custom',
			is_equipped: skin.is_equipped,
		}

		if (skinsMatchIgnoringSource(selectedSkin.value, skin)) {
			selectedSkin.value = preservedSkin
		}

		if (skinsMatchIgnoringSource(originalSelectedSkin.value, skin)) {
			originalSelectedSkin.value = preservedSkin
			void minecraftAccounts.value?.setEquippedSkin(preservedSkin)
		}

		preservedSkins.push(preservedSkin)
	}

	return preservedSkins
}

function schedulePendingSkinRefresh() {
	if (pendingSkinRefreshTimeout !== null) {
		window.clearTimeout(pendingSkinRefreshTimeout)
	}

	const pendingProfileId = currentUserId.value

	pendingSkinRefreshTimeout = window.setTimeout(async () => {
		pendingSkinRefreshTimeout = null

		if (isUnmounted) {
			return
		}

		try {
			if (pendingProfileId) {
				await flush_pending_skin_change_for_profile(pendingProfileId)
			} else {
				await flush_pending_skin_change()
			}
		} catch (error) {
			handleError(error as Error)
			schedulePendingSkinRefresh()
			return
		}

		if (minecraftAccounts.value) {
			await minecraftAccounts.value.refreshValues()
		}

		await loadCapes()
		await loadSkins()
	}, PENDING_SKIN_REFRESH_DELAY_MS)
}

async function applySelectedSkin() {
	const skinToApply = selectedSkin.value
	if (
		!currentUser.value ||
		!skinToApply ||
		!hasPendingSkinChange.value ||
		isApplyingSkin.value ||
		isSkinManagementReadOnly.value
	)
		return

	isApplyingSkin.value = true
	try {
		await equip_skin(skinToApply)
		setLocallyEquippedSkin(skinToApply)
		schedulePendingSkinRefresh()
	} catch (error) {
		if (isMinecraftSkinRateLimitError(error)) {
			notifications.addNotification({
				type: 'error',
				title: formatMessage(messages.rateLimitTitle),
				text: formatMessage(messages.rateLimitText),
			})
		} else {
			handleError(error as Error)
		}
	} finally {
		isApplyingSkin.value = false
	}
}

async function onSkinSaved(options: { applied: boolean; skin?: Skin; previousSkin?: Skin }) {
	if (options.skin) {
		updateLocalSkin(options.skin, options.applied, options.previousSkin)
	}

	if (!options.skin) {
		await loadCapes()
		await loadSkins()
	}

	if (options.applied) {
		schedulePendingSkinRefresh()
	}
}

async function loadCurrentUser() {
	try {
		const defaultId = await get_default_user()
		currentUserId.value = defaultId

		const allAccounts = await users()
		currentUser.value = allAccounts.find((acc) => acc.profile.id === defaultId)
	} catch (e) {
		handleError(e as Error)
		currentUser.value = undefined
		currentUserId.value = undefined
	}
}

async function login() {
	minecraftAccounts.value?.setLoginDisabled(true)
	const loggedIn = await login_flow().catch(handleSevereError)

	if (loggedIn && minecraftAccounts.value) {
		await minecraftAccounts.value.refreshValues()
	}

	trackEvent('AccountLogIn')
	minecraftAccounts.value?.setLoginDisabled(false)
}

function openAddSkinFileBrowser() {
	if (isSkinManagementReadOnly.value) return

	addSkinFileInput.value?.click()
}

async function onAddSkinFileInputChange(e: Event) {
	if (isSkinManagementReadOnly.value) return

	const files = (e.target as HTMLInputElement).files
	const file = files?.[0]

	if (!file) {
		return
	}

	await processSkinFileBuffer(await file.arrayBuffer())

	if (addSkinFileInput.value) {
		addSkinFileInput.value.value = ''
	}
}

function isSkinImagePath(path: string) {
	return path.toLowerCase().endsWith('.png')
}

function isSkinFileDrag(event: DragEvent) {
	const items = Array.from(event.dataTransfer?.items ?? [])
	const files = Array.from(event.dataTransfer?.files ?? [])

	return (
		items.some((item) => item.kind === 'file' && item.type === 'image/png') ||
		files.some((file) => file.type === 'image/png' || isSkinImagePath(file.name))
	)
}

function isPositionOverAddSkinButton(position: { x: number; y: number }) {
	const element = studioDropzoneEl.value

	if (!element) {
		return true
	}

	const { x, y } = position
	const rect = element.getBoundingClientRect()

	return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom
}

async function handleAddSkinNativeDragDrop(event: NativeDragDropEvent) {
	if (isSkinManagementReadOnly.value) return

	if (event.type === 'leave') {
		isDraggingSkinFile.value = false
		isAddSkinButtonDragActive.value = false
		return
	}

	if (event.type === 'enter') {
		isDraggingSkinFile.value = event.paths.some(isSkinImagePath)
	}

	if (event.type === 'enter' || event.type === 'over') {
		isAddSkinButtonDragActive.value =
			isDraggingSkinFile.value && isPositionOverAddSkinButton(event.position)
		return
	}

	const hasSkinPath = event.paths.some(isSkinImagePath)
	const shouldUpload =
		(isDraggingSkinFile.value || hasSkinPath) && isPositionOverAddSkinButton(event.position)

	isDraggingSkinFile.value = false
	isAddSkinButtonDragActive.value = false

	if (!shouldUpload) {
		return
	}

	const skinPath = event.paths.find(isSkinImagePath)

	if (!skinPath) {
		return
	}

	try {
		const data = await get_dragged_skin_data(skinPath)
		await processSkinFileBuffer(data)
	} catch (error) {
		addNotification({
			title: formatMessage(messages.droppedFileErrorTitle),
			text: getErrorMessage(error) || formatMessage(messages.droppedFileErrorText),
			type: 'error',
		})
	}
}

function onAddSkinDragOver(event: DragEvent) {
	if (isSkinManagementReadOnly.value) return

	if (!isSkinFileDrag(event)) {
		return
	}

	isAddSkinButtonDragActive.value = true
}

function onAddSkinDragLeave() {
	if (isSkinManagementReadOnly.value) return

	isAddSkinButtonDragActive.value = false
}

async function onAddSkinDrop(event: DragEvent) {
	if (isSkinManagementReadOnly.value) return

	isAddSkinButtonDragActive.value = false

	const file = Array.from(event.dataTransfer?.files ?? []).find(
		(file) => file.type === 'image/png' || isSkinImagePath(file.name),
	)

	if (!file) {
		return
	}

	await processSkinFileBuffer(await file.arrayBuffer())
}

async function setupAddSkinDragDropListener() {
	try {
		const unlisten = await listenNativeDragDrop(handleAddSkinNativeDragDrop)

		if (isUnmounted) {
			unlisten()
			return
		}

		unlistenAddSkinDragDrop = unlisten
	} catch (error) {
		handleError(error as Error)
	}
}

async function processSkinFileBuffer(buffer: Uint8Array | ArrayBuffer) {
	if (isSkinManagementReadOnly.value) return

	const fakeEvent = new MouseEvent('click')
	const originalSkinTexUrl = `data:image/png;base64,` + arrayBufferToBase64(buffer)
	try {
		const skinTextureNormalized = await normalize_skin_texture(originalSkinTexUrl)
		const skinTexUrl: SkinTextureUrl = {
			original: originalSkinTexUrl,
			normalized: `data:image/png;base64,` + arrayBufferToBase64(skinTextureNormalized),
		}
		editSkinModal.value?.showNew(fakeEvent, skinTexUrl)
	} catch (error) {
		handleError(error as Error)
	}
}

watch(
	() => selectedSkin.value?.cape_id,
	() => {},
)

watch(selectedSkin, () => {
	earsFeaturesEnabled.value = true
})

watch(isSkinManagementReadOnly, (readOnly) => {
	if (readOnly) {
		isDraggingSkinFile.value = false
		isAddSkinButtonDragActive.value = false
	}
})

onMounted(() => {
	window.addEventListener('offline', onOffline)
	window.addEventListener('online', onOnline)
	userCheckInterval = window.setInterval(checkUserChanges, 250)
	void setupAddSkinDragDropListener()
})

onUnmounted(() => {
	isUnmounted = true
	window.removeEventListener('offline', onOffline)
	window.removeEventListener('online', onOnline)

	if (userCheckInterval !== null) {
		window.clearInterval(userCheckInterval)
	}

	if (pendingSkinRefreshTimeout !== null) {
		window.clearTimeout(pendingSkinRefreshTimeout)
		pendingSkinRefreshTimeout = null
	}

	if (unlistenAddSkinDragDrop) {
		unlistenAddSkinDragDrop()
		unlistenAddSkinDragDrop = null
	}
})

function onOffline() {
	offline.value = true
}

function onOnline() {
	offline.value = false
	void authServerQuery.refetch()
}

async function checkUserChanges() {
	try {
		const defaultId = await get_default_user()
		if (defaultId !== currentUserId.value) {
			await minecraftAccounts.value?.refreshValues()
			await loadCurrentUser()
			await loadCapes()
			await loadSkins()
		}
	} catch (error) {
		if (currentUser.value && error instanceof Error) {
			handleError(error)
		}
	}
}

await Promise.all([loadCapes(), loadCurrentUser()])
await loadSkins()
</script>

<template>
	<EditSkinModal
		ref="editSkinModal"
		:capes="capes"
		:demo="!currentUser"
		@saved="onSkinSaved"
		@deleted="() => loadSkins()"
	/>
	<input
		ref="addSkinFileInput"
		type="file"
		accept="image/png"
		class="hidden"
		@change="onAddSkinFileInputChange"
	/>
	<ConfirmModal
		ref="deleteSkinModal"
		:title="formatMessage(messages.deleteSkinTitle)"
		:description="formatMessage(messages.deleteSkinDescription)"
		:proceed-label="formatMessage(commonMessages.deleteLabel)"
		@proceed="deleteSkin"
	/>

	<div class="solara-skin-studio w-full max-w-[1440px] mx-auto px-6 md:px-8 py-6 flex flex-col gap-6" :class="{ 'pb-28': !currentUser }">
		<!-- ============================================================
		     TOP SOLARA STUDIO COMMAND BAR
		     ============================================================ -->
		<header class="solara-studio-header rounded-2xl border border-solid border-surface-5 bg-surface-2/85 backdrop-blur-md px-5 py-4 flex flex-col lg:flex-row lg:items-center justify-between gap-4 shadow-sm">
			<div class="flex flex-col gap-1">
				<div class="flex items-center gap-2.5 flex-wrap">
					<span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[11px] font-bold uppercase tracking-[0.16em] bg-brand/12 text-brand border border-solid border-brand/25">
						<SparklesIcon class="size-3" />
						<span>Solara Skin Studio</span>
					</span>
					<span
						v-if="username"
						class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-surface-3 text-contrast border border-solid border-surface-5"
					>
						<span class="size-1.5 rounded-full bg-emerald-500" />
						<span>{{ username }}</span>
					</span>
					<span
						v-else
						class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-surface-3 text-secondary border border-solid border-surface-5"
					>
						<span>Guest Preview</span>
					</span>
				</div>
				<h1 class="m-0 text-2xl md:text-3xl font-black text-contrast tracking-tight">
					Wardrobe &amp; Cape Editor
				</h1>
			</div>

			<div class="flex flex-wrap items-center gap-3">
				<!-- Segmented Studio Tabs -->
				<div class="inline-flex items-center gap-1 p-1 rounded-xl bg-surface-3 border border-solid border-surface-5">
					<button
						type="button"
						class="solara-studio-tab px-3.5 py-1.5 rounded-lg text-xs font-bold border-0 cursor-pointer transition-all flex items-center gap-1.5"
						:class="
							studioTab === 'my-skins'
								? 'bg-brand text-[var(--color-accent-contrast)] shadow-sm'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="studioTab = 'my-skins'"
					>
						<ShirtIcon class="size-3.5" />
						<span>My Skins</span>
						<span class="opacity-80">({{ savedSkins.length }})</span>
					</button>
					<button
						type="button"
						class="solara-studio-tab px-3.5 py-1.5 rounded-lg text-xs font-bold border-0 cursor-pointer transition-all flex items-center gap-1.5"
						:class="
							studioTab === 'presets'
								? 'bg-brand text-[var(--color-accent-contrast)] shadow-sm'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="studioTab = 'presets'"
					>
						<LayersIcon class="size-3.5" />
						<span>Presets</span>
						<span class="opacity-80">({{ defaultSkins.length }})</span>
					</button>
					<button
						type="button"
						class="solara-studio-tab px-3.5 py-1.5 rounded-lg text-xs font-bold border-0 cursor-pointer transition-all flex items-center gap-1.5"
						:class="
							studioTab === 'capes'
								? 'bg-brand text-[var(--color-accent-contrast)] shadow-sm'
								: 'bg-transparent text-secondary hover:text-contrast'
						"
						@click="studioTab = 'capes'"
					>
						<SparklesIcon class="size-3.5" />
						<span>Capes</span>
						<span class="opacity-80">({{ capes.length }})</span>
					</button>
				</div>

				<Button
					type="colored"
					color="brand"
					size="md"
					:disabled="isSkinManagementReadOnly"
					@click="openAddSkinFileBrowser"
				>
					<UploadIcon class="size-4" />
					<span>Upload .PNG</span>
				</Button>
			</div>
		</header>

		<!-- ============================================================
		     MAIN 2-PANEL STUDIO WORKSPACE
		     Left (~62%): Wardrobe / Presets / Cape Vault
		     Right (~38%): Sticky Elevated 3D Character Pedestal Card
		     ============================================================ -->
		<div class="solara-studio-grid">
			<!-- LEFT PANEL: WARDROBE & CAPE DECK -->
			<section class="flex flex-col gap-5 min-w-0">
				<!-- TAB 1: MY SKINS -->
				<template v-if="studioTab === 'my-skins'">
					<!-- Interactive Drag & Drop Upload Banner -->
					<div
						ref="studioDropzoneEl"
						class="solara-dropzone rounded-2xl border-2 border-dashed p-5 flex flex-col sm:flex-row items-center justify-between gap-4 cursor-pointer transition-all"
						:class="
							isAddSkinButtonDragActive
								? 'border-brand bg-brand/10 shadow-lg'
								: 'border-surface-5 bg-surface-2/60 hover:border-brand/50 hover:bg-surface-2'
						"
						@click="openAddSkinFileBrowser"
						@dragenter.prevent="onAddSkinDragOver"
						@dragover.prevent="onAddSkinDragOver"
						@dragleave.prevent="onAddSkinDragLeave"
						@drop.prevent="onAddSkinDrop"
					>
						<div class="flex items-center gap-4">
							<div class="size-12 rounded-2xl bg-brand/15 border border-solid border-brand/30 flex items-center justify-center text-brand shrink-0">
								<UploadIcon class="size-6" />
							</div>
							<div class="flex flex-col gap-0.5 text-center sm:text-left">
								<span class="text-sm font-bold text-contrast">
									{{ isAddSkinButtonDragActive ? 'Drop your Minecraft skin .PNG now' : 'Import a Custom Skin (.PNG)' }}
								</span>
								<span class="text-xs text-secondary">
									Click to browse or drag and drop a 64x64 / 64x32 Minecraft skin file directly here.
								</span>
							</div>
						</div>
						<Button type="outlined" size="sm" :disabled="isSkinManagementReadOnly" @click.stop="openAddSkinFileBrowser">
							Choose File
						</Button>
					</div>

					<!-- Saved Skins Grid -->
					<div v-if="savedSkins.length > 0" class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 gap-4">
						<div
							v-for="skin in savedSkins"
							:key="skin.texture_key"
							class="solara-skin-card group relative rounded-2xl border border-solid p-2.5 flex flex-col gap-2 transition-all cursor-pointer"
							:class="
								isSkinSelected(skin)
									? 'border-brand bg-brand/8 shadow-md'
									: 'border-surface-5 bg-surface-2 hover:border-brand/45 hover:bg-surface-3/70'
							"
							@click="changeSkin(skin)"
						>
							<div class="flex items-center justify-between gap-1 px-1">
								<span
									v-if="isSkinActive(skin)"
									class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-emerald-500/15 text-emerald-400 border border-solid border-emerald-500/30"
								>
									<span class="size-1.5 rounded-full bg-emerald-400" />
									Active
								</span>
								<span
									v-else-if="isSkinSelected(skin)"
									class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-brand/15 text-brand border border-solid border-brand/30"
								>
									Preview
								</span>
								<span v-else class="text-[10px] font-semibold uppercase tracking-wider text-secondary">
									{{ skin.variant === 'SLIM' ? 'Slim' : 'Classic' }}
								</span>

								<div class="flex items-center gap-1 opacity-80 group-hover:opacity-100 transition-opacity">
									<button
										type="button"
										title="Customize skin"
										:disabled="isSkinManagementReadOnly"
										class="size-7 rounded-lg border border-solid border-surface-5 bg-surface-3 hover:border-brand/50 hover:text-brand text-secondary flex items-center justify-center cursor-pointer transition-colors"
										@click.stop="(e: MouseEvent) => editSkinModal?.show(e, skin)"
									>
										<EditIcon class="size-3.5" />
									</button>
									<button
										v-if="skin.source === 'custom'"
										type="button"
										title="Delete skin"
										:disabled="isSkinManagementReadOnly"
										class="size-7 rounded-lg border border-solid border-surface-5 bg-surface-3 hover:border-red/50 hover:text-red text-secondary flex items-center justify-center cursor-pointer transition-colors"
										@click.stop="confirmDeleteSkin(skin)"
									>
										<TrashIcon class="size-3.5" />
									</button>
								</div>
							</div>

							<div class="solara-skin-card-thumb rounded-xl overflow-hidden bg-surface-3/60">
								<BakedSkinButton
									:skin="skin"
									:capes="capes"
									:selected="isSkinSelected(skin)"
									class="w-full"
									@select="changeSkin(skin)"
								/>
							</div>
						</div>
					</div>

					<!-- Friendly Empty Wardrobe State -->
					<div
						v-else
						class="rounded-2xl border border-solid border-surface-5 bg-surface-2/70 p-8 text-center flex flex-col items-center gap-3"
					>
						<div class="size-12 rounded-2xl bg-surface-3 border border-solid border-surface-5 flex items-center justify-center text-secondary">
							<ShirtIcon class="size-6" />
						</div>
						<div class="flex flex-col gap-1 max-w-md">
							<h3 class="m-0 text-base font-bold text-contrast">Your Custom Wardrobe Is Empty</h3>
							<p class="m-0 text-xs text-secondary leading-relaxed">
								Upload a custom Minecraft .PNG skin above, or browse built-in presets to get started right away.
							</p>
						</div>
						<div class="flex items-center gap-2.5 mt-1">
							<Button type="colored" color="brand" size="sm" :disabled="isSkinManagementReadOnly" @click="openAddSkinFileBrowser">
								<UploadIcon class="size-4" />
								Upload Skin
							</Button>
							<Button type="outlined" size="sm" @click="studioTab = 'presets'">
								Browse Presets
							</Button>
						</div>
					</div>
				</template>

				<!-- TAB 2: BUILT-IN PRESETS -->
				<template v-else-if="studioTab === 'presets'">
					<!-- Section Filter Pills -->
					<div class="flex items-center gap-2 flex-wrap">
						<button
							type="button"
							class="px-3 py-1.5 rounded-full text-xs font-bold border border-solid cursor-pointer transition-all"
							:class="
								selectedPresetSection === 'all'
									? 'bg-brand text-[var(--color-accent-contrast)] border-brand'
									: 'bg-surface-2 text-secondary border-surface-5 hover:text-contrast hover:border-brand/40'
							"
							@click="selectedPresetSection = 'all'"
						>
							All Presets
						</button>
						<button
							v-for="section in defaultSkinSections"
							:key="section.section"
							type="button"
							class="px-3 py-1.5 rounded-full text-xs font-bold border border-solid cursor-pointer transition-all"
							:class="
								selectedPresetSection === section.section
									? 'bg-brand text-[var(--color-accent-contrast)] border-brand'
									: 'bg-surface-2 text-secondary border-surface-5 hover:text-contrast hover:border-brand/40'
							"
							@click="selectedPresetSection = section.section"
						>
							{{ section.title }} ({{ section.skins.length }})
						</button>
					</div>

					<div class="flex flex-col gap-6">
						<div
							v-for="section in filteredPresetSections"
							:key="section.section"
							class="flex flex-col gap-3"
						>
							<div class="flex items-center justify-between">
								<h2 class="m-0 text-sm font-extrabold uppercase tracking-wider text-secondary">
									{{ section.title }}
								</h2>
								<span class="text-xs text-secondary">{{ section.skins.length }} skins</span>
							</div>

							<div class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 gap-3.5">
								<div
									v-for="skin in section.skins"
									:key="skin.texture_key"
									class="solara-skin-card group relative rounded-2xl border border-solid p-2.5 flex flex-col gap-2 transition-all cursor-pointer"
									:class="
										isSkinSelected(skin)
											? 'border-brand bg-brand/8 shadow-md'
											: 'border-surface-5 bg-surface-2 hover:border-brand/45 hover:bg-surface-3/70'
									"
									@click="changeSkin(skin)"
								>
									<div class="flex items-center justify-between gap-1 px-1">
										<span
											v-if="isSkinActive(skin)"
											class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-emerald-500/15 text-emerald-400 border border-solid border-emerald-500/30"
										>
											Active
										</span>
										<span
											v-else-if="isSkinSelected(skin)"
											class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider bg-brand/15 text-brand border border-solid border-brand/30"
										>
											Preview
										</span>
										<span v-else class="text-[10px] font-semibold uppercase tracking-wider text-secondary">
											{{ skin.variant === 'SLIM' ? 'Slim' : 'Classic' }}
										</span>

										<button
											type="button"
											title="Customize with cape"
											:disabled="isSkinManagementReadOnly"
											class="size-7 rounded-lg border border-solid border-surface-5 bg-surface-3 hover:border-brand/50 hover:text-brand text-secondary flex items-center justify-center cursor-pointer transition-colors opacity-80 group-hover:opacity-100"
											@click.stop="(e: MouseEvent) => editSkinModal?.show(e, skin)"
										>
											<EditIcon class="size-3.5" />
										</button>
									</div>

									<div class="solara-skin-card-thumb rounded-xl overflow-hidden bg-surface-3/60">
										<BakedSkinButton
											:skin="skin"
											:capes="capes"
											:selected="isSkinSelected(skin)"
											class="w-full"
											@select="changeSkin(skin)"
										/>
									</div>
								</div>
							</div>
						</div>
					</div>
				</template>

				<!-- TAB 3: CAPE VAULT -->
				<template v-else>
					<div class="rounded-2xl border border-solid border-surface-5 bg-surface-2/80 p-5 flex flex-col gap-4">
						<div class="flex items-center justify-between gap-2">
							<div class="flex flex-col gap-0.5">
								<h2 class="m-0 text-base font-extrabold text-contrast">Cape Vault</h2>
								<p class="m-0 text-xs text-secondary">
									Click any unlocked cape to preview it live on your 3D character, then hit Apply Skin.
								</p>
							</div>
							<span class="px-2.5 py-1 rounded-lg text-xs font-bold bg-surface-3 text-contrast border border-solid border-surface-5">
								{{ capes.length }} Available
							</span>
						</div>

						<div class="flex flex-wrap gap-3">
							<!-- No Cape Option -->
							<button
								type="button"
								class="solara-cape-none-card w-[5.75rem] h-[7.25rem] rounded-xl border-2 border-solid flex flex-col items-center justify-center gap-2 cursor-pointer transition-all"
								:class="
									!currentCape
										? 'border-brand bg-brand/10 text-brand'
										: 'border-surface-5 bg-surface-3 text-secondary hover:border-brand/40 hover:text-contrast'
								"
								@click="selectCapeForCurrentSkin(undefined)"
							>
								<XIcon class="size-5" />
								<span class="text-xs font-bold">No Cape</span>
							</button>

							<CapeButton
								v-for="cape in capes"
								:id="cape.id"
								:key="cape.id"
								:texture="cape.texture"
								:name="cape.name || 'Cape'"
								:selected="currentCape?.id === cape.id"
								@select="selectCapeForCurrentSkin(cape)"
							/>
						</div>

						<div
							v-if="capes.length === 0"
							class="rounded-xl border border-solid border-surface-5 bg-surface-3/60 p-4 text-xs text-secondary"
						>
							{{ currentUser ? 'No Minecraft capes found on this account yet.' : 'Sign in with your Microsoft account to view and equip your unlocked Minecraft capes.' }}
						</div>
					</div>
				</template>
			</section>

			<!-- RIGHT PANEL: STICKY ELEVATED 3D CHARACTER PEDESTAL -->
			<aside class="solara-pedestal-col lg:sticky lg:top-6 self-start">
				<div class="solara-pedestal-card relative rounded-3xl border border-solid border-surface-5 bg-surface-2/90 p-5 flex flex-col gap-4 shadow-xl overflow-hidden">
					<div class="solara-pedestal-ambient pointer-events-none absolute inset-0" />

					<!-- Pedestal Top Bar: Status + Arm Toggle + Nametag Toggle -->
					<div class="relative z-10 flex items-center justify-between gap-2 flex-wrap">
						<span
							v-if="hasPendingSkinChange"
							class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold bg-brand/15 text-brand border border-solid border-brand/35"
						>
							<EyeIcon class="size-3.5" />
							<span>{{ formatMessage(messages.previewingBadge) }}</span>
						</span>
						<span
							v-else
							class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-bold bg-emerald-500/12 text-emerald-400 border border-solid border-emerald-500/30"
						>
							<span class="size-2 rounded-full bg-emerald-400" />
							<span>{{ currentUser ? 'Equipped' : 'Preview' }}</span>
						</span>

						<div class="flex items-center gap-1.5">
							<!-- Quick Classic / Slim Switcher -->
							<div
								v-if="canQuickToggleVariant"
								class="inline-flex p-0.5 rounded-lg bg-surface-3 border border-solid border-surface-5"
							>
								<button
									type="button"
									class="px-2 py-1 rounded-md text-[11px] font-bold border-0 cursor-pointer transition-colors"
									:class="
										skinVariant !== 'SLIM'
											? 'bg-brand text-[var(--color-accent-contrast)]'
											: 'bg-transparent text-secondary hover:text-contrast'
									"
									@click="setQuickArmVariant('CLASSIC')"
								>
									Classic
								</button>
								<button
									type="button"
									class="px-2 py-1 rounded-md text-[11px] font-bold border-0 cursor-pointer transition-colors"
									:class="
										skinVariant === 'SLIM'
											? 'bg-brand text-[var(--color-accent-contrast)]'
											: 'bg-transparent text-secondary hover:text-contrast'
									"
									@click="setQuickArmVariant('SLIM')"
								>
									Slim
								</button>
							</div>
							<span
								v-else
								class="px-2.5 py-1 rounded-lg text-[11px] font-bold bg-surface-3 text-secondary border border-solid border-surface-5"
							>
								{{ skinVariant === 'SLIM' ? 'Slim' : 'Classic' }}
							</span>

							<!-- Nametag Toggle -->
							<button
								type="button"
								:title="showNametagOverride ? 'Hide nametag' : 'Show nametag'"
								class="size-7 rounded-lg border border-solid border-surface-5 bg-surface-3 hover:border-brand/45 text-secondary hover:text-contrast flex items-center justify-center cursor-pointer transition-colors"
								@click="showNametagOverride = !showNametagOverride"
							>
								<EyeIcon v-if="showNametagOverride" class="size-3.5" />
								<EyeOffIcon v-else class="size-3.5" />
							</button>
						</div>
					</div>

					<!-- 3D Animated Character Viewport -->
					<div class="relative z-10 w-full h-[24rem] xl:h-[27rem] flex items-center justify-center">
						<SkinPreviewRenderer
							:cape-src="capeTexture"
							:texture-src="activeStudioTexture"
							:ears-texture-src="selectedSkin?.texture"
							:variant="skinVariant"
							:nametag="skinNametag"
							framing="modal"
							:initial-rotation="-Math.PI / 10"
							:ears-enabled="earsFeaturesEnabled"
							class="h-full w-full"
							@ears-features-detected="selectedSkinHasEarsFeatures = $event"
						/>
					</div>

					<!-- Ears Mod Notice (if detected) -->
					<div
						v-if="selectedSkinHasEarsFeatures"
						class="relative z-10 rounded-xl border border-solid border-surface-5 bg-surface-3/80 px-3 py-2.5 flex items-center justify-between gap-2.5"
					>
						<div class="flex items-center gap-2 min-w-0">
							<img
								:src="EarsModIcon"
								alt=""
								class="size-8 shrink-0 rounded-md border border-solid border-surface-5 object-cover"
							/>
							<span class="text-xs font-semibold text-primary truncate">Ears mod features</span>
						</div>
						<Toggle v-model="earsFeaturesEnabled" small />
					</div>

					<!-- Pedestal Bottom Action Dock -->
					<div class="relative z-10 pt-3 border-0 border-t border-solid border-surface-5 flex items-center gap-2.5">
						<template v-if="hasPendingSkinChange">
							<Button
								type="outlined"
								size="md"
								class="flex-1 justify-center"
								:disabled="isApplyingSkin || isSkinManagementReadOnly"
								@click="resetSelectedSkin"
							>
								<RotateCounterClockwiseIcon class="size-4" />
								<span>{{ formatMessage(commonMessages.resetButton) }}</span>
							</Button>
							<Button
								v-tooltip="!currentUser ? formatMessage(messages.demoApplyTooltip) : undefined"
								type="colored"
								color="brand"
								size="md"
								class="flex-1 justify-center"
								:disabled="!currentUser || isApplyingSkin || isSkinManagementReadOnly"
								@click="applySelectedSkin"
							>
								<SpinnerIcon v-if="isApplyingSkin" class="size-4 animate-spin" />
								<CheckIcon v-else class="size-4" />
								<span>{{ formatMessage(messages.applyButton) }}</span>
							</Button>
						</template>
						<template v-else>
							<Button
								type="outlined"
								size="md"
								class="flex-1 justify-center"
								:disabled="!selectedSkin || isSkinManagementReadOnly"
								@click="(e: MouseEvent) => selectedSkin && editSkinModal?.show(e, selectedSkin)"
							>
								<EditIcon class="size-4" />
								<span>Customize</span>
							</Button>
							<Button
								type="colored"
								color="brand"
								size="md"
								class="flex-1 justify-center"
								:disabled="isSkinManagementReadOnly"
								@click="openAddSkinFileBrowser"
							>
								<UploadIcon class="size-4" />
								<span>Upload</span>
							</Button>
						</template>
					</div>
				</div>
			</aside>
		</div>
	</div>

	<!-- Guest Sign-In Strip (when not signed in) -->
	<div v-if="!currentUser" class="sticky w-full bottom-0 z-20 px-6 pb-4">
		<div
			class="mx-auto flex w-full max-w-4xl items-center justify-between gap-4 rounded-2xl border border-solid border-brand/35 bg-surface-2/95 backdrop-blur-xl px-5 py-3.5 shadow-xl"
		>
			<div class="flex min-w-0 items-center gap-3">
				<div class="size-9 rounded-xl bg-brand/15 text-brand flex items-center justify-center shrink-0">
					<InfoIcon class="size-5" />
				</div>
				<div class="flex min-w-0 flex-col">
					<p class="m-0 text-sm font-bold text-contrast">
						{{ formatMessage(messages.demoTitle) }}
					</p>
					<p class="m-0 text-xs text-secondary">
						{{ formatMessage(messages.demoDescription) }}
					</p>
				</div>
			</div>
			<Button
				v-show="minecraftAccounts"
				type="colored"
				color="brand"
				size="md"
				:disabled="minecraftAccounts?.loginDisabled"
				@click="login"
			>
				<SpinnerIcon v-if="minecraftAccounts?.loginDisabled" class="size-4 animate-spin" />
				<LogInIcon v-else class="size-4" />
				<span>Sign In</span>
			</Button>
		</div>
	</div>
</template>

<style lang="scss" scoped>
.solara-studio-grid {
	display: grid;
	grid-template-columns: minmax(0, 1.55fr) minmax(20rem, 1fr);
	gap: 1.75rem;
	align-items: start;

	@media (max-width: 980px) {
		grid-template-columns: 1fr;
	}
}

.solara-pedestal-ambient {
	background: radial-gradient(
		65% 55% at 50% 45%,
		color-mix(in srgb, var(--color-brand) 14%, transparent) 0%,
		transparent 72%
	);
}

.solara-skin-card {
	transition:
		transform 0.16s ease,
		border-color 0.16s ease,
		background-color 0.16s ease,
		box-shadow 0.16s ease;
}

.solara-skin-card:hover {
	transform: translateY(-2px);
	box-shadow: 0 12px 24px -14px color-mix(in srgb, var(--color-brand) 40%, transparent);
}

.solara-skin-card-thumb :deep(button) {
	background: transparent !important;
	border: none !important;
	box-shadow: none !important;
}
</style>
