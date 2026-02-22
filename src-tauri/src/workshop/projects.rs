use super::{
    find_config_file, is_valid_project_name, load_mod_project, load_workshop_project,
    CreateProjectArgs, SaveProjectConfigArgs, Workshop, WorkshopProject,
};
use crate::error::{AppError, AppResult};
use crate::state::Settings;
use ltk_mod_project::{
    default_layers, ModMap, ModProject, ModProjectAuthor, ModProjectLayer, ModTag,
};
use std::fs;
use std::path::PathBuf;

impl Workshop {
    /// Get all workshop projects from the configured workshop directory.
    pub fn get_projects(&self, settings: &Settings) -> AppResult<Vec<WorkshopProject>> {
        let workshop_path = self.workshop_dir(settings)?;

        if !workshop_path.exists() {
            return Ok(Vec::new());
        }

        let mut projects = Vec::new();

        for entry in fs::read_dir(&workshop_path)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            if find_config_file(&path).is_none() {
                continue;
            }

            match load_workshop_project(&path) {
                Ok(project) => projects.push(project),
                Err(e) => {
                    tracing::warn!("Skipping invalid project at {}: {}", path.display(), e);
                }
            }
        }

        // Sort by last modified (newest first)
        projects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        Ok(projects)
    }

    /// Create a new workshop project.
    pub fn create_project(
        &self,
        settings: &Settings,
        args: CreateProjectArgs,
    ) -> AppResult<WorkshopProject> {
        let workshop_path = self.workshop_dir(settings)?;

        if !is_valid_project_name(&args.name) {
            return Err(AppError::ValidationFailed(
                "Project name must be lowercase alphanumeric with hyphens only".to_string(),
            ));
        }

        let project_dir = workshop_path.join(&args.name);

        if project_dir.exists() {
            return Err(AppError::ProjectAlreadyExists(args.name));
        }

        // Create project structure
        fs::create_dir_all(&project_dir)?;
        fs::create_dir_all(project_dir.join("content").join("base"))?;

        let authors: Vec<ModProjectAuthor> = args
            .authors
            .into_iter()
            .map(ModProjectAuthor::Name)
            .collect();

        let mod_project = ModProject {
            name: args.name.clone(),
            display_name: args.display_name,
            version: "1.0.0".to_string(),
            description: args.description,
            authors,
            license: None,
            tags: Vec::new(),
            champions: Vec::new(),
            maps: Vec::new(),
            transformers: Vec::new(),
            layers: default_layers(),
            thumbnail: None,
        };

        let config_path = project_dir.join("mod.config.json");
        let config_content = serde_json::to_string_pretty(&mod_project)?;
        fs::write(&config_path, config_content)?;

        let readme_content = format!(
            "# {}\n\n{}\n",
            mod_project.display_name, mod_project.description
        );
        fs::write(project_dir.join("README.md"), readme_content)?;

        load_workshop_project(&project_dir)
    }

    /// Get a single workshop project by path.
    pub fn get_project(&self, project_path: &str) -> AppResult<WorkshopProject> {
        let path = PathBuf::from(project_path);
        if !path.exists() {
            return Err(AppError::ProjectNotFound(project_path.to_string()));
        }
        load_workshop_project(&path)
    }

    /// Save project configuration changes.
    pub fn save_config(&self, args: SaveProjectConfigArgs) -> AppResult<WorkshopProject> {
        let path = PathBuf::from(&args.project_path);
        if !path.exists() {
            return Err(AppError::ProjectNotFound(args.project_path));
        }

        let config_path = find_config_file(&path)
            .ok_or_else(|| AppError::ProjectNotFound(args.project_path.clone()))?;
        let mut mod_project = load_mod_project(&config_path)?;

        mod_project.display_name = args.display_name;
        mod_project.version = args.version;
        mod_project.description = args.description;
        mod_project.authors = args
            .authors
            .into_iter()
            .map(|a| match a.role {
                Some(role) => ModProjectAuthor::Role { name: a.name, role },
                None => ModProjectAuthor::Name(a.name),
            })
            .collect();
        mod_project.tags = args.tags.into_iter().map(ModTag::from).collect();
        mod_project.champions = args.champions;
        mod_project.maps = args.maps.into_iter().map(ModMap::from).collect();

        let json_config_path = path.join("mod.config.json");
        let config_content = serde_json::to_string_pretty(&mod_project)?;
        fs::write(&json_config_path, config_content)?;

        load_workshop_project(&path)
    }

    /// Rename a workshop project (change its slug/directory name).
    pub fn rename_project(&self, project_path: &str, new_name: &str) -> AppResult<WorkshopProject> {
        let new_name = new_name.trim().to_string();

        if !is_valid_project_name(&new_name) {
            return Err(AppError::ValidationFailed(
                "Project name must be lowercase alphanumeric with hyphens only".to_string(),
            ));
        }

        let old_path = PathBuf::from(project_path);
        if !old_path.exists() {
            return Err(AppError::ProjectNotFound(project_path.to_string()));
        }

        // Verify it's a valid project
        if find_config_file(&old_path).is_none() {
            return Err(AppError::ProjectNotFound(project_path.to_string()));
        }

        let parent_dir = old_path.parent().ok_or_else(|| {
            AppError::InvalidPath("Cannot determine parent directory".to_string())
        })?;
        let new_path = parent_dir.join(&new_name);

        // Check if old name is the same as new name
        if old_path == new_path {
            return load_workshop_project(&old_path);
        }

        if new_path.exists() {
            return Err(AppError::ProjectAlreadyExists(new_name));
        }

        // Rename the directory
        fs::rename(&old_path, &new_path)?;

        // Update mod_project.name in the config file
        let config_path = find_config_file(&new_path)
            .ok_or_else(|| AppError::ProjectNotFound(new_path.display().to_string()))?;
        let mut mod_project = load_mod_project(&config_path)?;
        mod_project.name = new_name;

        let json_config_path = new_path.join("mod.config.json");
        let config_content = serde_json::to_string_pretty(&mod_project)?;
        fs::write(&json_config_path, config_content)?;

        load_workshop_project(&new_path)
    }

    /// Delete a workshop project.
    pub fn delete_project(&self, project_path: &str) -> AppResult<()> {
        let path = PathBuf::from(project_path);
        if !path.exists() {
            return Err(AppError::ProjectNotFound(project_path.to_string()));
        }

        // Safety check: ensure it contains a mod.config file
        if find_config_file(&path).is_none() {
            return Err(AppError::ValidationFailed(
                "Directory does not appear to be a mod project".to_string(),
            ));
        }

        fs::remove_dir_all(&path)?;
        Ok(())
    }

    /// Import a .modpkg file as a new workshop project.
    pub fn import_from_modpkg(
        &self,
        settings: &Settings,
        file_path: &str,
    ) -> AppResult<WorkshopProject> {
        let workshop_path = self.workshop_dir(settings)?;

        let file = fs::File::open(file_path)?;
        let mut modpkg = ltk_modpkg::Modpkg::mount_from_reader(file)
            .map_err(|e| AppError::Modpkg(e.to_string()))?;

        let metadata = modpkg
            .load_metadata()
            .map_err(|e| AppError::Modpkg(e.to_string()))?;

        // Check if project already exists
        let project_dir = workshop_path.join(&metadata.name);
        if project_dir.exists() {
            return Err(AppError::ProjectAlreadyExists(metadata.name));
        }

        // Create project directory
        fs::create_dir_all(&project_dir)?;

        // Extract content
        let content_dir = project_dir.join("content");
        fs::create_dir_all(&content_dir)?;

        let mut extractor = ltk_modpkg::ModpkgExtractor::new(&mut modpkg);
        extractor
            .extract_all(&content_dir)
            .map_err(|e| AppError::Modpkg(e.to_string()))?;

        // Build layers from header, preserving string overrides from metadata
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

        if !layers.iter().any(|l| l.name == "base") {
            layers.insert(0, ModProjectLayer::base());
        }

        // Create mod project config
        let mod_project = ModProject {
            name: metadata.name,
            display_name: metadata.display_name,
            version: metadata.version.to_string(),
            description: metadata.description.unwrap_or_default(),
            authors: metadata
                .authors
                .into_iter()
                .map(|a| ModProjectAuthor::Name(a.name))
                .collect(),
            license: None,
            tags: metadata
                .tags
                .into_iter()
                .map(ltk_mod_project::ModTag::from)
                .collect(),
            champions: metadata.champions,
            maps: metadata
                .maps
                .into_iter()
                .map(ltk_mod_project::ModMap::from)
                .collect(),
            transformers: Vec::new(),
            layers,
            thumbnail: None,
        };

        let config_path = project_dir.join("mod.config.json");
        fs::write(&config_path, serde_json::to_string_pretty(&mod_project)?)?;

        // Extract README and thumbnail
        if let Ok(readme_bytes) = modpkg.load_readme() {
            let _ = fs::write(project_dir.join("README.md"), readme_bytes);
        }
        if let Ok(thumbnail_bytes) = modpkg.load_thumbnail() {
            let _ = fs::write(project_dir.join("thumbnail.webp"), thumbnail_bytes);
        }

        load_workshop_project(&project_dir)
    }
}
