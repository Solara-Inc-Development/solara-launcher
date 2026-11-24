<template>
	<div
		class="popup-notification-group"
		:class="{
			'has-sidebar': hasSidebar,
			'has-modal': hasModalActive,
		}"
		:style="notificationGroupStyle"
	>
		<transition-group name="popup-notifs">
			<div
				v-for="item in notifications"
				:key="item.id"
				class="popup-notification-wrapper"
				@mouseenter="stopTimer(item)"
				@mouseleave="setNotificationTimer(item)"
			>
				<NotificationToast
					v-if="item.contentType === 'toast'"
					class="min-w-full"
					:type="item.type"
					:action-loading="toastActionLoading(item.id)"
					:actor-name="item.actorName"
					:actor-avatar-url="item.actorAvatarUrl"
					:entity-name="item.entityName"
					:entity-icon-url="item.entityIconUrl"
					:status-text="item.statusText"
					:progress="item.progress"
					:waiting="item.waiting"
					:show-progress="item.showProgress"
					:progress-type="item.progressType"
					:progress-current="item.progressCurrent"
					:progress-total="item.progressTotal"
					@accept="handleToastAccept(item, item.onAccept)"
					@review="handleToastReview(item)"
					@decline="handleToastAction(item, item.onDecline)"
					@dismiss="handleToastAction(item, item.onDismiss)"
					@launch="handleToastAction(item, item.onLaunch)"
					@open-actor="item.onOpenActor?.()"
					@open-instance="handleToastAction(item, item.onOpenInstance)"
					@delete="handleToastAction(item, item.onDelete)"
				/>
				<TransitionGroup
					v-else-if="isDownloadNotification(item)"
					name="popup-downloads"
					tag="div"
					class="flex flex-col gap-3"
				>
					<div v-for="progressItem in downloadToastItems(item)" :key="progressItem.id">
						<NotificationToast
							class="min-w-full"
							type="instance-download"
							:entity-name="progressItem.title || item.title"
							:entity-icon-url="progressItem.iconUrl ?? item.iconUrl ?? MinecraftServerIcon"
							:status-text="progressItem.text"
							:progress="progressItem.progress"
							:waiting="progressItem.waiting"
							:show-progress="progressItem.showProgress"
							:wrap-text="progressItem.wrapText"
							:progress-type="progressItem.progressType"
							:progress-current="progressItem.progressCurrent"
							:progress-total="progressItem.progressTotal"
							:actions="progressItem.buttons"
							:dismissible="progressItem.dismissible"
							@dismiss="handleProgressItemDismiss(item, progressItem)"
							@action="(index) => handleProgressItemAction(item, progressItem, index)"
						/>
					</div>
				</TransitionGroup>
				<component
					:is="item.component"
					v-else-if="item.contentType === 'custom'"
					v-bind="item.componentProps"
					@dismiss="popupNotificationManager.removeNotification(item.id)"
				/>
				<div
					v-else
					class="solara-popup-card relative flex w-full flex-col overflow-hidden rounded-2xl"
				>
					<div class="solara-popup-hairline pointer-events-none h-[2px] w-full shrink-0" />
					<div class="flex flex-col gap-3 p-3.5">
						<div class="flex flex-col gap-1.5 w-full">
							<div class="flex items-center justify-between gap-2.5">
								<div class="flex min-w-0 flex-1 items-center gap-2.5">
									<component
										:is="item.titleLogo"
										v-if="item.titleLogo"
										class="h-6 w-auto min-w-0 max-w-full text-contrast"
									/>
									<template v-else>
										<div
											v-if="!item.hideIcon"
											class="flex size-8 shrink-0 items-center justify-center rounded-xl"
											:class="{
												'bg-red/15 text-red': item.type === 'error',
												'bg-orange/15 text-orange': item.type === 'warning',
												'bg-brand/15 text-brand':
													!item.type ||
													item.type === 'download' ||
													item.type === 'success' ||
													!['error', 'warning'].includes(item.type),
											}"
										>
											<IssuesIcon v-if="item.type === 'warning'" class="size-4" />
											<DownloadIcon v-else-if="item.type === 'download'" class="size-4" />
											<CheckCircleIcon v-else-if="item.type === 'success'" class="size-4" />
											<XCircleIcon v-else-if="item.type === 'error'" class="size-4" />
											<InfoIcon v-else class="size-4" />
										</div>
										<div class="text-[13.5px] font-bold text-contrast m-0 grow truncate">
											{{ item.title }}
										</div>
									</template>
								</div>
								<button
									v-if="item.dismissible !== false"
									type="button"
									:aria-label="formatMessage(messages.close)"
									class="flex size-6 shrink-0 cursor-pointer items-center justify-center rounded-lg border-0 bg-transparent p-0 text-secondary/75 transition-all hover:bg-surface-4 hover:text-contrast"
									@click="handleStandardNotificationDismiss(item)"
								>
									<XIcon class="size-3.5" />
								</button>
							</div>
							<span v-if="item.text" class="text-[12.5px] leading-relaxed text-secondary">
								{{ item.text }}
							</span>
						</div>
						<div v-if="item.progressItems?.length" class="flex flex-col gap-2.5">
							<div
								v-for="progressItem in item.progressItems"
								:key="progressItem.id"
								class="flex flex-col gap-1.5"
							>
								<div class="text-xs font-semibold text-contrast truncate">
									{{ progressItem.title }}
								</div>
								<ProgressBar
									:progress="progressItem.progress"
									:max="1"
									:waiting="progressItem.waiting"
									:color="progressColorForType(item.type)"
									:gradient-border="false"
									full-width
								/>
								<div v-if="progressItem.text" class="text-xs text-secondary truncate">
									{{ progressItem.text }}
								</div>
							</div>
						</div>
						<ProgressBar
							v-else-if="item.progress != null || item.waiting"
							:progress="item.progress ?? 0"
							:max="1"
							:waiting="item.waiting ?? false"
							:color="progressColorForType(item.type)"
							:gradient-border="false"
							full-width
						/>
						<div v-if="item.buttons?.length" class="flex flex-wrap gap-1.5">
							<Button
								v-for="(btn, idx) in item.buttons"
								:key="idx"
								size="sm"
								class="!rounded-xl"
								:type="notificationButtonColor(btn, idx) ? 'colored' : 'base'"
								:color="notificationButtonColor(btn, idx)"
								@click="handleButtonClick(item.id, btn)"
							>
								<component :is="btn.icon" v-if="btn.icon" />
								{{ btn.label }}
							</Button>
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
	DownloadIcon,
	InfoIcon,
	IssuesIcon,
	MinecraftServerIcon,
	XCircleIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, ref } from 'vue'

