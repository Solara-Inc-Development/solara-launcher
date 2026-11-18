<template>
	<div
		v-if="accounts.length === 0"
		class="flex flex-col gap-3 bg-button-bg border border-solid border-surface-5 rounded-xl p-3 mt-2"
	>
		<span>{{ formatMessage(messages.notSignedIn) }}</span>
		<Button type="colored" color="brand" :disabled="loginDisabled" @click="login()">
			<LogInIcon v-if="!loginDisabled" />
			<SpinnerIcon v-else class="animate-spin" />
			{{ formatMessage(messages.signInToMinecraft) }}
		</Button>
	</div>
	<Accordion
		v-else
		class="w-full mt-2 bg-button-bg border border-solid border-surface-5 rounded-xl overflow-clip"
		button-class="button-base w-full bg-transparent px-3 py-2 border-0 cursor-pointer"
		:open-by-default="false"
	>
		<template #title>
			<div class="flex gap-2 w-full min-w-0">
				<Avatar
					size="36px"
					:src="
						selectedAccount
							? avatarUrl
							: 'https://launcher-files.modrinth.com/assets/steve_head.png'
					"
				/>
				<div class="flex flex-col items-start w-full min-w-0">
					<span class="truncate w-full text-left">{{
						selectedAccount ? selectedAccount.profile.name : formatMessage(messages.selectAccount)
					}}</span>
					<span class="text-secondary text-xs">{{ formatMessage(messages.minecraftAccount) }}</span>
				</div>
			</div>
		</template>
		<div class="bg-button-bg pt-1 pb-2 border-0 border-t border-solid border-surface-5">
			<template v-if="accounts.length > 0">
				<div v-for="account in accounts" :key="account.profile.id" class="flex gap-1 items-center">
					<button
						class="flex items-center flex-shrink flex-grow overflow-clip gap-2 p-2 border-0 bg-transparent cursor-pointer button-base min-w-0"
						@click="setAccount(account)"
					>
						<RadioButtonCheckedIcon
							v-if="selectedAccount && selectedAccount.profile.id === account.profile.id"
							class="w-5 h-5 text-brand shrink-0"
						/>
						<RadioButtonIcon v-else class="w-5 h-5 text-secondary shrink-0" />
						<Avatar :src="getAccountAvatarUrl(account)" size="24px" />
						<p
							class="m-0 truncate min-w-0"
							:class="
								selectedAccount && selectedAccount.profile.id === account.profile.id
									? 'text-contrast font-semibold'
									: 'text-primary'
							"
						>
							{{ account.profile.name }}
						</p>
					</button>
					<IconButton
						v-tooltip="formatMessage(messages.removeAccount)"
						type="quiet"
						color="red"
						:label="formatMessage(messages.removeAccount)"
						class="mr-2 !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)] hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
						@click="logout(account.profile.id)"
					>
						<TrashIcon />
					</IconButton>
				</div>
			</template>
			<div class="flex flex-col gap-2 px-2 pt-2">
				<Button
					v-if="accounts.length > 0"
					class="w-full !bg-button-bg !text-primary ![box-shadow:var(--shadow-button)]"
					:disabled="loginDisabled"
					@click="login()"
				>
					<PlusIcon />
					{{ formatMessage(messages.addAccount) }}
				</Button>
			</div>
		</div>
	</Accordion>
</template>

<script setup lang="ts">
import {
	LogInIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Accordion,
	Avatar,
	Button,
	defineMessages,
	IconButton,
	useVIntl,
} from '@modrinth/ui'

import { useMinecraftAccounts } from './use-minecraft-accounts'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	change: []
}>()

const mc = useMinecraftAccounts()
const {
	accounts,
	loginDisabled,
	selectedAccount,
	avatarUrl,
	getAccountAvatarUrl,
	refreshValues,
	setEquippedSkin,
	setLoginDisabled,
} = mc

async function setAccount(account: any) {
	await mc.setAccount(account)
	emit('change')
}

async function login() {
	await mc.login()
	emit('change')
}

async function logout(id: string) {
	await mc.logout(id)
	emit('change')
}

defineExpose({
	refreshValues,
	setEquippedSkin,
	setLoginDisabled,
	login,
	loginDisabled,
})

const messages = defineMessages({
	notSignedIn: {
		id: 'minecraft-account.not-signed-in',
		defaultMessage: 'Not signed in',
	},
	addAccount: {
		id: 'minecraft-account.add-account',
		defaultMessage: 'Add account',
	},
	removeAccount: {
		id: 'minecraft-account.remove-account',
		defaultMessage: 'Remove account',
	},
	selectAccount: {
		id: 'minecraft-account.select-account',
		defaultMessage: 'Select account',
	},
	minecraftAccount: {
		id: 'minecraft-account.label',
		defaultMessage: 'Minecraft account',
	},
	signInToMinecraft: {
		id: 'minecraft-account.sign-in',
		defaultMessage: 'Sign in to Minecraft',
	},
})
</script>
