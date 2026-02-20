# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.1.0](https://github.com/LeagueToolkit/ltk-manager/releases/tag/v0.1.0) - 2026-02-20

### Added

- workshop backend cleanup
- library refactor
- adapt repo for standalone ltk-manager
- *(ltk-manager)* add wad blocklist for scripts and tft wads
- *(ltk-manager)* enable devtools in prod
- *(ltk-manager)* add Copy ID functionality to ModCard
- *(ltk-manager)* use ProfileSlug instead of profile IDs for filesystem
- *(ltk-manager)* persist library view mode
- *(ltk-manager)* enhance button and switch components with disabled state handling
- *(ltk-manager)* add Progress component and integrate into dialogs
- *(ltk-manager)* enhance import progress dialog and handle empty file paths
- *(ltk-manager)* add bulk mod installation and progress tracking
- *(ltk-manager)* implement overlay invalidation after mod operations
- *(ltk-manager)* clean up library backend
- *(ltk-manager)* implement layer management features including creation, deletion, reordering, and description updates
- *(ltk-manager)* add rename workshop project functionality
- add locale awareness to string overrides
- add per-layer string overrides support (#83, #84)
- implement mod content providers for Fantome and Modpkg archives
- start using overlay crate
- add profiles
- update mod installation with archive support and metadata extraction
- support animated thumbnails
- workshop mvp
- *(library)* refactor mod thumbnail loading to use asset protocol
- *(manager)* add theme system with accent colors and mod thumbnails
- separate legacy/new patcher, add overlay progress display, fix mod import
- introduce legacy patcher system and mod management API with frontend integration
- add cslol dll
- *(ltk-manager)* enhance patcher functionality with DLL path resolution and resource bundling
- patcher api
- *(ltk-manager)* implement app settings persistence and first-run setup
- *(ltk-manager)* configure auto-updater for Tauri app
- add mod core crate
- adjust border color and fix clippy
- add error handling utils and react query inegration
- add custom titlebar
- *(ltk-manager)* add biome
- add initial ltk-manager dummy project

### Documentation

- add CLAUDE.md and ERROR_HANDLING.md for project guidance and error handling patterns

### Fixed

- move from prerelease postfix to 0.x.x release pattern
- tauri workspace root
- log version
- release workflow
- move cslol-dll.dll and update PATCHER_DLL_NAME reference
- update release workflow to handle MSI and NSIS artifacts correctly
- improve logging in release workflow and refactor overlay path handling
- *(ltk-manager)* update content security policy for asset loading
- *(ltk-manager)* refresh cslol dll
- *(ltk-manager)* update log file naming convention
- *(ltk-manager)* non-blocking patcher stop and overlay log visibility
- *(ltk-manager)* deterministic layer priority ordering
- add dead code allowance for IpcResult methods in error handling
- add dead code allowance for AppError enum in error handling
- update error handling in toggle_mod function to use ok_or

### Other

- update updater public key in tauri configuration
- remove redundant overlay path join in invalidate_overlay function
- update updater public key in tauri configuration
- change log level in legacy patcher from Trace to Info
- update version to 1.0.0-beta.1 and add release configuration
- *(ltk-manager)* fix clippy
- small fix
- fix clippy
- format
- format
- format
- update licenses across multiple crates to MIT or Apache-2.0

### Refactored

- improve image handling and project configuration loading
- *(ltk-manager)* simplify mod reordering logic and enhance profile structure
- *(ltk-manager)* address PR review feedback
- update IpcResponse and IpcResult to use boolean values for 'ok' field
