<template>
	<div
		ref="outerRef"
		data-tauri-drag-region
		class="min-w-0 overflow-hidden"
		:class="{
			'[mask-image:linear-gradient(to_right,transparent,black_12px,black_calc(100%_-_12px),transparent)]':
				isOverflowing,
		}"
		:style="isOverflowing ? { '--scroll-distance': `-${overflowAmount}px` } : undefined"
		@mouseenter="onMouseEnter"
		@mouseleave="onMouseLeave"
	>
		<div
			v-if="breadcrumbs.length > 0"
			ref="innerRef"
			data-tauri-drag-region
			class="flex w-fit items-center gap-1.5 pr-4"
			:class="{ 'breadcrumbs-scroll': isAnimating }"
			@animationiteration="onAnimationIteration"
		>
			<template v-for="(breadcrumb, index) in breadcrumbs" :key="breadcrumb.slot">
				<component
					:is="index < breadcrumbs.length - 1 && breadcrumb.to ? RouterLink : 'span'"
					v-bind="index < breadcrumbs.length - 1 && breadcrumb.to ? { to: breadcrumb.to } : {}"
					:data-tauri-drag-region="index === breadcrumbs.length - 1 ? '' : undefined"
					class="flex shrink-0 items-center gap-1.5 whitespace-nowrap text-[13px] font-semibold leading-5 no-underline transition-colors"
					:class="
						index === breadcrumbs.length - 1
							? 'cursor-default select-none text-contrast'
							: 'text-secondary hover:text-brand'
					"
					:aria-current="index === breadcrumbs.length - 1 ? 'page' : undefined"
				>
					<Avatar
						v-if="breadcrumb.visual?.type === 'image'"
						:src="breadcrumb.visual.src"
						:alt="breadcrumb.visual.alt ?? breadcrumb.label"
						:circle="breadcrumb.visual.circle"
						:tint-by="breadcrumb.visual.tintBy ?? breadcrumb.id"
						size="18px"
						no-shadow
						raised
						class="inline-block shrink-0 align-middle"
						:class="{ '!rounded-md': !breadcrumb.visual.circle }"
					/>
					<component
						:is="breadcrumb.visual.component"
						v-else-if="breadcrumb.visual?.type === 'icon'"
						class="size-4 shrink-0"
						:class="index === breadcrumbs.length - 1 ? 'text-brand' : 'text-secondary'"
						aria-hidden="true"
					/>
					<span>{{ breadcrumb.label }}</span>
				</component>
				<span
					v-if="index < breadcrumbs.length - 1"
					data-tauri-drag-region
					class="text-xs font-bold text-secondary/40 select-none px-0.5"
					aria-hidden="true"
				>
					/
				</span>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Avatar } from '@modrinth/ui'
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { RouterLink } from 'vue-router'

import { injectBreadcrumbManager } from '@/shared/breadcrumbs'

const { entries: breadcrumbs } = injectBreadcrumbManager()

// Overflow detection
const outerRef = ref<HTMLDivElement | null>(null)
const innerRef = ref<HTMLDivElement | null>(null)
const isOverflowing = ref(false)
const isAnimating = ref(false)
const overflowAmount = ref(0)

let hovered = false
let stopping = false

function checkOverflow() {
	if (!outerRef.value || !innerRef.value) return
	const outerStyles = window.getComputedStyle(outerRef.value)
	const horizontalPadding =
		Number.parseFloat(outerStyles.paddingLeft) + Number.parseFloat(outerStyles.paddingRight)
	const availableWidth = outerRef.value.clientWidth - horizontalPadding
	const overflow = innerRef.value.scrollWidth - availableWidth
	isOverflowing.value = overflow > 0
	overflowAmount.value = overflow
}

function onMouseEnter() {
	hovered = true
	stopping = false
	if (isOverflowing.value) {
		isAnimating.value = true
	}
}

function onMouseLeave() {
	hovered = false
	if (isAnimating.value) {
		stopping = true
	}
}

function onAnimationIteration() {
	if (stopping && !hovered) {
		isAnimating.value = false
		stopping = false
	}
}

let resizeObserver: ResizeObserver | null = null

onMounted(() => {
	checkOverflow()
	resizeObserver = new ResizeObserver(checkOverflow)
	if (outerRef.value) resizeObserver.observe(outerRef.value)
	if (innerRef.value) resizeObserver.observe(innerRef.value)
})

onBeforeUnmount(() => {
	resizeObserver?.disconnect()
})

watch(breadcrumbs, () => {
	requestAnimationFrame(checkOverflow)
})
</script>

<style scoped>
.solara-breadcrumb-pill {
	background: rgba(255, 255, 255, 0.045);
}

:global(.light-mode) .solara-breadcrumb-pill {
	background: rgba(15, 23, 42, 0.06);
}

.breadcrumbs-scroll {
	animation: breadcrumb-scroll 10s ease-in-out infinite;
}

@keyframes breadcrumb-scroll {
	0% {
		transform: translateX(0);
	}
	35%,
	65% {
		transform: translateX(var(--scroll-distance));
	}
	100% {
		transform: translateX(0);
	}
}
</style>
