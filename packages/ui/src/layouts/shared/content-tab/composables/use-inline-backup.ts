import { computed, ref, toValue } from 'vue'

import { injectAppBackup } from '#ui/providers'

export function useInlineBackup(_backupName: string | (() => string)) {
	const appBackup = injectAppBackup(null)

	if (appBackup) {
		const isBackingUp = ref(false)
		const backupFailed = ref(false)
		const backupComplete = ref(false)

		return {
			available: computed(() => toValue(appBackup.available) ?? true),
			isServer: false as const,
			isBackingUp,
			isCancelling: ref(false),
			backupFailed,
			backupComplete,
			backupCancelled: ref(false),
			externalBackupInProgress: computed(() => false),
			startBackup: async () => {
				isBackingUp.value = true
				backupFailed.value = false
				backupComplete.value = false
				try {
					await appBackup.createBackup()
					backupComplete.value = true
				} catch {
					backupFailed.value = true
				} finally {
					isBackingUp.value = false
				}
			},
			cancelBackup: async () => {},
		}
	}

	return {
		available: computed(() => false),
		isServer: false as const,
		isBackingUp: ref(false),
		isCancelling: ref(false),
		backupFailed: ref(false),
		backupComplete: ref(false),
		backupCancelled: ref(false),
		externalBackupInProgress: ref(false),
		startBackup: async () => {},
		cancelBackup: async () => {},
	}
}
