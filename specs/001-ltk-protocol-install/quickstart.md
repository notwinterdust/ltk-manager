# Quickstart: LTK Protocol Install

## Prerequisites

- Rust (stable) toolchain
- Node.js + pnpm
- Tauri CLI v2

## Setup

### 1. Add Tauri plugin dependencies

```bash
cd src-tauri
cargo add tauri-plugin-deep-link@2
cargo add tauri-plugin-single-instance@2
```

### 2. Add npm plugin packages

```bash
pnpm add @tauri-apps/plugin-deep-link
```

### 3. Configure deep-link scheme

In `src-tauri/tauri.conf.json`, add under `plugins`:

```json
{
  "plugins": {
    "deep-link": {
      "desktop": {
        "schemes": ["ltk"]
      }
    }
  }
}
```

### 4. Register plugins in main.rs

The single-instance plugin must be registered **before** the deep-link plugin. The single-instance callback receives deep-link URLs when the app is already running.

### 5. Test locally

```bash
# Build the app (protocol registration requires installed app on Windows)
pnpm tauri build

# Test protocol link (from Run dialog or browser)
# ltk://install?url=https://example.com/test-mod.modpkg&name=Test%20Mod&source=Test
```

For development, use `register_all()` to register the protocol at runtime without a full build.

## Key Files to Modify

| File                                  | Change                                                       |
| ------------------------------------- | ------------------------------------------------------------ |
| `src-tauri/Cargo.toml`                | Add `tauri-plugin-deep-link`, `tauri-plugin-single-instance` |
| `src-tauri/tauri.conf.json`           | Add deep-link plugin config with `ltk` scheme                |
| `src-tauri/src/main.rs`               | Register both plugins, wire deep-link handler                |
| `src-tauri/src/deep_link/mod.rs`      | NEW: URL parsing, validation, rate limiting                  |
| `src-tauri/src/deep_link/download.rs` | NEW: HTTPS download + file validation                        |
| `src-tauri/src/commands/deep_link.rs` | NEW: Tauri command wrapper                                   |
| `src-tauri/src/commands/mod.rs`       | Export `deep_link` module                                    |
| `src-tauri/src/state.rs`              | Add `protocol_domain_allowlist` to Settings                  |
| `src/lib/tauri.ts`                    | Add types + API bindings                                     |
| `src/modules/deep-link/`              | NEW: Frontend module (hooks + dialog)                        |
| `src/stores/deepLink.ts`              | NEW: Zustand store for pending install state                 |

## Architecture Overview

```
Browser click → OS protocol handler → Tauri app launch/focus
                                            │
                            ┌───────────────┤
                            │ (not running) │ (already running)
                            ▼               ▼
                      get_current()   single-instance callback
                            │               │
                            └───────┬───────┘
                                    ▼
                          Parse & validate URL
                                    │
                          Rate-limit check
                                    │
                          Domain allowlist check (if enabled)
                                    │
                          Emit "deep-link-install" event
                                    │
                                    ▼
                          Frontend: Show confirmation dialog
                                    │
                          User confirms → call deep_link_install_mod
                                    │
                          Download to temp → validate → install
                                    │
                          Success toast + cache update
```
