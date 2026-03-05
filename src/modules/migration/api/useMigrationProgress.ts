import { useState } from "react";

import type { InstallProgress, MigrationProgress } from "@/lib/tauri";
import { useTauriEvent } from "@/lib/useTauriEvent";

export function useMigrationProgress() {
  const [progress, setProgress] = useState<MigrationProgress | null>(null);

  useTauriEvent<MigrationProgress>("migration-progress", setProgress);

  useTauriEvent<InstallProgress>("install-progress", (payload) => {
    setProgress({
      phase: "installing",
      current: payload.current,
      total: payload.total,
      currentFile: payload.currentFile,
    });
  });

  function reset() {
    setProgress(null);
  }

  return { progress, reset };
}
