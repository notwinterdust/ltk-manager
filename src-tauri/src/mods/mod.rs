mod inspect;
mod library;
mod migration;
mod profiles;

pub use migration::*;

pub use inspect::{inspect_modpkg_file, ModpkgInfo};

use crate::error::{AppError, AppResult};
use crate::state::{get_app_data_dir, Settings};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use ts_rs::TS;
use uuid::Uuid;

/// Managed struct that encapsulates mod library operations.
///
/// Holds only the `AppHandle` (constant for app lifetime).
/// Settings are passed per-call since they can change at runtime.
#[derive(Clone)]
pub struct ModLibrary {
    app_handle: AppHandle,
}

/// Tauri managed state wrapper for `ModLibrary`.
pub struct ModLibraryState(pub ModLibrary);

impl ModLibrary {
    pub fn new(app_handle: &AppHandle) -> Self {
        Self {
            app_handle: app_handle.clone(),
        }
    }

    /// Expose the inner AppHandle (needed by overlay module for emitting events).
    pub fn app_handle(&self) -> &AppHandle {
        &self.app_handle
    }

    /// Resolve storage directory from settings snapshot.
    pub(crate) fn storage_dir(&self, settings: &Settings) -> AppResult<PathBuf> {
        settings
            .mod_storage_path
            .clone()
            .or_else(|| get_app_data_dir(&self.app_handle))
            .ok_or_else(|| AppError::Other("Failed to resolve mod storage directory".to_string()))
    }

    /// Read-only index access: load index, run closure.
    fn with_index<T>(
        &self,
        settings: &Settings,
        f: impl FnOnce(&Path, &LibraryIndex) -> AppResult<T>,
    ) -> AppResult<T> {
        let storage_dir = self.storage_dir(settings)?;
        let index = load_library_index(&storage_dir)?;
        f(&storage_dir, &index)
    }

    /// Mutate index: load, run closure, save, invalidate overlay.
    fn mutate_index<T>(
        &self,
        settings: &Settings,
        f: impl FnOnce(&Path, &mut LibraryIndex) -> AppResult<T>,
    ) -> AppResult<T> {
        let storage_dir = self.storage_dir(settings)?;
        let mut index = load_library_index(&storage_dir)?;
        let result = f(&storage_dir, &mut index)?;
        save_library_index(&storage_dir, &index)?;
        if let Err(e) = self.invalidate_overlay(settings) {
            tracing::warn!("Failed to invalidate overlay: {}", e);
        }
        Ok(result)
    }

    /// Delete the active profile's `overlay.json` to force the next build to rebuild.
    fn invalidate_overlay(&self, settings: &Settings) -> AppResult<()> {
        let storage_dir = self.storage_dir(settings)?;
        let index = load_library_index(&storage_dir)?;
        let active_profile = get_active_profile(&index)?;
        let overlay_json = storage_dir
            .join("profiles")
            .join(active_profile.slug.as_str())
            .join("overlay.json");
        if overlay_json.exists() {
            std::fs::remove_file(&overlay_json)?;
            tracing::info!("Invalidated overlay for profile {}", active_profile.slug);
        }
        Ok(())
    }
}

/// Slugified profile name used as the filesystem directory name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(transparent)]
pub struct ProfileSlug(pub String);

impl ProfileSlug {
    /// Create a slug from a profile name. Returns `None` if the name produces an empty slug.
    pub fn from_name(name: &str) -> Option<Self> {
        let s = slug::slugify(name);
        if s.is_empty() {
            None
        } else {
            Some(Self(s))
        }
    }

    /// Check whether this slug is unique among profiles in the index.
    pub fn is_unique_in(&self, index: &LibraryIndex, exclude_id: Option<&str>) -> bool {
        !index
            .profiles
            .iter()
            .any(|p| p.slug == *self && exclude_id.is_none_or(|id| p.id != id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProfileSlug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<String> for ProfileSlug {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A mod profile for organizing different mod configurations.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    /// Unique identifier (UUID)
    pub id: String,
    /// User-friendly name
    pub name: String,
    /// Slugified name used as the filesystem directory name
    #[serde(default)]
    pub slug: ProfileSlug,
    /// List of mod IDs enabled in this profile (maintains overlay priority order)
    pub enabled_mods: Vec<String>,
    /// Display order of all mods (enabled and disabled) in the UI
    pub mod_order: Vec<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last time this profile was used/switched to
    pub last_used: DateTime<Utc>,
}

/// A mod layer shown in the UI.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ModLayer {
    pub name: String,
    pub priority: i32,
    pub enabled: bool,
}

/// A mod entry shown in the UI Library.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InstalledMod {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Vec<String>,
    pub enabled: bool,
    pub installed_at: DateTime<Utc>,
    pub layers: Vec<ModLayer>,
    pub tags: Vec<String>,
    pub champions: Vec<String>,
    pub maps: Vec<String>,
    /// Directory where the mod is installed
    pub mod_dir: String,
}

/// Result of a bulk mod install operation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BulkInstallResult {
    pub installed: Vec<InstalledMod>,
    pub failed: Vec<BulkInstallError>,
}

/// Error info for a single file that failed during bulk install.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BulkInstallError {
    pub file_path: String,
    pub file_name: String,
    pub message: String,
}

