import { invoke } from "@tauri-apps/api/core";
import { useMemo } from "react";
import {
  LuChevronRight,
  LuEllipsisVertical,
  LuFolderOpen,
  LuPackage,
  LuPencil,
  LuPlay,
  LuTrash2,
} from "react-icons/lu";

import { Button, Checkbox, IconButton, Menu } from "@/components";
import type { WorkshopProject } from "@/lib/tauri";
import { usePatcherStatus } from "@/modules/patcher";
import {
  usePatcherSessionStore,
  useWorkshopDialogsStore,
  useWorkshopSelectionStore,
} from "@/stores";

import { useProjectThumbnail } from "../api/useProjectThumbnail";
import { useTestProjects } from "../api/useTestProject";
import type { ViewMode } from "./WorkshopToolbar";

interface ProjectCardProps {
  project: WorkshopProject;
  viewMode: ViewMode;
  onEdit: (project: WorkshopProject) => void;
}

export function ProjectCard({ project, viewMode, onEdit }: ProjectCardProps) {
  const { data: thumbnailUrl } = useProjectThumbnail(project.path, project.thumbnailPath);

  const selected = useWorkshopSelectionStore((s) => s.selectedPaths.has(project.path));
  const toggle = useWorkshopSelectionStore((s) => s.toggle);

  const { data: patcherStatus } = usePatcherStatus();
  const isPatcherActive = patcherStatus?.running ?? false;

  const testingProjects = usePatcherSessionStore((s) => s.testingProjects);
  const isTesting = useMemo(
    () => testingProjects.some((p) => p.path === project.path),
    [testingProjects, project.path],
  );

  const openPackDialog = useWorkshopDialogsStore((s) => s.openPackDialog);
  const openDeleteDialog = useWorkshopDialogsStore((s) => s.openDeleteDialog);

  const testProjects = useTestProjects();

  function handleTest() {
    testProjects.mutate(
      { projects: [{ path: project.path, displayName: project.displayName }] },
      { onError: (err) => console.error("Failed to test project:", err.message) },
    );
  }

  async function handleOpenLocation() {
    try {
      await invoke("reveal_in_explorer", { path: project.path });
    } catch (error) {
      console.error("Failed to open location:", error);
    }
  }

  const listBorderClass = isTesting
    ? "border-green-500/40"
    : selected
      ? "border-brand-500/40"
      : "border-surface-700";

  if (viewMode === "list") {
    return (
      <div
        className={`group flex items-center gap-4 rounded-lg border bg-surface-900 p-4 transition-all hover:border-surface-600 ${listBorderClass}`}
      >
        <Checkbox
          size="md"
          checked={isPatcherActive ? isTesting : selected}
          onCheckedChange={() => toggle(project.path)}
          disabled={isPatcherActive}
        />

        <div className="flex h-12 w-12 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-linear-to-br from-surface-700 to-surface-800">
          {thumbnailUrl ? (
            <img src={thumbnailUrl} alt="" className="h-full w-full object-cover" />
          ) : (
            <span className="text-lg font-bold text-surface-500">
              {project.displayName.charAt(0).toUpperCase()}
            </span>
          )}
        </div>

        <div className="min-w-0 flex-1">
          <h3
            className="group/title flex cursor-pointer items-center gap-1 font-medium text-surface-100 hover:text-brand-400"
            onClick={() => onEdit(project)}
          >
            <span className="truncate">{project.displayName}</span>
            <LuChevronRight className="h-3.5 w-3.5 shrink-0 opacity-0 transition-opacity group-hover/title:opacity-100" />
          </h3>
          <p className="truncate text-sm text-surface-500">
            v{project.version} • {project.authors.map((a) => a.name).join(", ") || "Unknown author"}
          </p>
        </div>

        {isTesting && (
          <span className="shrink-0 rounded-full bg-green-500/10 px-2.5 py-0.5 text-xs font-medium text-green-400">
            Testing
          </span>
        )}

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            left={<LuPlay className="h-4 w-4" />}
            onClick={handleTest}
            disabled={isPatcherActive}
          >
            Test
          </Button>
          <Button
            variant="outline"
            size="sm"
            left={<LuPackage className="h-4 w-4" />}
            onClick={() => openPackDialog(project)}
          >
            Pack
          </Button>
          <Menu.Root>
            <Menu.Trigger
              render={
                <IconButton
                  icon={<LuEllipsisVertical className="h-4 w-4" />}
                  variant="ghost"
                  size="sm"
                />
              }
            />
            <Menu.Portal>
              <Menu.Positioner>
                <Menu.Popup>
                  <Menu.Item
                    icon={<LuPencil className="h-4 w-4" />}
                    onClick={() => onEdit(project)}
                  >
                    Edit Project
                  </Menu.Item>
                  <Menu.Item
                    icon={<LuPlay className="h-4 w-4" />}
                    onClick={handleTest}
                    disabled={isPatcherActive}
                  >
                    Test
                  </Menu.Item>
                  <Menu.Item
                    icon={<LuPackage className="h-4 w-4" />}
                    onClick={() => openPackDialog(project)}
                  >
                    Pack
                  </Menu.Item>
                  <Menu.Item
                    icon={<LuFolderOpen className="h-4 w-4" />}
                    onClick={handleOpenLocation}
                  >
                    Open Location
                  </Menu.Item>
                  <Menu.Separator />
                  <Menu.Item
                    icon={<LuTrash2 className="h-4 w-4" />}
                    variant="danger"
                    onClick={() => openDeleteDialog(project)}
                  >
                    Delete
                  </Menu.Item>
                </Menu.Popup>
              </Menu.Positioner>
            </Menu.Portal>
          </Menu.Root>
          <IconButton
            icon={<LuChevronRight />}
            variant="ghost"
            size="sm"
            onClick={() => onEdit(project)}
          />
        </div>
      </div>
    );
  }

  const gridBorderClass = isTesting
    ? "border-green-500/40"
    : selected
      ? "border-brand-500/40"
      : "border-surface-600";

  return (
    <div
      className={`group relative rounded-xl border bg-surface-800 transition-all hover:border-surface-400 ${gridBorderClass}`}
    >
      <div
        className={`absolute top-0 left-0 z-10 p-2 ${isPatcherActive ? "" : "cursor-pointer"}`}
        onClick={(e) => {
          if (!isPatcherActive && e.target === e.currentTarget) toggle(project.path);
        }}
      >
        <Checkbox
          size="md"
          checked={isPatcherActive ? isTesting : selected}
          onCheckedChange={() => toggle(project.path)}
          disabled={isPatcherActive}
        />
      </div>

      <div className="relative flex aspect-video items-center justify-center overflow-hidden rounded-t-xl bg-linear-to-br from-surface-700 to-surface-800">
        {thumbnailUrl ? (
          <img src={thumbnailUrl} alt="" className="h-full w-full object-cover" />
        ) : (
          <span className="text-4xl font-bold text-surface-400">
            {project.displayName.charAt(0).toUpperCase()}
          </span>
        )}
      </div>

      <div className="flex items-start gap-1 p-3">
        <div className="min-w-0 flex-1">
          <h3
            className="group/title mb-1 flex cursor-pointer items-center gap-1 text-sm font-medium text-surface-100 hover:text-brand-400"
            onClick={() => onEdit(project)}
          >
            <span className="line-clamp-1">{project.displayName}</span>
            <LuChevronRight className="h-3.5 w-3.5 shrink-0 opacity-0 transition-opacity group-hover/title:opacity-100" />
          </h3>
          <div className="flex items-center gap-1.5 text-xs text-surface-500">
            <span>v{project.version}</span>
            <span>•</span>
            <span className="flex-1 truncate">
              {project.authors.length > 0 ? project.authors[0].name : "Unknown"}
            </span>
            {isTesting && (
              <span className="shrink-0 rounded-full bg-green-500/10 px-2 py-0.5 text-xs font-medium text-green-400">
                Testing
              </span>
            )}
          </div>
        </div>
        <Menu.Root>
          <Menu.Trigger
            render={<IconButton icon={<LuEllipsisVertical />} variant="ghost" size="md" compact />}
          />
          <Menu.Portal>
            <Menu.Positioner>
              <Menu.Popup>
                <Menu.Item icon={<LuPencil className="h-4 w-4" />} onClick={() => onEdit(project)}>
                  Edit Project
                </Menu.Item>
                <Menu.Item
                  icon={<LuPlay className="h-4 w-4" />}
                  onClick={handleTest}
                  disabled={isPatcherActive}
                >
                  Test
                </Menu.Item>
                <Menu.Item
                  icon={<LuPackage className="h-4 w-4" />}
                  onClick={() => openPackDialog(project)}
                >
                  Pack
                </Menu.Item>
                <Menu.Item icon={<LuFolderOpen className="h-4 w-4" />} onClick={handleOpenLocation}>
                  Open Location
                </Menu.Item>
                <Menu.Separator />
                <Menu.Item
                  icon={<LuTrash2 className="h-4 w-4" />}
                  variant="danger"
                  onClick={() => openDeleteDialog(project)}
                >
                  Delete
                </Menu.Item>
              </Menu.Popup>
            </Menu.Positioner>
          </Menu.Portal>
        </Menu.Root>
      </div>
    </div>
  );
}
