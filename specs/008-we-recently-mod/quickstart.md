# Quickstart: Folder Sorting

**Feature**: 008-we-recently-mod | **Date**: 2026-03-28

## Overview

Add folder sorting and drag-and-drop reordering to the library view. Folders should be sortable using the same sort dropdown that controls mod sorting, and manually reorderable via drag-and-drop in priority sort mode. Additionally, improve drag handle discoverability in list view and enable mod reordering within expanded folders in list view.

## Key Files to Modify

| File                                                 | Change                                                                         |
| ---------------------------------------------------- | ------------------------------------------------------------------------------ |
| `src/modules/library/utils/sorting.ts`               | Add `sortFolders()` function                                                   |
| `src/modules/library/api/useLibraryContent.ts`       | Apply folder sorting to `orderedUserFolders`                                   |
| `src/modules/library/components/UnifiedDndGrid.tsx`  | Make folders sortable items in DnD context                                     |
| `src/modules/library/components/FolderRow.tsx`       | Use `SortableModList` for expanded folder mods; add grip handle for folder DnD |
| `src/modules/library/components/SortableModCard.tsx` | Change drag handle resting opacity from 0% to ~30-40%                          |
| `src/modules/library/api/useFolderDnd.ts`            | **NEW** — DnD hook for folder reordering (mirrors `useRootModDnd`)             |
| `src/modules/library/api/index.ts`                   | Export new hook                                                                |

## Key Files to Reference (Read-Only)

| File                                                 | Why                                           |
| ---------------------------------------------------- | --------------------------------------------- |
| `src/modules/library/api/useRootModDnd.ts`           | Pattern to follow for folder DnD hook         |
| `src/modules/library/api/useSortableModDnd.ts`       | Pattern for folder-internal mod DnD           |
| `src/modules/library/api/useMoveMod.ts`              | `useReorderFolders()` mutation already exists |
| `src/modules/library/components/SortableModList.tsx` | Reuse for expanded folder mods in list view   |
| `src/stores/libraryFilter.ts`                        | SortConfig type and store                     |
| `src/lib/tauri.ts`                                   | `api.reorderFolders` already defined          |

## Implementation Order

1. **SortableModCard.tsx** — Update drag handle resting opacity (~30-40%)
2. **sorting.ts** — Add `sortFolders()` utility
3. **useLibraryContent.ts** — Apply folder sorting in the content hook
4. **useFolderDnd.ts** — Create folder DnD hook
5. **UnifiedDndGrid.tsx** — Wire up folder sortable items and DnD
6. **FolderRow.tsx** — Add sortable mods inside expanded folders + folder grip handle for list view

## No Backend Changes

The `reorder_folders` Tauri command, `folder_order` persistence, and `useReorderFolders` mutation all exist. This is a frontend-only feature.

## Verification

1. `pnpm check` — typecheck + lint + format
2. `cargo clippy -p ltk-manager` — should have no changes but verify
3. Manual testing:
   - **Grid view:**
     - Drag folders (whole-card) to reorder in priority sort mode
     - Drag mods onto folders to move them
     - Verify existing mod DnD still works
   - **List view:**
     - Verify drag handles visible at rest (~30-40% opacity) for both folders and mods
     - Drag handles reach full opacity on hover
     - Drag folders via grip handle to reorder
     - Expand folder → drag mods within folder to reorder
     - Expand folder → drag mod to "remove from folder" zone
   - **Sort modes:**
     - Name (A-Z) / Name (Z-A) — folders sort alphabetically
     - Installed At / Enabled First — folders keep manual order, mods sort
     - Priority — manual order, DnD enabled
   - **DnD disable conditions:**
     - Search active → DnD disabled
     - Filters active → DnD disabled
     - Patcher running → DnD disabled
     - Non-priority sort → DnD disabled
   - **Persistence:**
     - Reorder folders → close app → reopen → order preserved
   - **Themes:** Test in both dark and light themes
