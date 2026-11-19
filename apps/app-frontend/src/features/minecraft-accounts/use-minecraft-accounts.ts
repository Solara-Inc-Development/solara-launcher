import { computed, onUnmounted, ref } from 'vue'

import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import {
	get_default_user,
	login as login_flow,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'
import { getPlayerHeadUrl } from '@/helpers/rendering/player-head'
import type { Skin } from '@/helpers/skins'
import { get_available_skins } from '@/helpers/skins'
import { type AppEvents, injectAppEvents } from '@/providers/app-events'

export type MinecraftCredential = {
	profile: {
		id: string
		name: string
	}
}

const accounts = ref<MinecraftCredential[]>([])
const loginDisabled = ref(false)
const defaultUser = ref<string | undefined>()
const equippedSkin = ref<Skin | null>(null)
const equippedHeadUrl = ref<string | undefined>()
let headRequest = 0
let initialized = false

async function updateHeadUrl(skin: Skin | null) {
	const request = ++headRequest
	if (equippedHeadUrl.value) URL.revokeObjectURL(equippedHeadUrl.value)
	equippedHeadUrl.value = undefined
	if (!skin) return
	try {
		const url = await getPlayerHeadUrl(skin)
		if (request !== headRequest) {
			URL.revokeObjectURL(url)
		} else {
			equippedHeadUrl.value = url
		}
	} catch (e) {
		console.warn('Failed to load player head:', e)
	}
}

async function refreshValues() {
	try {
		defaultUser.value = await get_default_user().catch(() => undefined)
		const userList = await users().catch(() => [])
		accounts.value = Array.isArray(userList) ? [...userList] : []
		accounts.value.sort((a, b) => (a.profile?.name ?? '').localeCompare(b.profile?.name ?? ''))

		const skins = await get_available_skins().catch(() => [])
		equippedSkin.value = skins.find((skin) => skin.is_equipped) ?? null
		await updateHeadUrl(equippedSkin.value)
	} catch (error) {
		console.warn('Failed to refresh Minecraft account values:', error)
	}
}

async function setEquippedSkin(skin: Skin) {
	equippedSkin.value = skin
	await updateHeadUrl(skin)
}

function setLoginDisabled(value: boolean) {
	loginDisabled.value = value
}

async function setAccount(account: MinecraftCredential) {
	defaultUser.value = account.profile.id
	await set_default_user(account.profile.id).catch((e) => console.warn('Failed to set default user:', e))
	await refreshValues()
}

async function login() {
	loginDisabled.value = true
	try {
		const loggedIn = await login_flow().catch(handleSevereError)
		if (loggedIn) {
			await setAccount(loggedIn)
		}
		trackEvent('AccountLogIn')
	} finally {
		loginDisabled.value = false
	}
}

async function logout(id: string) {
	await remove_user(id).catch((e) => console.warn('Failed to remove user:', e))
	await refreshValues()
	if (!selectedAccount.value && accounts.value.length > 0) {
		await setAccount(accounts.value[0])
	}
	trackEvent('AccountLogOut')
}

const selectedAccount = computed(() =>
	accounts.value.find((account) => account.profile.id === defaultUser.value) ?? accounts.value[0] ?? null,
)

const username = computed(() => selectedAccount.value?.profile?.name ?? null)

const avatarUrl = computed(() => {
	if (equippedSkin.value?.texture_key) {
		const cachedUrl = equippedHeadUrl.value
		if (cachedUrl) return cachedUrl
		return `https://mc-heads.net/avatar/${equippedSkin.value.texture_key}/128`
	}
	if (selectedAccount.value?.profile?.id) {
		return `https://mc-heads.net/avatar/${selectedAccount.value.profile.id}/128`
	}
	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
})

function getAccountAvatarUrl(account: MinecraftCredential) {
	if (
		account.profile.id === selectedAccount.value?.profile?.id &&
		equippedSkin.value?.texture_key
	) {
		const cachedUrl = equippedHeadUrl.value
		if (cachedUrl) return cachedUrl
	}
	return `https://mc-heads.net/avatar/${account.profile.id}/128`
}

export function useMinecraftAccounts(customEvents?: AppEvents | null) {
	if (!initialized) {
		initialized = true
		void refreshValues()

		const events = customEvents ?? injectAppEvents(null)
		if (events) {
			events.on('process', async (e) => {
				if (e.event === 'launched') {
					await refreshValues()
				}
			})
		}
	}

	return {
		accounts,
		defaultUser,
		selectedAccount,
		username,
		loginDisabled,
		equippedSkin,
		equippedHeadUrl,
		avatarUrl,
		getAccountAvatarUrl,
		refreshValues,
		setEquippedSkin,
		setLoginDisabled,
		setAccount,
		login,
		logout,
	}
}
