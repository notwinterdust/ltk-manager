# Tasks: LTK Protocol Install

**Input**: Design documents from `/specs/001-ltk-protocol-install/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/protocol-schema.md

**Tests**: Not explicitly requested. No test tasks included.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add plugin dependencies and configure the `ltk://` protocol scheme

- [x] T001 Add `tauri-plugin-deep-link` and `tauri-plugin-single-instance` Rust dependencies in `src-tauri/Cargo.toml`
- [x] T002 Add `@tauri-apps/plugin-deep-link` npm package via pnpm
- [x] T003 Add deep-link plugin configuration with `"desktop": { "schemes": ["ltk"] }` to the `plugins` section in `src-tauri/tauri.conf.json`
- [x] T004 Add deep-link and single-instance plugin permissions to Tauri capabilities in `src-tauri/capabilities/default.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core backend infrastructure that MUST be complete before user story implementation

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Register `tauri-plugin-single-instance` in `src-tauri/src/main.rs` — must be registered BEFORE deep-link plugin. The single-instance callback should extract deep-link URLs from CLI args and emit a `"deep-link-install"` event to the frontend via `app.emit()`
- [x] T006 Register `tauri-plugin-deep-link` in `src-tauri/src/main.rs` — wire `on_open_url` callback to parse and emit `"deep-link-install"` events. On startup, also check `deep_link::get_current()` for URLs received at launch
- [x] T007 Create `src-tauri/src/deep_link/mod.rs` — implement `DeepLinkInstallRequest` struct (fields: `url: String`, `name: Option<String>`, `source: Option<String>`) with `Serialize`/`Deserialize`. Implement `parse_deep_link_url(raw_url: &str) -> AppResult<DeepLinkInstallRequest>` that validates: scheme is `ltk`, action is `install`, `url` param is present and uses `https` scheme, `name`/`source` are URL-decoded and max 256 chars, `url` host is not loopback/private IP. Unknown params silently ignored.
- [x] T008 Implement rate limiting in `src-tauri/src/deep_link/mod.rs` — add `DeepLinkState` struct with `last_invocation: Mutex<Option<Instant>>` and `should_rate_limit(&self) -> bool` method (returns true if <1 second since last accepted invocation). Register as Tauri managed state in `src-tauri/src/main.rs`
- [x] T009 Create `src-tauri/src/deep_link/download.rs` — implement `download_mod_file(url: &str, app_handle: &AppHandle) -> AppResult<PathBuf>` that: downloads via `reqwest` to `std::env::temp_dir()/{uuid}.{ext}` (infer extension from URL path, default to `.modpkg`), emits `"protocol-install-progress"` events with `ProtocolInstallProgress` payload during download (stage: downloading/validating/installing/complete/error, bytes_downloaded, total_bytes from Content-Length), validates downloaded file is non-empty with valid `.modpkg` or `.fantome` extension, returns temp file path
- [x] T010 Export `deep_link` module from `src-tauri/src/deep_link/mod.rs` — ensure `mod.rs` re-exports `DeepLinkInstallRequest`, `DeepLinkState`, `ProtocolInstallProgress`, `parse_deep_link_url`, and `download_mod_file`

**Checkpoint**: Foundation ready — deep-link URLs are received, parsed, validated, rate-limited, and files can be downloaded. User story implementation can now begin.

---

## Phase 3: User Story 1 — One-Click Mod Install from Website (Priority: P1) MVP

**Goal**: Users click an `ltk://install?url=...` link on a website and the mod is installed in LTK Manager after confirmation.

**Independent Test**: Click an `ltk://install?url=https://example.com/test.modpkg&name=Test&source=TestSite` link in a browser. App should launch (or focus), show confirmation dialog, and install the mod on confirm.

### Implementation for User Story 1

