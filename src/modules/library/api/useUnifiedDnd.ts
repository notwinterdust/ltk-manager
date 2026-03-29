import {
  closestCenter,
  type CollisionDetection,
  type DragEndEvent,
  type DragOverEvent,
  type DragStartEvent,
  pointerWithin,
} from "@dnd-kit/core";
import { useCallback, useMemo, useState } from "react";

import type { InstalledMod, LibraryFolder } from "@/lib/tauri";
import {
  parseSortableFolderId,
  REMOVE_FROM_FOLDER_ID,
  resolveFolderId,
} from "@/modules/library/utils";

import { useFolderDnd } from "./useFolderDnd";
import { useMoveModToFolder, useReorderFolderMods } from "./useMoveMod";
import { useRootModDnd } from "./useRootModDnd";

interface UseUnifiedDndArgs {
  folders: LibraryFolder[];
  rootMods: InstalledMod[];
  modsByFolder: Map<string, InstalledMod[]>;
  onReorder: (modIds: string[]) => void;
}

export function useUnifiedDnd({ folders, rootMods, modsByFolder, onReorder }: UseUnifiedDndArgs) {
  const {
    localOrder: modLocalOrder,
    orderedRootMods,
    activeMod,
    handleDragStart: handleModDragStart,
    handleDragOver: handleModDragOver,
    handleDragEnd: handleModDragEnd,
    handleDragCancel: handleModDragCancel,
  } = useRootModDnd({ rootMods, onReorder });

  const {
    folderLocalOrder,
    activeFolder,
    handleFolderDragStart,
    handleFolderDragOver,
    handleFolderDragEnd,
    handleFolderDragCancel,
  } = useFolderDnd({ folders });

  const moveModToFolder = useMoveModToFolder();
  const reorderFolderMods = useReorderFolderMods();

  const [activeFolderMod, setActiveFolderMod] = useState<InstalledMod | null>(null);
  const [activeFolderModSource, setActiveFolderModSource] = useState<string | null>(null);

  const folderModLookup = useMemo(() => {
    const map = new Map<string, { mod: InstalledMod; folderId: string }>();
    for (const [folderId, mods] of modsByFolder) {
      if (folderId === "root") continue;
      for (const mod of mods) {
        map.set(mod.id, { mod, folderId });
      }
    }
    return map;
  }, [modsByFolder]);

  const isDraggingMod = !!activeMod;
  const isDraggingFolder = !!activeFolder;
  const isDraggingFolderMod = !!activeFolderMod;
  const activeModForOverlay = activeMod ?? activeFolderMod;

  const sortableItems = useMemo(() => {
    if (isDraggingMod) return modLocalOrder;
    if (isDraggingFolder) return folderLocalOrder;
    return [...folderLocalOrder, ...modLocalOrder];
  }, [folderLocalOrder, modLocalOrder, isDraggingMod, isDraggingFolder]);

  const handleDragStart = useCallback(
    (event: DragStartEvent) => {
      const id = event.active.id as string;
      if (parseSortableFolderId(id)) {
        handleFolderDragStart(event);
        return;
      }
      const folderMod = folderModLookup.get(id);
      if (folderMod) {
        setActiveFolderMod(folderMod.mod);
        setActiveFolderModSource(folderMod.folderId);
        return;
      }
      handleModDragStart(event);
    },
    [handleFolderDragStart, handleModDragStart, folderModLookup],
  );

  const handleDragOver = useCallback(
    (event: DragOverEvent) => {
      const id = event.active.id as string;
      if (parseSortableFolderId(id)) {
        handleFolderDragOver(event);
        return;
      }
      if (folderModLookup.has(id)) {
        return;
      }
      handleModDragOver(event);
    },
    [handleFolderDragOver, handleModDragOver, folderModLookup],
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const id = event.active.id as string;

      if (parseSortableFolderId(id)) {
        handleFolderDragEnd(event);
        return;
      }

      if (activeFolderMod && activeFolderModSource) {
        const overId = event.over?.id as string | undefined;
        setActiveFolderMod(null);
        setActiveFolderModSource(null);

        if (overId) {
          if (overId === REMOVE_FROM_FOLDER_ID) {
            moveModToFolder.mutate({ modId: id, folderId: "root" });
            return;
          }
          const targetFolderId = resolveFolderId(overId);
          if (targetFolderId && targetFolderId !== activeFolderModSource) {
            moveModToFolder.mutate({ modId: id, folderId: targetFolderId });
            return;
          }

          const overFolderMod = folderModLookup.get(overId);
          if (overFolderMod && overFolderMod.folderId === activeFolderModSource) {
            const currentOrder = (modsByFolder.get(activeFolderModSource) ?? []).map((m) => m.id);
            const oldIndex = currentOrder.indexOf(id);
            const newIndex = currentOrder.indexOf(overId);
            if (oldIndex !== -1 && newIndex !== -1 && oldIndex !== newIndex) {
              const newOrder = [...currentOrder];
              newOrder.splice(oldIndex, 1);
              newOrder.splice(newIndex, 0, id);
              reorderFolderMods.mutate({ folderId: activeFolderModSource, modIds: newOrder });
            }
          }
        }
        return;
      }

      handleModDragEnd(event);
    },
    [
      handleFolderDragEnd,
      handleModDragEnd,
      activeFolderMod,
      activeFolderModSource,
      folderModLookup,
      modsByFolder,
      moveModToFolder,
      reorderFolderMods,
    ],
  );

  const handleDragCancel = useCallback(() => {
    handleFolderDragCancel();
    handleModDragCancel();
    setActiveFolderMod(null);
    setActiveFolderModSource(null);
  }, [handleFolderDragCancel, handleModDragCancel]);

  const collisionDetection: CollisionDetection = useCallback(
    (args) => {
      const activeId = args.active.id as string;

      if (parseSortableFolderId(activeId)) {
        return closestCenter(args);
      }

      const removeHit = pointerWithin(args).find((c) => c.id === REMOVE_FROM_FOLDER_ID);
      if (removeHit) return [removeHit];

      const activeSourceFolderId = folderModLookup.get(activeId)?.folderId;

      if (activeSourceFolderId) {
        const withoutSource = args.droppableContainers.filter(
          (c) => c.id !== `sortable-folder:${activeSourceFolderId}`,
        );
        return closestCenter({ ...args, droppableContainers: withoutSource });
      }

      return closestCenter(args);
    },
    [folderModLookup],
  );

  return {
    folderLocalOrder,
    orderedRootMods,
    activeFolder,
    activeModForOverlay,
    isDraggingMod,
    isDraggingFolder,
    isDraggingFolderMod,
    sortableItems,
    collisionDetection,
    handleDragStart,
    handleDragOver,
    handleDragEnd,
    handleDragCancel,
  };
}
