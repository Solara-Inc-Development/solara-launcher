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
		class="app-sidebar mt-px relative flex h-[calc(100vh-var(--top-bar-height))] w-[300px] shrink-0 flex-col overflow-visible border-0 border-l-[1px] border-solid border-white/[0.06]"
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
					<div class="p-3.5">
						<div class="rounded-2xl bg-bg-raised p-3.5 flex flex-col gap-2">
							<h3 class="text-[11px] uppercase tracking-[0.12em] text-brand font-bold m-0">
								{{ formatMessage(messages.playingAs) }}
							</h3>
							<Suspense>
								<AccountsCard ref="accounts" />
							</Suspense>
						</div>
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
	background: #000000;
}

.sidebar-teleport-content:empty + .sidebar-default-content.sidebar-enabled {
	display: contents;
}

.sidebar-scroll-fade {
	background: linear-gradient(to bottom, rgba(0, 0, 0, 0), rgba(0, 0, 0, 0.92));
}
</style>
