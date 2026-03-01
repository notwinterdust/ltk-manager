use crate::error::{AppError, AppResult, IpcResult, MutexResultExt};
use crate::mods::ModLibraryState;
use crate::patcher::PatcherState;
use crate::state::{save_settings_to_disk, SettingsState};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use super::patcher::{start_patcher_inner, PatcherConfig};

/// Register a global shortcut for hot-reloading mods.
fn register_reload_hotkey(app_handle: &AppHandle, accelerator: &str) -> AppResult<()> {
    let manager = app_handle.global_shortcut();
    let handle = app_handle.clone();

    manager
        .on_shortcut(accelerator, move |_app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            tracing::info!("Hot reload hotkey pressed");
            let handle_inner = handle.clone();
            std::thread::spawn(move || {
                if let Err(e) = execute_hot_reload(&handle_inner) {
                    tracing::error!("Hot reload failed: {}", e);
                    let _ = handle_inner.emit("hotkey-error", e.to_string());
                }
            });
        })
        .map_err(|e| {
            AppError::ValidationFailed(format!(
                "Hotkey \"{}\" could not be registered: {}",
                accelerator, e
            ))
        })?;

    tracing::info!("Registered reload-mods hotkey: {}", accelerator);
    Ok(())
}

/// Register a global shortcut for killing League.
fn register_kill_hotkey(app_handle: &AppHandle, accelerator: &str) -> AppResult<()> {
    let manager = app_handle.global_shortcut();
    let handle = app_handle.clone();

    manager
        .on_shortcut(accelerator, move |_app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            tracing::info!("Kill league hotkey pressed");
            let handle_inner = handle.clone();
            std::thread::spawn(move || {
                if let Err(e) = execute_kill_league(&handle_inner) {
                    tracing::error!("Kill league failed: {}", e);
                    let _ = handle_inner.emit("hotkey-error", e.to_string());
                }
            });
        })
        .map_err(|e| {
            AppError::ValidationFailed(format!(
                "Hotkey \"{}\" could not be registered: {}",
                accelerator, e
            ))
        })?;

    tracing::info!("Registered kill-league hotkey: {}", accelerator);
    Ok(())
}

/// Unregister a global shortcut if it exists.
fn unregister_hotkey(app_handle: &AppHandle, accelerator: &str) {
    let manager = app_handle.global_shortcut();
    if let Err(e) = manager.unregister(accelerator) {
        tracing::warn!("Failed to unregister hotkey {}: {}", accelerator, e);
    } else {
        tracing::info!("Unregistered global hotkey: {}", accelerator);
    }
}

/// Register persisted hotkeys from settings on app startup.
pub fn register_startup_hotkeys(app_handle: &AppHandle, settings_state: &SettingsState) {
    let settings = match settings_state.0.lock() {
        Ok(s) => s.clone(),
        Err(e) => {
            tracing::error!("Failed to lock settings for hotkey registration: {}", e);
            return;
        }
    };

    if let Some(ref hotkey) = settings.reload_mods_hotkey {
        let trimmed = hotkey.trim();
        if !trimmed.is_empty() {
            if let Err(e) = register_reload_hotkey(app_handle, trimmed) {
                tracing::error!("Failed to register reload-mods hotkey on startup: {}", e);
            }
        }
    }

    if let Some(ref hotkey) = settings.kill_league_hotkey {
        let trimmed = hotkey.trim();
        if !trimmed.is_empty() {
            if let Err(e) = register_kill_hotkey(app_handle, trimmed) {
                tracing::error!("Failed to register kill-league hotkey on startup: {}", e);
            }
        }
    }
}

// ── Hotkey action implementations (called from shortcut callbacks) ──

