//! Session storage for the compact game.
//!
//! This module moves an opaque string to and from the platform's storage. It
//! never inspects the string. `factory_sim` owns the save format, its version,
//! and every validation decision. See docs/compact-persistence.md.

/// Storage key on wasm, and the file basename on native.
const SAVE_SLOT: &str = "factory-game-v3.compact-save";

#[cfg(target_arch = "wasm32")]
mod backend {
  use super::SAVE_SLOT;

  fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
  }

  pub fn load() -> Option<String> {
    storage()?.get_item(SAVE_SLOT).ok()?
  }

  pub fn store(raw: &str) -> Result<(), String> {
    // A quota or private-mode failure is reported, never panicked on. Losing a
    // save must not take the session down with it.
    storage()
      .ok_or_else(|| "local storage is unavailable".to_string())?
      .set_item(SAVE_SLOT, raw)
      .map_err(|_| "local storage rejected the write".to_string())
  }

  pub fn clear() {
    if let Some(storage) = storage() {
      let _ = storage.remove_item(SAVE_SLOT);
    }
  }
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
  use super::SAVE_SLOT;
  use std::path::PathBuf;

  fn save_path() -> PathBuf {
    // XDG-style when HOME is present, current directory otherwise, so a
    // sandboxed run still persists somewhere predictable.
    match std::env::var_os("HOME") {
      Some(home) => PathBuf::from(home)
        .join(".local/share/factory-game-v3")
        .join(format!("{SAVE_SLOT}.json")),
      None => PathBuf::from(format!("{SAVE_SLOT}.json")),
    }
  }

  pub fn load() -> Option<String> {
    std::fs::read_to_string(save_path()).ok()
  }

  pub fn store(raw: &str) -> Result<(), String> {
    let path = save_path();
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    std::fs::write(&path, raw).map_err(|error| error.to_string())
  }

  pub fn clear() {
    let _ = std::fs::remove_file(save_path());
  }
}

pub use backend::{clear, load, store};
