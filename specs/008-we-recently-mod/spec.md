# Feature Specification: Folder Sorting

**Feature Branch**: `008-we-recently-mod`
**Created**: 2026-03-28
**Status**: Draft
**Input**: User description: "We recently did mod folders in the library, however we also need to let the users be able to sort them just like the mods inside."

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Drag-and-Drop Folder Reordering (Priority: P1)

A user wants to manually arrange their mod folders in a specific order that makes sense for their workflow. They drag a folder to a new position in the library view, and the folder order updates immediately with smooth visual feedback.

**Why this priority**: Manual reordering is the most fundamental sorting interaction and mirrors the existing mod drag-and-drop behavior. It gives users direct control over folder arrangement and is the natural extension of the existing "priority" sort mode.

**Independent Test**: Can be fully tested by dragging a folder from one position to another and verifying the new order persists across app restarts.

**Acceptance Scenarios**:

1. **Given** a library with 3+ folders in priority sort mode, **When** the user drags a folder to a new position, **Then** the folder moves to the target position and all other folders shift accordingly.
2. **Given** a folder has been reordered via drag-and-drop, **When** the user closes and reopens the app, **Then** the folder order is preserved.
3. **Given** the patcher is running, **When** the user attempts to drag a folder, **Then** drag-and-drop is disabled (consistent with existing mod DnD behavior).
4. **Given** the user is searching or has active filters, **When** they view the library, **Then** folder drag-and-drop is disabled (consistent with existing mod DnD behavior).
5. **Given** the library is in list view with priority sort, **When** the user hovers over a folder row, **Then** a drag handle appears with improved discoverability (larger hover zone, subtle hint styling).
6. **Given** a folder is expanded in list view with priority sort, **When** the user drags a mod within that folder via its grip handle, **Then** the mod reorders within the folder.

---

### User Story 2 - Sort Folders by Name (Priority: P2)

A user with many folders wants to quickly find folders by sorting them alphabetically. They select a name-based sort option and all folders reorder alphabetically (A-Z or Z-A), mirroring the existing mod sorting behavior.

**Why this priority**: Alphabetical sorting is the most universally expected sorting mode after manual ordering, and it directly mirrors the existing mod sorting experience.

**Independent Test**: Can be tested by creating folders with different names, selecting "Name (A-Z)" sort, and verifying folders appear in alphabetical order.

**Acceptance Scenarios**:

1. **Given** a library with multiple folders, **When** the user selects "Name (A-Z)" sort, **Then** folders are displayed in ascending alphabetical order.
2. **Given** folders are sorted by name, **When** the user switches to "Name (Z-A)", **Then** folders are displayed in descending alphabetical order.
3. **Given** folders are sorted by name, **When** the user switches back to "Priority" sort, **Then** folders return to their manual drag-and-drop order.

---

### User Story 3 - Unified Sort Behavior (Priority: P2)

When a user changes the sort mode, both mods and folders sort consistently using the same criteria where applicable. The sort dropdown applies its setting to both folders and the mods within them, providing a coherent experience.

**Why this priority**: Consistency between folder and mod sorting prevents user confusion and matches the expectation set by the feature description ("sort them just like the mods inside").

**Independent Test**: Can be tested by changing sort mode and verifying both folders and their contained mods reflect the same sort criteria.

**Acceptance Scenarios**:

1. **Given** the user selects "Name (A-Z)" sort, **When** the library renders, **Then** both folders and mods within each folder are sorted alphabetically.
2. **Given** the user selects "Priority" sort, **When** the library renders, **Then** both folders and mods appear in their manual drag-and-drop order.
3. **Given** the user selects "Enabled First" sort, **When** the library renders, **Then** mods within folders are sorted by enabled status while folders retain their manual order (enabled sort is mod-specific).

---

### Edge Cases

- What happens when a folder is renamed while sorted by name? The folder list should re-sort to reflect the new name.
- What happens when the root (ungrouped) mods section exists alongside folders? The root section should remain in a fixed position regardless of folder sort order.
- What happens with a single folder? Sorting should work without errors, though the visual effect is negligible.
- How does folder sorting interact with "Installed At" and "Enabled First" sort modes? These are mod-specific attributes — folders retain their manual order when these sorts are active.

## Clarifications

### Session 2026-03-28

- Q: Should list view drag handles be always visible or hover-revealed? → A: Keep hover-reveal pattern but improve discoverability (larger hover zone, subtle hint styling).
- Q: Should folders in grid view use whole-card drag or grip handle? → A: Whole-card drag in grid view (match mod behavior), grip handle in list view.
- Q: Should mods within expanded folders in list view be reorderable via DnD? → A: Yes, draggable with grip handles (same pattern as root mods in list view).
- Q: What specific discoverability improvement for hover-reveal drag handles? → A: Faint grip dots visible at rest (~30-40% opacity), full opacity on hover.

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: System MUST allow users to reorder folders via drag-and-drop when in "Priority" sort mode.
- **FR-002**: System MUST sort folders alphabetically (ascending or descending) when the user selects "Name" sort mode.
- **FR-003**: System MUST persist manually-set folder order across application restarts.
- **FR-004**: System MUST disable folder drag-and-drop when search is active, filters are applied, patcher is running, or a non-priority sort mode is selected (consistent with existing mod DnD constraints).
- **FR-005**: System MUST keep the root/ungrouped mods section in a fixed position regardless of folder sort order.
- **FR-006**: System MUST apply folder sorting through the same sort dropdown that controls mod sorting (no separate folder sort control).
- **FR-007**: When sort mode is "Installed At" or "Enabled First" (mod-specific sorts), folders MUST retain their manual (priority) order — only mods within folders are affected.
- **FR-008**: In list view, drag handles for both folders and mods MUST be visible at rest at moderate opacity (~30-40%) and transition to full opacity on hover, providing a persistent visual hint of draggability.
- **FR-009**: Folder drag handles in list view MUST follow the same interaction pattern as mod drag handles (GripVertical icon, hover-revealed).
- **FR-010**: In grid view, folders MUST be draggable by the entire card (no visible grip handle), matching existing mod card drag behavior.
- **FR-011**: Mods within expanded folders in list view MUST be reorderable via drag-and-drop using the same grip handle pattern as root-level mods in list view.

### Key Entities

- **LibraryFolder**: A named group of mods with an ID, name, and ordered list of mod IDs. Has a display position determined by the active sort mode.
- **Folder Order**: The persisted sequence of folder IDs representing the user's manual arrangement. Serves as the canonical order when in "Priority" sort mode.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Users can reorder folders via drag-and-drop with the same interaction pattern used for mods, completing the action in under 2 seconds.
- **SC-002**: Folder sort options (priority, name A-Z, name Z-A) are accessible from the existing sort dropdown with no additional UI controls needed.
- **SC-003**: 100% of existing mod sorting and drag-and-drop functionality continues to work unchanged after this feature is added.
- **SC-004**: Folder order changes persist reliably across application restarts.

## Assumptions

- The existing backend folder reorder command is functional and can be wired into the UI without backend changes.
- The existing drag-and-drop infrastructure can be extended to support folder dragging alongside mod dragging.
- Folder sorting by name is purely a frontend concern — the backend only stores the manual/priority order.
- The root/ungrouped mods section is not a sortable folder — it stays in a fixed position.
- Sort mode applies globally to the library view (not per-folder).