import { Button, type ButtonColor } from '#ui/components/base/buttons'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { useModalStack } from '../../composables/modal-stack'
import {
	injectPopupNotificationManager,
	type PopupNotification,
	type PopupNotificationButton,
	type PopupNotificationProgressItem,
	type PopupNotificationStandard,
	type PopupNotificationToast,
} from '../../providers'
import ProgressBar from '../base/ProgressBar.vue'
import NotificationToast from '../notifications/NotificationToast.vue'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	close: { id: 'notifications.close', defaultMessage: 'Close' },
})

const popupNotificationManager = injectPopupNotificationManager()
const notifications = computed<PopupNotification[]>(() =>
	popupNotificationManager.getVisibleNotifications(),
)
const { stackCount } = useModalStack()
const hasModalActive = computed(() => stackCount.value > 0)
const notificationGroupStyle = computed(() => ({
	zIndex: hasModalActive.value ? 100 + stackCount.value * 10 + 8 : 200,
}))
const activeToastActions = ref<Record<string, 'accept' | 'review'>>({})

const stopTimer = (n: PopupNotification) => popupNotificationManager.stopNotificationTimer(n)
const setNotificationTimer = (n: PopupNotification) =>
	popupNotificationManager.setNotificationTimer(n)
const toastActionLoading = (id: string | number) => activeToastActions.value[String(id)] ?? null

function notificationButtonColor(
	button: PopupNotificationButton,
	index: number,
): ButtonColor | undefined {
	const color = button.color ?? (index === 0 ? 'brand' : undefined)
	return color === 'standard' ? undefined : color
}

function isDownloadNotification(
	item: PopupNotification,
): item is PopupNotificationStandard & { type: 'download' } {
	return !!(
		item.contentType === 'standard' &&
		item.type === 'download' &&
		(!!item.progressItems?.length || item.progress != null || item.waiting)
	)
}

function downloadToastItems(item: PopupNotificationStandard): PopupNotificationProgressItem[] {
	if (item.progressItems?.length) {
		return popupNotificationManager.getVisibleDownloadProgressItems(item)
	}

	return [
		{
			id: `${item.id}`,
			title: item.title,
			text: item.text,
			iconUrl: item.iconUrl,
			progress: item.progress ?? 0,
			waiting: item.waiting ?? false,
			showProgress: true,
			progressType: 'percentage',
		},
	]
}

function handleProgressItemDismiss(
	item: PopupNotification,
	progressItem: PopupNotificationProgressItem,
) {
	popupNotificationManager.hideDownloadItem(item.id, downloadProgressItemId(item, progressItem))
}

async function handleProgressItemAction(
	item: PopupNotification,
	progressItem: PopupNotificationProgressItem,
	index: number,
) {
	const button = progressItem.buttons?.[index]
	if (button) {
		await handleProgressItemButtonClick(item, progressItem, button)
	}
}

async function handleProgressItemButtonClick(
	item: PopupNotification,
	progressItem: PopupNotificationProgressItem,
	btn: PopupNotificationButton,
) {
	await btn.action()
	if (!btn.keepOpen) {
		popupNotificationManager.hideDownloadItem(item.id, downloadProgressItemId(item, progressItem))
	}
}

