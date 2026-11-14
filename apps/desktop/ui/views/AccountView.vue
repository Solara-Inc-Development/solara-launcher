<script setup lang="ts">
import { ref } from 'vue';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import { Check, ExternalLink, Globe, LogIn, LogOut, Shield, Trash2, User } from '../icons';
import {
  native,
  navigate,
  notify,
  openExternal,
  playerAvatarUrl,
  refreshSnapshot,
  refreshSolaraProfile,
  store,
} from '../state';

const emit = defineEmits<{ (e: 'start-signin'): void }>();
const busyId = ref<string | null>(null);
const solaraBusy = ref(false);

async function selectAccount(id: string): Promise<void> {
  busyId.value = id;
  try {
    await native('select_minecraft_account', { id });
    await refreshSnapshot();
    notify('success', 'Active account switched');
  } catch (err) {
    notify('error', "Couldn't switch account", String(err));
  } finally {
    busyId.value = null;
  }
}

async function signOut(id: string, username: string): Promise<void> {
  busyId.value = id;
  try {
    await native('minecraft_sign_out', { id });
    await refreshSnapshot();
    notify('info', `Signed out ${username}`);
  } catch (err) {
    notify('error', "Couldn't sign out", String(err));
  } finally {
    busyId.value = null;
  }
}

async function signInSolara(): Promise<void> {
  if (!store.desktop) return;
  solaraBusy.value = true;
  try {
    await native('solara_sign_in');
    await refreshSolaraProfile();
    notify('success', 'Signed in to Solara Inc');
  } catch (err) {
    notify('error', 'Solara sign-in failed', String(err));
  } finally {
    solaraBusy.value = false;
  }
}

async function signOutSolara(): Promise<void> {
  if (!store.desktop) return;
  solaraBusy.value = true;
  try {
    await native('solara_sign_out');
    await refreshSolaraProfile();
    notify('info', 'Signed out of Solara Inc');
  } catch (err) {
    notify('error', 'Solara sign-out failed', String(err));
  } finally {
    solaraBusy.value = false;
  }
}
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-view-wrap">
      <!-- Microsoft Minecraft Java Accounts -->
      <header class="sol-section-head">
        <div>
          <h1 class="sol-page-title">Minecraft Accounts</h1>
          <p class="sol-muted sol-text-xs">
            Official Microsoft OAuth sign-in stored locally in your OS credential vault.
          </p>
        </div>
        <SolaraButton variant="primary" size="sm" @click="emit('start-signin')">
          <LogIn :size="14" /> Sign in with Microsoft
        </SolaraButton>
      </header>

      <div class="sol-banner-info">
        <Shield :size="15" />
        <span>
          Minecraft access tokens never leave your computer and are stored separately from Solara Inc web identity.
        </span>
      </div>

      <SolaraEmptyState
        v-if="store.accounts.length === 0"
        title="No Minecraft account connected"
        description="Sign in with Microsoft to launch Minecraft: Java Edition and manage skins and capes."
      >
        <template #icon><User :size="26" /></template>
        <template #actions>
          <SolaraButton variant="primary" size="sm" @click="emit('start-signin')">
            <LogIn :size="14" /> Sign in with Microsoft
          </SolaraButton>
        </template>
      </SolaraEmptyState>

      <div v-else class="sol-row-list">
        <div
          v-for="acc in store.accounts"
          :key="acc.id"
          :class="['sol-row-item', 'sol-account-row', { 'is-selected': acc.selected }]"
        >
          <div class="sol-row-gap">
            <img :src="playerAvatarUrl(acc.username)" :alt="acc.username" class="sol-avatar-40" />
            <div>
              <div class="sol-row-gap">
                <strong class="sol-account-name">{{ acc.username }}</strong>
                <span v-if="acc.selected" class="sol-badge sol-badge--success">Active</span>
              </div>
              <div class="sol-muted sol-text-xs">Microsoft · Minecraft: Java Edition</div>
            </div>
          </div>

          <div class="sol-row-gap">
            <SolaraButton
              v-if="!acc.selected"
              variant="secondary"
              size="xs"
              :loading="busyId === acc.id"
              @click="selectAccount(acc.id)"
            >
              <Check :size="12" /> Set Active
            </SolaraButton>
            <SolaraButton
              variant="ghost"
              size="xs"
              @click="navigate('skins')"
            >
              Skins &amp; Capes
            </SolaraButton>
            <SolaraButton
              variant="ghost"
              size="xs"
              :loading="busyId === acc.id"
              @click="signOut(acc.id, acc.username)"
            >
              <Trash2 :size="12" /> Sign Out
            </SolaraButton>
          </div>
        </div>
      </div>

      <!-- Solara Inc Account Identity -->
      <section class="sol-stack" style="margin-top: 20px">
        <div class="sol-section-head">
          <div>
            <h2 class="sol-section-title">Solara Inc Identity (Optional)</h2>
            <p class="sol-muted sol-text-xs">
              Connect your Solara ID via auth.solara-inc.in to manage Solara Hub cloud hosting and network profile.
            </p>
          </div>
          <SolaraButton variant="ghost" size="xs" @click="openExternal('https://app.solara-inc.in')">
            <ExternalLink :size="12" /> app.solara-inc.in
          </SolaraButton>
        </div>

        <div class="sol-row-item">
          <div class="sol-row-gap">
            <div class="sol-network-emblem sol-network-emblem--sm">
              <Globe :size="16" />
            </div>
            <div>
              <strong>
                {{
                  store.solaraProfile.signedIn
                    ? store.solaraProfile.displayName || store.solaraProfile.email || 'Solara Member'
                    : 'Not signed in to Solara Inc'
                }}
              </strong>
              <div class="sol-muted sol-text-xs">
                {{ store.solaraProfile.signedIn ? 'Connected via PKCE SSO' : 'Sign in for cloud nodes and Solara ecosystem tools.' }}
              </div>
            </div>
          </div>

          <div class="sol-row-gap">
            <SolaraButton variant="secondary" size="xs" @click="navigate('solara')">Open Hub</SolaraButton>
            <SolaraButton
              v-if="store.solaraProfile.signedIn"
              variant="ghost"
              size="xs"
              :loading="solaraBusy"
              @click="signOutSolara"
            >
              <LogOut :size="12" /> Sign Out
            </SolaraButton>
            <SolaraButton
              v-else
              variant="primary"
              size="xs"
              :loading="solaraBusy"
              @click="signInSolara"
            >
              <LogIn :size="12" /> Sign in with Solara ID
            </SolaraButton>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
