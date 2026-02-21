import { useMemo } from "react";

import type { InstalledMod } from "@/lib/tauri";

export interface FilterOptions {
  tags: string[];
  champions: string[];
  maps: string[];
}

export function useFilterOptions(mods: InstalledMod[]): FilterOptions {
  return useMemo(() => {
    const tags = new Set<string>();
    const champions = new Set<string>();
    const maps = new Set<string>();

    for (const mod of mods) {
      for (const t of mod.tags) tags.add(t);
      for (const c of mod.champions) champions.add(c);
      for (const m of mod.maps) maps.add(m);
    }

    return {
      tags: [...tags].sort(),
      champions: [...champions].sort(),
      maps: [...maps].sort(),
    };
  }, [mods]);
}
