<template>
	<component
		:is="isLink ? RouterLink : 'button'"
		:to="isLink ? to : undefined"
		v-bind="$attrs"
		:disabled="isLink ? undefined : disabled"
		:active-class="isLink && isSubpage ? '' : undefined"
		:class="[
			'solara-dock-btn',
			{
				'router-link-active': isPrimary && isPrimary(route),
				'subpage-active': isSubpage && isSubpage(route),
				'is-accent-cta': variant === 'primary',
				disabled: disabled,
			},
		]"
		@click="onClick"
	>
		<span class="solara-dock-pip" aria-hidden="true" />
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
.solara-dock-btn {
	position: relative;
	display: flex;
	align-items: center;
	justify-content: center;
	width: 42px;
	height: 42px;
	padding: 0;
	border: 1px solid transparent;
	border-radius: 14px;
	background: transparent;
	color: var(--color-secondary);
	text-decoration: none;
	cursor: pointer;
	user-select: none;
	flex-shrink: 0;
	transition:
		background-color 160ms ease,
		border-color 160ms ease,
		color 160ms ease,
		transform 160ms cubic-bezier(0.22, 1, 0.36, 1),
		box-shadow 180ms ease;

	:deep(svg) {
		width: 20px;
		height: 20px;
		flex-shrink: 0;
		transition: transform 160ms ease, color 160ms ease;
	}

	&:hover:not(.disabled) {
		background: rgba(255, 255, 255, 0.08);
		color: var(--color-contrast);
		transform: translateY(-1px);
	}

	&:active:not(.disabled) {
		transform: scale(0.94);
	}

	&.disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
}

:global(.light-mode) .solara-dock-btn:hover:not(.disabled) {
	background: rgba(15, 23, 42, 0.07);
}

.solara-dock-pip {
	position: absolute;
	left: -7px;
	top: 50%;
	width: 3px;
	height: 18px;
	border-radius: 999px;
	background: #3dd6f5;
	box-shadow: 0 0 10px rgba(61, 214, 245, 0.9);
	opacity: 0;
	transform: translateY(-50%) scaleY(0.35);
	transition:
		opacity 160ms ease,
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.solara-dock-btn.router-link-active {
	background: linear-gradient(145deg, rgba(61, 214, 245, 0.22), rgba(56, 189, 248, 0.09));
	border-color: rgba(61, 214, 245, 0.38);
	color: #7ee8ff;
	box-shadow:
		0 0 18px rgba(61, 214, 245, 0.14),
		inset 0 1px 0 rgba(255, 255, 255, 0.12);

	:deep(svg) {
		color: #5ee2ff;
	}

	.solara-dock-pip {
		opacity: 1;
		transform: translateY(-50%) scaleY(1);
	}
}

:global(.light-mode) .solara-dock-btn.router-link-active {
	background: rgba(14, 165, 233, 0.14);
	border-color: rgba(14, 165, 233, 0.35);
	color: #0284c7;
	box-shadow: none;

	:deep(svg) {
		color: #0284c7;
	}
}

.solara-dock-btn.subpage-active {
	background: rgba(255, 255, 255, 0.075);
	color: var(--color-contrast);

	.solara-dock-pip {
		opacity: 0.55;
		transform: translateY(-50%) scaleY(0.65);
	}
}

.solara-dock-btn.is-accent-cta {
	background: rgba(61, 214, 245, 0.14);
	border-color: rgba(61, 214, 245, 0.28);
	color: #6ce4ff;

	&:hover:not(.disabled) {
		background: #3dd6f5;
		border-color: #3dd6f5;
		color: #041017;
		box-shadow: 0 4px 16px rgba(61, 214, 245, 0.35);

		:deep(svg) {
			color: #041017;
		}
	}
}
</style>
