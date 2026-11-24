<template>
	<component
		:is="isLink ? RouterLink : 'button'"
		:to="isLink ? to : undefined"
		v-bind="$attrs"
		:disabled="isLink ? undefined : disabled"
		:active-class="isLink && isSubpage ? '' : undefined"
		:class="[
			'solara-nav-item',
			{
				'router-link-active': isPrimary && isPrimary(route),
				'subpage-active': isSubpage && isSubpage(route),
				'is-primary-cta': variant === 'primary',
				disabled: disabled,
			},
		]"
		@click="onClick"
	>
		<span class="solara-nav-indicator" aria-hidden="true" />
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
		variant?: 'default' | 'primary'
	}>(),
	{
		disabled: false,
		variant: 'default',
	},
)

const isLink = computed(() => typeof props.to === 'string')

function onClick() {
	if (typeof props.to === 'function') {
		props.to()
	}
}

defineOptions({
	inheritAttrs: false,
})
</script>

<style lang="scss" scoped>
.solara-nav-item {
	position: relative;
	display: flex;
	align-items: center;
	gap: 0.65rem;
	width: 100%;
	height: 2.45rem;
	padding: 0 0.78rem;
	border: 0;
	border-radius: 12px;
	background: transparent;
	color: var(--color-secondary);
	font-size: 0.84rem;
	font-weight: 600;
	letter-spacing: -0.005em;
	text-decoration: none;
	cursor: pointer;
	user-select: none;
	transition:
		background-color 150ms ease,
		color 150ms ease,
		transform 120ms ease;

	:deep(svg) {
		width: 1.1rem;
		height: 1.1rem;
		flex-shrink: 0;
		transition: color 150ms ease, transform 150ms ease;
	}

	&:hover:not(.disabled) {
		background: rgba(255, 255, 255, 0.055);
		color: var(--color-contrast);
	}

	&:active:not(.disabled) {
		transform: scale(0.985);
	}

	&.disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
}

.solara-nav-indicator {
	position: absolute;
	left: 0;
	top: 22%;
	bottom: 22%;
	width: 3px;
	border-radius: 999px;
	background: #3dd6f5;
	box-shadow: 0 0 10px rgba(61, 214, 245, 0.75);
	opacity: 0;
	transform: scaleY(0.4);
	transition:
		opacity 160ms ease,
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.solara-nav-item.router-link-active {
	background: rgba(61, 214, 245, 0.13);
	color: #7ee8ff;

	:deep(svg) {
		color: #3dd6f5;
	}

	.solara-nav-indicator {
		opacity: 1;
		transform: scaleY(1);
	}
}

.solara-nav-item.subpage-active {
	background: rgba(255, 255, 255, 0.06);
	color: var(--color-contrast);

	.solara-nav-indicator {
		opacity: 0.55;
		transform: scaleY(0.75);
	}
}

.solara-nav-item.is-primary-cta {
	background: rgba(61, 214, 245, 0.14);
	color: #7ee8ff;
	font-weight: 700;
	justify-content: center;

	:deep(svg) {
		color: #3dd6f5;
	}

	&:hover:not(.disabled) {
		background: #3dd6f5;
		color: #041017;

		:deep(svg) {
			color: #041017;
		}
	}
}
</style>
