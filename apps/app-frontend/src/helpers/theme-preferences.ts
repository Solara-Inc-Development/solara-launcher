/** Read old installs without changing saved theme choices during the product rename. */
export const themePreferenceKeys = Object.freeze({
	theme: 'solara:theme',
	darkTheme: 'solara:preferred-dark-theme',
})

export function readThemePreference<T extends string>(
	storage: Pick<Storage, 'getItem' | 'setItem'>,
	key: string,
	legacyKey: string,
	options: readonly T[],
): T | null {
	try {
		const current = storage.getItem(key)
		if (options.includes(current as T)) return current as T
		const legacy = storage.getItem(legacyKey)
		if (!options.includes(legacy as T)) return null
		try {
			storage.setItem(key, legacy!)
		} catch {
			// Storage can be read-only. The previous preference still works this session.
		}
		return legacy as T
	} catch {
		return null
	}
}
