<script setup lang="ts" generic="T extends string">
import {
	MoonIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SolaraIcon,
	SunIcon,
} from '@modrinth/assets'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

const { formatMessage } = useVIntl()

const { ariaLabel, modelValue, themeOptions, systemThemeColor, preferredDarkTheme } = defineProps<{
	ariaLabel: string
	modelValue: T
	themeOptions: readonly T[]
	systemThemeColor: T
	preferredDarkTheme: T
}>()

const emit = defineEmits<{
	'update:modelValue': [theme: T]
}>()

const themeLabels = defineMessages({
	system: {
		id: 'settings.display.theme.system',
		defaultMessage: 'Sync with system',
	},
	light: {
		id: 'settings.display.theme.light',
		defaultMessage: 'White',
	},
	dark: {
		id: 'settings.display.theme.dark',
		defaultMessage: 'Dark',
	},
	oled: {
		id: 'settings.display.theme.oled',
		defaultMessage: 'OLED',
	},
	retro: {
		id: 'settings.display.theme.retro',
		defaultMessage: 'Retro',
	},
})

const themeTooltips = defineMessages({
	preferredLight: {
		id: 'settings.display.theme.preferred-light-theme',
		defaultMessage: 'Preferred light theme',
	},
	preferredDark: {
		id: 'settings.display.theme.preferred-dark-theme',
		defaultMessage: 'Preferred dark theme',
	},
})

function formatTheme(theme: T): string {
	const message = themeLabels[theme as keyof typeof themeLabels]
	return message ? formatMessage(message) : theme
}

function getPreviewClass(option: T): string {
	const base = option === 'system' ? systemThemeColor : option
	return base.endsWith('-mode') ? base : `${base}-mode`
}

function isLightOption(option: T): boolean {
	const base = option === 'system' ? systemThemeColor : option
	return base === 'light'
}
</script>

<template>
	<div class="theme-options" role="group" :aria-label="ariaLabel">
		<button
			v-for="option in themeOptions"
			:key="option"
			type="button"
			class="solara-theme-card preview-radio button-base"
			:class="{ selected: modelValue === option }"
			:aria-pressed="modelValue === option"
			@click="emit('update:modelValue', option)"
		>
			<div
				class="solara-preview-stage"
				:class="[getPreviewClass(option), isLightOption(option) ? 'is-white' : 'is-oled']"
				aria-hidden="true"
			>
				<!-- Mini Solara Title Bar -->
				<div class="mini-titlebar">
					<div class="mini-brand">
						<SolaraIcon class="mini-brand-logo" />
						<span class="mini-brand-bar" />
					</div>
					<div class="mini-pills">
						<span class="mini-pill" />
						<span class="mini-pill is-wide" />
					</div>
				</div>

				<!-- Mini Solara Workspace -->
				<div class="mini-body">
					<div class="mini-sidebar">
						<span class="mini-nav-item is-active" />
						<span class="mini-nav-item" />
						<span class="mini-nav-item" />
						<span class="mini-nav-item is-bottom" />
					</div>
					<div class="mini-canvas">
						<div class="mini-ambient-glow" />
						<div class="mini-hero-logo">
							<SolaraIcon class="mini-hero-svg" />
						</div>
						<div class="mini-hero-title" />
						<div class="mini-hero-name" />
						<div class="mini-hero-cta" />
					</div>
				</div>
			</div>

			<div class="solara-theme-footer label">
				<RadioButtonCheckedIcon
					v-if="modelValue === option"
					class="radio shrink-0"
					aria-hidden="true"
				/>
				<RadioButtonIcon v-else class="radio shrink-0" aria-hidden="true" />
				<span class="theme-name">{{ formatTheme(option) }}</span>
				<span class="theme-subtag">
					{{ isLightOption(option) ? 'Clean Daylight' : 'True Black' }}
				</span>
				<SunIcon
					v-if="'light' === option"
					v-tooltip="formatMessage(themeTooltips.preferredLight)"
					class="theme-icon ml-auto shrink-0"
					aria-hidden="true"
				/>
				<MoonIcon
					v-else-if="option === preferredDarkTheme"
					v-tooltip="formatMessage(themeTooltips.preferredDark)"
					class="theme-icon ml-auto shrink-0"
					aria-hidden="true"
				/>
			</div>
		</button>
	</div>
</template>

<style scoped lang="scss">
.theme-options {
	display: grid;
	grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
	gap: 1.125rem;
}

.solara-theme-card {
	display: flex;
	flex-direction: column;
	overflow: hidden;
	border-radius: 1rem;
	padding: 0;
	border: 1.5px solid color-mix(in srgb, var(--surface-5) 75%, transparent);
	background: var(--surface-3);
	transition:
		transform 180ms ease,
		border-color 180ms ease,
		box-shadow 180ms ease;

	&:hover {
		transform: translateY(-2px);
		border-color: rgba(56, 189, 248, 0.45);
	}

	&.selected {
		border-color: #38bdf8;
		box-shadow: 0 0 0 1px rgba(56, 189, 248, 0.35), 0 10px 26px rgba(56, 189, 248, 0.14);
	}
}

