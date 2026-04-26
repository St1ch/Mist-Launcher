use crate::api::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Runtime;
use theseus::prelude::*;
use uuid::Uuid;

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("local-servers")
        .invoke_handler(tauri::generate_handler![
            local_servers_list,
            local_servers_create,
            local_servers_delete,
        ])
        .build()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServer {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub port: u16,
    pub max_players: u16,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServerCreateRequest {
    pub name: String,
    pub game_version: String,
    pub loader: String,
    pub port: u16,
    pub max_players: u16,
}

#[tauri::command]
pub async fn local_servers_list() -> Result<Vec<LocalServer>> {
    read_servers().await
}

#[tauri::command]
pub async fn local_servers_create(
    request: LocalServerCreateRequest,
) -> Result<LocalServer> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err(other_error("Server name cannot be empty").into());
    }

    if request.port == 0 {
        return Err(other_error("Server port must be greater than 0").into());
    }

    if request.max_players == 0 {
        return Err(other_error("Max players must be greater than 0").into());
    }

    let now = Utc::now().to_rfc3339();
    let server = LocalServer {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        game_version: request.game_version,
        loader: request.loader,
        port: request.port,
        max_players: request.max_players,
        status: "Draft".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    let mut servers = read_servers().await?;
    servers.push(server.clone());
    write_servers(&servers).await?;

    Ok(server)
}

#[tauri::command]
pub async fn local_servers_delete(id: String) -> Result<()> {
    let mut servers = read_servers().await?;
    let original_len = servers.len();
    servers.retain(|server| server.id != id);

    if servers.len() == original_len {
        return Err(other_error("Local server not found").into());
    }

    write_servers(&servers).await?;
    Ok(())
}

async fn servers_file_path() -> Result<PathBuf> {
    let state = State::get().await?;
    Ok(state
        .directories
        .config_dir
        .join("servers")
        .join("local_servers.json"))
}

async fn read_servers() -> Result<Vec<LocalServer>> {
    let path = servers_file_path().await?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let bytes = tokio::fs::read(path).await?;
    let servers = serde_json::from_slice(&bytes)
        .map_err(|error| other_error(format!("Failed to read local servers: {error}")))?;
    Ok(servers)
}

async fn write_servers(servers: &[LocalServer]) -> Result<()> {
    let path = servers_file_path().await?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let bytes = serde_json::to_vec_pretty(servers)
        .map_err(|error| other_error(format!("Failed to write local servers: {error}")))?;
    tokio::fs::write(path, bytes).await?;
    Ok(())
}

fn other_error(message: impl Into<String>) -> theseus::Error {
    theseus::Error::from(theseus::ErrorKind::OtherError(message.into()))
}
