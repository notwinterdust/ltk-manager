import { useState } from "react";

import {
  DragDropOverlay,
  FilterBar,
  ImportProgressDialog,
  LibraryContent,
  LibraryToolbar,
  useFilterOptions,
  useInstalledMods,
  useLibraryActions,
  useModFileDrop,
} from "@/modules/library";
import { MigrationBanner, MigrationWizardDialog } from "@/modules/migration";
import { usePatcherStatus, useStartPatcher, useStopPatcher } from "@/modules/patcher";
import { useSaveSettings, useSettings } from "@/modules/settings";

export function Library() {
  const [searchQuery, setSearchQuery] = useState("");
  const [migrationOpen, setMigrationOpen] = useState(false);

  const { data: mods = [], isLoading, error } = useInstalledMods();
  const actions = useLibraryActions();
  const isDragOver = useModFileDrop(actions.handleBulkInstallFiles);

  const { data: settings } = useSettings();
  const saveSettings = useSaveSettings();

  const { data: patcherStatus } = usePatcherStatus();
  const startPatcher = useStartPatcher();
  const stopPatcher = useStopPatcher();

  const isStarting = patcherStatus?.phase === "building";
  const isPatcherActive = patcherStatus?.running ?? false;

  const filterOptions = useFilterOptions(mods);
  const hasEnabledMods = mods.some((m) => m.enabled);

  function handleStartPatcher() {
    startPatcher.mutate(
      {},
      {
        onError: (error) => {
          console.error("Failed to start patcher:", error.message);
        },
      },
    );
  }

  function handleStopPatcher() {
    stopPatcher.mutate(undefined, {
      onError: (error) => {
        console.error("Failed to stop patcher:", error.message);
      },
    });
  }

  function handleDismissMigration() {
    if (!settings) return;
    saveSettings.mutate({ ...settings, migrationDismissed: true });
  }

  return (
    <div className="relative flex h-full flex-col">
      <DragDropOverlay visible={isDragOver} />
      {settings && !settings.migrationDismissed && (
        <MigrationBanner
          onImport={() => setMigrationOpen(true)}
          onDismiss={handleDismissMigration}
        />
      )}
      <LibraryToolbar
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        actions={actions}
        patcher={{
          status: patcherStatus,
          isStarting: isStarting,
          isStopping: stopPatcher.isPending,
          onStart: handleStartPatcher,
          onStop: handleStopPatcher,
        }}
        hasEnabledMods={hasEnabledMods}
        isLoading={isLoading}
        isPatcherActive={isPatcherActive}
      />
      <FilterBar filterOptions={filterOptions} />
      <LibraryContent
        mods={mods}
        searchQuery={searchQuery}
        actions={actions}
        isLoading={isLoading}
        error={error}
        onInstall={actions.handleInstallMod}
        isPatcherActive={isPatcherActive}
      />
      <ImportProgressDialog
        open={actions.importDialogOpen}
        onClose={actions.handleCloseImportDialog}
        progress={actions.installProgress}
        result={actions.importResult}
      />
      <MigrationWizardDialog open={migrationOpen} onClose={() => setMigrationOpen(false)} />
    </div>
  );
}
