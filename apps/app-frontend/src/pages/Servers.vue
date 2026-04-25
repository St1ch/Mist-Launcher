<script setup lang="ts">
import { ButtonStyled } from '@modrinth/ui'
import { computed, ref } from 'vue'

const loaderOptions = [
	{ id: 'paper', name: 'Paper', description: 'Recommended for modern plugin servers.' },
	{ id: 'spigot', name: 'Spigot', description: 'Classic plugin server compatibility.' },
	{ id: 'bukkit', name: 'Bukkit', description: 'Lightweight legacy plugin base.' },
	{ id: 'folia', name: 'Folia', description: 'Experimental region-threaded server core.' },
	{ id: 'vanilla', name: 'Vanilla', description: 'Clean Minecraft server without plugins.' },
]

const versionOptions = ['1.21.11', '1.21.10', '1.21.9', '1.20.6', '1.20.4']

const servers = ref([
	{
		id: 'local-dev',
		name: 'Local test server',
		version: '1.21.11',
		loader: 'Paper',
		status: 'Draft',
		players: '0/20',
		port: 25565,
	},
])

const selectedLoader = ref(loaderOptions[0].id)
const selectedVersion = ref(versionOptions[0])
const serverName = ref('Mist local server')

const selectedLoaderInfo = computed(
	() => loaderOptions.find((loader) => loader.id === selectedLoader.value) ?? loaderOptions[0],
)
</script>

<template>
	<div class="local-servers-page">
		<section class="server-hero">
			<div>
				<p class="eyebrow">Local server lab</p>
				<h1>Servers</h1>
				<p class="hero-copy">
					Create and manage local Minecraft servers directly from Mist Launcher. This is the
					new offline-first shell for server creation, runtime control, and logs.
				</p>
			</div>
			<div class="hero-card">
				<span class="status-dot"></span>
				<div>
					<strong>Next milestone</strong>
					<p>Wire this UI to a local server runtime, downloads, process control, and logs.</p>
				</div>
			</div>
		</section>

		<section class="server-grid">
			<div class="panel create-panel">
				<div class="panel-heading">
					<div>
						<p class="eyebrow">New server</p>
						<h2>Choose version and core</h2>
					</div>
				</div>

				<label class="field">
					<span>Name</span>
					<input v-model="serverName" type="text" />
				</label>

				<label class="field">
					<span>Minecraft version</span>
					<select v-model="selectedVersion">
						<option v-for="version in versionOptions" :key="version" :value="version">
							{{ version }}
						</option>
					</select>
				</label>

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

				<ButtonStyled color="brand" size="large">
					<button disabled title="Local server backend is next">
						Create {{ selectedLoaderInfo.name }} server
					</button>
				</ButtonStyled>
			</div>

			<div class="panel">
				<div class="panel-heading">
					<div>
						<p class="eyebrow">Dashboard</p>
						<h2>Local servers</h2>
					</div>
				</div>

				<div class="server-list">
					<article v-for="server in servers" :key="server.id" class="server-row">
						<div>
							<strong>{{ server.name }}</strong>
							<p>{{ server.loader }} {{ server.version }} - Port {{ server.port }}</p>
						</div>
						<div class="server-meta">
							<span>{{ server.players }}</span>
							<span>{{ server.status }}</span>
						</div>
					</article>
				</div>
			</div>
		</section>
	</div>
</template>

<style scoped lang="scss">
.local-servers-page {
	min-height: 100%;
	padding: 2.5rem;
	background:
		radial-gradient(circle at top left, rgba(143, 255, 210, 0.16), transparent 34rem),
		linear-gradient(135deg, rgba(13, 21, 25, 0.92), rgba(8, 11, 17, 0.98));
}

.server-hero {
	display: grid;
	grid-template-columns: minmax(0, 1fr) 22rem;
	gap: 1.5rem;
	align-items: stretch;
	margin-bottom: 1.5rem;
}

.eyebrow {
	margin: 0 0 0.45rem;
	color: var(--color-brand);
	font-size: 0.78rem;
	font-weight: 800;
	letter-spacing: 0.12em;
	text-transform: uppercase;
}

h1,
h2,
p {
	margin-top: 0;
}

h1 {
	margin-bottom: 0.75rem;
	color: var(--color-contrast);
	font-size: clamp(2.5rem, 5vw, 4.8rem);
	line-height: 0.95;
}

h2 {
	margin-bottom: 0;
	color: var(--color-contrast);
}

.hero-copy {
	max-width: 44rem;
	color: var(--color-secondary);
	font-size: 1.05rem;
	line-height: 1.7;
}

.hero-card,
.panel {
	border: 1px solid rgba(143, 255, 210, 0.16);
	border-radius: 1.5rem;
	background: rgba(23, 31, 38, 0.78);
	box-shadow: 0 1.5rem 4rem rgba(0, 0, 0, 0.22);
}

.hero-card {
	display: flex;
	gap: 1rem;
	align-items: flex-start;
	padding: 1.25rem;
	color: var(--color-secondary);
}

.hero-card strong {
	color: var(--color-contrast);
}

.status-dot {
	width: 0.8rem;
	height: 0.8rem;
	margin-top: 0.25rem;
	border-radius: 999px;
	background: var(--color-brand);
	box-shadow: 0 0 1.3rem rgba(143, 255, 210, 0.85);
}

.server-grid {
	display: grid;
	grid-template-columns: minmax(24rem, 1.05fr) minmax(20rem, 0.95fr);
	gap: 1.5rem;
}

.panel {
	padding: 1.4rem;
}

.panel-heading {
	display: flex;
	justify-content: space-between;
	margin-bottom: 1.25rem;
}

.field {
	display: grid;
	gap: 0.45rem;
	margin-bottom: 1rem;
	color: var(--color-secondary);
	font-weight: 700;
}

.field input,
.field select {
	border: 1px solid var(--color-button-border);
	border-radius: 0.9rem;
	background: var(--color-button-bg);
	color: var(--color-contrast);
	padding: 0.85rem 1rem;
	font: inherit;
}

.loader-list {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 0.75rem;
	margin: 1.2rem 0;
}

.loader-card {
	display: grid;
	gap: 0.35rem;
	text-align: left;
	border: 1px solid var(--color-button-border);
	border-radius: 1rem;
	background: rgba(255, 255, 255, 0.03);
	color: var(--color-secondary);
	padding: 1rem;
	cursor: pointer;
}

.loader-card strong {
	color: var(--color-contrast);
}

.loader-card.selected {
	border-color: var(--color-brand);
	background: rgba(143, 255, 210, 0.1);
	box-shadow: inset 0 0 0 1px rgba(143, 255, 210, 0.16);
}

.server-list {
	display: grid;
	gap: 0.85rem;
}

.server-row {
	display: flex;
	justify-content: space-between;
	gap: 1rem;
	border-radius: 1rem;
	background: rgba(255, 255, 255, 0.04);
	padding: 1rem;
}

.server-row strong {
	color: var(--color-contrast);
}

.server-row p,
.hero-card p {
	margin: 0.25rem 0 0;
	color: var(--color-secondary);
}

.server-meta {
	display: flex;
	flex-direction: column;
	align-items: flex-end;
	gap: 0.35rem;
	color: var(--color-brand);
	font-weight: 800;
}

@media (max-width: 900px) {
	.local-servers-page {
		padding: 1.25rem;
	}

	.server-hero,
	.server-grid {
		grid-template-columns: 1fr;
	}

	.loader-list {
		grid-template-columns: 1fr;
	}
}
</style>
