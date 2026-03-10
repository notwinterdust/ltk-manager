use crate::error::{AppError, AppResult};
use crate::state::Settings;
use chrono::Utc;
use ltk_mod_project::{ModMap, ModProject, ModProjectLayer, ModTag};
use ltk_modpkg::Modpkg;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::{
    BulkInstallError, BulkInstallResult, InstallProgress, InstalledMod, LibraryIndex,
    LibraryModEntry, ModArchiveFormat, ModLayer, ModLibrary,
};
use tauri::Emitter;

impl ModLibrary {
    pub fn get_installed_mods(&self, settings: &Settings) -> AppResult<Vec<InstalledMod>> {
        self.with_index(settings, |storage_dir, index| {
            let active_profile_id = index.active_profile_id.clone();
            let active_profile = index
                .profiles
                .iter()
                .find(|p| p.id == active_profile_id)
                .ok_or_else(|| AppError::Other("Active profile not found".to_string()))?;

            let enabled_set: std::collections::HashSet<&str> = active_profile
                .enabled_mods
                .iter()
                .map(|s| s.as_str())
                .collect();

            let mut result = Vec::new();
            for mod_id in &active_profile.mod_order {
                let Some(entry) = index.mods.iter().find(|m| &m.id == mod_id) else {
                    continue;
                };
                let enabled = enabled_set.contains(mod_id.as_str());
                match read_installed_mod(entry, enabled, storage_dir) {
                    Ok(m) => result.push(m),
                    Err(e) => {
                        tracing::warn!("Skipping broken mod entry {}: {}", entry.id, e);
                    }
                }
            }

            Ok(result)
        })
    }

    /// Reorder all mods for the active profile.
    /// The provided `mod_ids` must exactly match the active profile's mod order.
    /// The `enabled_mods` order is derived from the new display order.
    pub fn reorder_mods(&self, settings: &Settings, mod_ids: Vec<String>) -> AppResult<()> {
        self.mutate_index(settings, |_storage_dir, index| {
            let active_profile_id = index.active_profile_id.clone();
            let profile = index
                .profiles
                .iter_mut()
                .find(|p| p.id == active_profile_id)
                .ok_or_else(|| AppError::Other("Active profile not found".to_string()))?;

            // Validate that the provided IDs exactly match the profile's mod order
            let mut expected_sorted: Vec<&str> =
                profile.mod_order.iter().map(|s| s.as_str()).collect();
            expected_sorted.sort();
            let mut new_sorted: Vec<&str> = mod_ids.iter().map(|s| s.as_str()).collect();
            new_sorted.sort();

            if expected_sorted != new_sorted {
                return Err(AppError::ValidationFailed(
                    "Provided mod IDs do not match the profile's mod order".to_string(),
                ));
            }

            // Derive enabled_mods order from new display order
            let enabled_set: std::collections::HashSet<&str> =
                profile.enabled_mods.iter().map(|s| s.as_str()).collect();
            profile.enabled_mods = mod_ids
                .iter()
                .filter(|id| enabled_set.contains(id.as_str()))
                .cloned()
                .collect();

            profile.mod_order = mod_ids;

            Ok(())
        })
    }

    pub fn install_mod_from_package(
        &self,
        settings: &Settings,
        file_path: &str,
    ) -> AppResult<InstalledMod> {
        self.mutate_index(settings, |storage_dir, index| {
            let (_entry, installed_mod) =
                install_single_mod_to_index(storage_dir, index, file_path)?;
            Ok(installed_mod)
        })
    }

