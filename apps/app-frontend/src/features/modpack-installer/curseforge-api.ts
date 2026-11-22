import { invoke } from '@tauri-apps/api/core'

export const SOLARA_CURSEFORGE_API_BASE = 'https://api.solara-inc.in/v1/launcher/curseforge'

export interface CurseForgeCategory {
	id: number
	name: string
	slug: string
}

export interface CurseForgeFile {
	id: number
	modId: number
	displayName: string
	fileName: string
	fileDate: string
	fileLength: number
	releaseType: 'release' | 'beta' | 'alpha'
	downloadUrl: string | null
	restricted: boolean
	gameVersions: string[]
	loaders: string[]
	sha1: string | null
	md5: string | null
}

export interface CurseForgeLinks {
	websiteUrl?: string
	wikiUrl?: string
	issuesUrl?: string
	sourceUrl?: string
}

export interface CurseForgeModpackSummary {
	id: number
	slug: string
	name: string
	summary: string
	author: string
	authors: string[]
	downloads: number
	dateModified: string
	dateCreated: string
	logoUrl: string | null
	bannerUrl?: string | null
	websiteUrl: string
	links?: CurseForgeLinks
	loaders: string[]
	gameVersions: string[]
	categories: CurseForgeCategory[]
	latestFile: CurseForgeFile | null
}

export interface CurseForgeScreenshot {
	title: string
	url: string
	thumbnailUrl: string
}

export interface CurseForgeModpackDetail extends CurseForgeModpackSummary {
	descriptionHtml?: string
	screenshots: CurseForgeScreenshot[]
	files: CurseForgeFile[]
}

export interface CurseForgeSearchResponse {
	configured: boolean
	items: CurseForgeModpackSummary[]
	pagination: {
		index: number
		pageSize: number
		resultCount: number
		totalCount: number
	}
}

export interface CurseForgeDetailResponse {
	configured: boolean
	modpack: CurseForgeModpackDetail
}

export interface CurseForgeInstallRequest {
	modId: number
	fileId: number
	name: string
	downloadUrl: string
	iconUrl?: string | null
	fileName?: string | null
}

export interface CurseForgeRestrictedMod {
	modId: number
	fileId: number
	fileName?: string | null
}

export interface CurseForgeInstallResult {
	instanceId: string
	name: string
	gameVersion: string
	loader: string
	installedMods: number
	restrictedMods: CurseForgeRestrictedMod[]
}

export type CurseForgeSort = 'popularity' | 'downloads' | 'updated' | 'newest' | 'name'

export const CURSEFORGE_CATEGORY_MAP: Record<string, number> = {
	adventure: 4475,
	tech: 4472,
	magic: 4473,
	optimization: 4481,
	combat: 4483,
	multiplayer: 4484,
	quests: 4478,
	'kitchen-sink': 4471,
	lightweight: 4481,
	skyblock: 4736,
	hardcore: 4479,
}

export async function searchCurseForgeModpacks(params: {
	q?: string
	gameVersion?: string
	loader?: string
	categoryId?: number
	sort?: CurseForgeSort
	index?: number
	pageSize?: number
}): Promise<CurseForgeSearchResponse> {
	const search = new URLSearchParams()
	if (params.q?.trim()) search.set('q', params.q.trim())
	if (params.gameVersion) search.set('gameVersion', params.gameVersion)
	if (params.loader) search.set('loader', params.loader)
	if (params.categoryId && params.categoryId > 0) search.set('categoryId', String(params.categoryId))
	if (params.sort) search.set('sort', params.sort)
	if (typeof params.index === 'number' && params.index > 0) search.set('index', String(params.index))
	if (typeof params.pageSize === 'number') search.set('pageSize', String(params.pageSize))

	const url = `${SOLARA_CURSEFORGE_API_BASE}/search${search.toString() ? `?${search.toString()}` : ''}`
	const response = await fetch(url, {
		method: 'GET',
		headers: { Accept: 'application/json' },
	})

	if (!response.ok) {
		const errBody = await response.json().catch(() => null)
		const message =
			errBody?.error?.message || `CurseForge catalog request failed (HTTP ${response.status})`
		throw new Error(message)
	}

	return (await response.json()) as CurseForgeSearchResponse
}

export async function getCurseForgeModpackDetail(modId: number): Promise<CurseForgeDetailResponse> {
	const response = await fetch(`${SOLARA_CURSEFORGE_API_BASE}/modpacks/${modId}`, {
		method: 'GET',
		headers: { Accept: 'application/json' },
	})

	if (!response.ok) {
		const errBody = await response.json().catch(() => null)
		const message =
			errBody?.error?.message || `Failed to load CurseForge modpack details (HTTP ${response.status})`
		throw new Error(message)
	}

	return (await response.json()) as CurseForgeDetailResponse
}

export async function installCurseForgeModpack(
	request: CurseForgeInstallRequest,
): Promise<CurseForgeInstallResult> {
	return await invoke<CurseForgeInstallResult>('plugin:curseforge|curseforge_install_modpack', {
		request,
	})
}
