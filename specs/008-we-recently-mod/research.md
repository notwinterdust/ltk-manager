# Research: Folder Sorting

**Feature**: 008-we-recently-mod | **Date**: 2026-03-28

## R1: Can folders and mods coexist in the same SortableContext?

**Decision**: Use a single `SortableContext` with prefixed IDs to distinguish folders from mods.

**Rationale**: @dnd-kit's `SortableContext` accepts any string IDs. By prefixing folder IDs (e.g., `"sortable-folder:{id}"`) and keeping mod IDs unprefixed, the drag handlers can distinguish between folder drags and mod drags. This avoids needing nested DnD contexts which add complexity and collision detection issues.

**Alternatives considered**:

- **Separate DnD contexts for folders and mods**: Rejected — would prevent drag interactions between them (e.g., dragging a mod onto a folder) and duplicate sensor setup.
- **Two `SortableContext`s in one `DndContext`**: Possible but adds complexity for collision detection — items from both contexts can collide. Single context with ID prefixing is cleaner.

## R2: How to handle folder sorting alongside mod sorting in the same view?

**Decision**: Apply folder sort separately from mod sort in `useLibraryContent`. The `sortFolders()` function handles folder-level ordering, while `sortModsByFolder()` continues to handle mod-level ordering within each folder.

**Rationale**: Folder sorting only applies to the "name" sort field. For "priority", folders use `folder_order` from the backend. For "installedAt" and "enabled", these are mod-specific fields that don't exist on folders — folders keep their manual order. This maps naturally to a separate `sortFolders()` utility.

**Alternatives considered**:

- **Unified sort function for both folders and mods**: Rejected — folders and mods have different fields (folders lack `installedAt`, `enabled`), so a shared comparator would need type guards and special cases.

## R3: Backend changes needed?

**Decision**: No backend changes required.

**Rationale**: The `reorder_folders` command already exists and handles persistence of manual folder order. Frontend-only name sorting doesn't need backend support — it's a view-layer concern. The existing `folder_order` field in `LibraryIndex` stores the canonical manual order, and the frontend can sort folders by name in a `useMemo` without touching the backend.

## R4: How to distinguish folder drags from mod drags in handlers?

**Decision**: Use ID prefixing with parser utilities. Folder sortable IDs use `"sortable-folder:{id}"` pattern. A `parseSortableFolderId()` utility extracts the folder ID, returning `null` for non-folder items.

**Rationale**: This follows the existing `parseFolderDropId()` pattern used for folder drop zones (`"folder:{id}"`). Adding a parallel prefix for sortable folders keeps the patterns consistent.

**Alternatives considered**:

- **Data attribute on drag event**: @dnd-kit doesn't natively support typed data on sortable items without wrapping. ID parsing is simpler and already established.

## R5: Interaction between folder DnD and mod-onto-folder drops

**Decision**: When a mod is dragged over a sortable folder, the folder acts as a drop target (move mod into folder). When a folder is dragged, it reorders among other folders only. These are distinguished by checking the active item's ID prefix.

**Rationale**: The `handleDragEnd` handler can check: if active item is a folder → reorder folders; if active item is a mod and target is a folder → move mod to folder; if active item is a mod and target is a mod → reorder mods. This preserves existing behavior while adding folder reordering.

## R6: How to make expanded folder mods sortable in list view?

**Decision**: Reuse the existing `SortableModList` component inside `FolderRow` when DnD is enabled, replacing the current plain `ModCard` rendering.

**Rationale**: `SortableModList` already wraps `useSortableModDnd` with full DnD UI (SortableModCard with grip handles, DndContext, SortableContext, DragOverlay). It accepts a `folderId` prop for folder-specific behavior (including "remove from folder" drop zone). It also accepts a `disabled` prop to fall back to plain ModCards. This is the exact component used in the folder drilldown view.

**Alternatives considered**:

- **Nested SortableContext inside the parent DndContext**: Rejected — @dnd-kit supports nested contexts but collision detection becomes complex. Using `SortableModList` which creates its own `DndContext` avoids this.
- **New `SortableFolderRow` component**: Unnecessary — `FolderRow` just needs to conditionally render `SortableModList` instead of plain `ModCard` list.

## R7: Drag handle discoverability in list view

**Decision**: Change existing drag handle opacity from `opacity-0` (invisible at rest) to `opacity-30` (~30-40% at rest), keeping `opacity-100` on hover. Apply to both mod and folder drag handles.

**Rationale**: User feedback indicated drag handles were not discoverable. The hover-reveal pattern is standard (VS Code, Linear) but needs a persistent visual hint. A faint grip icon at rest communicates draggability without cluttering the UI.

**Alternatives considered**:

- **Always fully visible**: Rejected — adds visual noise in a dense list view.
- **Visible only in priority sort mode**: Rejected — more complex state management for marginal benefit; DnD is already disabled in non-priority modes so the handle won't respond anyway.

## R8: Grid vs list view DnD behavior for folders

**Decision**: Folders follow the same pattern as mods — whole-card drag in grid view, grip handle in list view.

**Rationale**: Consistency with existing mod behavior. In grid view, users expect to grab anywhere on a card. In list view, grip handles prevent accidental drags from click interactions on the row's other controls (expand/collapse, checkbox).
