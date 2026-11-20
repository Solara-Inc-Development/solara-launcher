<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	injectAuth,
	injectUserPreferences,
	provideAppearanceSettings,
	useSavable,
} from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, watch } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import {
	type ColorTheme,
	isDarkTheme,
	normalizeSolaraTheme,
	useTheme,
} from '@/composables/use-theme.ts'
import { catalogFeatures } from '@/config'
import { injectAppSettingsModal } from '@/features/settings/app-settings-modal/context'
import { type AppSettings, get, set } from '@/helpers/settings.ts'

const theme = useTheme()
const appSettings = useAppSettings()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const settingsModal = injectAppSettingsModal()

type AppearanceSettingsState = {
	theme: ColorTheme
	syncAcrossDevices: boolean
}

function getAppearanceSettingsState(): AppearanceSettingsState {
	return {
		theme: normalizeSolaraTheme(theme.preferred),
		syncAcrossDevices: theme.syncAcrossDevices,
	}
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	getAppearanceSettingsState,
	async (appearanceChanges) => {
		const value = current.value
		if (
			value.syncAcrossDevices &&
			auth.user.value &&
			(appearanceChanges.theme !== undefined || appearanceChanges.syncAcrossDevices !== undefined)
		) {
			await updatePreferences({
				appearance: value.theme === 'system' ? { auto: true } : { auto: false, theme: value.theme },
			})
		}

		const nextSettings: AppSettings = {
			...(await get()),
			theme: value.theme,
			sync_theme_across_devices: value.syncAcrossDevices,
			advanced_rendering: false,
			native_decorations: false,
		}

		await set(nextSettings)
		if (isDarkTheme(value.theme)) {
			theme.preferredDark = value.theme
		}
		theme.preferred = value.theme
		theme.syncAcrossDevices = value.syncAcrossDevices
		theme.advancedRendering = false
		appSettings.nativeDecorations = false
	},
)

const themeOptions = computed<ColorTheme[]>(() => ['oled', 'light'])

const preferredDarkTheme = computed(() => 'oled' as const)

function setTheme(value: ColorTheme): void {
	current.value.theme = normalizeSolaraTheme(value)
}

function setSyncAcrossDevices(enabled: boolean): void {
	current.value.syncAcrossDevices = enabled
}

watch(
	[() => current.value.theme, () => saved.value.theme],
	([selectedTheme, savedTheme]) => {
		theme.preview = selectedTheme === savedTheme ? null : selectedTheme
	},
	{ immediate: true },
)

async function saveAppearanceSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
	}
}

onMounted(() => {
	settingsModal.registerUnsavedChangesController({
		hasChanges: () => hasChanges.value,
		getOriginal: () => saved.value,
		getModified: () => changes.value,
		isSaving: () => saving.value,
		reset,
		save: saveAppearanceSettings,
	})
})

onBeforeUnmount(() => {
	theme.preview = null
	settingsModal.registerUnsavedChangesController(null)
})

provideAppearanceSettings({
	deferPersistence: true,
	theme: {
		current: computed(() => current.value.theme),
		options: themeOptions,
		system: computed(() => (theme.native === 'light' ? 'light' : preferredDarkTheme.value)),
		preferredDark: preferredDarkTheme,
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => current.value.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	updatePreferences,
})
</script>

<template>
	<AppearanceSettingsLayout :show-account-sync="catalogFeatures.cloudSettings" />
</template>
