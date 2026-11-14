<script setup lang="ts">
import { ref } from 'vue';
import { User, Shield, Trash2, Check, Plus } from '../icons';
import {
  store,
  playerAvatarUrl,
  refreshSnapshot,
  native,
  notify,
} from '../state';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';

const emit = defineEmits<{ (e: 'start-signin'): void }>();
const busyId = ref('');

async function selectAccount(id: string) {
  busyId.value = id;
  try {
    await native('select_minecraft_account', { id });
    await refreshSnapshot();
    notify('success', 'Active account switched');
  } catch (err) {
    notify('error', "Couldn't switch account", String(err));
  } finally {
    busyId.value = '';
  }
}

async function signOut(id: string, username: string) {
  busyId.value = id;
  try {
    await native('minecraft_sign_out', { id });
    await refreshSnapshot();
    notify('info', `Signed out of ${username}`);
  } catch (err) {
    notify('error', "Couldn't sign out", String(err));
  } finally {
    busyId.value = '';
  }
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-page-inner sol-account-inner">
      <header class="sol-page-header">
        <div>
          <h1 class="sol-page-title">Minecraft Accounts</h1>
          <p class="sol-page-sub">Manage verified Microsoft accounts owning Minecraft: Java Edition</p>
        </div>
        <SolaraButton variant="primary" size="sm" @click="emit('start-signin')">
          <Plus :size="14" /> Add Microsoft Account
        </SolaraButton>
      </header>

      <div class="sol-banner sol-banner--info" style="margin-bottom: 16px">
        <Shield :size="16" />
        <span>
          Solara signs in via your system browser using Microsoft OAuth PKCE. Account tokens remain in your OS Credential Vault and never touch web storage.
        </span>
      </div>

      <SolaraEmptyState
        v-if="store.accounts.length === 0"
        title="No Microsoft account connected"
        description="Sign in with Microsoft to verify Minecraft Java Edition ownership, launch instances, and customize your skin or capes."
      >
        <template #icon><User :size="26" /></template>
        <template #actions>
          <SolaraButton variant="primary" @click="emit('start-signin')">
            Sign in with Microsoft
          </SolaraButton>
        </template>
      </SolaraEmptyState>

      <div v-else class="sol-stack">
        <article
          v-for="acc in store.accounts"
          :key="acc.id"
          :class="['sol-card', 'sol-account-row', { 'is-selected': acc.selected }]"
        >
          <div class="sol-row-gap">
            <img :src="playerAvatarUrl(acc.username)" :alt="acc.username" class="sol-avatar-44" />
            <div>
              <div class="sol-row-gap">
                <strong class="sol-account-name">{{ acc.username }}</strong>
                <span v-if="acc.selected" class="sol-badge sol-badge--success">Active Account</span>
              </div>
              <div class="sol-muted sol-text-xs">Minecraft: Java Edition Ownership Verified</div>
            </div>
          </div>

          <div class="sol-row-gap">
            <SolaraButton
              v-if="!acc.selected"
              variant="secondary"
              size="sm"
              :loading="busyId === acc.id"
              @click="selectAccount(acc.id)"
            >
              <Check :size="13" /> Set Active
            </SolaraButton>
            <SolaraButton
              variant="danger"
              size="sm"
              :loading="busyId === acc.id"
              @click="signOut(acc.id, acc.username)"
            >
              <Trash2 :size="13" /> Sign Out
            </SolaraButton>
          </div>
        </article>
      </div>
    </div>
  </div>
</template>