- [x] T011 [US1] Create Tauri command `deep_link_install_mod` in `src-tauri/src/commands/deep_link.rs` — accepts `url: String`, `name: Option<String>`, `source: Option<String>`. Rejects if patcher is running (reuse `reject_if_patcher_running` pattern from `commands/mods.rs`). Calls `download_mod_file()` to download to temp, then calls `ModLibrary::install_mod_from_package()` with the temp path, cleans up temp file, returns `IpcResult<InstalledMod>`
- [x] T012 [US1] Export `deep_link` command module in `src-tauri/src/commands/mod.rs` and register `deep_link_install_mod` in the `generate_handler![]` macro in `src-tauri/src/main.rs`
- [x] T013 [P] [US1] Add TypeScript types and API binding in `src/lib/tauri.ts` — add `DeepLinkInstallRequest` type (`{ url: string; name: string | null; source: string | null }`), `ProtocolInstallProgress` type (`{ stage: string; bytes_downloaded: number; total_bytes: number | null; error: string | null }`), and `api.deepLinkInstallMod` method calling `invokeResult<InstalledMod>("deep_link_install_mod", { url, name, source })`
- [x] T014 [P] [US1] Create Zustand store in `src/stores/deepLink.ts` — implement `PendingProtocolInstall` state with fields: `request: DeepLinkInstallRequest | null`, `status: "idle" | "downloading" | "installing" | "complete" | "error"`, `progress: ProtocolInstallProgress | null`, `error: string | null`. Actions: `setRequest(req)`, `setStatus(status)`, `setProgress(progress)`, `setError(error)`, `reset()`. Export from `src/stores/index.ts`
- [x] T015 [US1] Create `src/modules/deep-link/api/useDeepLinkListener.ts` — hook that uses `listen<DeepLinkInstallRequest>()` from `@tauri-apps/api/event` to listen for `"deep-link-install"` events. On event, calls `useDeepLinkStore.setRequest(payload)`. Include cleanup via `unlisten()` in useEffect return. This hook should be mounted in the root layout (`src/routes/__root.tsx`)
- [x] T016 [US1] Create `src/modules/deep-link/api/useProtocolInstall.ts` — TanStack `useMutation` hook that calls `api.deepLinkInstallMod(url, name, source)`. On success: update installed mods query cache (same pattern as `useInstallMod`), show success toast via `useToast()`, reset deep-link store. On error: set error in store, show error toast
- [x] T017 [US1] Create `src/modules/deep-link/api/useProtocolInstallProgress.ts` — hook that listens for `"protocol-install-progress"` Tauri event and updates the deep-link store's `progress` and `status` fields based on the `stage` value
- [x] T018 [US1] Create `src/modules/deep-link/components/ProtocolInstallDialog.tsx` — confirmation dialog using `Dialog` from `@/components`. Opens when `useDeepLinkStore.request` is non-null. Displays: mod name (from `request.name` or "Unknown Mod"), source site (from `request.source` or extract domain from URL), download URL domain. Shows download progress bar when status is `"downloading"`. Footer has Cancel and Install buttons. Cancel calls `store.reset()`. Install calls `useProtocolInstall` mutation. Loading state on Install button during download/install. Error state shows error message with dismiss
- [x] T019 [US1] Create barrel exports: `src/modules/deep-link/api/index.ts`, `src/modules/deep-link/components/index.ts`, `src/modules/deep-link/index.ts` — export all hooks and components through the module barrel
- [x] T020 [US1] Mount `useDeepLinkListener` hook and render `ProtocolInstallDialog` in `src/routes/__root.tsx` — import from `@/modules/deep-link`. The listener must be active on all routes so deep links work regardless of which page the user is on. The dialog renders as a modal overlay

**Checkpoint**: User Story 1 is fully functional. Users can click `ltk://install` links and install mods after confirmation. Test with both app-not-running and app-already-running scenarios.

---

## Phase 4: User Story 2 — Domain Allowlist (Priority: P3)

**Goal**: Users can optionally restrict protocol installs to a list of trusted domains.

**Independent Test**: Enable domain allowlist in settings, add `runeforge.dev`, click a link with a different domain — should be blocked. Click a link from `runeforge.dev` — should proceed normally.

### Implementation for User Story 2

