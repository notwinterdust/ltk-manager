import { useMutation, useQueryClient } from "@tanstack/react-query";

import { useToast } from "@/components";
import { api, type AppError, type InstalledMod } from "@/lib/tauri";
import { checkModForSkinhack } from "@/modules/library/utils/skinhackCheck";
import { unwrapForQuery } from "@/utils/query";

import { libraryKeys } from "./keys";

/**
 * Hook to install a mod from a .modpkg file.
 */
export function useInstallMod() {
  const queryClient = useQueryClient();
  const toast = useToast();

  return useMutation<InstalledMod, AppError, string>({
    mutationFn: async (filePath) => {
      const result = await api.installMod(filePath);
      return unwrapForQuery(result);
    },
    onSuccess: (newMod) => {
      queryClient.setQueryData<InstalledMod[]>(libraryKeys.mods(), (old) =>
        old ? [newMod, ...old] : [newMod],
      );

      const flag = checkModForSkinhack(newMod);
      if (flag) {
        api.toggleMod(newMod.id, false);
        toast.warning("Skinhack Detected", `Skinhack detected in "${newMod.displayName}"`);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: libraryKeys.mods() });
    },
  });
}
