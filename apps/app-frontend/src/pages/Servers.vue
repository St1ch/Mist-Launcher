<script setup lang="ts">
import { ButtonStyled, injectNotificationManager, StyledInput } from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'

import * as localServers from '@/helpers/local_servers'
import type { LocalServer } from '@/helpers/local_servers'

const { handleError, addNotification } = injectNotificationManager()

const loaderOptions = [
	{ id: 'paper', name: 'Paper', description: 'Recommended for modern plugin servers.' },
	{ id: 'spigot', name: 'Spigot', description: 'Classic plugin server compatibility.' },
	{ id: 'bukkit', name: 'Bukkit', description: 'Lightweight legacy plugin base.' },
	{ id: 'folia', name: 'Folia', description: 'Experimental region-threaded server core.' },
	{ id: 'vanilla', name: 'Vanilla', description: 'Clean Minecraft server without plugins.' },
]

const versionOptions = ['1.21.11', '1.21.10', '1.21.9', '1.20.6', '1.20.4']

const servers = ref<LocalServer[]>([])
const loading = ref(true)
const creating = ref(false)
const operatingServerId = ref<string | null>(null)
const selectedLogServerId = ref<string | null>(null)
const selectedLog = ref('')

const selectedLoader = ref(loaderOptions[0].id)
const selectedVersion = ref(versionOptions[0])
const serverName = ref('Mist local server')
const port = ref(25565)
const maxPlayers = ref(20)

const selectedLoaderInfo = computed(
	() => loaderOptions.find((loader) => loader.id === selectedLoader.value) ?? loaderOptions[0],
)

onMounted(fetchServers)

async function fetchServers() {
	loading.value = true
	try {
		servers.value = await localServers.list()
	} catch (error) {
		handleError(error)
	} finally {
		loading.value = false
	}
}

async function createServer() {
	if (!serverName.value.trim()) {
		addNotification({
			title: 'Server name is required',
			text: 'Pick a name before creating a local server.',
			type: 'error',
		})
		return
	}

	creating.value = true
	try {
		const server = await localServers.create({
			name: serverName.value,
			gameVersion: selectedVersion.value,
			loader: selectedLoader.value,
			port: Number(port.value),
			maxPlayers: Number(maxPlayers.value),
		})
		servers.value = [...servers.value, server]
		addNotification({
			title: 'Server created',
			text: `${server.name} was added to your local server dashboard.`,
			type: 'success',
		})
	} catch (error) {
		handleError(error)
	} finally {
		creating.value = false
	}
}

async function prepareServer(id: string) {
	await runServerAction(id, async () => {
		const server = await localServers.prepare(id)
		updateServer(server)
		addNotification({
			title: 'Server prepared',
			text: `${server.name} is ready to start.`,
			type: 'success',
		})
	})
}

async function startServer(id: string) {
	await runServerAction(id, async () => {
		const server = await localServers.start(id)
		updateServer(server)
		addNotification({
			title: 'Server started',
			text: `${server.name} is running locally.`,
			type: 'success',
		})
	})
}

async function stopServer(id: string) {
	await runServerAction(id, async () => {
		const server = await localServers.stop(id)
		updateServer(server)
		addNotification({
			title: 'Server stopped',
			text: `${server.name} was stopped.`,
			type: 'success',
		})
	})
}

async function showLogs(id: string) {
	await runServerAction(id, async () => {
		selectedLogServerId.value = id
		selectedLog.value = await localServers.logs(id)
	})
}

async function runServerAction(id: string, action: () => Promise<void>) {
	operatingServerId.value = id
	try {
		await action()
	} catch (error) {
		handleError(error)
	} finally {
		operatingServerId.value = null
	}
}

async function deleteServer(id: string) {
	try {
		await localServers.remove(id)
		servers.value = servers.value.filter((server) => server.id !== id)
		if (selectedLogServerId.value === id) {
			selectedLogServerId.value = null
			selectedLog.value = ''
		}
	} catch (error) {
		handleError(error)
	}
}

function updateServer(updatedServer: LocalServer) {
	servers.value = servers.value.map((server) =>
		server.id === updatedServer.id ? updatedServer : server,
	)
}

function loaderName(loaderId: string) {
	return loaderOptions.find((loader) => loader.id === loaderId)?.name ?? loaderId
}

function isOperating(server: LocalServer) {
	return operatingServerId.value === server.id
}
</script>

