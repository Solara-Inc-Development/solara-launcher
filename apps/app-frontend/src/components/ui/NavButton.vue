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
			variant === 'cta'
				? 'solara-nav-cta h-8 px-3 rounded-xl text-[13px] font-bold flex items-center justify-center gap-1.5 cursor-pointer select-none transition-all duration-200 border border-solid border-brand/35 bg-brand/15 text-brand hover:bg-brand hover:text-[var(--color-accent-contrast)] hover:border-brand active:scale-95'
				: 'solara-nav-tab h-8 px-3.5 rounded-xl text-[13px] font-semibold flex items-center justify-center gap-2 cursor-pointer select-none transition-all duration-200 border-0 bg-transparent text-secondary hover:text-contrast',
			{
				'router-link-active': (isPrimary && isPrimary(route)) || highlightOverride,
				'subpage-active': isSubpage && isSubpage(route) && !(isPrimary && isPrimary(route)),
				'opacity-45 !cursor-not-allowed pointer-events-none': disabled,
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
		variant?: 'tab' | 'cta'
	}>(),
	{
		disabled: false,
		variant: 'tab',
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
.solara-nav-tab {
	position: relative;
	z-index: 0;
	text-decoration: none;

	:deep(svg) {
		width: 1.05rem;
		height: 1.05rem;
		flex-shrink: 0;
		transition:
			color 0.2s ease,
			transform 0.25s var(--ease-out-expo),
			filter 0.2s ease;
	}

	&:hover:not(.router-link-active) {
		background-color: color-mix(in srgb, var(--color-contrast) 6%, transparent);

		:deep(svg) {
			transform: translateY(-0.5px);
		}
	}

	&::before {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: 0.75rem;
		background: linear-gradient(
			135deg,
			color-mix(in srgb, var(--color-brand) 20%, transparent),
			color-mix(in srgb, var(--color-brand) 8%, transparent)
		);
		border: 1px solid color-mix(in srgb, var(--color-brand) 38%, transparent);
		box-shadow: 0 0 14px color-mix(in srgb, var(--color-brand) 14%, transparent);
		opacity: 0;
		scale: 0.78;
		z-index: -1;
		transition:
			opacity 0.24s var(--ease-out-expo),
			scale 0.24s var(--ease-out-expo);
	}

	&::after {
		content: '';
		position: absolute;
		bottom: 2px;
		left: 50%;
		transform: translateX(-50%) scaleX(0.3);
		width: 1.15rem;
		height: 2.5px;
		border-radius: 999px;
		background-color: var(--color-brand);
		box-shadow: 0 0 8px var(--color-brand);
		opacity: 0;
		transition:
			opacity 0.24s var(--ease-out-expo),
			transform 0.24s var(--ease-out-expo);
	}
}

.solara-nav-tab.router-link-active {
	color: var(--color-contrast);
	font-weight: 700;

	:deep(svg) {
		color: var(--color-brand);
		filter: drop-shadow(0 0 6px color-mix(in srgb, var(--color-brand) 45%, transparent));
	}

	&::before {
		opacity: 1;
		scale: 1;
	}

	&::after {
		opacity: 1;
		transform: translateX(-50%) scaleX(1);
	}
}

.solara-nav-tab.subpage-active {
	color: var(--color-contrast);
	background-color: color-mix(in srgb, var(--color-brand) 10%, transparent);

	:deep(svg) {
		color: var(--color-brand);
	}
}

.solara-nav-cta {
	:deep(svg) {
		width: 1rem;
		height: 1rem;
		flex-shrink: 0;
	}
}
</style>
