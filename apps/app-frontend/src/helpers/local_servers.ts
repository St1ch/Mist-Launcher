import { invoke } from '@tauri-apps/api/core'

export type LocalServer = {
	id: string
	name: string
	gameVersion: string
	loader: string
	port: number
	maxPlayers: number
	status: string
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
