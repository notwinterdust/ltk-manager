# Tasks: Folder Sorting

**Input**: Design documents from `/specs/008-we-recently-mod/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Not requested — no test tasks generated.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Foundational (Shared Utilities & Improvements)

**Purpose**: Add sorting utility, DnD ID parsing, and drag handle discoverability improvements that all user stories depend on

- [x] T001 Add `sortFolders()` function in `src/modules/library/utils/sorting.ts` — accepts `(folders: LibraryFolder[], sort: SortConfig)`, returns folders sorted by `name` (asc/desc via `localeCompare`) when sort field is `"name"`, otherwise returns the original array unchanged (priority/installedAt/enabled keep manual order)
- [x] T002 [P] Add `SORTABLE_FOLDER_PREFIX` constant and `parseSortableFolderId()` utility in `src/modules/library/utils/dnd.ts` — follows the existing `parseFolderDropId()` pattern; prefix is `"sortable-folder:"`, parser returns the folder ID string or `null`; also add `toSortableFolderId(folderId: string)` helper; export from `src/modules/library/utils/index.ts`
- [x] T003 [P] Update drag handle resting opacity in `src/modules/library/components/SortableModCard.tsx` — change the GripVertical icon container from `opacity-0 group-hover/sortable:opacity-100` to `opacity-30 group-hover/sortable:opacity-100` so drag handles are faintly visible at rest in list view

**Checkpoint**: Sorting utility, DnD ID parsing, and drag handle discoverability ready — user story implementation can begin

---

## Phase 2: User Story 1 — Drag-and-Drop Folder Reordering (Priority: P1) 🎯 MVP

**Goal**: Users can drag folders to reorder them in priority sort mode, in both grid and list views, with the new order persisted across app restarts.

**Independent Test**: Create 3+ folders, drag one to a new position in grid view (whole-card) and list view (grip handle), verify order updates. Close and reopen app, verify order persists. Verify DnD is disabled during search, filters active, or patcher running.

### Implementation for User Story 1

- [x] T004 [US1] Create `useFolderDnd` hook in `src/modules/library/api/useFolderDnd.ts` — mirror `useRootModDnd.ts` pattern: accept `{ folders: LibraryFolder[], onReorder: (folderIds: string[]) => void }` args; track `localOrder` (folder sortable IDs using `toSortableFolderId()`), `activeFolderId`, expose `handleDragStart/Over/End/Cancel` handlers; on drag start, check if active ID is a folder via `parseSortableFolderId()` — if not, ignore (let mod handlers take over); on drag end, extract real folder IDs from `localOrder` and call `onReorder`; use `useReorderFolders()` mutation from `src/modules/library/api/useMoveMod.ts`
- [x] T005 [US1] Export `useFolderDnd` from `src/modules/library/api/index.ts` barrel export
- [x] T006 [US1] Apply folder sorting in `src/modules/library/api/useLibraryContent.ts` — wrap `orderedUserFolders` with `useMemo(() => sortFolders(orderedUserFolders, sort), [orderedUserFolders, sort])` so folders are sorted when name sort is active and returned in manual order otherwise
- [x] T007 [US1] Modify `src/modules/library/components/UnifiedDndGrid.tsx` to make folders sortable — when `dndDisabled` is false: (a) import and call `useFolderDnd` with folders and `useReorderFolders` mutation callback; (b) include folder sortable IDs (via `toSortableFolderId()`) in the `SortableContext` items array alongside root mod IDs; (c) replace `DroppableFolderCard`/`DroppableFolderRow` with new `SortableFolderCard`/`SortableFolderRow` wrappers that use `useSortable()` from @dnd-kit; (d) merge folder DnD handlers with existing root mod DnD handlers in the `DndContext` — on drag start/over/end, check ID prefix to route to the correct handler; (e) add folder drag overlay (render `FolderCard` or `FolderRow` preview based on viewMode when active item is a folder); (f) ensure existing mod-onto-folder drop behavior still works — when active is a mod and target is a sortable folder, extract real folder ID via `parseSortableFolderId()` and call `moveModToFolder`
- [x] T008 [US1] Add grip handle to folder rows in `src/modules/library/components/FolderRow.tsx` — when folder is sortable (receives DnD attributes/listeners from parent `SortableFolderRow`), render a GripVertical icon with `opacity-30 group-hover:opacity-100` pattern matching the mod drag handle style from `SortableModCard.tsx`; in grid view, the whole card is draggable (no visible handle needed, attributes on wrapper)

**Checkpoint**: Folder DnD reordering works in both grid and list views. Drag handles visible at rest in list view. Existing mod DnD unaffected.

---

## Phase 3: User Story 1b — Expanded Folder Mod Reordering in List View (Priority: P1)

**Goal**: Mods within expanded folders in list view are reorderable via drag-and-drop with grip handles.

**Independent Test**: Expand a folder in list view, drag a mod to a new position within the folder via its grip handle, verify order updates. Drag a mod to the "remove from folder" zone, verify it moves to root.

### Implementation for User Story 1b

- [x] T009 [US1] Modify `src/modules/library/components/FolderRow.tsx` to use `SortableModList` for expanded folder content — replace the current plain `ModCard` map with `SortableModList` component; pass `folderId={folder.id}` for folder-specific DnD behavior (enables "remove from folder" drop zone); pass `disabled={dndDisabled}` to fall back to plain ModCards when DnD is disabled; pass `viewMode="list"` and `onReorder` callback that calls `useReorderFolderMods()` with the folder ID; accept `dndDisabled` as a prop from the parent `UnifiedDndGrid`
- [x] T010 [US1] Pass `dndDisabled` prop through to `FolderRow` in `src/modules/library/components/UnifiedDndGrid.tsx` — both the sortable and non-sortable folder rendering paths should pass `dndDisabled` to `FolderRow` (or the wrapper that renders it) so `SortableModList` knows whether to enable DnD

**Checkpoint**: Expanded folder mods are reorderable in list view. Remove-from-folder drop zone works. DnD correctly disabled when conditions apply.

---

## Phase 4: User Story 2 — Sort Folders by Name (Priority: P2)

**Goal**: When user selects Name (A-Z) or Name (Z-A) sort, folders reorder alphabetically alongside mods.

**Independent Test**: Create folders with names like "Zeta", "Alpha", "Mango". Select Name (A-Z) — verify folders appear as Alpha, Mango, Zeta. Select Name (Z-A) — verify reverse. Switch to Priority — verify manual order restored.

### Implementation for User Story 2

- [x] T011 [US2] Verify `sortFolders()` integration in `src/modules/library/api/useLibraryContent.ts` — confirm that name sort (asc/desc) correctly reorders the folders array returned in the `unified` content view; verify that switching back to priority sort restores manual `folder_order` from backend; no new code expected if T001 and T006 are correct

**Checkpoint**: Folder name sorting works. DnD correctly disabled when name sort active.

---

## Phase 5: User Story 3 — Unified Sort Behavior (Priority: P2)

**Goal**: Folder and mod sorting behave consistently — same sort dropdown controls both, with mod-specific sorts (installedAt, enabled) not affecting folder order.

**Independent Test**: Select each sort mode and verify: Priority — both folders and mods in manual order, DnD enabled. Name — both sorted alphabetically, DnD disabled. Installed At / Enabled First — mods sort by field, folders keep manual order, DnD disabled.

### Implementation for User Story 3

- [x] T012 [US3] Verify unified behavior across all sort modes in `src/modules/library/api/useLibraryContent.ts` — confirm `sortFolders()` returns original order for `installedAt` and `enabled` sort fields while `sortModsByFolder()` still sorts mods within folders for those fields; verify root section stays in fixed position regardless of sort
- [x] T013 [US3] Verify edge case: folder rename while sorted by name — rename a folder and confirm the sorted view updates to reflect the new name (TanStack Query invalidation via the existing `useRenameFolder` mutation should handle this)

**Checkpoint**: All sort modes behave correctly for both folders and mods.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final verification and cleanup

- [x] T014 Run `pnpm check` (typecheck + lint + format) and fix any issues
- [x] T015 Run quickstart.md manual verification checklist — test all scenarios in both grid and list views, dark and light themes, with and without root-level mods, with expanded and collapsed folders

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 1)**: No dependencies — start immediately
- **US1 DnD (Phase 2)**: Depends on Phase 1 (T001, T002, T003)
- **US1b Expanded (Phase 3)**: Depends on Phase 2 (T007 wires DnD in UnifiedDndGrid)
- **US2 (Phase 4)**: Depends on Phase 2 (T006 applies sorting in useLibraryContent)
- **US3 (Phase 5)**: Depends on Phase 2 (same sorting integration)
- **Polish (Phase 6)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (P1)**: Depends on foundational utilities only — delivers folder DnD reordering + expanded folder mod DnD
- **US2 (P2)**: Depends on US1 (T006 integrates sortFolders into useLibraryContent) — adds name sorting
- **US3 (P2)**: Depends on US1 (same integration) — verifies unified behavior across all sort modes

### Within Phase 1

- T001, T002, and T003 can all run in parallel [P] — different files, no dependencies

### Parallel Opportunities

- T001, T002, T003 can run in parallel (different files)
- T004 and T005 are sequential (create hook then export)
- T006 is independent of T004/T005 (different file)
- T007 depends on T004/T005 (uses the hook) and T006 (uses sorted folders)
- T008 depends on T007 (FolderRow receives DnD props from UnifiedDndGrid)
- T009 depends on T007 (needs dndDisabled prop flow)
- T011, T012, T013 are verification tasks that can run after prerequisites

---

## Parallel Example: Phase 1

```bash
# Launch all foundational tasks together:
Task: "Add sortFolders() in src/modules/library/utils/sorting.ts"
Task: "Add parseSortableFolderId() in src/modules/library/utils/dnd.ts"
Task: "Update drag handle opacity in src/modules/library/components/SortableModCard.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Foundational utilities (T001-T003)
2. Complete Phase 2: US1 — folder DnD reordering (T004-T008)
3. Complete Phase 3: US1b — expanded folder mod reordering (T009-T010)
4. **STOP and VALIDATE**: Drag folders to reorder in both views, drag mods in expanded folders, verify persistence, verify DnD disable conditions, verify drag handle visibility
5. This alone delivers the core value — folder reordering + expanded folder mod DnD + improved handle discoverability

### Incremental Delivery

1. Foundational → utilities ready
2. US1 → folder DnD reordering → validate (MVP!)
3. US1b → expanded folder mod DnD → validate
4. US2 → name sorting → validate
5. US3 → unified sort verification → validate
6. Polish → final checks

---

## Notes

- No backend changes needed — `reorder_folders` and `reorder_folder_mods` commands already exist
- `useReorderFolders()` mutation already exists in `src/modules/library/api/useMoveMod.ts`
- `useReorderFolderMods()` mutation already exists in `src/modules/library/api/useMoveMod.ts`
- `api.reorderFolders` and `api.reorderFolderMods` already defined in `src/lib/tauri.ts`
- `SortableModList` component already handles full DnD for mods including "remove from folder" zone
- Folder sortable IDs use `"sortable-folder:{id}"` prefix to distinguish from mod IDs and folder drop zone IDs (`"folder:{id}"`)
- Existing `dnd.ts` has `parseFolderDropId()` — add `parseSortableFolderId()` alongside it
- Root/ungrouped mods section stays in fixed position regardless of sort
