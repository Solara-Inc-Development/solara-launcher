<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import SolaraButton from '../components/SolaraButton.vue';
import SolaraEmptyState from '../components/SolaraEmptyState.vue';
import SolaraModal from '../components/SolaraModal.vue';
import {
  Check,
  Copy,
  ExternalLink,
  Globe,
  LogIn,
  LogOut,
  Play,
  Plus,
  RefreshCw,
  Server,
  ShieldCheck,
  Square,
  Terminal,
  Trash2,
} from '../icons';
import {
  loadMinecraftVersions,
  native,
  notify,
  openExternal,
  refreshSnapshot,
  refreshSolaraBootstrap,
  refreshSolaraProfile,
  store,
} from '../state';
import type { LocalServerRecord } from '../types';

const signingInSolara = ref(false);
const copiedIp = ref(false);
const remoteServers = ref<Array<{ id?: string; name?: string; status?: string; address?: string; plan?: string }>>([]);
const loadingRemoteServers = ref(false);
const remoteServersError = ref<string | null>(null);

// Create Local Server dialog
const showCreateServer = ref(false);
const srvName = ref('Solara Local Realm');
const srvVersion = ref('1.21.4');
const srvPort = ref(25565);
const srvMemory = ref(2048);
const srvEula = ref(false);
const creatingServer = ref(false);

// Local Server Console Modal
const activeConsoleServer = ref<LocalServerRecord | null>(null);
const consoleLogs = ref('');
const consoleCommand = ref('');
const loadingConsole = ref(false);

const networkInfo = computed(() => {
  const net = store.solaraBootstrap?.network;
  return {
    name: String(net?.name || 'Solara Network'),
    ip: String(net?.ip || net?.address || 'play.solaranetwork.in'),
    status: String(net?.status || 'Online'),
    version: String(net?.version || 'Java 1.21+'),
  };
});

const servicesList = computed(() => store.solaraBootstrap?.services ?? []);

const releaseVersions = computed(() => {
  const rel = store.mcVersions.filter((v) => v.type === 'release').slice(0, 16);
  return rel.length > 0 ? rel.map((r) => r.id) : ['1.21.4', '1.21.1', '1.20.4', '1.20.1'];
});

async function copyIp(): Promise<void> {
  try {
    await navigator.clipboard.writeText(networkInfo.value.ip);
    copiedIp.value = true;
    notify('success', 'Copied Server Address', networkInfo.value.ip);
    window.setTimeout(() => {
      copiedIp.value = false;
    }, 1800);
  } catch {
    notify('info', 'Solara Network', networkInfo.value.ip);
  }
}

async function signInSolara(): Promise<void> {
  if (!store.desktop) {
    notify('info', 'Desktop required', 'Solara SSO PKCE runs in the desktop launcher.');
    return;
  }
  signingInSolara.value = true;
  try {
    await native('solara_sign_in');
    await refreshSolaraProfile();
    notify('success', 'Connected to Solara Inc', 'Your Solara identity is signed in.');
    await loadRemoteServers();
  } catch (err) {
    notify('error', 'Solara sign-in failed', String(err));
  } finally {
    signingInSolara.value = false;
  }
}

async function signOutSolara(): Promise<void> {
  if (!store.desktop) return;
  try {
    await native('solara_sign_out');
    remoteServers.value = [];
    await refreshSolaraProfile();
    notify('info', 'Signed out of Solara Inc', 'Account credentials cleared from local vault.');
  } catch (err) {
    notify('error', 'Sign-out failed', String(err));
  }
}

async function loadRemoteServers(): Promise<void> {
  if (!store.desktop || !store.solaraProfile.signedIn) return;
  loadingRemoteServers.value = true;
  remoteServersError.value = null;
  try {
    const res = await native<unknown>('solara_request', {
      method: 'GET',
      path: '/v1/servers',
      body: null,
    });
    if (Array.isArray(res)) {
      remoteServers.value = res;
    } else if (res && typeof res === 'object' && Array.isArray((res as { servers?: unknown[] }).servers)) {
      remoteServers.value = (res as { servers: typeof remoteServers.value }).servers;
    } else {
      remoteServers.value = [];
    }
  } catch (err) {
    remoteServersError.value = err instanceof Error ? err.message : String(err);
  } finally {
    loadingRemoteServers.value = false;
  }
}

