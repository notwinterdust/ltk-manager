import { create } from "zustand";

export type SortField = "manual" | "name" | "installedAt" | "enabled";
export type SortDirection = "asc" | "desc";

export interface SortConfig {
  field: SortField;
  direction: SortDirection;
}

interface LibraryFilterStore {
  selectedTags: Set<string>;
  selectedChampions: Set<string>;
  selectedMaps: Set<string>;
  sort: SortConfig;

  toggleTag: (tag: string) => void;
  toggleChampion: (champion: string) => void;
  toggleMap: (map: string) => void;
  clearFilters: () => void;
  setSort: (sort: SortConfig) => void;
}

export const useLibraryFilterStore = create<LibraryFilterStore>((set) => ({
  selectedTags: new Set(),
  selectedChampions: new Set(),
  selectedMaps: new Set(),
  sort: { field: "manual", direction: "asc" },

  toggleTag: (tag) =>
    set((state) => {
      const next = new Set(state.selectedTags);
      if (next.has(tag)) next.delete(tag);
      else next.add(tag);
      return { selectedTags: next };
    }),

  toggleChampion: (champion) =>
    set((state) => {
      const next = new Set(state.selectedChampions);
      if (next.has(champion)) next.delete(champion);
      else next.add(champion);
      return { selectedChampions: next };
    }),

  toggleMap: (map) =>
    set((state) => {
      const next = new Set(state.selectedMaps);
      if (next.has(map)) next.delete(map);
      else next.add(map);
      return { selectedMaps: next };
    }),

  clearFilters: () =>
    set({
      selectedTags: new Set(),
      selectedChampions: new Set(),
      selectedMaps: new Set(),
    }),

  setSort: (sort) => set({ sort }),
}));

export function useHasActiveFilters() {
  const { selectedTags, selectedChampions, selectedMaps } = useLibraryFilterStore();
  return selectedTags.size > 0 || selectedChampions.size > 0 || selectedMaps.size > 0;
}
