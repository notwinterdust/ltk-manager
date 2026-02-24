use crate::error::{AppError, AppResult};
use crate::state::Settings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use super::{BulkInstallResult, MigrationPhase, MigrationProgress, ModLibrary};

/// Metadata for a discovered cslol-manager mod, shown in the UI selection step.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CslolModInfo {
    pub folder_name: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
}

/// Scan a cslol-manager directory for installed mods.
///
/// Expects `dir` to contain an `installed/` subdirectory where each child folder
/// has the standard fantome layout (`META/info.json`, `WAD/`, etc.).
pub fn scan_cslol_directory(dir: &Path) -> AppResult<Vec<CslolModInfo>> {
    let installed_dir = dir.join("installed");
    if !installed_dir.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "No 'installed' directory found in {}",
            dir.display()
        )));
    }

    let mut mods = Vec::new();

    for entry in fs::read_dir(&installed_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let folder_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let info_path = path.join("META").join("info.json");
        if !info_path.exists() {
            tracing::warn!(
                "Skipping cslol mod '{}': missing META/info.json",
                folder_name
            );
            continue;
        }

        match read_cslol_info(&info_path) {
            Ok(info) => {
                mods.push(CslolModInfo {
                    folder_name,
                    name: info.name,
                    author: info.author,
                    version: info.version,
                    description: info.description,
                });
            }
            Err(e) => {
                tracing::warn!("Skipping cslol mod '{}': {}", folder_name, e);
            }
        }
    }

    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(mods)
}

/// Import selected cslol-manager mods by creating temporary `.fantome` ZIPs
/// and piping them through the existing install pipeline.
///
/// Emits `"migration-progress"` events during both the packaging and installing phases.
pub fn import_cslol_mods(
    library: &ModLibrary,
    settings: &Settings,
    app_handle: &AppHandle,
    cslol_dir: &Path,
    folders: &[String],
) -> AppResult<BulkInstallResult> {
    let installed_dir = cslol_dir.join("installed");
    let temp_dir = std::env::temp_dir().join("ltk-migration");
    fs::create_dir_all(&temp_dir)?;

    let total = folders.len();
    let mut temp_paths: Vec<String> = Vec::new();

    for (i, folder) in folders.iter().enumerate() {
        let _ = app_handle.emit(
            "migration-progress",
            MigrationProgress {
                phase: MigrationPhase::Packaging,
                current: i + 1,
                total,
                current_file: folder.clone(),
            },
        );

        let mod_dir = installed_dir.join(folder);
        if !mod_dir.is_dir() {
            tracing::warn!("Skipping missing folder: {}", folder);
            continue;
        }

        let output_path = temp_dir.join(format!("{}.fantome", folder));
        match create_fantome_zip(&mod_dir, &output_path) {
            Ok(()) => {
                temp_paths.push(output_path.display().to_string());
            }
            Err(e) => {
                tracing::warn!("Failed to create fantome zip for '{}': {}", folder, e);
            }
        }
    }

    let result = library.install_mods_from_packages(settings, &temp_paths);

    for path in &temp_paths {
        let _ = fs::remove_file(path);
    }
    let _ = fs::remove_dir(&temp_dir);

    result
}

/// Create a `.fantome` ZIP archive from a cslol mod directory.
fn create_fantome_zip(mod_dir: &Path, output_path: &Path) -> AppResult<()> {
    let file = fs::File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut zip, mod_dir, mod_dir, options)?;
    zip.finish().map_err(AppError::ZipError)?;

    Ok(())
}

/// Recursively add directory contents to a ZIP archive.
fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    base_dir: &Path,
    current_dir: &Path,
    options: zip::write::SimpleFileOptions,
) -> AppResult<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        let relative = path
            .strip_prefix(base_dir)
            .map_err(|e| AppError::Other(e.to_string()))?;
        let name = relative.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            zip.add_directory(format!("{}/", name), options)
                .map_err(AppError::ZipError)?;
            add_dir_to_zip(zip, base_dir, &path, options)?;
        } else {
            zip.start_file(&name, options).map_err(AppError::ZipError)?;
            let mut f = fs::File::open(&path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }

    Ok(())
}

/// Read and parse a cslol-manager `META/info.json` file.
fn read_cslol_info(path: &Path) -> AppResult<ltk_fantome::FantomeInfo> {
    let content = fs::read_to_string(path)?;
    let content = content.trim_start_matches('\u{feff}').trim();

    serde_json::from_str(content).map_err(AppError::Serialization)
}
