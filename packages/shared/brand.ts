/** Solara product identity. Content-provider URLs and wire formats are independent. */
export const product = Object.freeze({
	name: 'Solara Launcher',
	shortName: 'Solara',
	website: 'https://app.solara-inc.in',
	source: 'https://github.com/Solara-Inc-Development/solara-launcher',
	support: 'https://github.com/Solara-Inc-Development/solara-launcher/issues',
	icon: '/logo.svg',
})

export function launcherUserAgent(version: string): string {
	return `solara/launcher/${version} (${product.website})`
}

/** Optional catalog accounts are unsupported; Minecraft ownership remains required. */
export const catalogFeatures = Object.freeze({
	accounts: false,
	cloudSettings: false,
	social: false,
})
