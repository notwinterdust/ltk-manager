mod layers;
mod packing;
mod projects;

use crate::error::{AppError, AppResult};
use crate::state::Settings;
use chrono::{DateTime, Utc};
use ltk_mod_project::{ModProject, ModProjectAuthor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// Managed struct that encapsulates workshop operations.
///
/// Holds the `AppHandle` for consistency with `ModLibrary`.
/// Settings are passed per-call since they can change at runtime.
pub struct Workshop {
    #[allow(dead_code)]
    app_handle: AppHandle,
}

/// Tauri managed state wrapper for `Workshop`.
pub struct WorkshopState(pub Workshop);

impl Workshop {
    pub fn new(app_handle: &AppHandle) -> Self {
        Self {
            app_handle: app_handle.clone(),
        }
    }

    /// Resolve the workshop directory from settings.
    pub(crate) fn workshop_dir(&self, settings: &Settings) -> AppResult<PathBuf> {
        settings
            .workshop_path
            .clone()
            .ok_or(AppError::WorkshopNotConfigured)
    }
}

/// A workshop project displayed in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopProject {
    /// Absolute path to the project directory
    pub path: String,
    /// Project slug name (directory name)
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Semantic version string
    pub version: String,
    /// Project description
    pub description: String,
    /// List of authors
    pub authors: Vec<WorkshopAuthor>,
    /// Categorization tags
    pub tags: Vec<String>,
    /// Champion names this mod applies to
    pub champions: Vec<String>,
    /// Map identifiers this mod applies to
    pub maps: Vec<String>,
    /// Project layers
    pub layers: Vec<WorkshopLayer>,
    /// Path to thumbnail image if exists
    pub thumbnail_path: Option<String>,
    /// Last modification time
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopAuthor {
    pub name: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopLayer {
    pub name: String,
    pub priority: i32,
    pub description: Option<String>,
    #[serde(default)]
    pub string_overrides: HashMap<String, HashMap<String, String>>,
}

/// Arguments for creating a new project.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectArgs {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub authors: Vec<String>,
}

/// Arguments for saving project configuration changes.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectConfigArgs {
    pub project_path: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<WorkshopAuthor>,
    pub tags: Vec<String>,
    pub champions: Vec<String>,
    pub maps: Vec<String>,
}

/// Arguments for packing a project.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackProjectArgs {
    pub project_path: String,
    pub output_dir: Option<String>,
    pub format: PackFormat,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PackFormat {
    Modpkg,
    Fantome,
}

/// Result of a successful pack operation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackResult {
    pub output_path: String,
    pub file_name: String,
    pub format: String,
}

/// Validation result for a project.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// ============================================================================
// Helpers
// ============================================================================

pub(crate) fn find_config_file(project_dir: &Path) -> Option<PathBuf> {
    let json_path = project_dir.join("mod.config.json");
    if json_path.exists() {
        return Some(json_path);
    }

    let toml_path = project_dir.join("mod.config.toml");
    if toml_path.exists() {
        return Some(toml_path);
    }

    None
}

pub(crate) fn load_mod_project(config_path: &Path) -> AppResult<ModProject> {
    let contents = fs::read_to_string(config_path)?;

    let is_toml = config_path
        .extension()
        .map(|e| e == "toml")
        .unwrap_or(false);

    if is_toml {
        toml::from_str(&contents).map_err(|e| AppError::Other(e.to_string()))
    } else {
        serde_json::from_str(&contents).map_err(AppError::from)
    }
}

pub(crate) fn load_workshop_project(project_dir: &Path) -> AppResult<WorkshopProject> {
    let config_path = find_config_file(project_dir)
        .ok_or_else(|| AppError::ProjectNotFound(project_dir.display().to_string()))?;

    let mod_project = load_mod_project(&config_path)?;

    let metadata = fs::metadata(&config_path)?;
    let last_modified = metadata
        .modified()
        .map(DateTime::<Utc>::from)
        .unwrap_or_else(|_| Utc::now());

    let thumbnail_path = if project_dir.join("thumbnail.webp").exists() {
        Some(project_dir.join("thumbnail.webp").display().to_string())
    } else if project_dir.join("thumbnail.png").exists() {
        Some(project_dir.join("thumbnail.png").display().to_string())
    } else {
        None
    };

    let authors = mod_project
        .authors
        .iter()
        .map(|a| match a {
            ModProjectAuthor::Name(name) => WorkshopAuthor {
                name: name.clone(),
                role: None,
            },
            ModProjectAuthor::Role { name, role } => WorkshopAuthor {
                name: name.clone(),
                role: Some(role.clone()),
            },
        })
        .collect();

    let layers = mod_project
        .layers
        .iter()
        .map(|l| WorkshopLayer {
            name: l.name.clone(),
            priority: l.priority,
            description: l.description.clone(),
            string_overrides: l.string_overrides.clone(),
        })
        .collect();

    let tags = mod_project.tags.iter().map(|t| t.to_string()).collect();
    let champions = mod_project.champions.clone();
    let maps = mod_project.maps.iter().map(|m| m.to_string()).collect();

    Ok(WorkshopProject {
        path: project_dir.display().to_string(),
        name: mod_project.name,
        display_name: mod_project.display_name,
        version: mod_project.version,
        description: mod_project.description,
        authors,
        tags,
        champions,
        maps,
        layers,
        thumbnail_path,
        last_modified,
    })
}

pub(crate) fn is_valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}
