// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One surface per editor window, made at most once and kept for the
//! window's life, a failure included so a broken GPU is not retried forever.
//! The twin of the Windows registry.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use screenwide_workspace::editing::SurfaceInner;

pub(super) type SurfaceSlot = Arc<OnceLock<Result<Arc<SurfaceInner>, String>>>;

static SURFACES: OnceLock<Mutex<HashMap<usize, SurfaceSlot>>> = OnceLock::new();

/// The slot for the window `key` names, made empty the first time.
pub(super) fn slot_for(key: usize) -> Result<SurfaceSlot, String> {
  let mut surfaces = SURFACES
    .get_or_init(|| Mutex::new(HashMap::new()))
    .lock()
    .map_err(|_| "The macOS preview surface registry is unusable".to_owned())?;
  Ok(Arc::clone(surfaces.entry(key).or_default()))
}

/// What names `window`'s surface in the registry, read while it is alive.
pub(crate) fn surface_key(window: &tauri::WebviewWindow) -> Option<usize> {
  window.ns_window().ok().map(|window| window as usize)
}

/// Lets go of the surface of the destroyed window `key` named.
pub(crate) fn forget(key: usize) {
  if let Some(surfaces) = SURFACES.get() {
    let removed = surfaces
      .lock()
      .ok()
      .and_then(|mut surfaces| surfaces.remove(&key));
    drop(removed);
  }
}