    /// Install multiple mods in a single batch operation.
    ///
    /// Acquires the index lock once, installs each mod, saves once, and invalidates
    /// the overlay once. Emits `"install-progress"` events per file.
    pub fn install_mods_from_packages(
        &self,
        settings: &Settings,
        file_paths: &[String],
    ) -> AppResult<BulkInstallResult> {
        if file_paths.is_empty() {
            return Ok(BulkInstallResult {
                installed: Vec::new(),
                failed: Vec::new(),
            });
        }

        let app_handle = self.app_handle().clone();
        let file_paths = file_paths.to_vec();

        self.mutate_index(settings, |storage_dir, index| {
            let total = file_paths.len();
            let mut installed = Vec::new();
            let mut failed = Vec::new();

            for (i, file_path) in file_paths.iter().enumerate() {
                let file_name = Path::new(file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(file_path)
                    .to_string();

                let _ = app_handle.emit(
                    "install-progress",
                    InstallProgress {
                        current: i + 1,
                        total,
                        current_file: file_name.clone(),
                    },
                );

                match install_single_mod_to_index(storage_dir, index, file_path) {
                    Ok((_entry, mod_info)) => {
                        installed.push(mod_info);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to install {}: {}", file_path, e);
                        failed.push(BulkInstallError {
                            file_path: file_path.clone(),
                            file_name,
                            message: e.to_string(),
                        });
                    }
                }
            }

            Ok(BulkInstallResult { installed, failed })
        })
    }

    pub fn toggle_mod_enabled(
        &self,
        settings: &Settings,
        mod_id: &str,
        enabled: bool,
    ) -> AppResult<()> {
        self.mutate_index(settings, |_storage_dir, index| {
            // Validate mod exists
            if !index.mods.iter().any(|m| m.id == mod_id) {
                return Err(AppError::ModNotFound(mod_id.to_string()));
            }

            // Update active profile's enabled mods
            let active_profile_id = index.active_profile_id.clone();
            let profile = index
                .profiles
                .iter_mut()
                .find(|p| p.id == active_profile_id)
                .ok_or_else(|| AppError::Other("Active profile not found".to_string()))?;

            if enabled {
                if !profile.enabled_mods.contains(&mod_id.to_string()) {
                    // Insert at position preserving relative order from mod_order
                    let insert_pos = if let Some(order_pos) =
                        profile.mod_order.iter().position(|id| id == mod_id)
                    {
                        profile
                            .enabled_mods
                            .iter()
                            .position(|id| {
                                profile
                                    .mod_order
                                    .iter()
                                    .position(|oid| oid == id)
                                    .is_none_or(|p| p > order_pos)
                            })
                            .unwrap_or(profile.enabled_mods.len())
                    } else {
                        0
                    };
                    profile.enabled_mods.insert(insert_pos, mod_id.to_string());
                }
            } else {
                profile.enabled_mods.retain(|id| id != mod_id);
            }

            Ok(())
        })
    }

    pub fn uninstall_mod_by_id(&self, settings: &Settings, mod_id: &str) -> AppResult<()> {
        self.mutate_index(settings, |storage_dir, index| {
            let Some(pos) = index.mods.iter().position(|m| m.id == mod_id) else {
                return Err(AppError::ModNotFound(mod_id.to_string()));
            };

            let entry = index.mods.remove(pos);

            // Remove from all profiles' mod_order and enabled_mods
            for profile in &mut index.profiles {
                profile.mod_order.retain(|id| id != mod_id);
                profile.enabled_mods.retain(|id| id != mod_id);
            }

            // Delete metadata directory
            let metadata_dir = entry.metadata_dir(storage_dir);
            if metadata_dir.exists() {
                fs::remove_dir_all(&metadata_dir)?;
            }

            // Delete archive file
            let archive_path = entry.archive_path(storage_dir);
            if archive_path.exists() {
                fs::remove_file(&archive_path)?;
                tracing::info!("Deleted mod archive at {}", archive_path.display());
            }

            Ok(())
        })
    }

    /// Get a mod's cached thumbnail path, extracting from the archive on first access.
    /// Returns `None` if the mod has no thumbnail.
    pub fn get_mod_thumbnail_path(
        &self,
        settings: &Settings,
        mod_id: &str,
    ) -> AppResult<Option<String>> {
        self.with_index(settings, |storage_dir, index| {
            let entry = index
                .mods
                .iter()
                .find(|m| m.id == mod_id)
                .ok_or_else(|| AppError::ModNotFound(mod_id.to_string()))?;

            let metadata_dir = entry.metadata_dir(storage_dir);

            // Check for already-cached thumbnail
            for filename in ["thumbnail.webp", "thumbnail.png"] {
                let cached = metadata_dir.join(filename);
                if cached.exists() {
                    return Ok(Some(cached.display().to_string()));
                }
            }

            // Lazy migration: extract from archive and cache
            let archive_path = entry.archive_path(storage_dir);
            if !archive_path.exists() {
                return Err(AppError::InvalidPath(archive_path.display().to_string()));
            }

            let cached_path = match entry.format {
                ModArchiveFormat::Fantome => {
                    extract_fantome_thumbnail(&archive_path, &metadata_dir)?
                }
                ModArchiveFormat::Modpkg => extract_modpkg_thumbnail(&archive_path, &metadata_dir)?,
            };

            Ok(cached_path.map(|p| p.display().to_string()))
        })
    }

    pub fn get_enabled_mods_for_overlay(
        &self,
        settings: &Settings,
    ) -> AppResult<(super::ProfileSlug, Vec<ltk_overlay::EnabledMod>)> {
        self.with_index(settings, |storage_dir, index| {
            let active_profile_id = index.active_profile_id.clone();
            let active_profile = index
                .profiles
                .iter()
                .find(|p| p.id == active_profile_id)
                .ok_or_else(|| AppError::Other("Active profile not found".to_string()))?;

            let mut enabled_mods = Vec::new();

            // Process mods in the order they appear in enabled_mods list (maintains priority)
            for mod_id in &active_profile.enabled_mods {
                let Some(entry) = index.mods.iter().find(|m| &m.id == mod_id) else {
                    tracing::warn!("Mod {} in profile but not found in library", mod_id);
                    continue;
                };

                let archive_path = entry.archive_path(storage_dir);

                if !archive_path.exists() {
                    tracing::warn!(
                        "Archive not found for mod {}: {}",
                        entry.id,
                        archive_path.display()
                    );
                    continue;
                }

                tracing::info!(
                    "Creating content provider for mod {} from archive {}",
                    entry.id,
                    archive_path.display()
                );

                let content: Box<dyn ltk_overlay::ModContentProvider> = match entry.format {
                    ModArchiveFormat::Fantome => {
                        let file = std::fs::File::open(&archive_path)?;
                        let provider = crate::overlay::fantome_content::FantomeContent::new(file)
                            .map_err(|e| {
                            AppError::Other(format!("Failed to open fantome archive: {}", e))
                        })?;
                        Box::new(provider)
                    }
                    ModArchiveFormat::Modpkg => {
                        let file = std::fs::File::open(&archive_path)?;
                        let modpkg = ltk_modpkg::Modpkg::mount_from_reader(file)
                            .map_err(|e| AppError::Modpkg(e.to_string()))?;
                        Box::new(crate::overlay::modpkg_content::ModpkgContent::new(modpkg))
                    }
                };

                enabled_mods.push(ltk_overlay::EnabledMod {
                    id: entry.id.clone(),
                    content,
                });
            }

            Ok((active_profile.slug.clone(), enabled_mods))
        })
    }
}

/// Core install logic for a single mod file.
///
/// Copies the archive, extracts metadata, and adds the mod to the index.
/// Does NOT load/save the index or invalidate the overlay.
pub(super) fn install_single_mod_to_index(
    storage_dir: &Path,
    index: &mut LibraryIndex,
    file_path: &str,
) -> AppResult<(LibraryModEntry, InstalledMod)> {
    let file_path = PathBuf::from(file_path);
    if !file_path.exists() {
        return Err(AppError::InvalidPath(file_path.display().to_string()));
    }

    let archives_dir = storage_dir.join("archives");
    let metadata_dir = storage_dir.join("mods");
    fs::create_dir_all(&archives_dir)?;
    fs::create_dir_all(&metadata_dir)?;

    let id = Uuid::new_v4().to_string();

    let format = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(ModArchiveFormat::from_extension)
        .unwrap_or(ModArchiveFormat::Modpkg);

    let installed_at = Utc::now();

    // Copy archive to archives directory
    let archive_filename = format!("{}.{}", id, format.extension());
    let archive_path = archives_dir.join(&archive_filename);
    fs::copy(&file_path, &archive_path)?;
    tracing::info!(
        "Copied mod archive from {} to {}",
        file_path.display(),
        archive_path.display()
    );

    // Extract only metadata to metadata directory
    let mod_metadata_dir = metadata_dir.join(&id);
    fs::create_dir_all(&mod_metadata_dir)?;

    match format {
        ModArchiveFormat::Fantome => extract_fantome_metadata(&archive_path, &mod_metadata_dir)?,
        ModArchiveFormat::Modpkg => extract_modpkg_metadata(&archive_path, &mod_metadata_dir)?,
    }

    let entry = LibraryModEntry {
        id: id.clone(),
        installed_at,
        format,
    };
    index.mods.push(entry.clone());

    // Enable in active profile and add to display order
    let active_profile_id = index.active_profile_id.clone();
    if let Some(profile) = index
        .profiles
        .iter_mut()
        .find(|p| p.id == active_profile_id)
    {
        profile.enabled_mods.insert(0, id.clone());
        profile.mod_order.insert(0, id.clone());
    }

    let installed_mod = read_installed_mod(&entry, true, storage_dir)?;
    Ok((entry, installed_mod))
}

fn read_installed_mod(
    entry: &LibraryModEntry,
    enabled: bool,
    storage_dir: &Path,
) -> AppResult<InstalledMod> {
    let mod_dir = entry.metadata_dir(storage_dir);
    let project = load_mod_project(&mod_dir)?;
    let authors = project
        .authors
        .iter()
        .map(|a| match a {
            ltk_mod_project::ModProjectAuthor::Name(name) => name.clone(),
            ltk_mod_project::ModProjectAuthor::Role { name, role: _ } => name.clone(),
        })
        .collect::<Vec<_>>();

    let layers = project
        .layers
        .iter()
        .map(|l| ModLayer {
            name: l.name.clone(),
            priority: l.priority,
            enabled: true,
        })
        .collect::<Vec<_>>();

    Ok(InstalledMod {
        id: entry.id.clone(),
        name: project.name,
        display_name: project.display_name,
        version: project.version,
        description: Some(project.description).filter(|s| !s.is_empty()),
        authors,
        enabled,
        installed_at: entry.installed_at,
        layers,
        tags: project.tags.iter().map(|t| t.to_string()).collect(),
        champions: project.champions.clone(),
        maps: project.maps.iter().map(|m| m.to_string()).collect(),
        mod_dir: mod_dir.display().to_string(),
    })
}

fn load_mod_project(mod_dir: &Path) -> AppResult<ModProject> {
    let config_path = mod_dir.join("mod.config.json");
    let contents = fs::read_to_string(&config_path).map_err(|e| {
        AppError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to read {}: {}", config_path.display(), e),
        ))
    })?;
    serde_json::from_str(&contents).map_err(AppError::from)
}