<template>
	<div class="servers-page app-viewport">
		<div class="servers-header">
			<div>
				<h1>Servers</h1>
				<p>Create and manage local Minecraft servers from Mist Launcher.</p>
			</div>
			<ButtonStyled>
				<button @click="fetchServers" :disabled="loading">Refresh</button>
			</ButtonStyled>
		</div>

		<div class="servers-layout">
			<section class="panel">
				<div class="panel-heading">
					<div>
						<h2>Create local server</h2>
						<p>Choose a Minecraft version and server core, then prepare and start it locally.</p>
					</div>
				</div>

				<div class="form-grid">
					<label class="field field-wide">
						<span>Name</span>
						<StyledInput
							id="local-server-name"
							v-model="serverName"
							autocomplete="off"
							type="text"
							wrapper-class="w-full"
						/>
					</label>

					<label class="field">
						<span>Minecraft version</span>
						<select v-model="selectedVersion">
							<option v-for="version in versionOptions" :key="version" :value="version">
								{{ version }}
							</option>
						</select>
					</label>

					<label class="field">
						<span>Port</span>
						<StyledInput
							id="local-server-port"
							v-model="port"
							autocomplete="off"
							type="number"
							wrapper-class="w-full"
						/>
					</label>

					<label class="field">
						<span>Max players</span>
						<StyledInput
							id="local-server-max-players"
							v-model="maxPlayers"
							autocomplete="off"
							type="number"
							wrapper-class="w-full"
						/>
					</label>
				</div>

				<div class="loader-list">
					<button
						v-for="loader in loaderOptions"
						:key="loader.id"
						class="loader-card"
						:class="{ selected: selectedLoader === loader.id }"
						@click="selectedLoader = loader.id"
					>
						<strong>{{ loader.name }}</strong>
						<span>{{ loader.description }}</span>
					</button>
				</div>

				<div class="panel-actions">
					<p>
						Creating: {{ selectedLoaderInfo.name }} {{ selectedVersion }} on port {{ port }}
					</p>
					<ButtonStyled color="brand">
						<button @click="createServer" :disabled="creating">
							{{ creating ? 'Creating...' : 'Create server' }}
						</button>
					</ButtonStyled>
				</div>
			</section>

			<section class="panel">
				<div class="panel-heading">
					<div>
						<h2>Local dashboard</h2>
						<p>Prepare server files, start Java, stop the process, and inspect logs.</p>
					</div>
				</div>

				<div v-if="loading" class="empty-state">Loading servers...</div>
				<div v-else-if="servers.length === 0" class="empty-state">
					No local servers yet. Create one to start building the dashboard.
				</div>
				<div v-else class="server-list">
					<article v-for="server in servers" :key="server.id" class="server-row">
						<div class="server-main">
							<strong>{{ server.name }}</strong>
							<p>
								{{ loaderName(server.loader) }} {{ server.gameVersion }} - Port {{ server.port }}
							</p>
							<small>{{ server.path }}</small>
						</div>
						<div class="server-meta">
							<span>{{ server.maxPlayers }} slots</span>
							<span class="status-pill" :class="server.status.toLowerCase()">
								{{ server.status }}
							</span>
						</div>
						<div class="server-actions">
							<ButtonStyled>
								<button @click="prepareServer(server.id)" :disabled="isOperating(server)">
									{{ isOperating(server) ? 'Working...' : 'Prepare' }}
								</button>
							</ButtonStyled>
							<ButtonStyled color="brand">
								<button
									@click="startServer(server.id)"
									:disabled="isOperating(server) || server.status === 'Running'"
								>
									Start
								</button>
							</ButtonStyled>
							<ButtonStyled>
								<button
									@click="stopServer(server.id)"
									:disabled="isOperating(server) || server.status !== 'Running'"
								>
									Stop
								</button>
							</ButtonStyled>
							<ButtonStyled>
								<button @click="showLogs(server.id)" :disabled="isOperating(server)">Logs</button>
							</ButtonStyled>
							<ButtonStyled>
								<button @click="deleteServer(server.id)" :disabled="isOperating(server)">Delete</button>
							</ButtonStyled>
						</div>
					</article>
				</div>
			</section>

			<section class="panel panel-wide">
				<div class="panel-heading">
					<div>
						<h2>Console preview</h2>
						<p>Latest server output captured by Mist Launcher.</p>
					</div>
				</div>
				<pre class="log-output">{{
					selectedLog || 'Select Logs on a prepared/running server to preview output.'
				}}</pre>
			</section>
		</div>
	</div>
</template>

<style scoped lang="scss">
.servers-page {
	display: flex;
	flex-direction: column;
	gap: 1.5rem;
	padding: 2rem;
	background: var(--color-bg);
}

.servers-header {
	display: flex;
	align-items: flex-start;
	justify-content: space-between;
	gap: 1rem;
}

