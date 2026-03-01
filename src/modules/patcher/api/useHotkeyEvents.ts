import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import { useToast } from "@/components";
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

  // Use a ref so the listener callback always sees the latest testing projects
  // without needing to re-register the event listener on every change.
  const testingProjectsRef = useRef(testingProjects);
  testingProjectsRef.current = testingProjects;

  useEffect(() => {
    let unlistenReload: UnlistenFn | null = null;
    let unlistenError: UnlistenFn | null = null;

    listen<string[] | null>("hotkey-reload-complete", (event) => {
      const workshopPaths = event.payload;
      const current = testingProjectsRef.current;

      // If the reload included workshop projects and we had testing projects before,
      // re-set them so the StatusBar continues to show the testing label.
      if (workshopPaths && workshopPaths.length > 0 && current.length > 0) {
        const pathSet = new Set(workshopPaths);
        const stillTesting = current.filter((p) => pathSet.has(p.path));

        if (stillTesting.length > 0) {
          setTestingProjects(stillTesting);
        }
      }
    }).then((fn) => {
      unlistenReload = fn;
    });

    listen<string>("hotkey-error", (event) => {
      toast.error("Hotkey Error", event.payload);
    }).then((fn) => {
      unlistenError = fn;
    });

    return () => {
      if (unlistenReload) unlistenReload();
      if (unlistenError) unlistenError();
    };
  }, [toast, setTestingProjects]);
}