pub(super) fn extract_fantome_metadata(file_path: &Path, metadata_dir: &Path) -> AppResult<()> {
    use std::io::Read;
    use zip::ZipArchive;

    let file = std::fs::File::open(file_path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| AppError::Other(format!("Failed to open fantome archive: {}", e)))?;

    // Read metadata from info.json
    let mut info_content = String::new();
    let mut found_metadata = false;

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| AppError::Other(format!("Failed to read archive entry: {}", e)))?;
        let name = file.name().to_lowercase();

        if name == "meta/info.json" {
            drop(file);
            let mut info_file = archive
                .by_index(i)
                .map_err(|e| AppError::Other(format!("Failed to read info.json: {}", e)))?;
            info_file
                .read_to_string(&mut info_content)
                .map_err(|e| AppError::Other(format!("Failed to read info.json content: {}", e)))?;
            found_metadata = true;
            break;
        }
    }

    if !found_metadata {
        return Err(AppError::Other(
            "Missing META/info.json in fantome archive".to_string(),
        ));
    }

    // Parse metadata
    let info_content = info_content.trim_start_matches('\u{feff}').trim();
    let info: ltk_fantome::FantomeInfo = serde_json::from_str(info_content)
        .map_err(|e| AppError::Other(format!("Failed to parse info.json: {}", e)))?;

    // Build layers from Fantome info, preserving string overrides
    let layers = if info.layers.is_empty() {
        ltk_mod_project::default_layers()
    } else {
        let mut layers: Vec<ltk_mod_project::ModProjectLayer> = info
            .layers
            .into_values()
            .map(|layer_info| ltk_mod_project::ModProjectLayer {
                name: layer_info.name,
                priority: layer_info.priority,
                description: None,
                string_overrides: layer_info.string_overrides,
            })
            .collect();
        // Ensure base layer exists
        if !layers.iter().any(|l| l.name == "base") {
            layers.insert(0, ltk_mod_project::ModProjectLayer::base());
        }
        layers.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.name.cmp(&b.name))
        });
        layers
    };

    // Create mod.config.json from metadata
    let project = ModProject {
        name: slug::slugify(&info.name),
        display_name: info.name,
        version: info.version,
        description: info.description,
        authors: vec![ltk_mod_project::ModProjectAuthor::Name(info.author)],
        license: None,
        tags: info.tags.into_iter().map(ModTag::from).collect(),
        champions: info.champions,
        maps: info.maps.into_iter().map(ModMap::from).collect(),
        transformers: Vec::new(),
        layers,
        thumbnail: None,
    };

    let config_path = metadata_dir.join("mod.config.json");
    fs::write(config_path, serde_json::to_string_pretty(&project)?)?;

    // Extract README and thumbnail if present
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::Other(format!("Failed to read archive entry: {}", e)))?;
        let name = file.name().to_string();
        let name_lower = name.to_lowercase();

        if name.eq_ignore_ascii_case("META/readme.md") || name.eq_ignore_ascii_case("readme.md") {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            fs::write(metadata_dir.join("README.md"), contents)?;
        } else if name_lower == "meta/image.png" {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            let _ = fs::write(metadata_dir.join("thumbnail.png"), &buffer);
        }
    }

    tracing::info!("Extracted fantome metadata to {}", metadata_dir.display());
    Ok(())
}

