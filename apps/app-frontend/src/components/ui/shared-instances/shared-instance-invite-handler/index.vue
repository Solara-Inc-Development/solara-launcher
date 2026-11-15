<template>
	<CatalogFeatureUnavailableModal ref="accountRequiredModal" />
	<SharedInstanceInstallModal ref="installModal" />
	<SharedInstanceAlreadyInstalledModal
		ref="alreadyInstalledModal"
		@cancel="handleAlreadyInstalledCancel"
		@go-to-instance="handleAlreadyInstalledGoToInstance"
		@install-anyway="handleAlreadyInstalledInstallAnyway"
	/>
</template>

<script setup lang="ts">
import { ref } from 'vue'

import CatalogFeatureUnavailableModal from '@/components/ui/modal/CatalogFeatureUnavailableModal.vue'
import SharedInstanceInstallModal from '@/components/ui/shared-instances/shared-instance-install-modal/index.vue'
import SharedInstanceAlreadyInstalledModal from '@/components/ui/shared-instances/SharedInstanceAlreadyInstalledModal.vue'

import type { SharedInstanceInviteHandler } from './shared-instance-invite-types'
import { useSharedInstanceInviteHandler } from './use-shared-instance-invite-handler'

const installModal = ref<InstanceType<typeof SharedInstanceInstallModal>>()
const alreadyInstalledModal = ref<InstanceType<typeof SharedInstanceAlreadyInstalledModal>>()
const accountRequiredModal = ref<InstanceType<typeof CatalogFeatureUnavailableModal>>()
const {
	handleNotification,
	installFromInviteId,
	clearNotifications,
	handleAlreadyInstalledCancel,
	handleAlreadyInstalledGoToInstance,
	handleAlreadyInstalledInstallAnyway,
} = useSharedInstanceInviteHandler(installModal, alreadyInstalledModal, accountRequiredModal)

defineExpose<SharedInstanceInviteHandler>({
	handleNotification,
	installFromInviteId,
	clearNotifications,
})
</script>
