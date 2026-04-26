use crate::api::Result;
use chrono::Utc;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::LazyLock;
use tauri::Runtime;
use theseus::prelude::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use uuid::Uuid;

static RUNNING_SERVERS: LazyLock<Mutex<HashMap<String, Child>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("local-servers")
        .invoke_handler(tauri::generate_handler![
            local_servers_list,
            local_servers_create,
            local_servers_delete,
            local_servers_prepare,
            local_servers_start,
            local_servers_stop,
            local_servers_logs,
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
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub jar_path: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalServerLogs {
    pub log: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionManifest {
    versions: Vec<VersionSummary>,
}

#[derive(Debug, Deserialize)]
struct VersionSummary {
    id: String,
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionDetails {
    downloads: VanillaDownloads,
}

#[derive(Debug, Deserialize)]
struct VanillaDownloads {
    server: VanillaDownload,
}

#[derive(Debug, Deserialize)]
struct VanillaDownload {
    url: String,
}

#[derive(Debug, Deserialize)]
struct PaperBuildsResponse {
    builds: Vec<PaperBuild>,
}

#[derive(Debug, Deserialize)]
struct PaperBuild {
    build: u32,
    downloads: PaperDownloads,
}

#[derive(Debug, Deserialize)]
struct PaperDownloads {
    application: PaperApplicationDownload,
}

#[derive(Debug, Deserialize)]
struct PaperApplicationDownload {
    name: String,
}

#[tauri::command]
pub async fn local_servers_list() -> Result<Vec<LocalServer>> {
    let mut servers = read_servers().await?;
    sync_running_statuses(&mut servers).await;
    Ok(servers)
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
    let id = Uuid::new_v4().to_string();
    let server_path = servers_root_path().await?.join(&id);
    let server = LocalServer {
        id,
        name: name.to_string(),
        game_version: request.game_version,
        loader: request.loader,
        port: request.port,
        max_players: request.max_players,
        status: "Created".to_string(),
        path: server_path.to_string_lossy().to_string(),
        jar_path: None,
        created_at: now.clone(),
        updated_at: now,
    };

    tokio::fs::create_dir_all(&server_path).await?;
    write_server_properties(&server).await?;
    write_eula(&server).await?;

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

#[tauri::command]
pub async fn local_servers_prepare(id: String) -> Result<LocalServer> {
    let mut servers = read_servers().await?;
    let server = find_server_mut(&mut servers, &id)?;
    server.status = "Preparing".to_string();
    server.updated_at = Utc::now().to_rfc3339();
    let server_snapshot = server.clone();
    write_servers(&servers).await?;

    tokio::fs::create_dir_all(server_dir(&server_snapshot)).await?;
    write_server_properties(&server_snapshot).await?;
    write_eula(&server_snapshot).await?;

    let jar_path = download_server_jar(&server_snapshot).await?;

    let mut servers = read_servers().await?;
    let server = find_server_mut(&mut servers, &id)?;
    server.jar_path = Some(jar_path.to_string_lossy().to_string());
    server.status = "Ready".to_string();
    server.updated_at = Utc::now().to_rfc3339();
    let prepared = server.clone();
    write_servers(&servers).await?;

    Ok(prepared)
}

#[tauri::command]
pub async fn local_servers_start(id: String) -> Result<LocalServer> {
    {
        let running = RUNNING_SERVERS.lock().await;
        if running.contains_key(&id) {
            return update_server_status(&id, "Running").await;
        }
    }

    let mut servers = read_servers().await?;
    let server = find_server_mut(&mut servers, &id)?;
    let jar_path = server
        .jar_path
        .clone()
        .ok_or_else(|| other_error("Prepare the server before starting it"))?;
    let server_path = server.path.clone();
    let server_id = server.id.clone();
    let server_name = server.name.clone();

    server.status = "Starting".to_string();
    server.updated_at = Utc::now().to_rfc3339();
    write_servers(&servers).await?;

    let log_path = PathBuf::from(&server_path).join("mist-server.log");
    let mut command = Command::new("java");
    command
        .arg("-Xms1G")
        .arg("-Xmx2G")
        .arg("-jar")
        .arg(jar_path)
        .arg("nogui")
        .current_dir(&server_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|error| {
        other_error(format!(
            "Failed to start {server_name}. Make sure Java is installed and available in PATH: {error}"
        ))
    })?;

    if let Some(stdout) = child.stdout.take() {
        pipe_log(stdout, log_path.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        pipe_log(stderr, log_path);
    }

    RUNNING_SERVERS.lock().await.insert(server_id.clone(), child);
    update_server_status(&server_id, "Running").await
}

#[tauri::command]
pub async fn local_servers_stop(id: String) -> Result<LocalServer> {
    let mut running = RUNNING_SERVERS.lock().await;
    if let Some(mut child) = running.remove(&id) {
        child.start_kill()?;
    }
    drop(running);

    update_server_status(&id, "Stopped").await
}

#[tauri::command]
pub async fn local_servers_logs(id: String) -> Result<LocalServerLogs> {
    let servers = read_servers().await?;
    let server = find_server(&servers, &id)?;
    let path = server_dir(server).join("mist-server.log");
    if !path.exists() {
        return Ok(LocalServerLogs { log: String::new() });
    }

    let log = tokio::fs::read_to_string(path).await?;
    let lines = log.lines().rev().take(220).collect::<Vec<_>>();
    Ok(LocalServerLogs {
        log: lines.into_iter().rev().collect::<Vec<_>>().join("\n"),
    })
}

async fn servers_file_path() -> Result<PathBuf> {
    let state = State::get().await?;
    Ok(state
        .directories
        .config_dir
        .join("servers")
        .join("local_servers.json"))
}

async fn servers_root_path() -> Result<PathBuf> {
    let state = State::get().await?;
    Ok(state.directories.config_dir.join("servers").join("local"))
}

async fn read_servers() -> Result<Vec<LocalServer>> {
    let path = servers_file_path().await?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let bytes = tokio::fs::read(path).await?;
    let mut servers: Vec<LocalServer> = serde_json::from_slice(&bytes)
        .map_err(|error| other_error(format!("Failed to read local servers: {error}")))?;
    let root = servers_root_path().await?;
    for server in &mut servers {
        if server.path.is_empty() {
            server.path = root.join(&server.id).to_string_lossy().to_string();
        }
    }
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

fn find_server<'a>(servers: &'a [LocalServer], id: &str) -> Result<&'a LocalServer> {
    servers
        .iter()
        .find(|server| server.id == id)
        .ok_or_else(|| other_error("Local server not found").into())
}

fn find_server_mut<'a>(
    servers: &'a mut [LocalServer],
    id: &str,
) -> Result<&'a mut LocalServer> {
    servers
        .iter_mut()
        .find(|server| server.id == id)
        .ok_or_else(|| other_error("Local server not found").into())
}

async fn update_server_status(id: &str, status: &str) -> Result<LocalServer> {
    let mut servers = read_servers().await?;
    let server = find_server_mut(&mut servers, id)?;
    server.status = status.to_string();
    server.updated_at = Utc::now().to_rfc3339();
    let updated = server.clone();
    write_servers(&servers).await?;
    Ok(updated)
}

async fn sync_running_statuses(servers: &mut [LocalServer]) {
    let running = RUNNING_SERVERS.lock().await;
    for server in servers {
        if running.contains_key(&server.id) {
            server.status = "Running".to_string();
        }
    }
}

fn server_dir(server: &LocalServer) -> PathBuf {
    PathBuf::from(&server.path)
}

async fn write_server_properties(server: &LocalServer) -> Result<()> {
    let properties = format!(
        "server-port={}\nmax-players={}\nonline-mode=false\nenable-command-block=true\nmotd={} powered by Mist Launcher\n",
        server.port, server.max_players, server.name
    );
    tokio::fs::write(server_dir(server).join("server.properties"), properties).await?;
    Ok(())
}

async fn write_eula(server: &LocalServer) -> Result<()> {
    tokio::fs::write(
        server_dir(server).join("eula.txt"),
        "# Generated by Mist Launcher\neula=true\n",
    )
    .await?;
    Ok(())
}

async fn download_server_jar(server: &LocalServer) -> Result<PathBuf> {
    match server.loader.as_str() {
        "vanilla" => download_vanilla_server(server).await,
        "paper" | "folia" => download_paper_family_server(server).await,
        "spigot" | "bukkit" => Err(other_error(
            "Automatic Spigot/Bukkit downloads are not implemented yet. Use Paper for now, or add a jar manually later.",
        )
        .into()),
        loader => Err(other_error(format!("Unsupported server loader: {loader}")).into()),
    }
}

async fn download_vanilla_server(server: &LocalServer) -> Result<PathBuf> {
    let client = Client::new();
    let manifest: VersionManifest = fetch_json(
        &client,
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
    )
    .await?;

    let version = manifest
        .versions
        .into_iter()
        .find(|version| version.id == server.game_version)
        .ok_or_else(|| other_error(format!("Minecraft {} was not found", server.game_version)))?;

    let details: VersionDetails = fetch_json(&client, &version.url).await?;

    download_to_file(
        &client,
        &details.downloads.server.url,
        server_dir(server).join("server.jar"),
    )
    .await
}

async fn download_paper_family_server(server: &LocalServer) -> Result<PathBuf> {
    let client = Client::new();
    let project = server.loader.as_str();
    let builds_url = format!(
        "https://api.papermc.io/v2/projects/{project}/versions/{}/builds",
        server.game_version
    );
    let builds: PaperBuildsResponse = fetch_json(&client, &builds_url).await?;
    let build = builds
        .builds
        .last()
        .ok_or_else(|| other_error(format!("No {project} builds for {}", server.game_version)))?;
    let download_url = format!(
        "https://api.papermc.io/v2/projects/{project}/versions/{}/builds/{}/downloads/{}",
        server.game_version, build.build, build.downloads.application.name
    );

    download_to_file(&client, &download_url, server_dir(server).join("server.jar")).await
}

async fn download_to_file(
    client: &Client,
    url: &str,
    destination: PathBuf,
) -> Result<PathBuf> {
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let bytes = client
        .get(url)
        .send()
        .await
        .map_err(network_error)?
        .error_for_status()
        .map_err(network_error)?
        .bytes()
        .await
        .map_err(network_error)?;
    tokio::fs::write(&destination, bytes).await?;
    Ok(destination)
}

async fn fetch_json<T>(client: &Client, url: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    client
        .get(url)
        .send()
        .await
        .map_err(network_error)?
        .error_for_status()
        .map_err(network_error)?
        .json()
        .await
        .map_err(network_error)
        .map_err(Into::into)
}

fn pipe_log<T>(stream: T, log_path: PathBuf)
where
    T: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut reader = BufReader::new(stream).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(mut file) = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .await
            {
                let _ = file.write_all(line.as_bytes()).await;
                let _ = file.write_all(b"\n").await;
            }
        }
    });
}

fn other_error(message: impl Into<String>) -> theseus::Error {
    theseus::Error::from(theseus::ErrorKind::OtherError(message.into()))
}

fn network_error(error: reqwest::Error) -> theseus::Error {
    other_error(format!("Server download request failed: {error}"))
}
