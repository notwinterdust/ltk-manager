import { useMemo } from "react";

import type { InstalledMod } from "@/lib/tauri";
import { useLibraryFilterStore } from "@/stores";

export function useFilteredMods(mods: InstalledMod[], searchQuery: string): InstalledMod[] {
  const { selectedTags, selectedChampions, selectedMaps, sort } = useLibraryFilterStore();

  return useMemo(() => {
    let result = mods;

    if (searchQuery) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (mod) => mod.displayName.toLowerCase().includes(q) || mod.name.toLowerCase().includes(q),
      );
    }

    if (selectedTags.size > 0) {
      result = result.filter((mod) => mod.tags.some((t) => selectedTags.has(t)));
    }
    if (selectedChampions.size > 0) {
      result = result.filter((mod) => mod.champions.some((c) => selectedChampions.has(c)));
    }
    if (selectedMaps.size > 0) {
      result = result.filter((mod) => mod.maps.some((m) => selectedMaps.has(m)));
    }

    if (sort.field === "priority") return result;

    const sorted = [...result];
    const dir = sort.direction === "asc" ? 1 : -1;

    sorted.sort((a, b) => {
      switch (sort.field) {
        case "name":
          return dir * a.displayName.localeCompare(b.displayName);
        case "installedAt":
          return dir * (new Date(a.installedAt).getTime() - new Date(b.installedAt).getTime());
        case "enabled":
          if (a.enabled !== b.enabled) return a.enabled ? -1 : 1;
          return a.displayName.localeCompare(b.displayName);
        default:
          return 0;
      }
    });

    return sorted;
  }, [mods, searchQuery, selectedTags, selectedChampions, selectedMaps, sort]);
}
