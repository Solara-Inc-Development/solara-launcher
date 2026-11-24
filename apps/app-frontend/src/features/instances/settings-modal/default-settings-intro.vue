<script setup lang="ts">
import { Settings2Icon } from '@modrinth/assets'
import { defineMessage, useVIntl } from '@modrinth/ui'

import {
	injectSettingsActions,
	type InstanceDefaultsTab,
} from '@/features/settings/settings-actions'

import { injectInstanceSettings } from './instance-settings-context'

const props = defineProps<{ description: string; tab: InstanceDefaultsTab }>()

const { formatMessage } = useVIntl()
const { closeModal } = injectInstanceSettings()
const { openInstanceDefaults } = injectSettingsActions()
const label = defineMessage({
	id: 'instance.settings.defaults.open-app-settings',
	defaultMessage: 'Manage default settings',
})

function showDefaultSettings() {
	const openSettings = () => openInstanceDefaults(props.tab)
	if (closeModal) {
		closeModal(openSettings)
	} else {
		openSettings()
	}
}
</script>

<template>
	<div class="flex flex-wrap items-center justify-between gap-3 rounded-2xl bg-brand/10 px-4 py-3">
		<p class="m-0 min-w-0 flex-1 text-xs leading-relaxed text-secondary">{{ description }}</p>
		<button
			type="button"
			class="inline-flex shrink-0 cursor-pointer items-center gap-1.5 rounded-xl border-0 bg-brand/15 px-3 py-1.5 text-xs font-bold text-brand transition-all hover:brightness-110"
			@click="showDefaultSettings"
		>
			<Settings2Icon class="size-3.5" />
			<span>{{ formatMessage(label) }}</span>
		</button>
	</div>
</template>
