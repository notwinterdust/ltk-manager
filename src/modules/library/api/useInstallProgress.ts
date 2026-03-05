import { useState } from "react";

import type { InstallProgress } from "@/lib/tauri";
import { useTauriEvent } from "@/lib/useTauriEvent";

export function useInstallProgress() {
  const [progress, setProgress] = useState<InstallProgress | null>(null);

  useTauriEvent<InstallProgress>("install-progress", setProgress);

  function reset() {
    setProgress(null);
  }

  return { progress, reset };
}
