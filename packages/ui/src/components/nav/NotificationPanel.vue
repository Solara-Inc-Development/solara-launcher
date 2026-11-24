<template>
	<div
		class="vue-notification-group"
		:class="{
			'intercom-present': isIntercomPresent,
			'location-left': notificationLocation === 'left',
			'location-right': notificationLocation === 'right',
			'has-sidebar': hasSidebar && !hasModalActive,
			'exit-fade': exitAnimation === 'fade',
		}"
	>
		<transition-group name="notifs">
			<div
				v-for="(item, index) in notifications"
				:key="item.id"
				class="vue-notification-wrapper"
				@mouseenter="stopTimer(item)"
				@mouseleave="setNotificationTimer(item)"
			>
				<div
					class="solara-toast group relative flex w-full flex-col overflow-hidden rounded-2xl"
					:class="[
						item.containerClass,
						{
							'solara-toast--error': item.type === 'error',
							'solara-toast--warning': item.type === 'warning',
							'solara-toast--success': item.type === 'success',
							'solara-toast--info': !item.type || item.type === 'info',
							'solara-toast--neutral': item.type === 'neutral',
						},
					]"
				>
					<!-- Signature thin luminous Solara top accent hairline -->
					<div class="solara-toast-hairline pointer-events-none h-[2px] w-full shrink-0" />

					<div class="flex items-start gap-3 px-3.5 py-3">
						<div
							v-if="!item.noIcon"
							class="solara-toast-icon-badge mt-0.5 flex size-8 shrink-0 items-center justify-center rounded-xl"
							:class="{
								'solara-badge--error': item.type === 'error',
								'solara-badge--warning': item.type === 'warning',
								'solara-badge--success': item.type === 'success',
								'solara-badge--info': !item.type || item.type === 'info',
								'solara-badge--neutral': item.type === 'neutral',
							}"
						>
							<IssuesIcon v-if="item.type === 'warning'" class="size-4" />
							<CheckCircleIcon v-else-if="item.type === 'success'" class="size-4" />
							<XCircleIcon v-else-if="item.type === 'error'" class="size-4" />
							<InfoIcon v-else class="size-4" />
						</div>

						<div class="flex min-w-0 flex-1 flex-col gap-1">
							<div class="flex items-start justify-between gap-2">
								<div class="flex min-w-0 flex-wrap items-center gap-1.5">
									<div class="m-0 text-[13.5px] font-bold leading-snug text-contrast">
										{{ item.title }}
									</div>
									<span
										v-if="item.count && item.count > 1"
										class="rounded-full bg-surface-4 px-1.5 py-0.5 text-[10.5px] font-bold leading-none text-contrast tabular-nums"
									>
										×{{ item.count }}
									</span>
								</div>

								<div class="flex shrink-0 items-center gap-0.5">
									<button
										v-if="item.copyable !== false"
										v-tooltip="
											item.supportData ? 'Copy error details for support' : 'Copy to clipboard'
										"
										type="button"
										:aria-label="
											item.supportData ? 'Copy error details for support' : 'Copy to clipboard'
										"
										class="flex size-6 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent p-0 text-secondary/75 transition-all hover:bg-surface-4 hover:text-contrast"
										@click="copyToClipboard(item)"
									>
										<CheckIcon v-if="copied[getCopyKey(item)]" class="size-3.5 text-brand" />
										<CopyIcon v-else class="size-3.5" />
									</button>
									<button
										v-if="item.dismissible !== false"
										v-tooltip="`Dismiss`"
										type="button"
										aria-label="Dismiss"
										class="flex size-6 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent p-0 text-secondary/75 transition-all hover:bg-surface-4 hover:text-contrast"
										@click="dismissNotification(index)"
									>
										<XIcon class="size-3.5" />
									</button>
								</div>
							</div>

							<div
								v-if="item.text"
								class="max-h-[45vh] overflow-y-auto whitespace-pre-wrap text-[12.5px] leading-relaxed text-secondary"
							>
								{{ item.text }}
							</div>

							<div v-if="item.errorCode" class="pt-0.5">
								<span
									class="inline-block rounded-md bg-surface-4 px-2 py-0.5 font-mono text-[11px] font-medium text-secondary"
								>
									{{ item.errorCode }}
								</span>
							</div>

							<div v-if="item.buttons?.length" class="flex flex-wrap gap-1.5 pt-1.5">
								<Button
									v-for="(button, buttonIndex) in item.buttons"
									:key="buttonIndex"
									size="sm"
									class="!rounded-xl"
									:type="button.color && button.color !== 'standard' ? 'colored' : 'base'"
									:color="
										button.color && button.color !== 'standard'
											? button.color === 'medal-promo'
												? 'medal_promotion'
												: button.color
											: undefined
									"
									@click="handleButtonClick(item, button)"
								>
									<component :is="button.icon" v-if="button.icon" />
									{{ button.label }}
								</Button>
							</div>
						</div>
					</div>
				</div>
			</div>
		</transition-group>
	</div>