.solara-preview-stage {
	position: relative;
	display: flex;
	flex-direction: column;
	width: 100%;
	height: 8.5rem;
	overflow: hidden;
	user-select: none;

	&.is-oled {
		background: #000000;
		--mini-bar-bg: #080b10;
		--mini-rail-bg: #070a0f;
		--mini-border: rgba(255, 255, 255, 0.08);
		--mini-muted: rgba(148, 163, 184, 0.25);
		--mini-title: #f8fafc;
		--mini-cta-bg: rgba(56, 189, 248, 0.14);
		--mini-cta-border: rgba(56, 189, 248, 0.55);
		--mini-nav-active: rgba(56, 189, 248, 0.24);
		--mini-nav-active-ring: #38bdf8;
		--mini-glow: radial-gradient(
			circle at 50% 42%,
			rgba(56, 189, 248, 0.24) 0%,
			rgba(251, 191, 36, 0.12) 42%,
			transparent 74%
		);
	}

	&.is-white {
		background: #f8fafc;
		--mini-bar-bg: #ffffff;
		--mini-rail-bg: #f1f5f9;
		--mini-border: rgba(15, 23, 42, 0.09);
		--mini-muted: rgba(100, 116, 139, 0.24);
		--mini-title: #0f172a;
		--mini-cta-bg: rgba(2, 132, 199, 0.1);
		--mini-cta-border: rgba(2, 132, 199, 0.45);
		--mini-nav-active: rgba(2, 132, 199, 0.16);
		--mini-nav-active-ring: #0284c7;
		--mini-glow: radial-gradient(
			circle at 50% 42%,
			rgba(56, 189, 248, 0.22) 0%,
			rgba(251, 191, 36, 0.14) 44%,
			transparent 76%
		);
	}
}

.mini-titlebar {
	display: flex;
	align-items: center;
	justify-content: space-between;
	height: 1.5rem;
	padding: 0 0.625rem;
	background: var(--mini-bar-bg);
	border-bottom: 1px solid var(--mini-border);
	flex-shrink: 0;
}

.mini-brand {
	display: flex;
	align-items: center;
	gap: 0.35rem;
}

.mini-brand-logo {
	width: 0.8rem;
	height: 0.8rem;
	flex-shrink: 0;
}

.mini-brand-bar {
	width: 2.8rem;
	height: 0.3rem;
	border-radius: 999px;
	background: var(--mini-muted);
}

.mini-pills {
	display: flex;
	align-items: center;
	gap: 0.25rem;
}

.mini-pill {
	width: 1.15rem;
	height: 0.45rem;
	border-radius: 999px;
	background: var(--mini-muted);

	&.is-wide {
		width: 1.85rem;
	}
}

.mini-body {
	display: flex;
	flex: 1;
	min-height: 0;
}

.mini-sidebar {
	display: flex;
	flex-direction: column;
	align-items: center;
	gap: 0.35rem;
	width: 1.65rem;
	padding: 0.45rem 0;
	background: var(--mini-rail-bg);
	border-right: 1px solid var(--mini-border);
	flex-shrink: 0;
}

.mini-nav-item {
	width: 0.8rem;
	height: 0.8rem;
	border-radius: 0.35rem;
	background: var(--mini-muted);

	&.is-active {
		background: var(--mini-nav-active);
		box-shadow: inset 0 0 0 1px var(--mini-nav-active-ring);
	}

	&.is-bottom {
		margin-top: auto;
	}
}

.mini-canvas {
	position: relative;
	display: flex;
	flex: 1;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	gap: 0.35rem;
	padding: 0.5rem;
	overflow: hidden;
}

.mini-ambient-glow {
	position: absolute;
	inset: 0;
	background: var(--mini-glow);
	pointer-events: none;
}

.mini-hero-logo {
	position: relative;
	z-index: 1;
	display: flex;
	align-items: center;
	justify-content: center;
	width: 1.9rem;
	height: 1.9rem;
	border-radius: 0.55rem;
	background: rgba(15, 23, 42, 0.65);
	box-shadow: 0 4px 14px rgba(56, 189, 248, 0.22);
}

.is-white .mini-hero-logo {
	background: #0f172a;
}

.mini-hero-svg {
	width: 1.45rem;
	height: 1.45rem;
}

.mini-hero-title {
	position: relative;
	z-index: 1;
	width: 4.8rem;
	height: 0.42rem;
	border-radius: 999px;
	background: var(--mini-title);
	opacity: 0.9;
}

.mini-hero-name {
	position: relative;
	z-index: 1;
	width: 3.1rem;
	height: 0.44rem;
	border-radius: 999px;
	background: linear-gradient(90deg, #38bdf8 0%, #60a5fa 52%, #fbbf24 100%);
}

.mini-hero-cta {
	position: relative;
	z-index: 1;
	margin-top: 0.2rem;
	width: 3.6rem;
	height: 0.85rem;
	border-radius: 999px;
	background: var(--mini-cta-bg);
	border: 1px solid var(--mini-cta-border);
}

.solara-theme-footer {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	width: 100%;
	padding: 0.75rem 0.9rem;
	border-top: 1px solid color-mix(in srgb, var(--surface-5) 55%, transparent);
	background: var(--surface-3);

	.theme-name {
		font-weight: 650;
		color: var(--color-contrast);
	}

	.theme-subtag {
		font-size: 0.72rem;
		font-weight: 500;
		color: var(--color-secondary);
	}
}
</style>
