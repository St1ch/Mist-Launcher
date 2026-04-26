import { invoke } from '@tauri-apps/api/core'

export type LocalServer = {
	id: string
	name: string
	gameVersion: string
	loader: string
	port: number
	maxPlayers: number
	status: string
	path: string
	jarPath: string | null
	createdAt: string
	updatedAt: string
}

export type LocalServerCreateRequest = {
	name: string
	gameVersion: string
	loader: string
	port: number
	maxPlayers: number
}

export async function list(): Promise<LocalServer[]> {
	return await invoke('plugin:local-servers|local_servers_list')
}

export async function create(request: LocalServerCreateRequest): Promise<LocalServer> {
	return await invoke('plugin:local-servers|local_servers_create', { request })
}

export async function remove(id: string): Promise<void> {
	return await invoke('plugin:local-servers|local_servers_delete', { id })
}

export async function prepare(id: string): Promise<LocalServer> {
	return await invoke('plugin:local-servers|local_servers_prepare', { id })
}

export async function start(id: string): Promise<LocalServer> {
	return await invoke('plugin:local-servers|local_servers_start', { id })
}

export async function stop(id: string): Promise<LocalServer> {
	return await invoke('plugin:local-servers|local_servers_stop', { id })
}

export async function logs(id: string): Promise<string> {
	const response = await invoke<{ log: string }>('plugin:local-servers|local_servers_logs', { id })
	return response.log
}