async function createLocalServer(): Promise<void> {
  if (!srvName.value.trim()) {
    notify('warning', 'Server name required', 'Enter a name for your local server.');
    return;
  }
  if (!srvEula.value) {
    notify('warning', 'Minecraft EULA required', 'Confirm acceptance of the Mojang Minecraft EULA before creating a local server.');
    return;
  }
  if (!store.desktop) {
    notify('info', 'Desktop required', 'Local servers run in the Solara desktop app.');
    return;
  }
  creatingServer.value = true;
  try {
    await native('create_local_server', {
      name: srvName.value.trim(),
      version: srvVersion.value,
      javaPath: 'auto',
      memory: srvMemory.value,
      port: srvPort.value,
      eula: srvEula.value,
    });
    showCreateServer.value = false;
    srvEula.value = false;
    notify('success', 'Local server created', `${srvName.value.trim()} is ready.`);
    await refreshSnapshot();
  } catch (err) {
    notify('error', "Couldn't create local server", String(err));
  } finally {
    creatingServer.value = false;
  }
}

async function serverAction(srv: LocalServerRecord, action: 'start' | 'stop' | 'accept-eula' | 'delete'): Promise<void> {
  try {
    await native('local_server_action', { id: srv.id, action });
    await refreshSnapshot();
    if (action === 'start') notify('success', 'Server starting', `${srv.name} on port ${srv.port}`);
    if (action === 'stop') notify('info', 'Server stopped', srv.name);
    if (action === 'delete') notify('info', 'Server removed', srv.name);
  } catch (err) {
    notify('error', 'Server action failed', String(err));
  }
}

async function openServerConsole(srv: LocalServerRecord): Promise<void> {
  activeConsoleServer.value = srv;
  loadingConsole.value = true;
  try {
    consoleLogs.value = (await native<string>('local_server_logs', { id: srv.id })) || 'No server output yet.';
  } catch (err) {
    consoleLogs.value = String(err);
  } finally {
    loadingConsole.value = false;
  }
}

async function sendConsoleCommand(): Promise<void> {
  if (!activeConsoleServer.value || !consoleCommand.value.trim()) return;
  const cmd = consoleCommand.value.trim();
  consoleCommand.value = '';
  try {
    await native('local_server_command', { id: activeConsoleServer.value.id, command: cmd });
    consoleLogs.value = (await native<string>('local_server_logs', { id: activeConsoleServer.value.id })) || consoleLogs.value;
  } catch (err) {
    notify('error', 'Command failed', String(err));
  }
}

onMounted(() => {
  void loadMinecraftVersions();
  void refreshSolaraBootstrap();
  void refreshSolaraProfile().then(() => {
    if (store.solaraProfile.signedIn) void loadRemoteServers();
  });
});
</script>

