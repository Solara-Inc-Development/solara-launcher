export type ModrinthCredentials = {
	session: string
	expires: string
	user_id: string
	active: boolean
}

export type ModrinthAuthFlow = 'sign-in' | 'sign-up'

export async function login(
	_flow: ModrinthAuthFlow = 'sign-in',
	_addAccount = false,
): Promise<ModrinthCredentials> {
	throw new Error('Catalog account login is unavailable in Solara Launcher.')
}

export async function logout(): Promise<void> {}

export async function get(): Promise<ModrinthCredentials | null> {
	return null
}

export async function getAll(): Promise<ModrinthCredentials[]> {
	return []
}

export async function setActive(_userId: string): Promise<void> {}

export async function removeUser(_userId: string): Promise<void> {}

export async function cancelLogin(): Promise<void> {}
