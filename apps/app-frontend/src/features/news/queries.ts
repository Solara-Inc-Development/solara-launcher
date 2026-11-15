import type { NewsArticle } from '@modrinth/ui'
import { queryOptions } from '@tanstack/vue-query'

import { API_ORIGIN, buildNumber, isDownloadable } from '../../../../../packages/shared/domain.mjs'
import { product } from '@/config'
import releaseThumbnail from '@modrinth/assets/solara/logo.svg?url'

type ReleaseNews = {
	publicVersion: string
	publishedAt?: string
	verifiedAt: string
	channel?: string }

export const newsKeys = {
	articles: ['news', 'articles'] as const,
}

export function newsArticlesQueryOptions() {
	return queryOptions({
		queryKey: newsKeys.articles,
		queryFn: async ({ signal }): Promise<NewsArticle[]> => {
			const response = await fetch(`${API_ORIGIN}/v1/launcher/bootstrap`, { signal,
				credentials: 'omit',
			})
			if (!response.ok) {
				throw new Error(`Failed to fetch news articles: HTTP ${response.status}`)
			}
			const feed: { releases?: ReleaseNews[] } | null = await response.json()
			return (Array.isArray(feed?.releases) ? feed.releases : [])
				.filter(isDownloadable)
				.sort((a, b) => buildNumber(b.publicVersion) - buildNumber(a.publicVersion))
				.slice(0, 4).map((release) => ({
					path: `${product.source}/releases/tag/${release.publicVersion}`,
					thumbnail: releaseThumbnail,
					title: `${product.name} ${release.publicVersion}`,
					summary:
						release.channel === 'stable'
							? 'Read the release notes.'
							: 'Read the Windows testing preview notes.',
					date: release.publishedAt || release.verifiedAt,
				}))
		},
		staleTime: Infinity,
		retry: false,
		refetchOnMount: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	})
}