pub(super) fn extract_modpkg_metadata(file_path: &Path, metadata_dir: &Path) -> AppResult<()> {
    let file = std::fs::File::open(file_path)?;
    let mut modpkg =
        Modpkg::mount_from_reader(file).map_err(|e| AppError::Modpkg(e.to_string()))?;

    // Build a mod project config from metadata/header layers (no content extraction).
    let metadata = modpkg
        .load_metadata()
        .map_err(|e| AppError::Modpkg(e.to_string()))?;

    // Use header layers as source of truth, preserving string overrides from metadata.
    let mut layers: Vec<ModProjectLayer> = modpkg
        .layers
        .values()
        .map(|l| {
            let meta_layer = metadata.layers.iter().find(|ml| ml.name == l.name);
            ModProjectLayer {
                name: l.name.clone(),
                priority: l.priority,
                description: meta_layer.and_then(|ml| ml.description.clone()),
                string_overrides: meta_layer
                    .map(|ml| ml.string_overrides.clone())
                    .unwrap_or_default(),
            }
        })
        .collect();
    layers.sort_by(|a, b| a.priority.cmp(&b.priority).then(a.name.cmp(&b.name)));

    // Ensure base exists.
    if !layers.iter().any(|l| l.name == "base") {
        layers.insert(0, ModProjectLayer::base());
    }

    let project = ModProject {
        name: metadata.name,
        display_name: metadata.display_name,
        version: metadata.version.to_string(),
        description: metadata.description.unwrap_or_default(),
        authors: metadata
            .authors
            .into_iter()
            .map(|a| ltk_mod_project::ModProjectAuthor::Name(a.name))
            .collect(),
        license: None,
        tags: metadata.tags.into_iter().map(ModTag::from).collect(),
        champions: metadata.champions,
        maps: metadata.maps.into_iter().map(ModMap::from).collect(),
        transformers: Vec::new(),
        layers,
        thumbnail: None,
    };

    let config_path = metadata_dir.join("mod.config.json");
    fs::write(config_path, serde_json::to_string_pretty(&project)?)?;

    if let Ok(readme_bytes) = modpkg.load_readme() {
        let _ = fs::write(metadata_dir.join("README.md"), readme_bytes);
    }

    if let Ok(thumbnail_bytes) = modpkg.load_thumbnail() {
        let _ = fs::write(metadata_dir.join("thumbnail.webp"), thumbnail_bytes);
    }

    tracing::info!("Extracted modpkg metadata to {}", metadata_dir.display());

    Ok(())
}

