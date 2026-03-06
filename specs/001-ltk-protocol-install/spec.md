# Feature Specification: LTK Protocol Install

**Feature Branch**: `001-ltk-protocol-install`
**Created**: 2026-03-06
**Status**: Draft
**Input**: User description: "Implement LTK protocol (`ltk://`) for one-click mod installation from web platforms like Runeforge/DivineSkins"
**Reference**: [league-mod#124](https://github.com/LeagueToolkit/league-mod/issues/124)

## Clarifications

### Session 2026-03-06

- Q: Should checksum verification be implemented? → A: No. Remove verification logic but keep `checksum` as a reserved/ignored parameter in the protocol schema for future use.
- Q: Which platforms should be supported in the initial release? → A: Windows only. Linux and macOS support deferred to a follow-up.
- Q: Should the `api=` parameter be implemented in the initial release? → A: No. Defer implementation; reserve the parameter but only support `url=` initially.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - One-Click Mod Install from Website (Priority: P1)

A user browsing a mod distribution site (e.g., Runeforge, DivineSkins) sees a mod they want. They click an "Install in LTK Manager" button. Their browser prompts them to open LTK Manager (or auto-opens if previously allowed). LTK Manager comes to focus, displays a confirmation dialog showing the mod name, source site, and file details, and the user clicks "Install" to download and add the mod to their library.

**Why this priority**: This is the core value proposition — reducing mod installation from a multi-step manual process (download file, open app, import) to a single click. Without this, the feature has no purpose.

**Independent Test**: Can be fully tested by clicking an `ltk://install?url=...` link in a browser and verifying the mod appears in the library after confirmation.

**Acceptance Scenarios**:

1. **Given** the app is not running, **When** the user clicks an `ltk://install?url=<download_url>` link, **Then** the app launches, shows a confirmation dialog with the mod name and source, and installs the mod on user approval.
2. **Given** the app is already running, **When** the user clicks an `ltk://install` link, **Then** the existing app window comes to focus and shows the confirmation dialog without launching a duplicate instance.
3. **Given** the user clicks "Cancel" on the confirmation dialog, **Then** no download occurs and the app returns to its previous state.
4. **Given** the mod installs successfully, **Then** the user sees a success notification with an option to view the mod in their library.

---

### User Story 2 - Domain Allowlist (Priority: P3)

A security-conscious user wants to restrict mod installations to only trusted domains. They configure a domain allowlist in settings, and any `ltk://` link from a non-allowlisted domain is blocked with an explanation.

**Why this priority**: Optional power-user feature that adds an extra layer of security. Most users will rely on the confirmation dialog, but advanced users benefit from domain-level controls.

**Independent Test**: Can be tested by enabling the allowlist in settings, adding specific domains, and verifying that links from allowed domains proceed normally while links from other domains are blocked.

**Acceptance Scenarios**:

1. **Given** the domain allowlist is enabled with `runeforge.dev`, **When** the user clicks a link downloading from `runeforge.dev`, **Then** the confirmation dialog appears normally.
2. **Given** the domain allowlist is enabled, **When** the user clicks a link downloading from a non-listed domain, **Then** the app shows a "blocked domain" message and does not proceed.
3. **Given** the domain allowlist is disabled (default), **When** the user clicks any valid link, **Then** the confirmation dialog appears regardless of domain.

---

### Edge Cases

- What happens when the URL uses `http://` instead of `https://`? The app rejects the link and shows an error requiring HTTPS.
- What happens when the URL is malformed or contains invalid parameters? The app shows a parse error and does not proceed.
- What happens when a user rapidly clicks multiple install links? The app ignores duplicate/rapid invocations and processes only the first.
- What happens when the download file is not a valid `.modpkg` or `.fantome`? The app validates the file format after download and rejects invalid files with an error message.
- What happens when the user is offline? The app shows a network error with a retry option.
- What happens when the mod is already installed? The app informs the user the mod already exists and offers to reinstall/update or cancel.

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: System MUST register a custom URI protocol (`ltk://`) with the operating system on Windows. Linux and macOS support is out of scope for the initial release.
- **FR-002**: System MUST parse `ltk://install` URLs and extract parameters (`url`, `name`, `source`). The `checksum` and `api` parameters are reserved for future use and MUST be accepted but ignored.
- **FR-003**: System MUST display a confirmation dialog before downloading or installing any mod, showing available mod details (name, source, file size).
- **FR-004**: System MUST download mod files to a temporary location, validate the file format (`.modpkg` or `.fantome`), and only then move to the mod library.
- **FR-005**: System MUST enforce HTTPS for all download URLs, rejecting `http://` URLs.
- **FR-006**: System MUST enforce single-instance behavior so that protocol links focus the existing window rather than launching a new instance.
- **FR-007**: System MUST rate-limit protocol invocations to prevent abuse from rapid-fire link clicks.
- **FR-008**: System MUST sanitize and strictly validate all URL input, rejecting malformed URLs and preventing path traversal.
- **FR-009**: System SHOULD provide a domain allowlist setting that users can optionally enable to restrict installs to trusted domains.
- **FR-010**: System MUST show a success notification after a mod is installed with an option to view it in the library.
- **FR-011**: System MUST handle the case where the mod is already installed, informing the user and offering to reinstall or cancel.

### Key Entities

- **Protocol Link**: A `ltk://install` URI containing parameters that identify a mod to install (download URL, name, and source). The `checksum` and `api` parameters are reserved for future use.
- **Mod Metadata**: Information about a mod (name, source, file size) derived from link parameters and the downloaded file.
- **Domain Allowlist**: A user-configurable list of trusted domains from which mod downloads are permitted when the feature is enabled.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Users can install a mod from a website link in under 30 seconds (from click to mod appearing in library), compared to the current multi-step manual process.
- **SC-002**: 100% of mod installs triggered via the protocol require explicit user confirmation before any download begins.
- **SC-003**: All download URLs using `http://` are rejected — zero unencrypted downloads permitted.
- **SC-004**: Duplicate app instances are never launched — protocol links always reuse the existing window when the app is already running.
- **SC-005**: Rapid-fire protocol invocations (more than 1 per second) are rate-limited, with only the first being processed.
- **SC-006**: The feature works on Windows with OS-native protocol registration. Linux and macOS support is deferred to a follow-up.

## Assumptions

- The app already supports installing `.modpkg` and `.fantome` files through its existing mod installation flow, and this feature reuses that capability.
- Partner sites (Runeforge, DivineSkins) will integrate the `ltk://` protocol links into their mod pages.
- The Mod Distribution API standard (for the `api=` parameter) is deferred and will be documented separately when that parameter is implemented.
- Single-instance enforcement is achievable through the Tauri plugin ecosystem.
- The protocol registration happens automatically during app installation (via the installer/setup process) rather than requiring manual user action.
