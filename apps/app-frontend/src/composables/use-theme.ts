import { prepareThemeColorTransition } from '@modrinth/ui'
import { computed, reactive, ref, watch } from 'vue'

import { readThemePreference, themePreferenceKeys } from '@/helpers/theme-preferences'

export const THEME_OPTIONS = ['dark', 'light', 'oled', 'retro', 'system'] as const
export const DARK_THEMES = ['dark', 'oled', 'retro'] as const

export type ColorTheme = (typeof THEME_OPTIONS)[number]
export type DarkTheme = (typeof DARK_THEMES)[number]
type Theme = Exclude<ColorTheme, 'system'>
type NativeTheme = 'light' | 'dark'

const PREFERRED_THEME_KEY = themePreferenceKeys.theme
const PREFERRED_DARK_THEME_KEY = themePreferenceKeys.darkTheme

export function isDarkTheme(theme: string): theme is DarkTheme {
	return (DARK_THEMES as readonly string[]).includes(theme)
}

export function normalizeSolaraTheme(theme: string | null | undefined): 'oled' | 'light' {
	return theme === 'light' ? 'light' : 'oled'
}

function loadPreferredTheme(): ColorTheme {
	try {
		const stored = readThemePreference(
			window.localStorage,
			PREFERRED_THEME_KEY,
			'modrinth-theme',
			THEME_OPTIONS,
		)
		if (stored) return normalizeSolaraTheme(stored)
	} catch {
		// storage blocked or full
	}

	if (document.documentElement.classList.contains('light-mode')) {
		return 'light'
	}

	return 'oled'
}

function loadPreferredDarkTheme(): DarkTheme {
	try {
		const stored = readThemePreference(
			window.localStorage,
			PREFERRED_DARK_THEME_KEY,
			'modrinth-preferred-dark-theme',
			DARK_THEMES,
		)
		if (stored) return 'oled'
	} catch {
		// storage blocked or full
	}

	return 'oled'
}

const preferred = ref<ColorTheme>(loadPreferredTheme())
const preview = ref<ColorTheme | null>(null)
const preferredDark = ref<DarkTheme>(loadPreferredDarkTheme())
const advancedRendering = ref(true)
const syncAcrossDevices = ref(false)
const nativeThemeQuery = window.matchMedia('(prefers-color-scheme: dark)')
const native = ref<NativeTheme>(nativeThemeQuery.matches ? 'dark' : 'light')
const active = computed<Theme>(() => {
	const selectedTheme = preview.value ?? preferred.value
	return normalizeSolaraTheme(selectedTheme)
})

nativeThemeQuery.addEventListener('change', (event) => {
	native.value = event.matches ? 'dark' : 'light'
})

watch([preferred, preview], ([selectedPreferred, selectedPreview]) => {
	const selectedTheme = selectedPreview ?? selectedPreferred
	if (isDarkTheme(selectedTheme)) {
		preferredDark.value = selectedTheme
	}
})

watch(
	preferred,
	(theme) => {
		try {
			window.localStorage.setItem(PREFERRED_THEME_KEY, theme)
		} catch {
			// storage blocked or full
		}
	},
	{ immediate: true },
)

watch(preferredDark, (theme) => {
	try {
		window.localStorage.setItem(PREFERRED_DARK_THEME_KEY, theme)
	} catch {
		// storage blocked or full
	}
})

watch(
	active,
	(theme, previousTheme) => {
		if (previousTheme && previousTheme !== theme) {
			prepareThemeColorTransition()
		}

		const html = document.documentElement
		for (const option of THEME_OPTIONS) {
			html.classList.remove(`${option}-mode`)
		}
		html.classList.add(`${theme}-mode`)
	},
	{ immediate: true },
)

function applyAccountAppearance(appearance: { auto: boolean; theme: string }): void {
	preferredDark.value = 'oled'
	preferred.value = normalizeSolaraTheme(appearance.theme)
}

const theme = reactive({
	preferred,
	preview,
	preferredDark,
	active,
	native,
	syncAcrossDevices,
	advancedRendering,
	options: THEME_OPTIONS,
	applyAccountAppearance,
})

export function useTheme() {
	return theme
}