/// Execute hot-reload: stop patcher → kill League → restart patcher.
fn execute_hot_reload(app_handle: &AppHandle) -> AppResult<()> {
    let patcher_state = app_handle.state::<PatcherState>();
    let settings_state = app_handle.state::<SettingsState>();
    let library_state = app_handle.state::<ModLibraryState>();

    // Get the last config before stopping
    let last_config = {
        let ps = patcher_state.0.lock().mutex_err()?;
        ps.last_config.clone()
    };

    let config = last_config
        .ok_or_else(|| AppError::Other("No previous patcher config to reload with".to_string()))?;

    // Stop patcher if running
    {
        let ps = patcher_state.0.lock().mutex_err()?;
        if ps.is_running() {
            tracing::info!("Hot reload: stopping patcher...");
            ps.stop_flag.store(true, Ordering::SeqCst);
        }
    }

    wait_for_patcher_stop(&patcher_state)?;
    kill_league_process();
    std::thread::sleep(std::time::Duration::from_millis(500));

    let workshop_projects = config.workshop_projects.clone();

    let patcher_config = PatcherConfig {
        log_file: config.log_file,
        timeout_ms: config.timeout_ms,
        flags: config.flags,
        workshop_projects: config.workshop_projects,
    };

    tracing::info!("Hot reload: restarting patcher");
    start_patcher_inner(
        patcher_config,
        app_handle,
        &patcher_state,
        &settings_state,
        &library_state,
    )?;

    // Best-effort LCU reconnect (in background — retries take time)
    let league_path = {
        let s = settings_state.0.lock().mutex_err()?;
        s.league_path.clone()
    };
    if let Some(path) = league_path {
        std::thread::spawn(move || try_lcu_reconnect(&path));
    }

    // Emit workshop project paths so frontend can re-sync testing state
    let _ = app_handle.emit("hotkey-reload-complete", workshop_projects);
    Ok(())
}

/// Execute kill-league action.
fn execute_kill_league(app_handle: &AppHandle) -> AppResult<()> {
    let patcher_state = app_handle.state::<PatcherState>();
    let settings_state = app_handle.state::<SettingsState>();

    let should_stop_patcher = {
        let s = settings_state.0.lock().mutex_err()?;
        s.kill_league_stops_patcher
    };

    if should_stop_patcher {
        let ps = patcher_state.0.lock().mutex_err()?;
        if ps.is_running() {
            tracing::info!("Kill league: also stopping patcher");
            ps.stop_flag.store(true, Ordering::SeqCst);
        }
        drop(ps);
        wait_for_patcher_stop(&patcher_state)?;
    }

    kill_league_process();
    Ok(())
}

// ── IPC commands (called from frontend) ──

/// Set (or clear) the global hotkey for reloading mods.
#[tauri::command]
pub fn set_reload_mods_hotkey(
    accelerator: Option<String>,
    app_handle: AppHandle,
    settings: State<SettingsState>,
) -> IpcResult<()> {
    set_reload_mods_hotkey_inner(accelerator, &app_handle, &settings).into()
}

fn set_reload_mods_hotkey_inner(
    accelerator: Option<String>,
    app_handle: &AppHandle,
    settings: &State<SettingsState>,
) -> AppResult<()> {
    let mut s = settings.0.lock().mutex_err()?;

    // Unregister old hotkey
    if let Some(ref old) = s.reload_mods_hotkey {
        unregister_hotkey(app_handle, old);
    }

    match accelerator {
        Some(ref accel) if !accel.trim().is_empty() => {
            let trimmed = accel.trim().to_string();
            register_reload_hotkey(app_handle, &trimmed)?;
            s.reload_mods_hotkey = Some(trimmed);
        }
        _ => {
            s.reload_mods_hotkey = None;
        }
    }

    save_settings_to_disk(app_handle, &s)?;
    Ok(())
}

/// Set (or clear) the global hotkey for killing League.
#[tauri::command]
pub fn set_kill_league_hotkey(
    accelerator: Option<String>,
    app_handle: AppHandle,
    settings: State<SettingsState>,
) -> IpcResult<()> {
    set_kill_league_hotkey_inner(accelerator, &app_handle, &settings).into()
}

fn set_kill_league_hotkey_inner(
    accelerator: Option<String>,
    app_handle: &AppHandle,
    settings: &State<SettingsState>,
) -> AppResult<()> {
    let mut s = settings.0.lock().mutex_err()?;

    // Unregister old hotkey
    if let Some(ref old) = s.kill_league_hotkey {
        unregister_hotkey(app_handle, old);
    }

    match accelerator {
        Some(ref accel) if !accel.trim().is_empty() => {
            let trimmed = accel.trim().to_string();
            register_kill_hotkey(app_handle, &trimmed)?;
            s.kill_league_hotkey = Some(trimmed);
        }
        _ => {
            s.kill_league_hotkey = None;
        }
    }

    save_settings_to_disk(app_handle, &s)?;
    Ok(())
}

