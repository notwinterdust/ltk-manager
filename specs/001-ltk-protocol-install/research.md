# Research: LTK Protocol Install

## R1: Tauri Deep-Link Plugin Integration

**Decision**: Use `tauri-plugin-deep-link` v2 with `tauri-plugin-single-instance` v2 for protocol handling.

**Rationale**: Tauri's official deep-link plugin handles OS-level protocol registration and URL routing. On Windows/Linux, it supports runtime `register()`. The single-instance plugin is required on desktop because deep links arrive as CLI arguments to new process launches — the single-instance plugin intercepts these and forwards the URL to the existing running instance.

**Key findings from plugin docs**:

- Desktop deep links arrive as command-line arguments to new app processes
- Windows/Linux support runtime registration via `register()` / `register_all()`
- macOS requires bundled app in `/Applications` (out of scope for initial release)
- The single-instance plugin must initialize **before** deep-link plugin and receives deep-link events via its callback
- Tauri validates CLI args against configured schemes to mitigate fake deep links
- Plugin config in `tauri.conf.json`: `plugins.deep-link.desktop.schemes: ["ltk"]`

**Alternatives considered**:

- Manual Windows registry manipulation: Rejected — plugin handles this cleanly and cross-platform
- Local HTTP server fallback: Deferred to future consideration (mentioned in GitHub issue)

## R2: Protocol URL Parsing and Validation

**Decision**: Parse `ltk://install?url=<download_url>&name=<mod_name>&source=<site_name>` using Rust's `url` crate (already a transitive dependency via Tauri).

**Rationale**: The `url` crate provides robust URL parsing with query parameter extraction. HTTPS enforcement is a simple scheme check on the `url` parameter value. Reserved parameters (`checksum`, `api`) are parsed but ignored.

**Validation rules**:

1. Scheme must be `ltk`
2. Host/path must be `install` (only supported action)
3. `url` parameter is required and must be a valid HTTPS URL
4. `name` and `source` are optional display-only parameters
5. `checksum` and `api` are accepted but ignored (reserved)
6. Reject any URL where the download URL host resolves to a private/loopback IP (SSRF prevention)

**Alternatives considered**:

- Custom parser: Rejected — `url` crate is battle-tested and already available
- Regex-based parsing: Rejected — fragile and hard to maintain

## R3: Download and Install Flow

**Decision**: Download to a temp file in the OS temp directory, validate the file extension and magic bytes, then delegate to existing `ModLibrary::install_mod_from_package()`.

**Rationale**: The existing install pipeline handles all the heavy lifting (UUID generation, archive copying, metadata extraction, profile updates, cache invalidation). We only need to get the file to a local path and validate it's a real mod file.

**Flow**:

1. Deep-link URL received → parsed and validated
2. Frontend shows confirmation dialog with mod name, source, download URL domain
3. User confirms → frontend calls `protocol_install_mod` command
4. Backend downloads file to `std::env::temp_dir()/{uuid}.{ext}` using `reqwest` (already a dependency)
5. Validate file: check extension (`.modpkg` or `.fantome`), check file size > 0
6. Call `ModLibrary::install_mod_from_package()` with temp file path
7. Clean up temp file
8. Return `InstalledMod` to frontend

**Alternatives considered**:

- Download in frontend (JS): Rejected — Tauri backend has direct filesystem access and can validate before install
- Stream directly to archive location: Rejected — temp file allows validation before committing to library

## R4: Single-Instance and Event Routing

**Decision**: Use `tauri-plugin-single-instance` to prevent duplicate launches and forward deep-link URLs to the running instance via Tauri events.

**Rationale**: On Windows, clicking a `ltk://` link when the app is already running would launch a second instance. The single-instance plugin intercepts this, extracts the deep-link URL from the CLI args, and sends it to the existing instance. The running instance then emits a frontend event to trigger the confirmation dialog.

**Event flow**:

1. **App not running**: OS launches app with `ltk://...` as CLI arg → `deep_link::get_current()` on startup → emit event to frontend
2. **App running**: OS tries to launch second instance → single-instance plugin catches it → forwards args to running instance → `on_open_url` callback fires → emit event to frontend
3. **Frontend**: `useDeepLinkListener()` hook listens for `"deep-link-install"` event → opens `ProtocolInstallDialog`

**Alternatives considered**:

- IPC pipe between instances: Rejected — single-instance plugin handles this natively
- Named mutex only: Rejected — doesn't support forwarding the URL to the running instance

## R5: Rate Limiting

**Decision**: Simple timestamp-based rate limiter in Rust — reject protocol invocations within 1 second of the last accepted one.

**Rationale**: Prevents abuse from malicious pages rapidly triggering `ltk://` links. A simple `Instant` comparison is sufficient; no need for token buckets or sliding windows.

**Implementation**: Store `last_deep_link_time: Mutex<Option<Instant>>` in app state. On each deep-link event, check if elapsed time > 1 second. If not, silently drop the invocation.

**Alternatives considered**:

- Frontend-side rate limiting: Rejected — deep links arrive at the backend first
- Per-domain rate limiting: Over-engineered for the threat model; simple global rate limit suffices

## R6: Domain Allowlist (P3)

**Decision**: Add optional `protocol_domain_allowlist: Option<Vec<String>>` to `Settings`. When `Some`, check the download URL's host against the list before showing the confirmation dialog.

**Rationale**: Lightweight addition to existing settings persistence. `None` means disabled (default). When enabled, domains are matched exactly (no wildcards in initial implementation).

**Alternatives considered**:

- Regex-based domain matching: Over-engineered for initial release
- Per-mod-site trust system: Better suited for the `api=` parameter (deferred)
