import {
	AuthFeature,
	NodeAuthFeature,
	nodeAuthState,
	PanelVersionFeature,
	TauriModrinthClient,
	VerboseLoggingFeature,
} from '@modrinth/api-client'
import { getVersion } from '@tauri-apps/api/app'

import { config, launcherUserAgent } from '@/config'
import { get as getCreds } from '@/helpers/mr_auth'
import { createLocalFileUploader } from '@/platform/adapters/local-file-upload'

export function createAppClient(): TauriModrinthClient {
	const appVersion = getVersion()

	return new TauriModrinthClient({
		userAgent: async () => launcherUserAgent(await appVersion),
		labrinthBaseUrl: config.labrinthBaseUrl,
		archonBaseUrl: config.archonBaseUrl,
		sharedInstancesBaseUrl: config.sharedInstancesBaseUrl,
		localFileUploader: createLocalFileUploader(),
		features: [
			new NodeAuthFeature({
				getAuth: () => nodeAuthState.getAuth?.() ?? null,
				refreshAuth: async () => {
					if (nodeAuthState.refreshAuth) {
						await nodeAuthState.refreshAuth()
					}
				},
			}),
			new AuthFeature({
				token: async () => (await getCreds())?.session,
			}),
			new PanelVersionFeature(),
			new VerboseLoggingFeature(),
		],
	})
}
