<script setup lang="ts">
import { CheckIcon, RadioButtonIcon } from '@modrinth/assets'
import { Accordion, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, onUnmounted, ref } from 'vue'

import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'

const emit = defineEmits<{
	'create-instance': []
	'login-minecraft': []
}>()

const { formatMessage } = useVIntl()
const {
	hasCreatedInstance,
	hasLoggedIntoMinecraft, isReady,
	showChecklist } = injectOnboardingChecklist()
const collapsedCornersVisible = ref(false)
let collapseTimer: ReturnType<typeof setTimeout> | undefined

const messages = defineMessages({
	title: {
		id: 'onboarding-checklist.title',
		defaultMessage: 'Getting started',
	},
	createInstance: {
		id: 'onboarding-checklist.create-instance',
		defaultMessage: 'Create first instance',
	},
	loginMinecraft: {
		id: 'onboarding-checklist.login-minecraft',
		defaultMessage: 'Sign in to Minecraft',
	},
})

const steps = computed(() => [
	{
		id: 'create-instance',
		label: formatMessage(messages.createInstance),
		complete: hasCreatedInstance.value,
		action: () => emit('create-instance'),
	},
	{
		id: 'login-minecraft',
		label: formatMessage(messages.loginMinecraft),
		complete: hasLoggedIntoMinecraft.value,
		action: () => emit('login-minecraft'),
	},
])

const accordionButtonClass = computed(
	() =>
		`flex w-full cursor-pointer items-center justify-between rounded-t-2xl border-0 bg-bg-raised p-3.5 text-left text-contrast transition-colors hover:bg-white/[0.06]${collapsedCornersVisible.value ? ' rounded-b-2xl' : ''}`,
)

function handleAccordionOpen() {
	clearTimeout(collapseTimer)
	collapsedCornersVisible.value = false
}

function handleAccordionClose() {
	clearTimeout(collapseTimer)
	collapseTimer = setTimeout(() => {
		collapsedCornersVisible.value = true
	}, 300)
}

onUnmounted(() => clearTimeout(collapseTimer))
</script>

<template>
	<div
		v-if="isReady && showChecklist"
		class="px-3.5 pt-3.5 pb-1"
	>
		<Accordion
			open-by-default
			class="w-full overflow-hidden rounded-2xl text-sm"
			:button-class="accordionButtonClass"
			content-class="flex flex-col gap-2 rounded-b-2xl border-0 bg-bg-raised px-3 pb-3 pt-1"
			@on-open="handleAccordionOpen"
			@on-close="handleAccordionClose"
		>
			<template #title>
				<span class="font-bold text-xs uppercase tracking-[0.1em] text-brand leading-5">
					{{ formatMessage(messages.title) }}
				</span>
			</template>
			<button
				v-for="step in steps"
				:key="step.id"
				type="button"
				class="flex h-10 w-full items-center gap-2.5 rounded-xl border-0 bg-white/[0.04] px-3.5 py-2 text-left text-primary transition-colors"
				:class="
					step.complete
						? '!cursor-default opacity-50'
						: 'cursor-pointer hover:bg-brand/15 hover:text-contrast'
				"
				:disabled="step.complete"
				@click="step.action"
			>
				<span
					v-if="step.complete"
					class="flex size-[18px] items-center justify-center rounded-full bg-brand mr-0.5 shrink-0"
				>
					<CheckIcon class="size-3 text-black [stroke-width:3]" />
				</span>
				<RadioButtonIcon v-else class="size-4 text-brand shrink-0" />
				<span
					class="min-w-0 truncate font-semibold text-xs leading-5"
					:class="{ 'text-secondary line-through': step.complete }"
				>
					{{ step.label }}
				</span>
			</button>
		</Accordion>
	</div>
</template>
