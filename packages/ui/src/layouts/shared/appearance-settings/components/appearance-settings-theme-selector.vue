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
						<div class="mini-header-row">
							<div class="mini-header-copy">
								<span class="mini-line is-primary" />
								<span class="mini-line is-secondary" />
							</div>
							<span class="mini-action-btn" />
						</div>
						<div class="mini-cards-grid">
							<div class="mini-card">
								<span class="mini-card-thumb" />
								<span class="mini-card-line" />
							</div>
							<div class="mini-card">
								<span class="mini-card-thumb" />
								<span class="mini-card-line" />
							</div>
							<div class="mini-card">
								<span class="mini-card-thumb" />
								<span class="mini-card-line" />
							</div>
						</div>
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
	grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr));
	gap: 1rem;
}

.solara-theme-card {
	display: flex;
	flex-direction: column;
	overflow: hidden;
	border-radius: 0.875rem;
	padding: 0;
	border: 2px solid var(--surface-5);
	background: var(--surface-3);
	transition: border-color 150ms ease;

	&:hover {
		border-color: var(--color-button-bg-selected);
	}

	&.selected {
		border-color: var(--color-brand);
	}
}

.solara-preview-stage {
	position: relative;
	display: flex;
	flex-direction: column;
	width: 100%;
	height: 8rem;
	overflow: hidden;
	user-select: none;

	&.is-oled {
		background: #000000;
		--mini-bar-bg: #090b0f;
		--mini-rail-bg: #07090c;
		--mini-card-bg: #0f1318;
		--mini-thumb-bg: #181e26;
		--mini-border: rgba(255, 255, 255, 0.08);
		--mini-muted: rgba(148, 163, 184, 0.24);
		--mini-title: #f1f5f9;
		--mini-cta-bg: #38bdf8;
		--mini-nav-active: rgba(56, 189, 248, 0.2);
		--mini-nav-active-ring: #38bdf8;
	}

	&.is-white {
		background: #f8fafc;
		--mini-bar-bg: #ffffff;
		--mini-rail-bg: #f1f5f9;
		--mini-card-bg: #ffffff;
		--mini-thumb-bg: #e2e8f0;
		--mini-border: rgba(15, 23, 42, 0.08);
		--mini-muted: rgba(100, 116, 139, 0.24);
		--mini-title: #0f172a;
		--mini-cta-bg: #0284c7;
		--mini-nav-active: rgba(2, 132, 199, 0.14);
		--mini-nav-active-ring: #0284c7;
	}
}

.mini-titlebar {
	display: flex;
	align-items: center;
	justify-content: space-between;
	height: 1.4rem;
	padding: 0 0.6rem;
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
	width: 0.75rem;
	height: 0.75rem;
	flex-shrink: 0;
}

.mini-brand-bar {
	width: 2.6rem;
	height: 0.28rem;
	border-radius: 999px;
	background: var(--mini-muted);
}

.mini-pills {
	display: flex;
	align-items: center;
	gap: 0.25rem;
}

.mini-pill {
	width: 1rem;
	height: 0.4rem;
	border-radius: 999px;
	background: var(--mini-muted);

	&.is-wide {
		width: 1.6rem;
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
	gap: 0.32rem;
	width: 1.55rem;
	padding: 0.4rem 0;
	background: var(--mini-rail-bg);
	border-right: 1px solid var(--mini-border);
	flex-shrink: 0;
}

.mini-nav-item {
	width: 0.72rem;
	height: 0.72rem;
	border-radius: 0.3rem;
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
	display: flex;
	flex: 1;
	flex-direction: column;
	justify-content: space-between;
	gap: 0.5rem;
	padding: 0.65rem 0.75rem;
	overflow: hidden;
}

.mini-header-row {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 0.5rem;
}

.mini-header-copy {
	display: flex;
	flex-direction: column;
	gap: 0.25rem;
}

.mini-line {
	display: block;
	border-radius: 999px;

	&.is-primary {
		width: 4.5rem;
		height: 0.4rem;
		background: var(--mini-title);
		opacity: 0.85;
	}

	&.is-secondary {
		width: 3rem;
		height: 0.28rem;
		background: var(--mini-muted);
	}
}

.mini-action-btn {
	width: 2.1rem;
	height: 0.75rem;
	border-radius: 0.3rem;
	background: var(--mini-cta-bg);
	flex-shrink: 0;
}

.mini-cards-grid {
	display: grid;
	grid-template-columns: repeat(3, minmax(0, 1fr));
	gap: 0.4rem;
	flex: 1;
	min-height: 0;
}

.mini-card {
	display: flex;
	flex-direction: column;
	justify-content: space-between;
	padding: 0.35rem;
	border-radius: 0.4rem;
	background: var(--mini-card-bg);
	border: 1px solid var(--mini-border);
}

.mini-card-thumb {
	width: 1.1rem;
	height: 1.1rem;
	border-radius: 0.25rem;
	background: var(--mini-thumb-bg);
}

.mini-card-line {
	width: 80%;
	height: 0.26rem;
	border-radius: 999px;
	background: var(--mini-muted);
}

.solara-theme-footer {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	width: 100%;
	padding: 0.7rem 0.85rem;
	border-top: 1px solid var(--surface-5);
	background: var(--surface-3);

	.theme-name {
		font-weight: 600;
		color: var(--color-contrast);
	}
}
</style>
