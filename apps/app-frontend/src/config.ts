const trimTrailingSlash = (url: string) => url.replace(/\/$/, '')

export { catalogFeatures, launcherUserAgent, product } from '../../../packages/shared/brand'

const siteUrl = trimTrailingSlash(import.meta.env.MODRINTH_URL || 'https://modrinth.com')
const labrinthBaseUrl = trimTrailingSlash(
	import.meta.env.MODRINTH_API_BASE_URL || 'https://api.modrinth.com',
)
const archonBaseUrl = trimTrailingSlash(
	import.meta.env.MODRINTH_ARCHON_BASE_URL || 'https://disabled.invalid',
)
const sharedInstancesBaseUrl = trimTrailingSlash(
	import.meta.env.SHARED_INSTANCES_API_BASE_URL || 'https://disabled.invalid',
)

export const config = {
	siteUrl,
	stripePublishableKey: import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY || '',
	labrinthBaseUrl,
	archonBaseUrl,
	sharedInstancesBaseUrl,
}
