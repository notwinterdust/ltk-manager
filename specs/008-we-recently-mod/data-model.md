# Data Model: Folder Sorting

**Feature**: 008-we-recently-mod | **Date**: 2026-03-28

## Existing Entities (No Changes)

### LibraryFolder

| Field  | Type     | Description                               |
| ------ | -------- | ----------------------------------------- |
| id     | string   | UUID identifier (or "root" for ungrouped) |
| name   | string   | User-defined display name                 |
| modIds | string[] | Ordered list of mod IDs in this folder    |

### LibraryIndex (Backend)

| Field             | Type              | Description                      |
| ----------------- | ----------------- | -------------------------------- |
| folders           | LibraryFolder[]   | All folders including root       |
| folder_order      | string[]          | Persisted manual folder ordering |
| mods              | LibraryModEntry[] | All installed mods               |
| profiles          | Profile[]         | Mod profiles                     |
| active_profile_id | string            | Currently active profile         |

### SortConfig (Frontend State)

| Field     | Type          | Description                                        |
| --------- | ------------- | -------------------------------------------------- |
| field     | SortField     | "priority" \| "name" \| "installedAt" \| "enabled" |
| direction | SortDirection | "asc" \| "desc"                                    |

## New Types (Frontend Only)

### Sort Applicability to Folders

| Sort Field  | Applies to Folders? | Folder Behavior                                    |
| ----------- | ------------------- | -------------------------------------------------- |
| priority    | Yes                 | Use `folder_order` from backend (manual DnD order) |
| name        | Yes                 | Sort alphabetically by `folder.name`               |
| installedAt | No                  | Folders keep manual order                          |
| enabled     | No                  | Folders keep manual order                          |

### DnD Behavior by View Mode

| View Mode | Folders                                             | Root Mods                                           | Expanded Folder Mods                            |
| --------- | --------------------------------------------------- | --------------------------------------------------- | ----------------------------------------------- |
| Grid      | Whole-card drag                                     | Whole-card drag                                     | N/A (folders not expandable in grid)            |
| List      | Grip handle (hover-reveal, ~30-40% resting opacity) | Grip handle (hover-reveal, ~30-40% resting opacity) | Grip handle (same pattern, via SortableModList) |

No new backend entities, database fields, or API contracts are needed. The data model is unchanged — this feature only changes how existing data is presented and reordered in the frontend.

## State Transitions

### Folder Order State

```
Manual Order (priority sort)
  ──[user drags folder]──> New Manual Order (persisted via reorderFolders API)
  ──[user selects name sort]──> Alphabetical View Order (frontend-only, manual order preserved)
  ──[user selects installedAt/enabled sort]──> Manual Order (mod-specific sorts don't affect folders)
  ──[user switches back to priority]──> Manual Order (restored from backend)
```

### DnD State

```
Idle
  ──[pointer down + 8px move on handle/card]──> Dragging Folder
  ──[drop on valid position]──> Reorder (call useReorderFolders)
  ──[drop on invalid / cancel]──> Reset to original order
```

### Expanded Folder Mod DnD State

```
Idle (folder expanded in list view)
  ──[pointer down + 8px move on grip handle]──> Dragging Mod
  ──[drop on position within folder]──> Reorder (call useReorderFolderMods)
  ──[drop on "remove from folder" zone]──> Move to root
  ──[drop on invalid / cancel]──> Reset to original order
```
