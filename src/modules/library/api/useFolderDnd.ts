import type { DragEndEvent, DragOverEvent, DragStartEvent } from "@dnd-kit/core";
import { arrayMove } from "@dnd-kit/sortable";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { LibraryFolder } from "@/lib/tauri";
import {
  hasOrderChanged,
  parseSortableFolderId,
  toSortableFolderId,
} from "@/modules/library/utils";

import { useReorderFolders } from "./useMoveMod";

interface UseFolderDndArgs {
  folders: LibraryFolder[];
}

export function useFolderDnd({ folders }: UseFolderDndArgs) {
  const reorderFolders = useReorderFolders();

  const folderSortableIds = useMemo(() => folders.map((f) => toSortableFolderId(f.id)), [folders]);
  const folderMap = useMemo(() => new Map(folders.map((f) => [f.id, f])), [folders]);

  const [activeFolderId, setActiveFolderId] = useState<string | null>(null);
  const [localOrder, setLocalOrder] = useState<string[]>(folderSortableIds);
  const lastPropsOrder = useRef<string[]>(folderSortableIds);

  useEffect(() => {
    if (hasOrderChanged(folderSortableIds, lastPropsOrder.current)) {
      lastPropsOrder.current = folderSortableIds;
      if (!activeFolderId) setLocalOrder(folderSortableIds);
    }
  }, [folderSortableIds, activeFolderId]);

  const activeFolder = activeFolderId ? (folderMap.get(activeFolderId) ?? null) : null;

  const handleFolderDragStart = useCallback((event: DragStartEvent) => {
    const folderId = parseSortableFolderId(event.active.id as string);
    if (!folderId) return;
    setActiveFolderId(folderId);
  }, []);

  const handleFolderDragOver = useCallback((event: DragOverEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const activeIsFolder = parseSortableFolderId(active.id as string);
    if (!activeIsFolder) return;

    const overIsFolder = parseSortableFolderId(over.id as string);
    if (!overIsFolder) return;

    setLocalOrder((prev) => {
      const oldIndex = prev.indexOf(active.id as string);
      const newIndex = prev.indexOf(over.id as string);
      if (oldIndex === -1 || newIndex === -1) return prev;
      return arrayMove(prev, oldIndex, newIndex);
    });
  }, []);

  const handleFolderDragEnd = useCallback(
    (_event: DragEndEvent) => {
      setActiveFolderId(null);

      if (hasOrderChanged(localOrder, folderSortableIds)) {
        const realIds = localOrder
          .map((sid) => parseSortableFolderId(sid))
          .filter(Boolean) as string[];
        reorderFolders.mutate(realIds);
      }
    },
    [localOrder, folderSortableIds, reorderFolders],
  );

  const handleFolderDragCancel = useCallback(() => {
    setActiveFolderId(null);
    setLocalOrder(folderSortableIds);
  }, [folderSortableIds]);

  return {
    folderLocalOrder: localOrder,
    activeFolder,
    activeFolderId,
    handleFolderDragStart,
    handleFolderDragOver,
    handleFolderDragEnd,
    handleFolderDragCancel,
  };
}