function downloadProgressItemId(
	item: PopupNotification,
	progressItem: PopupNotificationProgressItem,
): string | undefined {
	return item.contentType === 'standard' && item.progressItems?.length ? progressItem.id : undefined
}

async function handleButtonClick(id: string | number, btn: PopupNotificationButton) {
	await btn.action()
	if (!btn.keepOpen) {
		popupNotificationManager.removeNotification(id)
	}
}

async function handleStandardNotificationDismiss(item: PopupNotificationStandard) {
	await item.onDismiss?.()
	popupNotificationManager.removeNotification(item.id)
}

async function handleToastAction(item: PopupNotification, action?: () => void | Promise<void>) {
	popupNotificationManager.removeNotification(item.id)
	await action?.()
}

async function handleToastReview(item: PopupNotificationToast) {
	if (toastActionLoading(item.id) != null || !item.onReview) return
	const actionId = String(item.id)
	popupNotificationManager.stopNotificationTimer(item)
	activeToastActions.value = { ...activeToastActions.value, [actionId]: 'review' }
	try {
		await item.onReview()
	} finally {
		activeToastActions.value = Object.fromEntries(
			Object.entries(activeToastActions.value).filter(([key]) => key !== actionId),
		)
		popupNotificationManager.setNotificationTimer(item)
	}
}

async function handleToastAccept(item: PopupNotification, action?: () => void | Promise<void>) {
	if (toastActionLoading(item.id) != null) return

	const actionId = String(item.id)
	popupNotificationManager.stopNotificationTimer(item)
	activeToastActions.value = {
		...activeToastActions.value,
		[actionId]: 'accept',
	}

	try {
		await action?.()
	} finally {
		activeToastActions.value = Object.fromEntries(
			Object.entries(activeToastActions.value).filter(([key]) => key !== actionId),
		)
		popupNotificationManager.removeNotification(item.id)
	}
}

function progressColorForType(type: PopupNotificationStandard['type']) {
	if (type === 'error') {
		return 'red'
	} else if (type === 'warning') {
		return 'orange'
	} else if (type === 'download' || type === 'success' || type === 'info') {
		return 'brand'
	}
	return 'brand'
}

withDefaults(
	defineProps<{
		hasSidebar?: boolean
	}>(),
	{
		hasSidebar: false,
	},
)
</script>

<style scoped>
.popup-notification-group {
	position: fixed;
	top: calc(var(--top-bar-height, 3rem) + 1.25rem);
	right: 1.25rem;
	z-index: 200;
	width: min(376px, calc(100vw - 1.5rem));
	min-width: min(376px, calc(100vw - 1.5rem));
	max-width: min(376px, calc(100vw - 1.5rem));
	display: flex;
	flex-direction: column;
	gap: 0.625rem;
	transition:
		opacity 0.2s cubic-bezier(0.22, 1, 0.36, 1),
		transform 0.2s cubic-bezier(0.22, 1, 0.36, 1);
}

.solara-popup-card {
	background-color: color-mix(in srgb, var(--surface-3) 92%, transparent);
	border: 1px solid color-mix(in srgb, var(--surface-5) 45%, transparent);
	backdrop-filter: blur(18px);
	-webkit-backdrop-filter: blur(18px);
	box-shadow:
		0 16px 36px -14px rgba(0, 0, 0, 0.72),
		0 4px 12px -4px rgba(0, 0, 0, 0.38);
}

.solara-popup-hairline {
	background: linear-gradient(90deg, transparent 0%, #3dd6f5 32%, #ffb55a 78%, transparent 100%);
}

.popup-notification-group.has-sidebar {
	right: calc(var(--right-bar-width, 0px) + 1.25rem);
}

.popup-notification-group.has-modal {
	opacity: 0;
	pointer-events: none;
	transform: translateY(-0.5rem);
}

@media screen and (max-width: 500px) {
	.popup-notification-group {
		right: 0.75rem;
	}
}

.popup-notification-group .popup-notification-wrapper {
	width: 100%;
}

.popup-notifs-enter-active,
.popup-notifs-leave-active,
.popup-notifs-move {
	transition:
		opacity 240ms cubic-bezier(0.22, 1, 0.36, 1),
		transform 280ms cubic-bezier(0.22, 1, 0.36, 1);
}

.popup-notifs-enter-from {
	opacity: 0;
	transform: translateX(28px) scale(0.96);
}

.popup-notifs-leave-to {
	opacity: 0;
	transform: translateX(36px) scale(0.96);
}

.popup-downloads-move {
	transition: transform 260ms cubic-bezier(0.22, 1, 0.36, 1);
}

.popup-downloads-leave-active {
	transition:
		opacity 240ms cubic-bezier(0.22, 1, 0.36, 1),
		transform 260ms cubic-bezier(0.22, 1, 0.36, 1);
}

.popup-downloads-leave-to {
	opacity: 0;
	transform: translateX(36px);
}
</style>
