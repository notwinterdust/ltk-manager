import { LuPlay, LuX } from "react-icons/lu";

import { Button, Checkbox, IconButton } from "@/components";
import { usePatcherStatus } from "@/modules/patcher";
import { useWorkshopSelectionStore } from "@/stores";

import { useFilteredProjects } from "../api/useFilteredProjects";
import { useTestProjects } from "../api/useTestProject";

export function SelectionActionBar() {
  const selectedPaths = useWorkshopSelectionStore((s) => s.selectedPaths);
  const selectAll = useWorkshopSelectionStore((s) => s.selectAll);
  const clear = useWorkshopSelectionStore((s) => s.clear);

  const { data: patcherStatus } = usePatcherStatus();
  const isPatcherActive = patcherStatus?.running ?? false;

  const filteredProjects = useFilteredProjects();
  const testProjects = useTestProjects();

  const selectedCount = selectedPaths.size;
  const totalCount = filteredProjects.length;

  if (isPatcherActive || selectedCount === 0) return null;

  const allSelected = selectedCount === totalCount;
  const indeterminate = selectedCount > 0 && !allSelected;

  function handleTest() {
    const selected = filteredProjects.filter((p) => selectedPaths.has(p.path));
    if (selected.length === 0) return;
    testProjects.mutate(
      { projects: selected.map((p) => ({ path: p.path, displayName: p.displayName })) },
      {
        onSuccess: () => clear(),
        onError: (err) => console.error("Failed to test projects:", err.message),
      },
    );
  }

  return (
    <div className="animate-in slide-in-from-bottom-2 flex items-center gap-3 border-t border-surface-700 bg-surface-900 px-4 py-2">
      <Checkbox
        size="md"
        checked={allSelected}
        indeterminate={indeterminate}
        onCheckedChange={() => {
          if (allSelected) {
            clear();
          } else {
            selectAll(filteredProjects.map((p) => p.path));
          }
        }}
        aria-label="Toggle select all projects"
      />
      <span className="text-sm text-surface-300">
        {selectedCount} of {totalCount} selected
      </span>

      <IconButton
        icon={<LuX className="h-4 w-4" />}
        variant="ghost"
        size="md"
        onClick={clear}
        aria-label="Clear selection"
      />

      <div className="flex-1" />

      <Button
        variant="filled"
        size="md"
        left={<LuPlay className="h-4 w-4" />}
        onClick={handleTest}
        loading={testProjects.isPending}
      >
        {selectedCount === 1 ? "Test" : `Test ${selectedCount} Projects`}
      </Button>
    </div>
  );
}
