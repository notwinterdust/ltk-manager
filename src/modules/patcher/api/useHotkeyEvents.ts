import { useRef } from "react";

import { useToast } from "@/components";
import { useTauriEvent } from "@/lib/useTauriEvent";
import { usePatcherSessionStore } from "@/stores";

/**
 * Listen for hotkey-triggered reload/error events from the backend.
 *
 * - On `hotkey-reload-complete`: re-syncs the testingProjects store if the
 *   reload included workshop projects (preserves the "Testing X projects" UI).
 * - On `hotkey-error`: shows an error toast.
 */
export function useHotkeyEvents() {
  const toast = useToast();
  const testingProjects = usePatcherSessionStore((s) => s.testingProjects);
  const setTestingProjects = usePatcherSessionStore((s) => s.setTestingProjects);

  const testingProjectsRef = useRef(testingProjects);
  testingProjectsRef.current = testingProjects;

  useTauriEvent<string[] | null>("hotkey-reload-complete", (workshopPaths) => {
    const current = testingProjectsRef.current;

    if (workshopPaths && workshopPaths.length > 0 && current.length > 0) {
      const pathSet = new Set(workshopPaths);
      const stillTesting = current.filter((p) => pathSet.has(p.path));

      if (stillTesting.length > 0) {
        setTestingProjects(stillTesting);
      }
    }
  });

  useTauriEvent<string>("hotkey-error", (message) => {
    toast.error("Hotkey Error", message);
  });
}
