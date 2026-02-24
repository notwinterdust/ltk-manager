import { invoke } from "@tauri-apps/api/core";

import type {
  AppError,
  AppInfo,
  BulkInstallResult,
  CreateProjectArgs,
  CslolModInfo,
  FantomePeekResult,
  ImportFantomeArgs,
  ImportGitRepoArgs,
  InstalledMod,
  ModpkgInfo,
  PackProjectArgs,
  PackResult,
  PatcherConfig,
  PatcherStatus,
  Profile,
  SaveProjectConfigArgs,
  Settings,
  ValidationResult,
  WorkshopProject,
} from "@/lib/bindings";
import type { Result } from "@/utils/result";

export type * from "@/lib/bindings";
export type { Result } from "@/utils/result";
export { isErr, isOk, match, unwrap, unwrapOr } from "@/utils/result";

type IpcResponse<T> = { ok: true; value: T } | { ok: false; error: AppError };

/**
 * Transform the raw IPC response to our Result type.
 */
function toResult<T>(response: IpcResponse<T>): Result<T> {
  if (response.ok) {
    return { ok: true, value: response.value };
  }
  return { ok: false, error: response.error };
}

/**
 * Invoke a Tauri command and return a typed Result.
 */
async function invokeResult<T>(cmd: string, args?: Record<string, unknown>): Promise<Result<T>> {
  const response = await invoke<IpcResponse<T>>(cmd, args);
  return toResult(response);
}

// API functions
export const api = {
  getAppInfo: () => invokeResult<AppInfo>("get_app_info"),

  // Settings
  getSettings: () => invokeResult<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invokeResult<void>("save_settings", { settings }),
  autoDetectLeaguePath: () => invokeResult<string | null>("auto_detect_league_path"),
  validateLeaguePath: (path: string) => invokeResult<boolean>("validate_league_path", { path }),
  checkSetupRequired: () => invokeResult<boolean>("check_setup_required"),

  // Mods
  getInstalledMods: () => invokeResult<InstalledMod[]>("get_installed_mods"),
  installMod: (filePath: string) => invokeResult<InstalledMod>("install_mod", { filePath }),
  installMods: (filePaths: string[]) =>
    invokeResult<BulkInstallResult>("install_mods", { filePaths }),
  uninstallMod: (modId: string) => invokeResult<void>("uninstall_mod", { modId }),
  toggleMod: (modId: string, enabled: boolean) =>
    invokeResult<void>("toggle_mod", { modId, enabled }),
  getModThumbnail: (modId: string) => invokeResult<string | null>("get_mod_thumbnail", { modId }),
  getStorageDirectory: () => invokeResult<string>("get_storage_directory"),
  reorderMods: (modIds: string[]) => invokeResult<void>("reorder_mods", { modIds }),

  // Migration
  scanCslolMods: (directory: string) =>
    invokeResult<CslolModInfo[]>("scan_cslol_mods", { directory }),
  importCslolMods: (directory: string, selectedFolders: string[]) =>
    invokeResult<BulkInstallResult>("import_cslol_mods", { directory, selectedFolders }),

  // Inspector
  inspectModpkg: (filePath: string) => invokeResult<ModpkgInfo>("inspect_modpkg", { filePath }),

  // Patcher
  startPatcher: (config: PatcherConfig) => invokeResult<void>("start_patcher", { config }),
  stopPatcher: () => invokeResult<void>("stop_patcher"),
  getPatcherStatus: () => invokeResult<PatcherStatus>("get_patcher_status"),

  // Profiles
  listModProfiles: () => invokeResult<Profile[]>("list_mod_profiles"),
  getActiveModProfile: () => invokeResult<Profile>("get_active_mod_profile"),
  createModProfile: (name: string) => invokeResult<Profile>("create_mod_profile", { name }),
  deleteModProfile: (profileId: string) => invokeResult<void>("delete_mod_profile", { profileId }),
  switchModProfile: (profileId: string) =>
    invokeResult<Profile>("switch_mod_profile", { profileId }),
  renameModProfile: (profileId: string, newName: string) =>
    invokeResult<Profile>("rename_mod_profile", { profileId, newName }),

  // Shell
  revealInExplorer: (path: string) => invokeResult<void>("reveal_in_explorer", { path }),

  // Workshop
  getWorkshopProjects: () => invokeResult<WorkshopProject[]>("get_workshop_projects"),
  createWorkshopProject: (args: CreateProjectArgs) =>
    invokeResult<WorkshopProject>("create_workshop_project", { args }),
  getWorkshopProject: (projectPath: string) =>
    invokeResult<WorkshopProject>("get_workshop_project", { projectPath }),
  saveProjectConfig: (args: SaveProjectConfigArgs) =>
    invokeResult<WorkshopProject>("save_project_config", { args }),
  renameWorkshopProject: (projectPath: string, newName: string) =>
    invokeResult<WorkshopProject>("rename_workshop_project", { projectPath, newName }),
  deleteWorkshopProject: (projectPath: string) =>
    invokeResult<void>("delete_workshop_project", { projectPath }),
  packWorkshopProject: (args: PackProjectArgs) =>
    invokeResult<PackResult>("pack_workshop_project", { args }),
  importFromModpkg: (filePath: string) =>
    invokeResult<WorkshopProject>("import_from_modpkg", { filePath }),
  peekFantome: (filePath: string) => invokeResult<FantomePeekResult>("peek_fantome", { filePath }),
  importFromFantome: (args: ImportFantomeArgs) =>
    invokeResult<WorkshopProject>("import_from_fantome", { args }),
  importFromGitRepo: (args: ImportGitRepoArgs) =>
    invokeResult<WorkshopProject>("import_from_git_repo", { args }),
  validateProject: (projectPath: string) =>
    invokeResult<ValidationResult>("validate_project", { projectPath }),
  setProjectThumbnail: (projectPath: string, imagePath: string) =>
    invokeResult<WorkshopProject>("set_project_thumbnail", { projectPath, imagePath }),
  getProjectThumbnail: (thumbnailPath: string) =>
    invokeResult<string>("get_project_thumbnail", { thumbnailPath }),
  saveLayerStringOverrides: (
    projectPath: string,
    layerName: string,
    stringOverrides: Record<string, Record<string, string>>,
  ) =>
    invokeResult<WorkshopProject>("save_layer_string_overrides", {
      projectPath,
      layerName,
      stringOverrides,
    }),
  createProjectLayer: (projectPath: string, name: string, description?: string) =>
    invokeResult<WorkshopProject>("create_project_layer", { projectPath, name, description }),
  deleteProjectLayer: (projectPath: string, layerName: string) =>
    invokeResult<WorkshopProject>("delete_project_layer", { projectPath, layerName }),
  reorderProjectLayers: (projectPath: string, layerNames: string[]) =>
    invokeResult<WorkshopProject>("reorder_project_layers", { projectPath, layerNames }),
  updateLayerDescription: (projectPath: string, layerName: string, description?: string) =>
    invokeResult<WorkshopProject>("update_layer_description", {
      projectPath,
      layerName,
      description,
    }),
};