/// Progress event emitted per-file during bulk mod install.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

/// Progress event emitted during cslol migration (both packaging and installing phases).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MigrationProgress {
    pub phase: MigrationPhase,
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum MigrationPhase {
    Packaging,
    Installing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LibraryIndex {
    pub(super) mods: Vec<LibraryModEntry>,
    pub(super) profiles: Vec<Profile>,
    pub(crate) active_profile_id: String,
}

impl Default for LibraryIndex {
    fn default() -> Self {
        let default_profile = Profile {
            id: Uuid::new_v4().to_string(),
            name: "Default".to_string(),
            slug: ProfileSlug::from("default".to_string()),
            enabled_mods: Vec::new(),
            mod_order: Vec::new(),
            created_at: Utc::now(),
            last_used: Utc::now(),
        };
        let active_profile_id = default_profile.id.clone();

        Self {
            mods: Vec::new(),
            profiles: vec![default_profile],
            active_profile_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum ModArchiveFormat {
    Modpkg,
    Fantome,
}

impl ModArchiveFormat {
    /// File extension for this format.
    pub(super) fn extension(self) -> &'static str {
        match self {
            ModArchiveFormat::Modpkg => "modpkg",
            ModArchiveFormat::Fantome => "fantome",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LibraryModEntry {
    pub(super) id: String,
    pub(super) installed_at: DateTime<Utc>,
    pub(super) format: ModArchiveFormat,
}

impl LibraryModEntry {
    /// Directory containing extracted metadata (mod.config.json, thumbnail, etc).
    pub(super) fn metadata_dir(&self, storage_dir: &Path) -> PathBuf {
        storage_dir.join("mods").join(&self.id)
    }

    /// Path to the stored mod archive file.
    pub(super) fn archive_path(&self, storage_dir: &Path) -> PathBuf {
        storage_dir
            .join("archives")
            .join(format!("{}.{}", self.id, self.format.extension()))
    }
}

pub(super) fn library_index_path(storage_dir: &Path) -> PathBuf {
    storage_dir.join("library.json")
}

/// Load the library index from disk.
/// Creates the index file if it doesn't exist.
pub(super) fn load_library_index(storage_dir: &Path) -> AppResult<LibraryIndex> {
    fs::create_dir_all(storage_dir)?;

    let path = library_index_path(storage_dir);
    if !path.exists() {
        return Ok(LibraryIndex::default());
    }

    serde_json::from_str(&fs::read_to_string(path)?).map_err(AppError::from)
}

pub(super) fn save_library_index(storage_dir: &Path, index: &LibraryIndex) -> AppResult<()> {
    fs::create_dir_all(storage_dir)?;
    let path = library_index_path(storage_dir);
    let contents = serde_json::to_string_pretty(index)?;
    fs::write(path, contents)?;
    Ok(())
}

pub(super) fn get_active_profile(index: &LibraryIndex) -> AppResult<&Profile> {
    index
        .profiles
        .iter()
        .find(|p| p.id == index.active_profile_id)
        .ok_or_else(|| AppError::Other("Active profile not found".to_string()))
}

pub(super) fn get_profile_by_id<'a>(
    index: &'a LibraryIndex,
    profile_id: &str,
) -> AppResult<&'a Profile> {
    index
        .profiles
        .iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| AppError::Other(format!("Profile {} not found", profile_id)))
}

pub(super) fn resolve_profile_dirs(
    storage_dir: &Path,
    profile_slug: &ProfileSlug,
) -> (PathBuf, PathBuf) {
    let profile_dir = storage_dir.join("profiles").join(profile_slug.as_str());
    let overlay_dir = profile_dir.join("overlay");
    let cache_dir = profile_dir.join("cache");
    (overlay_dir, cache_dir)
}
