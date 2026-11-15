<script setup lang="ts">
import { Button, defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

const modal = ref<InstanceType<typeof NewModal>>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	title: {
		id: 'app.catalog-feature-unavailable.title',
		defaultMessage: 'This online feature is unavailable',
	},
	description: {
		id: 'app.catalog-feature-unavailable.description',
		defaultMessage:
			'Catalog accounts and shared online instances are not supported in Solara Launcher. You can browse the catalog, install modpacks, and manage local instances without a catalog account.',
	},
	close: {
		id: 'app.catalog-feature-unavailable.close',
		defaultMessage: 'Got it',
	},
})

async function show(): Promise<boolean> {
	modal.value?.show()
	// Never treat a dismissed information panel as authorization for a remote action.
	return false
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)" max-width="30rem">
		<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>
		<div class="mt-4 flex justify-end">
			<Button color="brand" @click="modal?.hide()">{{ formatMessage(messages.close) }}</Button>
		</div>
	</NewModal>
</template>