/// Hot-reload: stop patcher, kill League, restart patcher with the last config.
#[tauri::command]
pub fn hot_reload_mods(
    app_handle: AppHandle,
    state: State<PatcherState>,
    settings: State<SettingsState>,
    library: State<ModLibraryState>,
) -> IpcResult<()> {
    hot_reload_mods_inner(&app_handle, &state, &settings, &library).into()
}

fn hot_reload_mods_inner(
    app_handle: &AppHandle,
    state: &State<PatcherState>,
    settings: &State<SettingsState>,
    library: &State<ModLibraryState>,
) -> AppResult<()> {
    tracing::info!("Hot reload triggered via command");

    let last_config = {
        let ps = state.0.lock().mutex_err()?;
        ps.last_config.clone()
    };

    let config = last_config
        .ok_or_else(|| AppError::Other("No previous patcher config to reload with".to_string()))?;

    // Stop patcher if running
    {
        let ps = state.0.lock().mutex_err()?;
        if ps.is_running() {
            tracing::info!("Stopping patcher for hot reload...");
            ps.stop_flag.store(true, Ordering::SeqCst);
        }
    }

    wait_for_patcher_stop(state)?;
    kill_league_process();
    std::thread::sleep(std::time::Duration::from_millis(500));

    let patcher_config = PatcherConfig {
        log_file: config.log_file,
        timeout_ms: config.timeout_ms,
        flags: config.flags,
        workshop_projects: config.workshop_projects,
    };

    tracing::info!("Restarting patcher after hot reload");
    start_patcher_inner(patcher_config, app_handle, state, settings, library)?;

    // Best-effort LCU reconnect (in background — retries take time)
    let league_path = {
        let s = settings.0.lock().mutex_err()?;
        s.league_path.clone()
    };
    if let Some(path) = league_path {
        std::thread::spawn(move || try_lcu_reconnect(&path));
    }

    Ok(())
}

/// Kill League of Legends process, optionally stopping the patcher.
#[tauri::command]
pub fn kill_league(state: State<PatcherState>, settings: State<SettingsState>) -> IpcResult<()> {
    kill_league_inner(&state, &settings).into()
}

fn kill_league_inner(
    state: &State<PatcherState>,
    settings: &State<SettingsState>,
) -> AppResult<()> {
    let should_stop_patcher = {
        let s = settings.0.lock().mutex_err()?;
        s.kill_league_stops_patcher
    };

    if should_stop_patcher {
        let ps = state.0.lock().mutex_err()?;
        if ps.is_running() {
            tracing::info!("Kill league: also stopping patcher");
            ps.stop_flag.store(true, Ordering::SeqCst);
        }
        drop(ps);
        wait_for_patcher_stop(state)?;
    }

    kill_league_process();
    Ok(())
}

// ── Helpers ──

/// Wait for the patcher thread to finish (with timeout).
fn wait_for_patcher_stop(state: &PatcherState) -> AppResult<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        {
            let ps = state.0.lock().mutex_err()?;
            if !ps.is_running() {
                break;
            }
        }
        if std::time::Instant::now() > deadline {
            tracing::warn!("Timed out waiting for patcher to stop");
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Ok(())
}

// ── LCU Reconnect ──

/// Parsed LCU lockfile data.
struct LockfileData {
    port: u16,
    password: String,
}