.servers-header h1 {
	margin: 0;
	color: var(--color-contrast);
	font-size: 2rem;
	line-height: 1.15;
}

.servers-header p,
.panel-heading p,
.panel-actions p,
.server-row p {
	margin: 0.35rem 0 0;
	color: var(--color-secondary);
}

.servers-layout {
	display: grid;
	grid-template-columns: minmax(24rem, 1.05fr) minmax(22rem, 0.95fr);
	gap: 1rem;
}

.panel {
	border: 1px solid var(--color-button-border);
	border-radius: var(--radius-lg);
	background: var(--color-raised-bg);
	padding: 1.25rem;
}

.panel-wide {
	grid-column: 1 / -1;
}

.panel-heading {
	display: flex;
	justify-content: space-between;
	gap: 1rem;
	margin-bottom: 1.25rem;
}

.panel-heading h2 {
	margin: 0;
	color: var(--color-contrast);
	font-size: 1.2rem;
}

.form-grid {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 1rem;
}

.field {
	display: grid;
	gap: 0.45rem;
	color: var(--color-secondary);
	font-weight: 700;
}

.field-wide {
	grid-column: 1 / -1;
}

.field select {
	border: 1px solid var(--color-button-border);
	border-radius: var(--radius-md);
	background: var(--color-button-bg);
	color: var(--color-contrast);
	padding: 0.75rem 1rem;
	font: inherit;
	min-height: 2.5rem;
}

.loader-list {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 0.75rem;
	margin: 1.25rem 0;
}

.loader-card {
	display: grid;
	gap: 0.35rem;
	text-align: left;
	border: 1px solid var(--color-button-border);
	border-radius: var(--radius-md);
	background: var(--color-button-bg);
	color: var(--color-secondary);
	padding: 1rem;
	cursor: pointer;
	transition:
		border-color 0.15s ease,
		background 0.15s ease;
}

.loader-card:hover,
.loader-card.selected {
	border-color: var(--color-brand);
	background: var(--color-brand-highlight);
}

.loader-card strong,
.server-row strong {
	color: var(--color-contrast);
}

.panel-actions {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 1rem;
	border-top: 1px solid var(--color-button-border);
	padding-top: 1rem;
}

.empty-state {
	display: grid;
	min-height: 11rem;
	place-items: center;
	border: 1px dashed var(--color-button-border);
	border-radius: var(--radius-lg);
	color: var(--color-secondary);
	text-align: center;
	padding: 1.5rem;
}

.server-list {
	display: flex;
	flex-direction: column;
	gap: 0.75rem;
}

.server-row {
	display: grid;
	grid-template-columns: minmax(0, 1fr) auto;
	align-items: center;
	gap: 1rem;
	border: 1px solid var(--color-button-border);
	border-radius: var(--radius-md);
	background: var(--color-button-bg);
	padding: 1rem;
}

.server-main {
	min-width: 0;
}

.server-main small {
	display: block;
	overflow: hidden;
	margin-top: 0.45rem;
	color: var(--color-secondary);
	text-overflow: ellipsis;
	white-space: nowrap;
}

.server-meta {
	display: flex;
	flex-direction: column;
	align-items: flex-end;
	gap: 0.35rem;
	color: var(--color-secondary);
	font-weight: 700;
}

.server-actions {
	display: flex;
	grid-column: 1 / -1;
	flex-wrap: wrap;
	gap: 0.5rem;
}

.status-pill {
	border: 1px solid var(--color-button-border);
	border-radius: var(--radius-sm);
	background: var(--color-button-bg);
	padding: 0.2rem 0.5rem;
	color: var(--color-secondary);
}

.status-pill.running,
.status-pill.ready {
	border-color: var(--color-brand);
	background: var(--color-brand-highlight);
	color: var(--color-contrast);
}

.status-pill.preparing,
.status-pill.starting {
	border-color: var(--color-orange);
	color: var(--color-contrast);
}

.log-output {
	overflow: auto;
	min-height: 12rem;
	max-height: 24rem;
	margin: 0;
	border: 1px solid var(--color-button-border);
	border-radius: var(--radius-md);
	background: var(--color-bg);
	color: var(--color-contrast);
	padding: 1rem;
	white-space: pre-wrap;
}

@media (max-width: 980px) {
	.servers-layout,
	.form-grid,
	.loader-list {
		grid-template-columns: 1fr;
	}

	.server-row {
		grid-template-columns: 1fr;
		align-items: stretch;
	}

	.server-meta {
		align-items: flex-start;
	}
}
</style>
