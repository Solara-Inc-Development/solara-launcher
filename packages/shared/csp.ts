type ContentSecurityPolicy = string | Record<string, string | string[]>

/** Keep Tauri's policy intact and add only the local Vite websocket. */
export function developmentCsp(policy: ContentSecurityPolicy, websocket: string): string {
	const directives = new Map<string, string[]>()
	if (typeof policy === 'string') {
		for (const directive of policy.split(';')) {
			const [name, ...sources] = directive.trim().split(/\s+/)
			if (name) directives.set(name, sources)
		}
	} else {
		for (const [name, sources] of Object.entries(policy)) {
			directives.set(name, Array.isArray(sources) ? [...sources] : sources.split(/\s+/))
		}
	}
	const connectionSources = directives.get('connect-src') ?? directives.get('default-src') ?? []
	directives.set('connect-src', [
		...new Set([...connectionSources.filter((s) => s !== "'none'"), websocket]),
	])
	return [...directives].map(([name, sources]) => `${name} ${sources.join(' ')}`).join('; ')
}
