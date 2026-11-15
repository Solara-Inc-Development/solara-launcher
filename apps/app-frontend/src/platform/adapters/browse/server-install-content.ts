import type { Archon, Labrinth } from '@modrinth/api-client'
import {
	type BrowseInstallPlan,
	type BrowseSelectedProject,
	createContext,
	type CreationFlowContextValue,
	type UpdateChannelPolicy,
} from '@modrinth/ui'
import { computed, type ComputedRef, type Ref, ref } from 'vue'
import { useRoute } from 'vue-router'

type ServerFlowFrom = 'onboarding' | 'reset-server'

export interface ServerPlatform {
	loader: string | null
	gameVersion: string | null
}

type InstallableSearchResult = Labrinth.Search.v3.ResultSearchProject & {
	installing?: boolean
	installed?: boolean
}

export interface ServerModpackSelectionRequest {
	contentType?: string
	projectId: string
	versionId: string
	name: string
	iconUrl?: string
}

interface ServerSetupModalHandle {
	show: () => void | Promise<void>
	hide: () => void
	ctx?: CreationFlowContextValue | null
}

export interface ServerInstallContentContext {
	serverIdQuery: ComputedRef<string | null>
	worldIdQuery: ComputedRef<string | null>
	browseFrom: ComputedRef<string | null>
	serverFlowFrom: ComputedRef<ServerFlowFrom | null>
	isFromWorlds: ComputedRef<boolean>
	isServerContext: ComputedRef<boolean>
	isSetupServerContext: ComputedRef<boolean>
	effectiveServerWorldId: ComputedRef<string | null>
	serverContextServerData: Ref<Archon.Servers.v0.Server | null>
	serverContextPlatform: ComputedRef<ServerPlatform>
	serverContentProjectIds: Ref<Set<string>>
	queuedServerInstallRootProjectIds: ComputedRef<Set<string>>
	queuedServerInstallProjectIds: ComputedRef<Set<string>>
	queuedServerInstallCount: ComputedRef<number>
	selectedServerInstallProjects: ComputedRef<BrowseSelectedProject[]>
	isInstallingQueuedServerInstalls: Ref<boolean>
	queuedInstallProgress: Ref<{ completed: number; total: number }>
	serverBackUrl: ComputedRef<string>
	serverBackLabel: ComputedRef<string>
	serverBrowseHeading: ComputedRef<string>
	clearQueuedServerInstalls: () => void
	removeQueuedServerInstall: (projectId: string) => void
	flushQueuedServerInstalls: () => Promise<boolean>
	discardQueuedServerInstallsAndBack: () => Promise<void>
	installQueuedServerInstallsAndBack: () => Promise<boolean>
	initServerContext: () => Promise<void>
	watchServerContextChanges: () => void
	getServerProjectVersions: (projectId: string) => Promise<{ id: string }[]>
	getServerUpdateChannel: () => Promise<UpdateChannelPolicy | undefined>
	enforceServerSetupRoute: (currentProjectType: string | undefined) => void
	getQueuedServerInstallPlans: () => Map<string, BrowseInstallPlan<InstallableSearchResult>>
	setQueuedServerInstallPlans: (
		plans: Map<string, BrowseInstallPlan<InstallableSearchResult>>,
	) => void
	resolveQueuedServerInstallPlan: (
		plan: BrowseInstallPlan<InstallableSearchResult>,
	) => Promise<void>
	activeServerModpackInstallProjectId: Ref<string | null>
	openServerModpackInstallFlow: (request: ServerModpackSelectionRequest) => Promise<void>
	onServerFlowHide: () => void
	onServerFlowBack: () => void
	handleServerModpackFlowCreate: (config: CreationFlowContextValue) => Promise<void>
	markServerProjectInstalled: (id: string) => void
}

export const [injectServerInstallContent, provideServerInstallContent] =
	createContext<ServerInstallContentContext>('Browse', 'serverInstallContent')

function readQueryString(value: unknown): string | null {
	if (Array.isArray(value)) return value[0] ?? null
	return typeof value === 'string' && value.length > 0 ? value : null
}

export function useServerInstallContent(_opts: {
	serverSetupModalRef: Ref<ServerSetupModalHandle | null>
	inheritBreadcrumbServer?: boolean
}): ServerInstallContentContext {
	const route = useRoute()

	const serverIdQuery = computed<string | null>(() => null)
	const worldIdQuery = computed<string | null>(() => null)
	const browseFrom = computed(() => readQueryString(route.query.from))
	const serverFlowFrom = computed<ServerFlowFrom | null>(() => null)
	const isFromWorlds = computed(() => browseFrom.value === 'worlds')
	const isServerContext = computed(() => false)
	const isSetupServerContext = computed(() => false)
	const effectiveServerWorldId = computed<string | null>(() => null)
	const serverContextServerData = ref<Archon.Servers.v0.Server | null>(null)
	const serverContextPlatform = computed<ServerPlatform>(() => ({
		loader: null,
		gameVersion: null,
	}))
	const serverContentProjectIds = ref<Set<string>>(new Set())
	const queuedServerInstalls = ref<Map<string, BrowseInstallPlan<InstallableSearchResult>>>(
		new Map(),
	)
	const queuedServerInstallRootProjectIds = computed(() => new Set<string>())
	const queuedServerInstallProjectIds = computed(() => new Set<string>())
	const queuedServerInstallCount = computed(() => 0)
	const selectedServerInstallProjects = computed<BrowseSelectedProject[]>(() => [])
	const isInstallingQueuedServerInstalls = ref(false)
	const queuedInstallProgress = ref({ completed: 0, total: 0 })
	const serverBackUrl = computed(() => '/')
	const serverBackLabel = computed(() => 'Back')
	const serverBrowseHeading = computed(() => 'Installing content')
	const activeServerModpackInstallProjectId = ref<string | null>(null)

	return {
		serverIdQuery,
		worldIdQuery,
		browseFrom,
		serverFlowFrom,
		isFromWorlds,
		isServerContext,
		isSetupServerContext,
		effectiveServerWorldId,
		serverContextServerData,
		serverContextPlatform,
		serverContentProjectIds,
		queuedServerInstallRootProjectIds,
		queuedServerInstallProjectIds,
		queuedServerInstallCount,
		selectedServerInstallProjects,
		isInstallingQueuedServerInstalls,
		queuedInstallProgress,
		serverBackUrl,
		serverBackLabel,
		serverBrowseHeading,
		clearQueuedServerInstalls: () => {},
		removeQueuedServerInstall: () => {},
		flushQueuedServerInstalls: async () => true,
		discardQueuedServerInstallsAndBack: async () => {},
		installQueuedServerInstallsAndBack: async () => true,
		initServerContext: async () => {},
		watchServerContextChanges: () => {},
		getServerProjectVersions: async () => [],
		getServerUpdateChannel: async () => undefined,
		enforceServerSetupRoute: () => {},
		getQueuedServerInstallPlans: () => queuedServerInstalls.value,
		setQueuedServerInstallPlans: (plans) => {
			queuedServerInstalls.value = plans
		},
		resolveQueuedServerInstallPlan: async () => {},
		activeServerModpackInstallProjectId,
		openServerModpackInstallFlow: async () => {},
		onServerFlowHide: () => {
			activeServerModpackInstallProjectId.value = null
		},
		onServerFlowBack: () => {},
		handleServerModpackFlowCreate: async (config) => {
			config.loading.value = false
		},
		markServerProjectInstalled: () => {},
	}
}
