// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// One compositor per editor window: a DirectComposition target belongs to the
/// host HWND it was created for, so a process-wide surface would make the
/// second editor window composite into the first one's window.
///
/// The per-window slot is an inner `OnceLock` handed out from under the map
/// lock rather than a `Result` stored in the map, because creating a surface
/// round-trips to the window's event-loop thread
/// ([`create_editor_on_owning_thread`]) and must not run while a lock that
/// thread could also want is held. The slot keeps the original semantics:
/// created at most once per window, and a failure cached so a broken GPU is not
/// retried forever.
pub(super) type SurfaceSlot =
  std::sync::Arc<OnceLock<Result<std::sync::Arc<SurfaceInner>, String>>>;

static PREVIEW_SURFACES: OnceLock<Mutex<HashMap<isize, SurfaceSlot>>> = OnceLock::new();

/// The reverse lookup, filled once a surface exists: the editor `window_proc`
/// is handed only its own HWND, which cannot name the host window.
#[derive(Default)]
pub(super) struct SurfaceIndex {
  pub(super) by_editor: HashMap<isize, std::sync::Arc<SurfaceInner>>,
}

static SURFACE_INDEX: OnceLock<Mutex<SurfaceIndex>> = OnceLock::new();

pub(super) fn preview_surfaces() -> &'static Mutex<HashMap<isize, SurfaceSlot>> {
  PREVIEW_SURFACES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) fn surface_index() -> &'static Mutex<SurfaceIndex> {
  SURFACE_INDEX.get_or_init(|| Mutex::new(SurfaceIndex::default()))
}

pub(super) fn surface_for_editor(hwnd: HWND) -> Option<std::sync::Arc<SurfaceInner>> {
  let index = surface_index().lock().ok()?;
  index
    .by_editor
    .get(&(hwnd.0 as isize))
    .map(std::sync::Arc::clone)
}

/// What names `window`'s surface in the registry: its handle, read while the
/// window is alive.
pub(crate) fn surface_key(window: &WebviewWindow) -> Option<usize> {
  window.hwnd().ok().map(|hwnd| hwnd.0 as usize)
}

/// Lets go of the surface of the destroyed window `key` named. Its
/// DirectComposition tree and swap chains go once nothing else holds it.
pub(crate) fn forget(key: usize) {
  let slot = preview_surfaces()
    .lock()
    .ok()
    .and_then(|mut surfaces| surfaces.remove(&(key as isize)));
  let Some(inner) = slot.and_then(|slot| slot.get().and_then(|made| made.as_ref().ok()).cloned())
  else {
    return;
  };
  if let Ok(mut index) = surface_index().lock() {
    index
      .by_editor
      .retain(|_, held| !std::sync::Arc::ptr_eq(held, &inner));
  }
}