- [x] T021 [US2] Add `trusted_domains: Vec<String>` field to the `Settings` struct in `src-tauri/src/state.rs` — defaults to `["runeforge.dev", "divineskins.gg"]`. When non-empty, the deep-link handler checks the download URL's host against the list. Add `#[serde(default)]` for backwards compatibility with existing settings.json files
- [x] T022 [US2] Add domain allowlist check to the deep-link handler in `src-tauri/src/main.rs` — before emitting the `"deep-link-install"` event, load settings and check if `trusted_domains` is non-empty. If so, extract the host from the download URL and check if it's in the list (including subdomains). If not in the list, emit a `"deep-link-blocked"` event with the domain name instead of `"deep-link-install"`
- [x] T023 [US2] Add TypeScript types for the blocked event in `src/lib/tauri.ts` — `DeepLinkBlockedPayload` type already existed (`{ domain: string; url: string }`)
- [x] T024 [US2] Update `src/modules/deep-link/api/useDeepLinkListener.ts` — added listener for `"deep-link-blocked"` event. On blocked event, shows a warning toast explaining the domain is not in the trusted providers list
- [x] T025 [P] [US2] Settings API already supports the new `trustedDomains` field via ts-rs auto-generation — verified existing `api.getSettings` and `api.saveSettings` work with updated `Settings` type
- [x] T026 [US2] Create `src/modules/settings/components/TrustedDomainsSection.tsx` — settings section with: list of trusted domains with remove buttons, input + Add button to add new domains. Uses existing settings save pattern (`onSave` callback). Added to GeneralSection in settings page

**Checkpoint**: Domain allowlist is functional. Can be enabled/disabled in settings, domains added/removed, and protocol links are blocked/allowed accordingly.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Edge case handling, error refinement, and final validation

- [ ] T027 Handle already-installed mod detection in `src-tauri/src/commands/deep_link.rs` — before downloading, check if a mod with the same file name already exists in the library. If so, return a specific error code so the frontend can show "Mod already installed — reinstall or cancel?" in the dialog
- [ ] T028 Handle edge cases in `src/modules/deep-link/components/ProtocolInstallDialog.tsx` — network errors show retry option, invalid file format shows clear error message, patcher-running state shows explanatory message
- [ ] T029 Run `pnpm check` (typecheck + lint + format:check) and fix any issues across all new/modified files
- [ ] T030 Run `cargo clippy -p ltk-manager` and `cargo fmt -p ltk-manager` and fix any warnings or formatting issues
- [ ] T031 Manual end-to-end verification: build with `pnpm tauri build`, test `ltk://install` links from browser with app not running and already running, verify single-instance behavior, test cancel flow, test with invalid URLs, test rate limiting with rapid clicks

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Phase 2 completion
- **User Story 2 (Phase 4)**: Depends on Phase 2 completion. Can run in parallel with US1 but US1 is the MVP priority
- **Polish (Phase 5)**: Depends on Phase 3 completion (US1). Phase 4 (US2) is optional before polish

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Phase 2 — no dependencies on other stories
- **User Story 2 (P3)**: Can start after Phase 2 — independent of US1, but US1 should complete first as the MVP

### Within Each User Story

- Backend commands before frontend hooks
- TypeScript types before hooks that use them
- Hooks before components that consume them
- Components before root-level integration

### Parallel Opportunities

- T001 + T002 + T003 + T004 can all run in parallel (different files)
- T007 + T008 (within Phase 2, different concerns in same file — sequential)
- T013 + T014 can run in parallel (different files: `tauri.ts` vs `stores/deepLink.ts`)
- T025 + T026 can run in parallel (different files)
- T029 + T030 can run in parallel (frontend vs backend checks)

---

## Parallel Example: User Story 1

```bash
# After T012 completes (backend command registered), launch in parallel:
Task T013: "Add TypeScript types and API binding in src/lib/tauri.ts"
Task T014: "Create Zustand store in src/stores/deepLink.ts"

# After T013 + T014 complete, these can be parallelized:
Task T015: "Create useDeepLinkListener hook in src/modules/deep-link/api/"
Task T016: "Create useProtocolInstall mutation hook in src/modules/deep-link/api/"
Task T017: "Create useProtocolInstallProgress hook in src/modules/deep-link/api/"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T010)
3. Complete Phase 3: User Story 1 (T011-T020)
4. **STOP and VALIDATE**: Build and test `ltk://install` links end-to-end
5. Ship if ready — domain allowlist can come in a follow-up

### Incremental Delivery

1. Setup + Foundational → Protocol infrastructure ready
2. Add User Story 1 → Test end-to-end → Ship (MVP!)
3. Add User Story 2 → Test allowlist → Ship
4. Polish phase → Final hardening → Ship

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- The existing `ModLibrary::install_mod_from_package()` handles all heavy lifting — this feature is primarily plumbing (receive URL → download → hand off to existing pipeline)
