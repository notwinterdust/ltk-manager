# Implementation Plan: Folder Sorting

**Branch**: `008-we-recently-mod` | **Date**: 2026-03-28 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/008-we-recently-mod/spec.md`

## Summary

Enable users to sort and reorder library folders using the same sort controls and drag-and-drop patterns already used for mods. Additionally, make mods within expanded folders in list view reorderable, and improve drag handle discoverability with moderate resting opacity. The backend `reorder_folders` command and `useReorderFolders` mutation already exist. The primary work is frontend: making folders sortable items in the DnD grid, applying sort logic to folders, enabling DnD inside expanded folder rows, and improving drag handle visibility.

## Technical Context

**Language/Version**: Rust (stable) + TypeScript (strict, React 19)
**Primary Dependencies**: @dnd-kit/core, @dnd-kit/sortable, TanStack Query, Zustand, Tailwind CSS v4
**Storage**: JSON file-based (LibraryIndex with `folder_order` field)
**Testing**: Manual UI verification + `pnpm check` + `cargo clippy`
**Target Platform**: Windows/macOS/Linux (Tauri v2 desktop app)
**Project Type**: Desktop application (Tauri)
**Performance Goals**: Folder DnD reorder completes in <200ms perceived latency, no frame drops during drag
**Constraints**: Must not break existing mod DnD; same disable conditions apply to folders
**Scale/Scope**: Typically 1-20 folders per user library

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

| Principle                         | Status | Notes                                                                                         |
| --------------------------------- | ------ | --------------------------------------------------------------------------------------------- |
| I. Code Quality & Maintainability | PASS   | Barrel imports, hooks for state, no ternaries in JSX, no premature abstractions               |
| II. Type Safety & Error Handling  | PASS   | No new Tauri commands; existing `IpcResult<T>` / `Result<T, E>` patterns unchanged            |
| III. Testing Standards            | PASS   | Manual UI verification; `pnpm check` + `cargo clippy` will be run                             |
| IV. User Experience Consistency   | PASS   | Uses existing `@/components`, same DnD patterns as mods, both themes tested                   |
| V. Performance Requirements       | PASS   | Frontend-only sort; DnD uses optimistic local state; Zustand selectors for minimal re-renders |

All gates pass. No violations to justify.

## Project Structure

### Documentation (this feature)

```text
specs/008-we-recently-mod/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── modules/library/
│   ├── api/
│   │   ├── useLibraryContent.ts    # MODIFY: add folder sorting logic
│   │   ├── useRootModDnd.ts        # REFERENCE: pattern for folder DnD hook
│   │   ├── useSortableModDnd.ts    # REFERENCE: pattern for folder-internal mod DnD
│   │   ├── useMoveMod.ts           # EXISTING: useReorderFolders() already defined
│   │   └── useFolderDnd.ts         # NEW: folder DnD hook (mirrors useRootModDnd)
│   ├── components/
│   │   ├── UnifiedDndGrid.tsx      # MODIFY: make folders sortable items
│   │   ├── FolderRow.tsx           # MODIFY: add sortable mods inside expanded folders
│   │   ├── SortableModCard.tsx     # MODIFY: update drag handle opacity (resting ~30-40%)
│   │   └── SortableModList.tsx     # REFERENCE: reuse for expanded folder mods
│   └── utils/
│       └── sorting.ts              # MODIFY: add sortFolders() function
├── stores/
│   └── libraryFilter.ts            # REFERENCE: sort config (no changes needed)
└── lib/
    └── tauri.ts                    # REFERENCE: api.reorderFolders already defined
```

**Structure Decision**: All changes are within the existing `src/modules/library/` module. One new file (`useFolderDnd.ts`) follows the established pattern of per-concern DnD hooks. No backend changes required.

## Complexity Tracking

No constitution violations. Table not applicable.
