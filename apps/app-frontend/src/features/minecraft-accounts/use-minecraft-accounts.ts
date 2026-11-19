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
import type { Cape, Skin } from '@/helpers/skins'
import {
	get_available_capes,
	get_available_skins,
	get_normalized_skin_texture,
} from '@/helpers/skins'
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
const equippedSkinTexture = ref<string>('')
const equippedCapeTexture = ref<string | undefined>(undefined)
const availableCapes = ref<Cape[]>([])
const equippedHeadUrl = ref<string | undefined>()
let headRequest = 0
let skinTextureRequest = 0
let initialized = false

async function updateSkinPreviewTexture(skin: Skin | null) {
	const request = ++skinTextureRequest
	if (!skin?.texture) {
		equippedSkinTexture.value = ''
		return
	}
	try {
		const normalized = await get_normalized_skin_texture(skin)
		if (request === skinTextureRequest) {
			equippedSkinTexture.value = normalized || skin.texture
		}
	} catch {
		if (request === skinTextureRequest) {
			equippedSkinTexture.value = skin.texture.startsWith('data:image/') ? skin.texture : ''
		}
	}
}

function resolveEquippedCapeTexture(skin: Skin | null, capes: Cape[]): string | undefined {
	if (skin?.cape_id) {
		const matched = capes.find((c) => c.id === skin.cape_id)
		if (matched?.texture) return matched.texture
	}
	const equipped = capes.find((c) => c.is_equipped)
	return equipped?.texture
}

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

		const [skins, capes] = await Promise.all([
			get_available_skins().catch(() => [] as Skin[]),
			get_available_capes().catch(() => [] as Cape[]),
		])
		const skinList = Array.isArray(skins) ? skins : []
		const capeList = Array.isArray(capes) ? capes : []
		availableCapes.value = capeList
		equippedSkin.value = skinList.find((skin) => skin.is_equipped) ?? skinList[0] ?? null
		equippedCapeTexture.value = resolveEquippedCapeTexture(equippedSkin.value, capeList)
		await Promise.all([
			updateHeadUrl(equippedSkin.value),
			updateSkinPreviewTexture(equippedSkin.value),
		])
	} catch (error) {
		console.warn('Failed to refresh Minecraft account values:', error)
	}
}

async function setEquippedSkin(skin: Skin) {
	equippedSkin.value = skin
	equippedCapeTexture.value = resolveEquippedCapeTexture(skin, availableCapes.value)
	await Promise.all([updateHeadUrl(skin), updateSkinPreviewTexture(skin)])
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
		equippedSkinTexture,
		equippedCapeTexture,
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
