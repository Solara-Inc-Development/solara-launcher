<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	injectAuth,
	injectUserPreferences,
	provideAppearanceSettings,
	Toggle,
	useSavable,
} from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

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

const discordRpcEnabled = ref(true)
const savingDiscordRpc = ref(false)

onMounted(async () => {
	try {
		const loaded = await get()
		discordRpcEnabled.value = Boolean(loaded.discord_rpc)
	} catch {
		discordRpcEnabled.value = true
	}
})

async function toggleDiscordRpc(nextValue: boolean) {
	if (savingDiscordRpc.value) return
	const previous = discordRpcEnabled.value
	discordRpcEnabled.value = nextValue
	savingDiscordRpc.value = true
	try {
		const currentSettings = await get()
		await set({
			...currentSettings,
			discord_rpc: nextValue,
		})
	} catch {
		discordRpcEnabled.value = previous
	} finally {
		savingDiscordRpc.value = false
	}
}

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
	<div class="flex flex-col gap-5">
		<AppearanceSettingsLayout :show-account-sync="catalogFeatures.cloudSettings" />

		<section class="flex items-center justify-between gap-4 rounded-2xl bg-bg-raised p-4">
			<div class="flex min-w-0 flex-col gap-1">
				<h3 class="m-0 text-sm font-bold text-contrast">Discord Rich Presence</h3>
				<p class="m-0 text-xs leading-relaxed text-secondary">
					Automatically show Solara Launcher, your active Minecraft instance, loader, and play timer
					on your Discord profile.
				</p>
			</div>
			<Toggle
				id="solara-discord-rpc"
				:model-value="discordRpcEnabled"
				:disabled="savingDiscordRpc"
				class="shrink-0"
				@update:model-value="toggleDiscordRpc"
			/>
		</section>
	</div>
</template>
