//! Tauri IPC command handlers.
//! ## Pattern
//!
//! ```rust
//! use crate::error::{AppResult, IpcResult};
//!
//! #[tauri::command]
//! pub fn my_command(args: String) -> IpcResult<ReturnType> {
//!     my_command_inner(&args).into()
//! }
//!
//! fn my_command_inner(args: &str) -> AppResult<ReturnType> {
//!     Ok(value)
//! }
//! ```
//!
//! See `docs/ERROR_HANDLING.md` for details.

mod app;
mod deep_link;
pub(crate) mod hotkeys;
mod migration;
mod mods;
pub(crate) mod patcher;
mod profiles;
mod settings;
mod shell;
mod workshop;

pub use app::*;
pub use deep_link::*;
pub use hotkeys::*;
pub use migration::*;
pub use mods::*;
pub use patcher::*;
pub use profiles::*;
pub use settings::*;
pub use shell::*;
pub use workshop::*;