</template>

<script setup lang="ts">
import {
	CheckCircleIcon,
	CheckIcon,
	CopyIcon,
	InfoIcon,
	IssuesIcon,
	XCircleIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import { useModalStack } from '#ui/composables/modal-stack.ts'

import {
	injectNotificationManager,
	type WebNotification,
	type WebNotificationButton,
} from '../../providers'

const notificationManager = injectNotificationManager()
const notifications = computed<WebNotification[]>(() => notificationManager.getNotifications())
const notificationLocation = computed(() => notificationManager.getNotificationLocation())

const isIntercomPresent = ref<boolean>(false)
const copied = ref<Record<string, boolean>>({})

const stopTimer = (n: WebNotification) => notificationManager.stopNotificationTimer(n)
const setNotificationTimer = (n: WebNotification) => notificationManager.setNotificationTimer(n)
const dismissNotification = (n: number) => notificationManager.removeNotificationByIndex(n)

async function handleButtonClick(item: WebNotification, button: WebNotificationButton) {
	await button.action()
	if (!button.keepOpen) {
		notificationManager.removeNotification(item.id)
	}
}

function createNotifText(notif: WebNotification): string {
	return [notif.title, notif.text, notif.errorCode].filter(Boolean).join('\n')
}

function getCopyKey(notif: WebNotification): string {
	return notif.supportData ? `support-${notif.id}` : createNotifText(notif)
}

function checkIntercomPresence(): void {
	isIntercomPresent.value = !!document.querySelector('.intercom-lightweight-app')
}

function copyToClipboard(notif: WebNotification): void {
	// If supportData is present, copy the full JSON for support; otherwise copy plain text
	const text = notif.supportData
		? JSON.stringify(notif.supportData, null, 2)
		: createNotifText(notif)

	const key = getCopyKey(notif)
	copied.value[key] = true
	navigator.clipboard.writeText(text)

	setTimeout(() => {
		const { [key]: _, ...rest } = copied.value
		copied.value = rest
	}, 2000)
}

onMounted(() => {
	checkIntercomPresence()

	const observer = new MutationObserver(() => {
		checkIntercomPresence()
	})

	observer.observe(document.body, {
		childList: true,
		subtree: true,
	})

	onBeforeUnmount(() => {
		observer.disconnect()
	})
})

const { hasModal: hasModalActive } = useModalStack()

withDefaults(
	defineProps<{
		hasSidebar?: boolean
		exitAnimation?: 'slide' | 'fade'
	}>(),
	{
		hasSidebar: false,
		exitAnimation: 'slide',
	},
)
</script>

<style lang="scss" scoped>
.vue-notification-group {
	position: fixed;
	bottom: 1.25rem;
	z-index: 200;
	width: 376px;
	max-width: calc(100vw - 2rem);
	transition: bottom 0.25s cubic-bezier(0.22, 1, 0.36, 1);

	&.location-right {
		right: 1.25rem;
		transition: right 0.25s cubic-bezier(0.22, 1, 0.36, 1);

		&.has-sidebar {
			right: 325px;
		}
	}

	&.location-left {
		left: 1.25rem;
	}

	@media screen and (max-width: 500px) {
		width: calc(100% - 0.75rem * 2);
		bottom: 0.75rem;

		&.location-right {
			right: 0.75rem;
			left: auto;
		}

		&.location-left {
			left: 0.75rem;
			right: auto;
		}
	}

	&.intercom-present {
		bottom: 5rem;
	}

	.vue-notification-wrapper {
		width: 100%;
		margin-bottom: 10px;

		&:last-child {
			margin: 0;
		}
	}

	@media screen and (max-width: 750px) {
		transition: bottom 0.25s cubic-bezier(0.22, 1, 0.36, 1);
		bottom: calc(var(--size-mobile-navbar-height) + 10px) !important;

		&.browse-menu-open {
			bottom: calc(var(--size-mobile-navbar-height-expanded) + 10px) !important;
		}
	}
}

.solara-toast {
	background-color: color-mix(in srgb, var(--surface-3) 92%, transparent);
	border: 1px solid color-mix(in srgb, var(--surface-5) 45%, transparent);
	backdrop-filter: blur(18px);
	-webkit-backdrop-filter: blur(18px);
	box-shadow:
		0 16px 36px -14px rgba(0, 0, 0, 0.72),
		0 4px 12px -4px rgba(0, 0, 0, 0.38);
	transition:
		transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
		border-color 180ms ease;

	&:hover {
		transform: translateY(-1px);
		border-color: color-mix(in srgb, var(--surface-5) 75%, transparent);
	}
}

.solara-toast--success .solara-toast-hairline {
	background: linear-gradient(90deg, transparent 0%, #3dd6f5 30%, #7be8ff 72%, transparent 100%);
}
.solara-toast--info .solara-toast-hairline {
	background: linear-gradient(90deg, transparent 0%, #3dd6f5 32%, #ffb55a 78%, transparent 100%);
}
.solara-toast--warning .solara-toast-hairline {
	background: linear-gradient(90deg, transparent 0%, #ffb55a 35%, #f59e0b 75%, transparent 100%);
}
.solara-toast--error .solara-toast-hairline {
	background: linear-gradient(90deg, transparent 0%, #f43f5e 35%, #fb7185 75%, transparent 100%);
}
.solara-toast--neutral .solara-toast-hairline {
	background: linear-gradient(
		90deg,
		transparent 0%,
		color-mix(in srgb, var(--color-contrast) 22%, transparent) 50%,
		transparent 100%
	);
}

.solara-badge--success {
	background-color: color-mix(in srgb, var(--color-brand) 16%, transparent);
	color: var(--color-brand);
}
.solara-badge--info {
	background-color: color-mix(in srgb, var(--color-brand) 16%, transparent);
	color: var(--color-brand);
}
.solara-badge--warning {
	background-color: color-mix(in srgb, #ffb55a 16%, transparent);
	color: #ffb55a;
}
.solara-badge--error {
	background-color: color-mix(in srgb, #f43f5e 16%, transparent);
	color: #f43f5e;
}
.solara-badge--neutral {
	background-color: var(--surface-4);
	color: var(--color-contrast);
}

.notifs-enter-active,
.notifs-leave-active,
.notifs-move {
	transition:
		opacity 240ms cubic-bezier(0.22, 1, 0.36, 1),
		transform 280ms cubic-bezier(0.22, 1, 0.36, 1);
}

.notifs-enter-from {
	opacity: 0;
	.location-right & {
		transform: translateX(28px) scale(0.96);
	}
	.location-left & {
		transform: translateX(-28px) scale(0.96);
	}
}

.notifs-leave-to {
	opacity: 0;
	.location-right & {
		transform: translateX(36px) scale(0.96);
	}
	.location-left & {
		transform: translateX(-36px) scale(0.96);
	}
}

.exit-fade .notifs-leave-active {
	transition: opacity 220ms ease-out;
}

.exit-fade .notifs-leave-to {
	transform: none;
}

body.floating-action-bar-shown .vue-notification-group {
	bottom: 90px;
}
</style>
