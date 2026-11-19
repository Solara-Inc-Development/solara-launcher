import type { NewsArticle } from '@modrinth/ui'
import { queryOptions } from '@tanstack/vue-query'

import { API_ORIGIN, buildNumber, isDownloadable } from '../../../../../packages/shared/domain.mjs'
import { product } from '@/config'
import releaseThumbnail from '@modrinth/assets/solara/logo.svg?url'

export type ReleaseNews = {
	publicVersion: string
	internalVersion?: string
	publishedAt?: string
	verifiedAt: string
	channel?: string
	notes?: string
}

export type SolaraNewsItem = NewsArticle & {
	id: string
	publicVersion: string
	channel: string
	notes: string
	bullets: string[]
}

export const newsKeys = {
	articles: ['news', 'articles'] as const,
	bootstrap: ['news', 'bootstrap-releases'] as const,
}

function cleanMarkdownLine(line: string): string {
	return line
		.replace(/^#{1,6}\s+/, '')
		.replace(/^[-*]\s+/, '')
		.replace(/\*\*(.*?)\*\*/g, '$1')
		.replace(/`([^`]+)`/g, '$1')
		.trim()
}

function extractHeadlineAndSummary(release: ReleaseNews): {
	headline: string
	summary: string
	bullets: string[]
} {
	const rawNotes = (release.notes ?? '').trim()
	if (!rawNotes) {
		return {
			headline: `${product.name} ${release.publicVersion}`,
			summary:
				release.channel === 'stable'
					? 'Read the release notes and update highlights.'
					: 'Read the Windows testing preview notes and changes.',
			bullets: [],
		}
	}

	const lines = rawNotes
		.split('\n')
		.map((l) => l.trim())
		.filter((l) => l.length > 0 && !l.startsWith('Updater-signed Windows testing preview'))

	let headline = `${product.name} ${release.publicVersion}`
	const bullets: string[] = []
	for (const line of lines) {
		if (line.startsWith('# Changelog')) continue
		if (line.startsWith('- ') || line.startsWith('* ')) {
			const cleaned = cleanMarkdownLine(line)
			if (cleaned) bullets.push(cleaned)
			continue
		}
		if (headline === `${product.name} ${release.publicVersion}` && !line.startsWith('## ')) {
			const cleaned = cleanMarkdownLine(line)
			if (cleaned.length >= 8 && cleaned.length <= 90) {
				headline = cleaned
			}
		} else if (headline === `${product.name} ${release.publicVersion}` && line.startsWith('## ')) {
			const cleaned = cleanMarkdownLine(line)
			if (cleaned.length >= 6 && cleaned.length <= 90) {
				headline = cleaned
			}
		}
	}

	const summary =
		bullets[0] ||
		(release.channel === 'stable'
			? 'Read the release notes.'
			: 'Read the Windows testing preview notes.')

	return { headline, summary, bullets }
}

async function fetchBootstrapReleases(signal?: AbortSignal): Promise<ReleaseNews[]> {
	const response = await fetch(`${API_ORIGIN}/v1/launcher/bootstrap`, {
		signal,
		credentials: 'omit',
	})
	if (!response.ok) {
		throw new Error(`Failed to fetch Solara bootstrap feed: HTTP ${response.status}`)
	}
	const feed: { releases?: ReleaseNews[] } | null = await response.json()
	return (Array.isArray(feed?.releases) ? feed.releases : [])
		.filter(isDownloadable)
		.sort((a, b) => buildNumber(b.publicVersion) - buildNumber(a.publicVersion))
		.slice(0, 30)
}

export function bootstrapReleasesQueryOptions() {
	return queryOptions({
		queryKey: newsKeys.bootstrap,
		queryFn: async ({ signal }): Promise<ReleaseNews[]> => fetchBootstrapReleases(signal),
		staleTime: 5 * 60 * 1000,
		retry: false,
		refetchOnMount: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	})
}

export function newsArticlesQueryOptions() {
	return queryOptions({
		queryKey: newsKeys.articles,
		queryFn: async ({ signal }): Promise<SolaraNewsItem[]> => {
			const releases = await fetchBootstrapReleases(signal)
			return releases.slice(0, 30).map((release) => {
				const { headline, summary, bullets } = extractHeadlineAndSummary(release)
				return {
					id: release.publicVersion,
					publicVersion: release.publicVersion,
					channel: release.channel === 'stable' ? 'Stable' : 'Testing Preview',
					path: `${product.source}/releases/tag/${release.publicVersion}`,
					thumbnail: releaseThumbnail,
					title:
						headline !== `${product.name} ${release.publicVersion}`
							? headline
							: `${product.name} ${release.publicVersion}`,
					summary,
					date: release.publishedAt || release.verifiedAt,
					notes: (release.notes ?? '').trim() || summary,
					bullets,
				}
			})
		},
		staleTime: 5 * 60 * 1000,
		retry: false,
		refetchOnMount: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	})
}
