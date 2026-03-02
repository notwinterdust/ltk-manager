import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useHotkeys } from "react-hotkeys-hook";

import type { WorkshopProject } from "@/lib/tauri";
import {
  BulkDeleteDialog,
  BulkPackDialog,
  DeleteConfirmDialog,
  ErrorState,
  ImportFantomeDialog,
  ImportGitRepoDialog,
  LoadingState,
  NewProjectDialog,
  NoProjectsState,
  NoSearchResultsState,
  PackDialog,
  ProjectGrid,
  useFilteredProjects,
  useWorkshopProjects,
  WorkshopToolbar,
} from "@/modules/workshop";
import { useWorkshopDialogsStore, useWorkshopSelectionStore, useWorkshopViewStore } from "@/stores";

export const Route = createFileRoute("/workshop/")({
  component: WorkshopIndex,
});

function WorkshopIndex() {
  const navigate = useNavigate();
  const { isLoading, error } = useWorkshopProjects();
  const searchQuery = useWorkshopViewStore((s) => s.searchQuery);
  const filteredProjects = useFilteredProjects();

  const openNewProjectDialog = useWorkshopDialogsStore((s) => s.openNewProjectDialog);
  const selectAll = useWorkshopSelectionStore((s) => s.selectAll);

  useHotkeys("ctrl+n", () => openNewProjectDialog(), { preventDefault: true });
  useHotkeys("ctrl+a", () => selectAll(filteredProjects.map((p) => p.path)), {
    preventDefault: true,
  });

  function handleEditProject(project: WorkshopProject) {
    navigate({ to: "/workshop/$projectName", params: { projectName: project.name } });
  }

  function renderContent() {
    if (isLoading) return <LoadingState />;
    if (error) return <ErrorState error={error} />;
    if (filteredProjects.length === 0) {
      if (searchQuery) return <NoSearchResultsState />;
      return <NoProjectsState />;
    }
    return <ProjectGrid projects={filteredProjects} onEdit={handleEditProject} />;
  }

  return (
    <div className="flex h-full flex-col">
      <WorkshopToolbar />
      <div className="flex-1 overflow-auto p-6">{renderContent()}</div>
      <PackDialog />
      <BulkPackDialog />
      <DeleteConfirmDialog />
      <BulkDeleteDialog />
      <NewProjectDialog />
      <ImportFantomeDialog />
      <ImportGitRepoDialog />
    </div>
  );
}
