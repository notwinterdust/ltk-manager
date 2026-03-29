const FOLDER_DROP_PREFIX = "folder:";

export function parseFolderDropId(id: string): string | null {
  if (id.startsWith(FOLDER_DROP_PREFIX)) return id.slice(FOLDER_DROP_PREFIX.length);
  return null;
}

export type DropTarget = { type: "folder"; folderId: string } | { type: "reorder" };

export function resolveDropTarget(overId: string): DropTarget {
  const folderId = parseFolderDropId(overId);
  if (folderId) return { type: "folder", folderId };
  return { type: "reorder" };
}

const SORTABLE_FOLDER_PREFIX = "sortable-folder:";

export function toSortableFolderId(folderId: string): string {
  return `${SORTABLE_FOLDER_PREFIX}${folderId}`;
}

export function parseSortableFolderId(id: string): string | null {
  if (id.startsWith(SORTABLE_FOLDER_PREFIX)) return id.slice(SORTABLE_FOLDER_PREFIX.length);
  return null;
}

export function hasOrderChanged(a: readonly string[], b: readonly string[]): boolean {
  if (a.length !== b.length) return true;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return true;
  }
  return false;
}

export const REMOVE_FROM_FOLDER_ID = "remove-from-folder";

export function resolveFolderId(id: string): string | null {
  return parseFolderDropId(id) ?? parseSortableFolderId(id);
}

export function gridClass(viewMode: "grid" | "list") {
  if (viewMode === "list") return "space-y-2";
  return "grid grid-cols-[repeat(auto-fill,minmax(var(--card-min-w,240px),var(--card-max-w,320px)))] justify-center gap-4";
}