/// Read and parse the League Client lockfile.
/// Format: `LeagueClient:pid:port:password:https` (5-part) or `process:port:password:protocol` (4-part).
fn read_lockfile(league_path: &Path) -> Option<LockfileData> {
    let lockfile_path = league_path.join("lockfile");

    let content = match std::fs::read_to_string(&lockfile_path) {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("Could not read lockfile at {:?}: {}", lockfile_path, e);
            return None;
        }
    };

    let parts: Vec<&str> = content.trim().split(':').collect();

    match parts.len() {
        4 => {
            // Old format: process:port:password:protocol
            let port = parts[1].parse::<u16>().ok()?;
            let password = parts[2].to_string();
            tracing::debug!("Parsed lockfile (4-part): port={}", port);
            Some(LockfileData { port, password })
        }
        5 => {
            // New format: process:pid:port:password:protocol
            let port = parts[2].parse::<u16>().ok()?;
            let password = parts[3].to_string();
            tracing::debug!("Parsed lockfile (5-part): port={}", port);
            Some(LockfileData { port, password })
        }
        n if n > 5 => {
            // Try 5-part interpretation
            let port = parts[2].parse::<u16>().ok()?;
            let password = parts[3].to_string();
            tracing::warn!(
                "Lockfile has {} parts, using 5-part format guess: port={}",
                n,
                port
            );
            Some(LockfileData { port, password })
        }
        _ => {
            tracing::warn!("Invalid lockfile format: {} parts", parts.len());
            None
        }
    }
}

/// Attempt to reconnect to League via the LCU API (best-effort, non-fatal).
/// Retries several times with delays to give the client time to process the game exit.
fn try_lcu_reconnect(league_path: &Path) {
    let lockfile = match read_lockfile(league_path) {
        Some(data) => data,
        None => {
            tracing::info!("No lockfile found, skipping LCU reconnect");
            return;
        }
    };

    let client = match reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to build HTTP client for LCU: {}", e);
            return;
        }
    };

    // Retry up to 5 times with increasing delays.
    // The League client needs time to process the game exit before it accepts reconnect.
    let retry_delays = [
        std::time::Duration::from_secs(3),
        std::time::Duration::from_secs(3),
        std::time::Duration::from_secs(5),
        std::time::Duration::from_secs(5),
        std::time::Duration::from_secs(5),
    ];

    for (attempt, delay) in retry_delays.iter().enumerate() {
        tracing::debug!(
            "LCU reconnect: waiting {}s before attempt {} of {}",
            delay.as_secs(),
            attempt + 1,
            retry_delays.len()
        );
        std::thread::sleep(*delay);

        if try_lcu_reconnect_once(&client, &lockfile) {
            return;
        }
    }

    tracing::info!("LCU reconnect: all attempts exhausted (client may not need reconnect)");
}

/// Single attempt to reconnect via LCU API. Returns true on success.
fn try_lcu_reconnect_once(client: &reqwest::blocking::Client, lockfile: &LockfileData) -> bool {
    let endpoints = [
        ("POST", "/lol-gameflow/v1/reconnect"),
        ("PUT", "/lol-gameflow/v1/reconnect"),
        ("POST", "/lol-login/v1/session/reconnect"),
    ];

    for (method, path) in &endpoints {
        let url = format!("https://127.0.0.1:{}{}", lockfile.port, path);
        tracing::debug!("Trying LCU reconnect: {} {}", method, url);

        let result = match *method {
            "POST" => client
                .post(&url)
                .basic_auth("riot", Some(&lockfile.password))
                .header("Content-Type", "application/json")
                .send(),
            "PUT" => client
                .put(&url)
                .basic_auth("riot", Some(&lockfile.password))
                .header("Content-Type", "application/json")
                .send(),
            _ => continue,
        };

        match result {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("LCU reconnect succeeded via {} {}", method, path);
                return true;
            }
            Ok(resp) => {
                tracing::debug!("LCU {} {} returned {}", method, path, resp.status());
            }
            Err(e) => {
                tracing::debug!("LCU {} {} failed: {}", method, path, e);
            }
        }
    }

    false
}

/// Kill the League of Legends process via taskkill.
fn kill_league_process() {
    tracing::info!("Killing League of Legends process");
    match Command::new("taskkill")
        .args(["/F", "/IM", "League of Legends.exe"])
        .spawn()
    {
        Ok(mut child) => {
            let timeout = std::time::Duration::from_secs(3);
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        if start.elapsed() > timeout {
                            tracing::warn!("taskkill timed out");
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(e) => {
                        tracing::warn!("Error waiting for taskkill: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to spawn taskkill: {}", e);
        }
    }
}
