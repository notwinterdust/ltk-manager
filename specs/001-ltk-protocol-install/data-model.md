# Data Model: LTK Protocol Install

## Entities

### DeepLinkInstallRequest (Rust)

Parsed representation of a `ltk://install` URL. Created in the backend when a deep-link is received, validated, and forwarded to the frontend.

| Field    | Type             | Required | Description                                    |
| -------- | ---------------- | -------- | ---------------------------------------------- |
| `url`    | `String` (URL)   | Yes      | HTTPS download URL for `.modpkg` or `.fantome` |
| `name`   | `Option<String>` | No       | Display name for the confirmation dialog       |
| `source` | `Option<String>` | No       | Name of the originating site                   |

**Validation rules**:

- `url` must be a valid URL with `https` scheme
- `url` host must not resolve to loopback/private IP ranges
- `name` and `source` are URL-decoded strings, max 256 characters
- Unknown parameters are silently ignored (forward compatibility)

### ProtocolInstallProgress (Rust → Frontend event)

Progress payload emitted during the download phase.

| Field              | Type             | Description                                                              |
| ------------------ | ---------------- | ------------------------------------------------------------------------ |
| `stage`            | `String`         | `"downloading"`, `"validating"`, `"installing"`, `"complete"`, `"error"` |
| `bytes_downloaded` | `u64`            | Bytes downloaded so far                                                  |
| `total_bytes`      | `Option<u64>`    | Total file size if known from Content-Length                             |
| `error`            | `Option<String>` | Error message if stage is `"error"`                                      |

### Settings (existing, extended)

Add to existing `Settings` struct in `state.rs`:

| Field                       | Type                  | Default | Description                                                            |
| --------------------------- | --------------------- | ------- | ---------------------------------------------------------------------- |
| `protocol_domain_allowlist` | `Option<Vec<String>>` | `None`  | When `Some`, restricts downloads to listed domains. `None` = disabled. |

### PendingProtocolInstall (Frontend Zustand store)

Client-only state for the confirmation dialog flow.

| Field      | Type                     | Description   |
| ---------- | ------------------------ | ------------- | -------------------------------------------------------- | ---------- | -------- | --------------------- |
| `request`  | `DeepLinkInstallRequest  | null`         | The parsed install request, null when no pending install |
| `status`   | `"idle"                  | "downloading" | "installing"                                             | "complete" | "error"` | Current install state |
| `progress` | `ProtocolInstallProgress | null`         | Download progress data                                   |
| `error`    | `string                  | null`         | Error message if failed                                  |

## State Transitions

### Protocol Install Lifecycle

```
[idle] ──deep-link received──→ [pending]
  │                                │
  │                        user confirms │ user cancels
  │                                │         │
  │                                ▼         │
  │                          [downloading]   │
  │                                │         │
  │                          [validating]    │
  │                                │         │
  │                          [installing]    │
  │                                │         │
  │                          [complete] ─────│──→ [idle]
  │                                          │
  │                            [error] ──────│──→ [idle] (on dismiss)
  │                                          │
  └──────────────────────────────────────────┘
```

## Relationships to Existing Entities

- **InstalledMod**: The final output of a successful protocol install. Returned by `ModLibrary::install_mod_from_package()`.
- **LibraryIndex**: Updated when mod is installed (new entry added to `mods`, `enabled_mods`, `mod_order` in active profile).
- **Settings**: Extended with `protocol_domain_allowlist` field. Persisted to `settings.json`.