<template>
  <div class="sol-page sol-scroll">
    <div class="sol-view-wrap">
      <!-- Solara Hub Top Header -->
      <header class="sol-hub-header">
        <div>
          <h1 class="sol-page-title">Solara Inc Ecosystem</h1>
          <p class="sol-muted sol-text-xs">
            Official Solara Network realm, connected cloud services via api.solara-inc.in, and local Minecraft server hosting.
          </p>
        </div>

        <div class="sol-row-gap">
          <SolaraButton variant="ghost" size="sm" @click="openExternal('https://app.solara-inc.in')">
            <ExternalLink :size="13" /> Web Portal
          </SolaraButton>
          <SolaraButton
            v-if="!store.solaraProfile.signedIn"
            variant="primary"
            size="sm"
            :loading="signingInSolara"
            @click="signInSolara"
          >
            <LogIn :size="13" /> Sign in with Solara ID
          </SolaraButton>
          <SolaraButton v-else variant="secondary" size="sm" @click="signOutSolara">
            <LogOut :size="13" /> Sign out ({{ store.solaraProfile.displayName || store.solaraProfile.email || 'Solara' }})
          </SolaraButton>
        </div>
      </header>

      <!-- Solara Network Featured Banner -->
      <section class="sol-network-banner">
        <div class="sol-row-gap">
          <div class="sol-network-emblem">
            <Globe :size="22" />
          </div>
          <div>
            <div class="sol-row-gap">
              <strong class="sol-hub-net-title">{{ networkInfo.name }}</strong>
              <span class="sol-badge sol-badge--success">{{ networkInfo.status }}</span>
              <span class="sol-badge sol-badge--gold">{{ networkInfo.version }}</span>
            </div>
            <div class="sol-mono sol-text-sm" style="margin-top: 2px">{{ networkInfo.ip }}</div>
          </div>
        </div>

        <div class="sol-row-gap">
          <SolaraButton variant="primary" size="sm" @click="copyIp">
            <Check v-if="copiedIp" :size="13" />
            <Copy v-else :size="13" />
            {{ copiedIp ? 'Copied Address' : 'Copy Server IP' }}
          </SolaraButton>
          <SolaraButton variant="secondary" size="sm" @click="refreshSolaraBootstrap">
            <RefreshCw :size="13" /> Sync Status
          </SolaraButton>
        </div>
      </section>

      <!-- Solara Cloud Platforms & Services (from api.solara-inc.in/v1/launcher/bootstrap) -->
      <section class="sol-stack">
        <div class="sol-section-head">
          <h2 class="sol-section-title">Solara Services &amp; Hosting</h2>
          <span v-if="store.solaraBootstrapError" class="sol-muted sol-text-xs">
            Offline / {{ store.solaraBootstrapError }}
          </span>
        </div>

        <div v-if="servicesList.length > 0" class="sol-services-grid">
          <article v-for="(svc, idx) in servicesList" :key="svc.id || idx" class="sol-service-row">
            <div>
              <div class="sol-row-gap">
                <strong>{{ svc.name || svc.title || 'Solara Service' }}</strong>
                <span v-if="svc.badge || svc.status" class="sol-badge sol-badge--gold">
                  {{ svc.badge || svc.status }}
                </span>
              </div>
              <p v-if="svc.description || svc.summary" class="sol-muted sol-text-xs" style="margin-top: 4px">
                {{ svc.description || svc.summary }}
              </p>
            </div>
            <SolaraButton
              v-if="svc.url || svc.href"
              variant="secondary"
              size="xs"
              @click="openExternal(String(svc.url || svc.href))"
            >
              Open <ExternalLink :size="12" />
            </SolaraButton>
          </article>
        </div>
        <div v-else class="sol-muted sol-text-xs">
          Syncing live service catalog from api.solara-inc.in...
        </div>

        <!-- Authenticated Hosted Servers if signed into Solara ID -->
        <div v-if="store.solaraProfile.signedIn" class="sol-stack" style="margin-top: 8px">
          <div class="sol-section-head">
            <h3 class="sol-section-title">Your Solara Hosted Nodes</h3>
            <SolaraButton variant="ghost" size="xs" :loading="loadingRemoteServers" @click="loadRemoteServers">
              <RefreshCw :size="12" /> Refresh
            </SolaraButton>
          </div>

          <div v-if="remoteServersError" class="sol-muted sol-text-xs">{{ remoteServersError }}</div>
          <div v-else-if="remoteServers.length === 0" class="sol-muted sol-text-xs">
            No remote Solara nodes provisioned on this account yet.
          </div>
          <div v-else class="sol-row-list">
            <div v-for="(rs, idx) in remoteServers" :key="rs.id || idx" class="sol-row-item">
              <div>
                <strong>{{ rs.name || 'Node' }}</strong>
                <div class="sol-muted sol-text-xs">{{ rs.address || rs.plan || 'Solara Node' }}</div>
              </div>
              <span class="sol-badge">{{ rs.status || 'Active' }}</span>
            </div>
          </div>
        </div>
      </section>

      <!-- Local Minecraft Server Manager (Desktop Native) -->
      <section class="sol-stack">
        <div class="sol-section-head">
          <div>
            <h2 class="sol-section-title">Local Minecraft Servers</h2>
            <p class="sol-muted sol-text-xs">Host an isolated local Paper/Vanilla server directly on this PC with live console access.</p>
          </div>
          <SolaraButton variant="primary" size="sm" @click="showCreateServer = true">
            <Plus :size="14" /> New Local Server
          </SolaraButton>
        </div>

        <SolaraEmptyState
          v-if="store.localServers.length === 0"
          title="No local servers yet"
          description="Spin up an isolated local Minecraft server on this machine with one-click start/stop and live console commands."
        >
          <template #icon><Server :size="26" /></template>
          <template #actions>
            <SolaraButton variant="primary" size="sm" @click="showCreateServer = true">
              <Plus :size="14" /> Create Local Server
            </SolaraButton>
          </template>
        </SolaraEmptyState>

        <div v-else class="sol-row-list">
          <div v-for="srv in store.localServers" :key="srv.id" class="sol-row-item">
            <div class="sol-row-gap">
              <Server :size="18" class="sol-gold" />
              <div>
                <div class="sol-row-gap">
                  <strong>{{ srv.name }}</strong>
                  <span class="sol-badge">{{ srv.version }} · Port {{ srv.port }}</span>
                  <span
                    :class="[
                      'sol-badge',
                      srv.state === 'running' ? 'sol-badge--success' : srv.state === 'failed' ? 'sol-badge--danger' : '',
                    ]"
                  >
                    {{ srv.state }}
                  </span>
                </div>
                <div class="sol-muted sol-text-xs">
                  {{ srv.memory }} MB RAM · {{ srv.eulaAccepted ? 'EULA Accepted' : 'EULA Required' }}
                </div>
              </div>
            </div>

            <div class="sol-row-gap">
              <SolaraButton
                v-if="!srv.eulaAccepted"
                variant="secondary"
                size="xs"
                @click="serverAction(srv, 'accept-eula')"
              >
                <ShieldCheck :size="12" /> Accept EULA
              </SolaraButton>
              <SolaraButton variant="ghost" size="xs" @click="openServerConsole(srv)">
                <Terminal :size="12" /> Console
              </SolaraButton>
              <SolaraButton
                v-if="srv.state === 'running' || srv.state === 'starting'"
                variant="danger"
                size="xs"
                @click="serverAction(srv, 'stop')"
              >
                <Square :size="11" /> Stop
              </SolaraButton>
              <SolaraButton
                v-else
                variant="primary"
                size="xs"
                :disabled="!srv.eulaAccepted"
                @click="serverAction(srv, 'start')"
              >
                <Play :size="11" /> Start
              </SolaraButton>
              <button
                type="button"
                class="sol-icon-btn sol-icon-btn--xs"
                title="Delete local server"
                @click="serverAction(srv, 'delete')"
              >
                <Trash2 :size="13" />
              </button>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- Create Local Server Modal -->
    <SolaraModal
      :open="showCreateServer"
      title="Create Local Minecraft Server"
      subtitle="Downloads verified official server artifacts with isolated directory storage"
      width="md"
      @close="showCreateServer = false"
    >
      <div class="sol-stack">
        <div class="sol-field">
          <label class="sol-label">Server Name</label>
          <input v-model="srvName" type="text" class="sol-input" maxlength="60" />
        </div>

        <div class="sol-split-2">
          <div class="sol-field">
            <label class="sol-label">Minecraft Version</label>
            <select v-model="srvVersion" class="sol-input">
              <option v-for="v in releaseVersions" :key="v" :value="v">{{ v }}</option>
            </select>
          </div>
          <div class="sol-field">
            <label class="sol-label">Server Port</label>
            <input v-model.number="srvPort" type="number" min="1024" max="65535" class="sol-input" />
          </div>
        </div>

        <div class="sol-field">
          <label class="sol-label">Memory Allocation ({{ srvMemory }} MB)</label>
          <input v-model.number="srvMemory" type="range" min="1024" max="12288" step="512" class="sol-range" />
        </div>

        <label class="sol-checkbox-label">
          <input v-model="srvEula" type="checkbox" />
          <span>I agree to the Mojang Minecraft End User License Agreement (EULA).</span>
        </label>
      </div>

      <template #footer>
        <SolaraButton variant="ghost" @click="showCreateServer = false">Cancel</SolaraButton>
        <SolaraButton variant="primary" :loading="creatingServer" :disabled="!srvEula" @click="createLocalServer">
          Create Server
        </SolaraButton>
      </template>
    </SolaraModal>

    <!-- Local Server Console Modal -->
    <SolaraModal
      :open="Boolean(activeConsoleServer)"
      :title="activeConsoleServer ? `${activeConsoleServer.name} — Console` : 'Server Console'"
      width="lg"
      @close="activeConsoleServer = null"
    >
      <div class="sol-stack">
        <pre class="sol-log-viewer sol-scroll selectable" style="max-height: 360px">{{ consoleLogs }}</pre>
        <form class="sol-row-gap" @submit.prevent="sendConsoleCommand">
          <input
            v-model="consoleCommand"
            type="text"
            class="sol-input"
            placeholder="Enter server command (e.g. list, say Hello, stop)..."
          />
          <SolaraButton variant="primary" size="sm" type="submit">Send</SolaraButton>
          <SolaraButton variant="secondary" size="sm" :loading="loadingConsole" @click="activeConsoleServer && openServerConsole(activeConsoleServer)">
            <RefreshCw :size="13" />
          </SolaraButton>
        </form>
      </div>
    </SolaraModal>
  </div>
</template>