/// Extract thumbnail from a fantome archive and save to the metadata directory.
/// Returns the path to the saved file, or `None` if the archive has no thumbnail.
fn extract_fantome_thumbnail(
    archive_path: &Path,
    metadata_dir: &Path,
) -> AppResult<Option<PathBuf>> {
    use std::io::Read;
    use zip::ZipArchive;

    let file = std::fs::File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| AppError::Other(format!("Failed to open fantome archive: {}", e)))?;

    for i in 0..archive.len() {
        let name = archive
            .by_index(i)
            .map_err(|e| AppError::Other(format!("Failed to read archive entry: {}", e)))?
            .name()
            .to_lowercase();

        if name == "meta/image.png" {
            let mut file = archive
                .by_index(i)
                .map_err(|e| AppError::Other(format!("Failed to read thumbnail: {}", e)))?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            let dest = metadata_dir.join("thumbnail.png");
            fs::write(&dest, &buffer)?;
            return Ok(Some(dest));
        }
    }

    Ok(None)
}

/// Extract thumbnail from a modpkg archive and save to the metadata directory.
/// Returns the path to the saved file, or `None` if the archive has no thumbnail.
fn extract_modpkg_thumbnail(
    archive_path: &Path,
    metadata_dir: &Path,
) -> AppResult<Option<PathBuf>> {
    let file = std::fs::File::open(archive_path)?;
    let mut modpkg =
        Modpkg::mount_from_reader(file).map_err(|e| AppError::Modpkg(e.to_string()))?;

    match modpkg.load_thumbnail() {
        Ok(thumbnail_bytes) => {
            let dest = metadata_dir.join("thumbnail.webp");
            fs::write(&dest, &thumbnail_bytes)?;
            Ok(Some(dest))
        }
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use std::collections::HashMap;
    use std::io::Write;

    fn make_test_mod_config_json() -> String {
        serde_json::to_string_pretty(&ltk_mod_project::ModProject {
            name: "test-mod".to_string(),
            display_name: "Test Mod".to_string(),
            version: "1.0.0".to_string(),
            description: "A test mod".to_string(),
            authors: vec![ltk_mod_project::ModProjectAuthor::Name(
                "Author".to_string(),
            )],
            license: None,
            tags: Vec::new(),
            champions: vec!["Aatrox".to_string()],
            maps: Vec::new(),
            transformers: Vec::new(),
            layers: ltk_mod_project::default_layers(),
            thumbnail: None,
        })
        .unwrap()
    }

    fn make_test_fantome_zip(dir: &Path, include_thumbnail: bool, include_readme: bool) -> PathBuf {
        let info = ltk_fantome::FantomeInfo {
            name: "Test Mod".to_string(),
            author: "Author".to_string(),
            version: "1.0.0".to_string(),
            description: "Description".to_string(),
            tags: Vec::new(),
            champions: Vec::new(),
            maps: Vec::new(),
            layers: HashMap::new(),
        };

        let zip_path = dir.join("test.fantome");
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        zip.start_file("META/info.json", options).unwrap();
        zip.write_all(serde_json::to_string_pretty(&info).unwrap().as_bytes())
            .unwrap();

        if include_thumbnail {
            zip.start_file("META/image.png", options).unwrap();
            zip.write_all(b"fake png data").unwrap();
        }

        if include_readme {
            zip.start_file("META/readme.md", options).unwrap();
            zip.write_all(b"# Test Mod\nReadme content").unwrap();
        }

        zip.finish().unwrap();
        zip_path
    }

    #[test]
    fn load_mod_project_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("mod.config.json"),
            make_test_mod_config_json(),
        )
        .unwrap();
        let project = load_mod_project(dir.path()).unwrap();
        assert_eq!(project.name, "test-mod");
        assert_eq!(project.version, "1.0.0");
        assert_eq!(project.display_name, "Test Mod");
    }

    #[test]
    fn load_mod_project_invalid_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("mod.config.json"), "not valid json").unwrap();
        assert!(load_mod_project(dir.path()).is_err());
    }

    #[test]
    fn load_mod_project_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_mod_project(dir.path()).is_err());
    }

    #[test]
    fn read_installed_mod_populates_all_fields() {
        let storage = tempfile::tempdir().unwrap();
        let id = "test-id";
        let mods_dir = storage.path().join("mods").join(id);
        fs::create_dir_all(&mods_dir).unwrap();
        fs::write(
            mods_dir.join("mod.config.json"),
            make_test_mod_config_json(),
        )
        .unwrap();

        let entry = LibraryModEntry {
            id: id.to_string(),
            installed_at: Utc::now(),
            format: ModArchiveFormat::Fantome,
        };

        let result = read_installed_mod(&entry, true, storage.path()).unwrap();
        assert_eq!(result.id, id);
        assert_eq!(result.name, "test-mod");
        assert_eq!(result.display_name, "Test Mod");
        assert_eq!(result.version, "1.0.0");
        assert_eq!(result.description.as_deref(), Some("A test mod"));
        assert_eq!(result.authors, vec!["Author"]);
        assert!(result.enabled);
        assert!(!result.layers.is_empty());
        assert_eq!(result.champions, vec!["Aatrox"]);
    }

    #[test]
    fn read_installed_mod_empty_description_becomes_none() {
        let storage = tempfile::tempdir().unwrap();
        let id = "test-id-2";
        let mods_dir = storage.path().join("mods").join(id);
        fs::create_dir_all(&mods_dir).unwrap();

        let config = serde_json::to_string_pretty(&ltk_mod_project::ModProject {
            name: "test-mod".to_string(),
            display_name: "Test Mod".to_string(),
            version: "1.0.0".to_string(),
            description: "".to_string(),
            authors: Vec::new(),
            license: None,
            tags: Vec::new(),
            champions: Vec::new(),
            maps: Vec::new(),
            transformers: Vec::new(),
            layers: ltk_mod_project::default_layers(),
            thumbnail: None,
        })
        .unwrap();
        fs::write(mods_dir.join("mod.config.json"), config).unwrap();

        let entry = LibraryModEntry {
            id: id.to_string(),
            installed_at: Utc::now(),
            format: ModArchiveFormat::Fantome,
        };

        let result = read_installed_mod(&entry, false, storage.path()).unwrap();
        assert!(result.description.is_none());
        assert!(!result.enabled);
    }

    #[test]
    fn read_installed_mod_missing_config_returns_error() {
        let storage = tempfile::tempdir().unwrap();
        let entry = LibraryModEntry {
            id: "nonexistent".to_string(),
            installed_at: Utc::now(),
            format: ModArchiveFormat::Fantome,
        };
        assert!(read_installed_mod(&entry, true, storage.path()).is_err());
    }

    #[test]
    fn extract_fantome_thumbnail_with_image() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = make_test_fantome_zip(dir.path(), true, false);
        let metadata_dir = dir.path().join("metadata");
        fs::create_dir_all(&metadata_dir).unwrap();

        let result = extract_fantome_thumbnail(&archive_path, &metadata_dir).unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().exists());
    }

    #[test]
    fn extract_fantome_thumbnail_without_image() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = make_test_fantome_zip(dir.path(), false, false);
        let metadata_dir = dir.path().join("metadata");
        fs::create_dir_all(&metadata_dir).unwrap();

        let result = extract_fantome_thumbnail(&archive_path, &metadata_dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn extract_fantome_metadata_valid() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = make_test_fantome_zip(dir.path(), false, false);
        let metadata_dir = dir.path().join("metadata");
        fs::create_dir_all(&metadata_dir).unwrap();

        extract_fantome_metadata(&archive_path, &metadata_dir).unwrap();

        let config_path = metadata_dir.join("mod.config.json");
        assert!(config_path.exists());
        let project = load_mod_project(&metadata_dir).unwrap();
        assert_eq!(project.display_name, "Test Mod");
    }

    #[test]
    fn extract_fantome_metadata_missing_info_json() {
        let dir = tempfile::tempdir().unwrap();

        let zip_path = dir.path().join("empty.fantome");
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("WAD/test.wad.client/file.bin", options)
            .unwrap();
        zip.write_all(b"data").unwrap();
        zip.finish().unwrap();

        let metadata_dir = dir.path().join("metadata");
        fs::create_dir_all(&metadata_dir).unwrap();

        assert!(extract_fantome_metadata(&zip_path, &metadata_dir).is_err());
    }

    #[test]
    fn extract_fantome_metadata_with_bom() {
        let dir = tempfile::tempdir().unwrap();

        let info_json = format!(
            "\u{feff}{}",
            serde_json::to_string(&ltk_fantome::FantomeInfo {
                name: "BOM Mod".to_string(),
                author: "Author".to_string(),
                version: "2.0.0".to_string(),
                description: "Has BOM".to_string(),
                tags: Vec::new(),
                champions: Vec::new(),
                maps: Vec::new(),
                layers: HashMap::new(),
            })
            .unwrap()
        );

        let zip_path = dir.path().join("bom.fantome");
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("META/info.json", options).unwrap();
        zip.write_all(info_json.as_bytes()).unwrap();
        zip.finish().unwrap();

        let metadata_dir = dir.path().join("metadata");
        fs::create_dir_all(&metadata_dir).unwrap();

        extract_fantome_metadata(&zip_path, &metadata_dir).unwrap();
        let project = load_mod_project(&metadata_dir).unwrap();
        assert_eq!(project.display_name, "BOM Mod");
    }

    #[test]
    fn extract_fantome_metadata_extracts_readme() {
        let dir = tempfile::tempdir().unwrap();
        let archive_path = make_test_fantome_zip(dir.path(), false, true);
        let metadata_dir = dir.path().join("metadata");
        fs::create_dir_all(&metadata_dir).unwrap();

        extract_fantome_metadata(&archive_path, &metadata_dir).unwrap();
        let readme = metadata_dir.join("README.md");
        assert!(readme.exists());
        let contents = fs::read_to_string(readme).unwrap();
        assert!(contents.contains("Test Mod"));
    }

    #[test]
    fn install_single_mod_to_index_missing_file() {
        let storage = tempfile::tempdir().unwrap();
        let mut index = LibraryIndex::default();
        let result =
            install_single_mod_to_index(storage.path(), &mut index, "/nonexistent/file.fantome");
        assert_matches!(result, Err(AppError::InvalidPath(_)));
    }

    #[test]
    fn install_single_mod_to_index_adds_to_index() {
        let storage = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        let archive_path = make_test_fantome_zip(source.path(), false, false);
        let mut index = LibraryIndex::default();
        assert!(index.mods.is_empty());

        let (_entry, installed) =
            install_single_mod_to_index(storage.path(), &mut index, archive_path.to_str().unwrap())
                .unwrap();

        assert_eq!(index.mods.len(), 1);
        assert_eq!(installed.display_name, "Test Mod");
        assert!(installed.enabled);

        let profile = index
            .profiles
            .iter()
            .find(|p| p.id == index.active_profile_id)
            .unwrap();
        assert_eq!(profile.mod_order[0], index.mods[0].id);
        assert_eq!(profile.enabled_mods[0], index.mods[0].id);
    }

    #[test]
    fn install_single_mod_format_detection_fantome() {
        let storage = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        let archive_path = make_test_fantome_zip(source.path(), false, false);
        let mut index = LibraryIndex::default();

        let (entry, _) =
            install_single_mod_to_index(storage.path(), &mut index, archive_path.to_str().unwrap())
                .unwrap();

        assert_eq!(entry.format, ModArchiveFormat::Fantome);
    }

    #[test]
    fn mod_archive_format_extension() {
        assert_eq!(ModArchiveFormat::Fantome.extension(), "fantome");
        assert_eq!(ModArchiveFormat::Modpkg.extension(), "modpkg");
    }

    #[test]
    fn library_mod_entry_paths() {
        let storage_dir = Path::new("/storage");
        let entry = LibraryModEntry {
            id: "abc-123".to_string(),
            installed_at: Utc::now(),
            format: ModArchiveFormat::Fantome,
        };

        let metadata_dir = entry.metadata_dir(storage_dir);
        assert!(metadata_dir.ends_with("mods/abc-123"));

        let archive_path = entry.archive_path(storage_dir);
        assert!(archive_path.ends_with("archives/abc-123.fantome"));
    }
}
