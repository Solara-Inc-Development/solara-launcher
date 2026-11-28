<template>
	<component
		:is="isLink ? RouterLink : 'button'"
		:to="isLink ? to : undefined"
		:type="isLink ? undefined : 'button'"
		v-bind="$attrs"
		:disabled="isLink ? undefined : disabled"
		:active-class="isLink && isSubpage ? '' : undefined"
		data-tauri-drag-region-exclude
		:class="[
			'solara-rail-btn size-11 rounded-[13px] flex items-center justify-center text-xl cursor-pointer select-none transition-all duration-200',
			variant === 'cta' ? 'solara-rail-cta' : 'solara-rail-squircle',
			{
				'router-link-active': (isPrimary && isPrimary(route)) || highlightOverride,
				'subpage-active': isSubpage && isSubpage(route) && !(isPrimary && isPrimary(route)),
				'opacity-40 !cursor-not-allowed pointer-events-none': disabled,
			},
		]"
		@click="onClick"
	>
		<slot />
	</component>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { RouteLocationNormalizedLoaded } from 'vue-router'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

type RouteFunction = (route: RouteLocationNormalizedLoaded) => boolean

const props = withDefaults(
	defineProps<{
		to: (() => void) | string
		isPrimary?: RouteFunction
		isSubpage?: RouteFunction
		highlightOverride?: boolean
		disabled?: boolean
		variant?: 'default' | 'cta'
	}>(),
	{
		disabled: false,
		variant: 'default',
	},
)

const isLink = computed(() => typeof props.to === 'string')

function onClick() {
	if (props.disabled) return
	if (typeof props.to === 'function') {
		props.to()
	}
}

defineOptions({
	inheritAttrs: false,
})
</script>

<style lang="scss" scoped>
.solara-rail-btn {
	position: relative;
	text-decoration: none;

	:deep(svg) {
		width: 1.25rem;
		height: 1.25rem;
		flex-shrink: 0;
		transition:
			color 0.2s ease,
			transform 0.22s var(--ease-out-expo),
			filter 0.2s ease;
	}

	/* Discord-style left wall glowing indicator pip */
	&::after {
		content: '';
		position: absolute;
		left: -12px;
		top: 50%;
		transform: translateY(-50%) scaleY(0);
		width: 3.5px;
		height: 10px;
		border-radius: 0 999px 999px 0;
		background-color: var(--color-contrast);
		opacity: 0;
		pointer-events: none;
		transition:
			height 0.24s var(--ease-out-expo),
			transform 0.24s var(--ease-out-expo),
			opacity 0.2s ease,
			background-color 0.2s ease,
			box-shadow 0.2s ease;
	}
}

.solara-rail-squircle {
	color: var(--color-secondary);
	background-color: color-mix(in srgb, var(--color-bg) 62%, transparent);
	border: 1px solid var(--surface-4);
	box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);

	&:hover:not(.router-link-active) {
		color: var(--color-contrast);
		border-radius: 15px;
		background-color: color-mix(in srgb, var(--color-bg) 85%, var(--color-contrast) 6%);
		border-color: var(--surface-5);

		:deep(svg) {
			transform: scale(1.06);
		}

		&::after {
			opacity: 0.85;
			height: 11px;
			transform: translateY(-50%) scaleY(1);
		}
	}

	&:active {
		transform: scale(0.95);
	}
}

.solara-rail-squircle.router-link-active {
	color: var(--color-brand);
	border-radius: 15px;
	background: linear-gradient(
		145deg,
		color-mix(in srgb, var(--color-brand) 22%, var(--color-bg)),
		color-mix(in srgb, var(--color-brand) 8%, var(--color-bg))
	);
	border-color: color-mix(in srgb, var(--color-brand) 48%, transparent);
	box-shadow:
		0 0 16px color-mix(in srgb, var(--color-brand) 16%, transparent),
		inset 0 1px 0 color-mix(in srgb, var(--color-brand) 24%, transparent);

	:deep(svg) {
		color: var(--color-brand);
		filter: drop-shadow(0 0 6px color-mix(in srgb, var(--color-brand) 45%, transparent));
	}

	&::after {
		opacity: 1;
		height: 22px;
		transform: translateY(-50%) scaleY(1);
		background-color: var(--color-brand);
		box-shadow: 0 0 10px var(--color-brand);
	}
}

.solara-rail-squircle.subpage-active {
	color: var(--color-brand);
	background-color: color-mix(in srgb, var(--color-brand) 12%, var(--color-bg));
	border-color: color-mix(in srgb, var(--color-brand) 32%, transparent);

	&::after {
		opacity: 0.9;
		height: 14px;
		transform: translateY(-50%) scaleY(1);
		background-color: var(--color-brand);
	}
}

.solara-rail-cta {
	color: var(--color-brand);
	background-color: color-mix(in srgb, var(--color-brand) 14%, var(--color-bg));
	border: 1px solid color-mix(in srgb, var(--color-brand) 38%, transparent);

	&:hover {
		color: var(--color-accent-contrast);
		background-color: var(--color-brand);
		border-color: var(--color-brand);
		border-radius: 15px;
		box-shadow: 0 0 16px color-mix(in srgb, var(--color-brand) 32%, transparent);
	}

	&:active {
		transform: scale(0.94);
	}
}
</style>
