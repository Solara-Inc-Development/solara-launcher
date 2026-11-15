<script setup lang="ts">
import { defineMessages, useScrollIndicator, useVIntl } from '@modrinth/ui'
import { OverlayScrollbars } from 'overlayscrollbars'
import { computed, onMounted, onScopeDispose, ref } from 'vue'

import { useTheme } from '@/composables/use-theme'
import AccountsCard from '@/features/minecraft-accounts/accounts-card.vue'
import type { MinecraftAccountsActions } from '@/features/minecraft-accounts/context'
import NewsSidebar from '@/features/news/news-sidebar.vue'
import OnboardingChecklist from '@/features/onboarding/checklist.vue'

defineProps<{
	visible: boolean
}>()

const emit = defineEmits<{
	'create-instance': []
}>()

const { formatMessage } = useVIntl()
const appTheme = useTheme()
const accounts = ref<InstanceType<typeof AccountsCard> | null>(null)
const scrollContainer = ref<HTMLElement | null>(null)
const scrollViewport = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState } = useScrollIndicator(scrollViewport)

let removeScrollbarListener: (() => void) | undefined

onMounted(() => {
	if (!scrollContainer.value) return
	const scrollbars = OverlayScrollbars(scrollContainer.value)
	if (!scrollbars) return
	scrollViewport.value = scrollbars.elements().viewport
	removeScrollbarListener = scrollbars.on('updated', checkScrollState)
})

onScopeDispose(() => removeScrollbarListener?.())

const minecraftAccounts = computed<MinecraftAccountsActions | null>(() => {
	const card = accounts.value
	if (!card) return null
	return {
		loginDisabled: card.loginDisabled,
		refreshValues: card.refreshValues,
		setEquippedSkin: card.setEquippedSkin,
		setLoginDisabled: card.setLoginDisabled,
		login: card.login,
	}
})

function showAddFriendModal() {}

defineExpose({ minecraftAccounts, showAddFriendModal })

const scrollbarOptions = Object.freeze({
	overflow: {
		x: 'hidden',
		y: 'scroll',
	},
})

const messages = defineMessages({
	playingAs: {
		id: 'app.sidebar.playing-as',
		defaultMessage: 'Playing as',
	},
})
</script>

<template>
	<div
		class="app-sidebar mt-px relative flex h-[calc(100vh-var(--top-bar-height))] w-[300px] shrink-0 flex-col overflow-visible border-0 border-l-[1px] border-solid border-[--brand-gradient-border] before:pointer-events-none before:absolute before:inset-y-0 before:-left-8 before:w-8 before:content-['']"
		:class="{
			'before:shadow-[-15px_0_15px_-15px_rgba(0,0,0,0.1)_inset]': appTheme.advancedRendering,
		}"
	>
		<div class="relative flex min-h-0 grow shrink flex-col">
			<div
				ref="scrollContainer"
				v-overlay-scrollbars="scrollbarOptions"
				class="app-sidebar-scrollable relative min-h-0 grow shrink"
				data-overlayscrollbars-initialize
			>
				<OnboardingChecklist
					@create-instance="emit('create-instance')"
					@login-minecraft="accounts?.login()"
				/>
				<div
					id="sidebar-teleport-target"
					class="sidebar-teleport-content empty:hidden"
				></div>
				<div class="sidebar-default-content hidden" :class="{ 'sidebar-enabled': visible }">
					<div
						class="p-4 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid"
					>
						<h3 class="text-base text-primary font-medium m-0">
							{{ formatMessage(messages.playingAs) }}
						</h3>
						<Suspense>
							<AccountsCard ref="accounts" />
						</Suspense>
					</div>
					<NewsSidebar />
				</div>
			</div>
			<div
				class="sidebar-scroll-fade pointer-events-none absolute inset-x-0 top-0 z-10 h-8 rotate-180 transition-opacity duration-200 motion-reduce:transition-none"
				:class="showTopFade ? 'opacity-100' : 'opacity-0'"
				aria-hidden="true"
			/>
			<div
				class="sidebar-scroll-fade pointer-events-none absolute inset-x-0 bottom-0 z-10 h-8 transition-opacity duration-200 motion-reduce:transition-none"
				:class="showBottomFade ? 'opacity-100' : 'opacity-0'"
				aria-hidden="true"
			/>
		</div>
	</div>
</template>

<style scoped>
.app-sidebar {
	background: var(--brand-gradient-bg);
	--color-button-bg: var(--brand-gradient-button);
	--surface-4: var(--brand-gradient-button);
	--color-button-bg-hover: var(--brand-gradient-border);
	--surface-5: var(--brand-gradient-border);
	--color-divider: var(--brand-gradient-border);
	--color-divider-dark: var(--brand-gradient-border);
}

.sidebar-teleport-content:empty + .sidebar-default-content.sidebar-enabled {
	display: contents;
}

.sidebar-scroll-fade {
	background: var(--brand-gradient-fade-out-color);
}
</style>
